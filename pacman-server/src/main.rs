use axum::{routing::get, Router};
use axum_cookie::CookieLayer;

use crate::{app::AppState, auth::AuthRegistry, config::Config};
mod routes;

mod app;
mod auth;
mod config;
mod errors;
mod session;
use std::sync::Arc;
use std::time::{Duration, Instant};
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{watch, Notify};

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
    let shutdown_timeout = std::time::Duration::from_secs(config.shutdown_timeout_seconds as u64);
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

    // coordinated graceful shutdown with timeout
    let notify = Arc::new(Notify::new());
    let (tx_signal, rx_signal) = watch::channel::<Option<Instant>>(None);

    {
        let notify = notify.clone();
        let tx = tx_signal.clone();
        tokio::spawn(async move {
            let signaled_at = shutdown_signal().await;
            let _ = tx.send(Some(signaled_at));
            notify.notify_waiters();
        });
    }

    let mut rx_for_timeout = rx_signal.clone();
    let timeout_task = async move {
        // wait until first signal observed
        while rx_for_timeout.borrow().is_none() {
            if rx_for_timeout.changed().await.is_err() {
                return; // channel closed
            }
        }
        tokio::time::sleep(shutdown_timeout).await;
        eprintln!("shutdown timeout elapsed (>{:.2?}) - forcing exit", shutdown_timeout);
        std::process::exit(1);
    };

    let notify_for_server = notify.clone();
    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        notify_for_server.notified().await;
    });

    tokio::select! {
        res = server => {
            // server finished; if we had a signal, print remaining time
            let now = Instant::now();
            if let Some(signaled_at) = *rx_signal.borrow() {
                let elapsed = now.duration_since(signaled_at);
                if elapsed < shutdown_timeout {
                    let remaining = shutdown_timeout - elapsed;
                    eprintln!("graceful shutdown complete, remaining time: {:.2?}", remaining);
                }
            }
            res.unwrap();
        }
        _ = timeout_task => {}
    }
}

async fn shutdown_signal() -> Instant {
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
        _ = ctrl_c => { Instant::now() }
        _ = sigterm => { Instant::now() }
    }
}
