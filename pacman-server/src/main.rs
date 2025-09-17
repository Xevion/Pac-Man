use axum::Router;

use crate::config::Config;

mod config;

#[tokio::main]
async fn main() {
    // Load environment variables
    #[cfg(debug_assertions)]
    dotenvy::from_path(format!("{}.env", env!("CARGO_MANIFEST_DIR"))).ok();
    #[cfg(not(debug_assertions))]
    dotenvy::dotenv().ok();

    // Load configuration
    let config: Config = config::load_config();

    let app = Router::new().fallback(|| async { "Hello, World!" });

    let addr = std::net::SocketAddr::new(config.host, config.port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
