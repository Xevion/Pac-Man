use axum::{routing::get, Router};
use axum_cookie::CookieLayer;

use crate::{app::AppState, auth::AuthRegistry, config::Config};
mod formatter;
mod logging;
mod routes;

mod app;
mod auth;
mod config;
mod data;
mod errors;
mod session;
use std::time::Instant;
use std::{sync::Arc, time::Duration};
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{watch, Notify};
use tracing::{info, trace, warn};

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

    let app_state = AppState::new(config, auth, db);
    {
        // migrations succeeded
        let mut h = app_state.health.write().await;
        h.set_migrations(true);
    }

    let app = Router::new()
        .route("/", get(|| async { "Hello, World! Visit /auth/github to start OAuth flow." }))
        .route("/health", get(routes::health_handler))
        .route("/auth/providers", get(routes::list_providers_handler))
        .route("/auth/{provider}", get(routes::oauth_authorize_handler))
        .route("/auth/{provider}/callback", get(routes::oauth_callback_handler))
        .route("/logout", get(routes::logout_handler))
        .route("/profile", get(routes::profile_handler))
        .with_state(app_state.clone())
        .layer(CookieLayer::default());

    info!(%addr, "Starting HTTP server bind");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!(%addr, "HTTP server listening");

    // coordinated graceful shutdown with timeout
    let notify = Arc::new(Notify::new());
    let (tx_signal, rx_signal) = watch::channel::<Option<Instant>>(None);

    // Spawn background health checker (listens for shutdown via notify)
    {
        let health_state = app_state.health.clone();
        let db_pool = app_state.db.clone();
        let notify_for_health = notify.clone();
        tokio::spawn(async move {
            trace!("Health checker task started");
            let mut backoff: u32 = 1;
            let mut next_sleep = Duration::from_secs(0);
            loop {
                tokio::select! {
                    _ = notify_for_health.notified() => {
                        trace!("Health checker received shutdown notification; exiting");
                        break;
                    }
                    _ = tokio::time::sleep(next_sleep) => {
                        let ok = sqlx::query("SELECT 1").execute(&*db_pool).await.is_ok();
                        {
                            let mut h = health_state.write().await;
                            h.set_database(ok);
                        }
                        if ok {
                            trace!(database_ok = true, "Health check succeeded; scheduling next run in 90s");
                            backoff = 1;
                            next_sleep = Duration::from_secs(90);
                        } else {
                            backoff = (backoff.saturating_mul(2)).min(60);
                            trace!(database_ok = false, backoff, "Health check failed; backing off");
                            next_sleep = Duration::from_secs(backoff as u64);
                        }
                    }
                }
            }
        });
    }

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
