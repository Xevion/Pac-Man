use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::{info, warn};

pub type PgPool = Pool<Postgres>;

/// Create a PostgreSQL database pool.
///
/// - `immediate`: If true, establishes connection immediately (panics on failure).
///                If false, uses lazy connection (for tests or when database may not be needed).
/// - `database_url`: The database connection URL.
/// - `max_connections`: Maximum number of connections in the pool.
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

/// Create a dummy pool that will fail on any actual database operation.
/// Used when database is not configured but the app still needs to start.
pub fn create_dummy_pool() -> PgPool {
    // This creates a pool with an invalid URL that will fail on actual use
    // The pool itself can be created (lazy), but any operation will fail
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://invalid:invalid@localhost:5432/invalid")
        .expect("Failed to create dummy pool")
}
