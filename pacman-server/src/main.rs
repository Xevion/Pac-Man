use axum::{routing::get, Router};
use axum_cookie::CookieLayer;

use crate::{app::AppState, auth::AuthRegistry, config::Config};
mod routes;

mod app;
mod auth;
mod config;
mod errors;
mod session;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() {
    // Load environment variables
    #[cfg(debug_assertions)]
    dotenvy::from_path(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env")).ok();
    #[cfg(not(debug_assertions))]
    dotenvy::dotenv().ok();

    // Load configuration
    let config: Config = config::load_config();

    let addr = std::net::SocketAddr::new(config.host, config.port);
    let auth = AuthRegistry::new(&config).expect("auth initializer");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World! Visit /auth/github to start OAuth flow." }))
        .route("/auth/{provider}", get(routes::oauth_authorize_handler))
        .route("/auth/{provider}/callback", get(routes::oauth_callback_handler))
        .route("/logout", get(routes::logout_handler))
        .route("/profile", get(routes::profile_handler))
        .with_state(AppState::new(config, auth))
        .layer(CookieLayer::default());

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
        eprintln!("received Ctrl+C, shutting down");
    };

    #[cfg(unix)]
    let sigterm = async {
        let mut term_stream = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        term_stream.recv().await;
        eprintln!("received SIGTERM, shutting down");
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = sigterm => {}
    }
}
