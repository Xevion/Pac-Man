#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

use crate::{
    app::{create_router, AppState},
    auth::AuthRegistry,
    config::Config,
    data::pool::{create_dummy_pool, create_pool},
};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, trace, warn};

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{watch, Notify};

#[cfg_attr(coverage_nightly, coverage(off))]
mod config;
#[cfg_attr(coverage_nightly, coverage(off))]
mod errors;
#[cfg_attr(coverage_nightly, coverage(off))]
mod formatter;

mod app;
mod auth;
mod data;
mod image;
mod logging;
mod routes;
mod session;

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install default crypto provider");

    // Load environment variables
    #[cfg(debug_assertions)]
    dotenvy::from_path(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env")).ok();
    #[cfg(not(debug_assertions))]
    dotenvy::dotenv().ok();

    // Load configuration
    let config: Config = config::load_config();

    // Initialize tracing subscriber
    logging::setup_logging();
    trace!(host = %config.host, port = config.port, shutdown_timeout_seconds = config.shutdown_timeout_seconds, "Loaded server configuration");

    // Log configuration status
    info!(
        database = config.database.is_some(),
        discord = config.discord.is_some(),
        github = config.github.is_some(),
        s3 = config.s3.is_some(),
        "Feature configuration"
    );

    let addr = std::net::SocketAddr::new(config.host, config.port);
    let shutdown_timeout = std::time::Duration::from_secs(config.shutdown_timeout_seconds as u64);

    // Initialize auth registry (only enabled providers will be registered)
    let auth = AuthRegistry::new(&config).expect("auth initializer");

    // Initialize database - either connect to configured database or create a dummy pool
    let db = if let Some(ref db_config) = config.database {
        info!("Connecting to configured database");
        let pool = create_pool(true, &db_config.url, 10).await;

        // Run migrations
        info!("Running database migrations");
        if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
            panic!("failed to run database migrations: {}", e);
        }

        pool
    } else {
        info!("No database configured, creating dummy pool (database-dependent features will be unavailable)");
        create_dummy_pool()
    };

    // Create the shutdown notification before creating AppState
    let notify = Arc::new(Notify::new());

    let app_state = AppState::new(config, auth, db, notify.clone()).await;
    {
        // Set health status based on configuration
        let mut h = app_state.health.write().await;
        if app_state.database_configured {
            // Database was configured - migrations ran successfully
            h.set_migrations(true);
            h.set_database(true);
        }
        // If database is not configured, Health::ok() returns true by default
        // because database_enabled is false
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
