use axum::Router;
use pacman_server::{
    app::{create_router, AppState},
    auth::AuthRegistry,
    config::Config,
};
use std::sync::Arc;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};
use tokio::sync::Notify;

/// Test configuration for integration tests
pub struct TestConfig {
    pub database_url: String,
    pub container: ContainerAsync<GenericImage>,
    pub config: Config,
}

impl TestConfig {
    /// Create a test configuration with a test database
    pub async fn new() -> Self {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install default crypto provider");

        let (database_url, container) = setup_test_database("testdb", "testuser", "testpass").await;

        let config = Config {
            database_url: database_url.clone(),
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

        Self {
            database_url,
            container,
            config,
        }
    }
}

/// Set up a test PostgreSQL database using testcontainers
async fn setup_test_database(db: &str, user: &str, password: &str) -> (String, ContainerAsync<GenericImage>) {
    let container = GenericImage::new("postgres", "15")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
        .with_env_var("POSTGRES_DB", db)
        .with_env_var("POSTGRES_USER", user)
        .with_env_var("POSTGRES_PASSWORD", password)
        .start()
        .await
        .unwrap();

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    (
        format!("postgresql://{user}:{password}@{host}:{port}/{db}?sslmode=disable"),
        container,
    )
}

/// Create a test app state with database and auth registry
pub async fn create_test_app_state(test_config: &TestConfig) -> AppState {
    // Create database pool
    let db = pacman_server::data::pool::create_pool(&test_config.database_url, 5).await;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run database migrations");

    // Create auth registry
    let auth = AuthRegistry::new(&test_config.config).expect("Failed to create auth registry");

    // Create app state
    let notify = Arc::new(Notify::new());
    let app_state = AppState::new(test_config.config.clone(), auth, db, notify).await;

    // Set health status to true for tests (migrations and database are both working)
    {
        let mut health = app_state.health.write().await;
        health.set_migrations(true);
        health.set_database(true);
    }

    app_state
}

/// Create a test router with the given app state
pub fn create_test_router(app_state: AppState) -> Router {
    create_router(app_state)
}
