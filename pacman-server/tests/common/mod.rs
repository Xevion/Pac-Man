use axum_test::TestServer;
use bon::builder;
use pacman_server::{
    app::{create_router, AppState},
    auth::AuthRegistry,
    config::Config,
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
    // Optional database
    pub container: Option<ContainerAsync<GenericImage>>,
}

#[builder]
pub async fn test_context(#[builder(default = false)] use_database: bool, auth_registry: Option<AuthRegistry>) -> TestContext {
    CRYPTO_INIT.call_once(|| {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install default crypto provider");
    });

    // Set up logging
    std::env::set_var("RUST_LOG", "debug,sqlx=info");
    pacman_server::logging::setup_logging();
    let (database_url, container) = if use_database {
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
        (
            Some(format!("postgresql://{user}:{password}@{host}:{port}/{db}?sslmode=disable")),
            Some(container),
        )
    } else {
        (None, None)
    };

    let config = Config {
        database_url: database_url.clone().unwrap_or_default(),
        discord_client_id: "test_discord_client_id".to_string(),
        discord_client_secret: "test_discord_client_secret".to_string(),
        github_client_id: "test_github_client_id".to_string(),
        github_client_secret: "test_github_client_secret".to_string(),
        s3_access_key: "test_s3_access_key".to_string(),
        s3_secret_access_key: "test_s3_secret_access_key".to_string(),
        s3_bucket_name: "test_bucket".to_string(),
        s3_public_base_url: "https://test.example.com".to_string(),
        port: 0, // Will be set by test server
        host: "127.0.0.1".parse().unwrap(),
        shutdown_timeout_seconds: 5,
        public_base_url: "http://localhost:3000".to_string(),
        jwt_secret: "test_jwt_secret_key_for_testing_only".to_string(),
    };

    let db = if use_database {
        let db = pacman_server::data::pool::create_pool(use_database, &database_url.unwrap(), 5).await;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&db)
            .instrument(debug_span!("running_migrations"))
            .await
            .expect("Failed to run database migrations");
        debug!("Database migrations ran successfully");

        db
    } else {
        // Create a dummy database pool that will fail gracefully
        let dummy_url = "postgresql://dummy:dummy@localhost:5432/dummy?sslmode=disable";
        pacman_server::data::pool::create_pool(false, dummy_url, 1).await
    };

    // Create auth registry
    let auth = auth_registry.unwrap_or_else(|| AuthRegistry::new(&config).expect("Failed to create auth registry"));

    // Create app state
    let notify = Arc::new(Notify::new());
    let app_state = AppState::new_with_database(config.clone(), auth, db, notify, use_database).await;

    // Set health status based on database usage
    {
        let mut health = app_state.health.write().await;
        health.set_migrations(use_database);
        health.set_database(use_database);
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
