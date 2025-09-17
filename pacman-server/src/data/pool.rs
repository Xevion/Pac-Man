use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::{info, warn};

pub type PgPool = Pool<Postgres>;

pub async fn create_pool(database_url: &str, max_connections: u32) -> PgPool {
    info!("Connecting to PostgreSQL");
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .unwrap_or_else(|e| {
            warn!(error = %e, "Failed to connect to PostgreSQL");
            panic!("database connect failed: {}", e);
        })
}
