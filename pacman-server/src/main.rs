use crate::{
    app::{create_router, AppState},
    auth::AuthRegistry,
    config::Config,
};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, trace, warn};

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{watch, Notify};

mod app;
mod auth;
mod config;
mod data;
mod errors;
mod formatter;
mod image;
mod logging;
mod routes;
mod session;

#[tokio::main]
async fn main() {
    // Load environment variables
    #[cfg(debug_assertions)]
    dotenvy::from_path(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env")).ok();
    #[cfg(not(debug_assertions))]
    dotenvy::dotenv().ok();

    // Load configuration
    let config: Config = config::load_config();

    // Initialize tracing subscriber
    logging::setup_logging(&config);
    trace!(host = %config.host, port = config.port, shutdown_timeout_seconds = config.shutdown_timeout_seconds, "Loaded server configuration");

    let addr = std::net::SocketAddr::new(config.host, config.port);
    let shutdown_timeout = std::time::Duration::from_secs(config.shutdown_timeout_seconds as u64);
    let auth = AuthRegistry::new(&config).expect("auth initializer");
    let db = data::pool::create_pool(&config.database_url, 10).await;

    // Run database migrations at startup
    if let Err(e) = sqlx::migrate!("./migrations").run(&db).await {
        panic!("failed to run database migrations: {}", e);
    }

    // Create the shutdown notification before creating AppState
    let notify = Arc::new(Notify::new());

    let app_state = AppState::new(config, auth, db, notify.clone()).await;
    {
        // migrations succeeded
        let mut h = app_state.health.write().await;
        h.set_migrations(true);
    }

    let app = create_router(app_state);

    info!(%addr, "Starting HTTP server bind");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!(%addr, "HTTP server listening");

    // coordinated graceful shutdown with timeout
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
        warn!(timeout = ?shutdown_timeout, "Shutdown timeout elapsed; forcing exit");
        std::process::exit(1);
    };

    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        notify.notified().await;
    });

    tokio::select! {
        res = server => {
            // server finished; if we had a signal, print remaining time
            let now = Instant::now();
            if let Some(signaled_at) = *rx_signal.borrow() {
                let elapsed = now.duration_since(signaled_at);
                if elapsed < shutdown_timeout {
                    let remaining = format!("{:.2?}", shutdown_timeout - elapsed);
                    info!(remaining = remaining, "Graceful shutdown complete");
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
        warn!(signal = "ctrl_c", "Received Ctrl+C; shutting down");
    };

    #[cfg(unix)]
    let sigterm = async {
        let mut term_stream = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        term_stream.recv().await;
        warn!(signal = "sigterm", "Received SIGTERM; shutting down");
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { Instant::now() }
        _ = sigterm => { Instant::now() }
    }
}
