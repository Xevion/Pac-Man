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
    pub database_url: Option<String>,
    pub container: Option<ContainerAsync<GenericImage>>,
    pub config: Config,
}

impl TestConfig {
    /// Create a test configuration with a test database
    pub async fn new() -> Self {
        Self::new_with_database(true).await
    }

    /// Create a test configuration with optional database setup
    pub async fn new_with_database(use_database: bool) -> Self {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install default crypto provider");

        let (database_url, container) = if use_database {
            let (url, container) = setup_test_database("testdb", "testuser", "testpass").await;
            (Some(url), Some(container))
        } else {
            (None, None)
        };

        let config = Config {
            database_url: database_url
                .clone()
                .unwrap_or_else(|| "postgresql://dummy:dummy@localhost:5432/dummy?sslmode=disable".to_string()),
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
    create_test_app_state_with_database(test_config, true).await
}

/// Create a test app state with optional database setup
pub async fn create_test_app_state_with_database(test_config: &TestConfig, use_database: bool) -> AppState {
    let db = if use_database {
        // Create database pool
        let db_url = test_config
            .database_url
            .as_ref()
            .expect("Database URL required when use_database is true");
        let db = pacman_server::data::pool::create_pool(use_database, db_url, 5).await;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("Failed to run database migrations");

        db
    } else {
        // Create a dummy database pool that will fail gracefully
        let dummy_url = "postgresql://dummy:dummy@localhost:5432/dummy?sslmode=disable";
        pacman_server::data::pool::create_pool(false, dummy_url, 1).await
    };

    // Create auth registry
    let auth = AuthRegistry::new(&test_config.config).expect("Failed to create auth registry");

    // Create app state
    let notify = Arc::new(Notify::new());
    let app_state = AppState::new_with_database(test_config.config.clone(), auth, db, notify, use_database).await;

    // Set health status based on database usage
    {
        let mut health = app_state.health.write().await;
        health.set_migrations(use_database);
        health.set_database(use_database);
    }

    app_state
}

/// Create a test router with the given app state
pub fn create_test_router(app_state: AppState) -> Router {
    create_router(app_state)
}
