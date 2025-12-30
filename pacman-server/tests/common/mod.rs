use axum_test::TestServer;
use bon::builder;
use pacman_server::{
    app::{create_router, AppState},
    auth::AuthRegistry,
    config::{Config, DatabaseConfig, DiscordConfig, GithubConfig},
    data::pool::{create_dummy_pool, create_pool},
};
use std::sync::{Arc, Once};
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};
use tokio::sync::Notify;
use tracing::{debug, debug_span, Instrument};

static CRYPTO_INIT: Once = Once::new();

/// Test configuration for integration tests
/// Do not destructure this struct if you need the database, it will be dropped implicitly, which will kill the database container prematurely.
#[allow(dead_code)]
pub struct TestContext {
    pub config: Config,
    pub server: TestServer,
    pub app_state: AppState,
    // Optional database container (only for Postgres tests)
    pub container: Option<ContainerAsync<GenericImage>>,
}

#[builder]
pub async fn test_context(
    /// Whether to use a real PostgreSQL database via testcontainers (default: false)
    #[builder(default = false)]
    use_database: bool,
    /// Optional custom AuthRegistry (otherwise built from config)
    auth_registry: Option<AuthRegistry>,
    /// Include Discord OAuth config (default: true for backward compatibility)
    #[builder(default = true)]
    with_discord: bool,
    /// Include GitHub OAuth config (default: true for backward compatibility)
    #[builder(default = true)]
    with_github: bool,
) -> TestContext {
    CRYPTO_INIT.call_once(|| {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install default crypto provider");
    });

    // Set up logging
    std::env::set_var("RUST_LOG", "debug,sqlx=info");
    pacman_server::logging::setup_logging();

    let (database_config, container) = if use_database {
        let db = "testdb";
        let user = "testuser";
        let password = "testpass";

        // Create container request
        let container_request = GenericImage::new("postgres", "15")
            .with_exposed_port(5432.tcp())
            .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
            .with_env_var("POSTGRES_DB", db)
            .with_env_var("POSTGRES_USER", user)
            .with_env_var("POSTGRES_PASSWORD", password);

        tracing::debug!(request_image = ?container_request.image(), "Acquiring postgres testcontainer");
        let start = std::time::Instant::now();
        let container = container_request.start().await.unwrap();
        let duration: std::time::Duration = start.elapsed();
        let host = container.get_host().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();

        tracing::debug!(host = %host, port = %port, duration = ?duration, "Test database ready");
        let url = format!("postgresql://{user}:{password}@{host}:{port}/{db}?sslmode=disable");
        (Some(DatabaseConfig { url }), Some(container))
    } else {
        (None, None)
    };

    // Build OAuth configs if requested
    let discord = if with_discord {
        Some(DiscordConfig {
            client_id: "test_discord_client_id".to_string(),
            client_secret: "test_discord_client_secret".to_string(),
        })
    } else {
        None
    };

    let github = if with_github {
        Some(GithubConfig {
            client_id: "test_github_client_id".to_string(),
            client_secret: "test_github_client_secret".to_string(),
        })
    } else {
        None
    };

    let config = Config {
        database: database_config,
        discord,
        github,
        s3: None, // Tests don't need S3
        port: 0,  // Will be set by test server
        host: "127.0.0.1".parse().unwrap(),
        shutdown_timeout_seconds: 5,
        public_base_url: "http://localhost:3000".to_string(),
        jwt_secret: "test_jwt_secret_key_for_testing_only".to_string(),
    };

    // Create database pool
    let db = if let Some(ref db_config) = config.database {
        let pool = create_pool(false, &db_config.url, 5).await;

        // Run migrations for Postgres
        sqlx::migrate!("./migrations")
            .run(&pool)
            .instrument(debug_span!("running_migrations"))
            .await
            .expect("Failed to run database migrations");
        debug!("Database migrations ran successfully");

        pool
    } else {
        // Create dummy pool for tests that don't need database
        create_dummy_pool()
    };

    // Create auth registry
    let auth = auth_registry.unwrap_or_else(|| AuthRegistry::new(&config).expect("Failed to create auth registry"));

    // Create app state
    let notify = Arc::new(Notify::new());
    let app_state = AppState::new_with_options(config.clone(), auth, db, notify, use_database).await;

    // Set health status
    {
        let mut health = app_state.health.write().await;
        if use_database {
            health.set_migrations(true);
            health.set_database(true);
        }
    }

    let router = create_router(app_state.clone());
    let mut server = TestServer::new(router).unwrap();
    server.save_cookies();

    TestContext {
        server,
        app_state,
        config,
        container,
    }
}
