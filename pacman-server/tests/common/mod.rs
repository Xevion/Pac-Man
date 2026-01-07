use axum_test::TestServer;
use bon::builder;
use pacman_server::{
    app::{create_router, AppState},
    auth::AuthRegistry,
    config::{Config, DiscordConfig, GithubConfig},
    data::pool::create_dummy_pool,
};
use std::sync::{Arc, Once};
use tokio::sync::Notify;

#[cfg(feature = "postgres-tests")]
use pacman_server::{config::DatabaseConfig, data::pool::create_pool};
#[cfg(feature = "postgres-tests")]
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};
#[cfg(feature = "postgres-tests")]
use tracing::debug;

#[allow(dead_code)]
static INIT: Once = Once::new();

/// Test configuration for integration tests.
/// Do not destructure this struct if you need the database container - it will be dropped
/// implicitly, which will kill the database container prematurely.
#[allow(dead_code)]
pub struct TestContext {
    pub config: Config,
    pub server: TestServer,
    pub app_state: AppState,
    /// Container handle (only present for PostgreSQL tests with postgres-tests feature)
    #[cfg(feature = "postgres-tests")]
    pub container: Option<ContainerAsync<GenericImage>>,
}

#[allow(dead_code)]
fn init_once() {
    INIT.call_once(|| {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install default crypto provider");
    });
}

/// Create a PostgreSQL test database via testcontainers.
#[cfg(feature = "postgres-tests")]
async fn create_postgres_test_pool() -> (pacman_server::data::pool::PgPool, ContainerAsync<GenericImage>) {
    let db = "testdb";
    let user = "testuser";
    let password = "testpass";

    let container_request = GenericImage::new("postgres", "15")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_DB", db)
        .with_env_var("POSTGRES_USER", user)
        .with_env_var("POSTGRES_PASSWORD", password);

    tracing::debug!(request_image = ?container_request.image(), "Acquiring postgres testcontainer");
    let start = std::time::Instant::now();
    let container = container_request.start().await.unwrap();
    let duration = start.elapsed();
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    tracing::debug!(host = %host, port = %port, duration = ?duration, "Test database ready");
    let url = format!("postgresql://{user}:{password}@{host}:{port}/{db}?sslmode=disable");

    let pool = create_pool(false, &url, 5).await;

    // Run migrations for Postgres
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    debug!("Database migrations ran successfully");

    (pool, container)
}

#[builder]
pub async fn test_context(
    /// Use real PostgreSQL via testcontainers (requires `postgres-tests` feature, default: false)
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
    init_once();

    // Set up logging
    std::env::set_var("RUST_LOG", "debug,sqlx=info");
    pacman_server::logging::setup_logging();

    // Create database pool based on configuration
    #[cfg(feature = "postgres-tests")]
    let (db, container, database_config, database_configured) = if use_database {
        let (pool, container) = create_postgres_test_pool().await;
        (pool, Some(container), Some(DatabaseConfig { url: "postgres://test".to_string() }), true)
    } else {
        let pool = create_dummy_pool();
        (pool, None, None, false)
    };

    #[cfg(not(feature = "postgres-tests"))]
    let (db, database_config, database_configured) = {
        if use_database {
            panic!(
                "Database tests require the `postgres-tests` feature. \
                 Run with: cargo test --features postgres-tests"
            );
        }
        let pool = create_dummy_pool();
        (pool, None, false)
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
        s3: None,
        port: 0,
        host: "127.0.0.1".parse().unwrap(),
        shutdown_timeout_seconds: 5,
        public_base_url: "http://localhost:3000".to_string(),
        jwt_secret: "test_jwt_secret_key_for_testing_only".to_string(),
    };

    // Create auth registry
    let auth =
        auth_registry.unwrap_or_else(|| AuthRegistry::new(&config).expect("Failed to create auth registry"));

    // Create app state
    let notify = Arc::new(Notify::new());
    let app_state = AppState::new_with_options(config.clone(), auth, db, notify, database_configured).await;

    // Set health status based on database configuration
    {
        let mut health = app_state.health.write().await;
        if database_configured {
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
        #[cfg(feature = "postgres-tests")]
        container,
    }
}
