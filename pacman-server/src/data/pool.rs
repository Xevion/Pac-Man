use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::{info, warn};

pub type PgPool = Pool<Postgres>;

pub async fn create_pool(immediate: bool, database_url: &str, max_connections: u32) -> PgPool {
    info!(immediate, "Connecting to PostgreSQL");

    let options = PgPoolOptions::new().max_connections(max_connections);

    if immediate {
        options.connect(database_url).await.unwrap_or_else(|e| {
            warn!(error = %e, "Failed to connect to PostgreSQL");
            panic!("database connect failed: {}", e);
        })
    } else {
        options
            .connect_lazy(database_url)
            .expect("Failed to create lazy database pool")
    }
}
