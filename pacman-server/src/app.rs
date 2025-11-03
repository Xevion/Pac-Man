use axum::{routing::get, Router};
use axum_cookie::CookieLayer;
use dashmap::DashMap;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Notify, RwLock};
use tokio::task::JoinHandle;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info_span;

use crate::data::pool::PgPool;
use crate::{auth::AuthRegistry, config::Config, image::ImageStorage, routes};

#[derive(Debug, Clone, Default)]
pub struct Health {
    migrations: bool,
    database: bool,
}

impl Health {
    pub fn ok(&self) -> bool {
        self.migrations && self.database
    }

    pub fn set_migrations(&mut self, done: bool) {
        self.migrations = done;
    }

    pub fn set_database(&mut self, ok: bool) {
        self.database = ok;
    }
}

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthRegistry>,
    pub sessions: Arc<DashMap<String, crate::auth::provider::AuthUser>>,
    pub jwt_encoding_key: Arc<EncodingKey>,
    pub jwt_decoding_key: Arc<DecodingKey>,
    pub db: PgPool,
    pub health: Arc<RwLock<Health>>,
    pub image_storage: Arc<ImageStorage>,
    pub healthchecker_task: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl AppState {
    pub async fn new(config: Config, auth: AuthRegistry, db: PgPool, shutdown_notify: Arc<Notify>) -> Self {
        Self::new_with_database(config, auth, db, shutdown_notify, true).await
    }

    pub async fn new_with_database(
        config: Config,
        auth: AuthRegistry,
        db: PgPool,
        shutdown_notify: Arc<Notify>,
        use_database: bool,
    ) -> Self {
        let jwt_secret = config.jwt_secret.clone();

        // Initialize image storage
        let image_storage = match ImageStorage::from_config(&config) {
            Ok(storage) => Arc::new(storage),
            Err(e) => {
                tracing::warn!(error = %e, "Failed to initialize image storage, avatar processing will be disabled");
                // Create a dummy storage that will fail gracefully
                Arc::new(ImageStorage::new(&config, "dummy").unwrap_or_else(|_| panic!("Failed to create dummy image storage")))
            }
        };

        let app_state = Self {
            auth: Arc::new(auth),
            sessions: Arc::new(DashMap::new()),
            jwt_encoding_key: Arc::new(EncodingKey::from_secret(jwt_secret.as_bytes())),
            jwt_decoding_key: Arc::new(DecodingKey::from_secret(jwt_secret.as_bytes())),
            db: db,
            health: Arc::new(RwLock::new(Health::default())),
            image_storage,
            healthchecker_task: Arc::new(RwLock::new(None)),
        };

        // Start the healthchecker task only if database is being used
        if use_database {
            let health_state = app_state.health.clone();
            let db_pool = app_state.db.clone();
            let healthchecker_task = app_state.healthchecker_task.clone();

            let task = tokio::spawn(async move {
                tracing::trace!("Health checker task started");
                let mut backoff: u32 = 1;
                let mut next_sleep = Duration::from_secs(0);
                loop {
                    tokio::select! {
                        _ = shutdown_notify.notified() => {
                            tracing::trace!("Health checker received shutdown notification; exiting");
                            break;
                        }

                        _ = tokio::time::sleep(next_sleep) => {
                            // Run health check
                        }
                    }

                    // Run the actual health check
                    let ok = sqlx::query("SELECT 1").execute(&db_pool).await.is_ok();
                    {
                        let mut h = health_state.write().await;
                        h.set_database(ok);
                    }
                    if ok {
                        tracing::trace!(database_ok = true, "Health check succeeded; scheduling next run in 90s");
                        backoff = 1;
                        next_sleep = Duration::from_secs(90);
                    } else {
                        backoff = (backoff.saturating_mul(2)).min(60);
                        tracing::trace!(database_ok = false, backoff, "Health check failed; backing off");
                        next_sleep = Duration::from_secs(backoff as u64);
                    }
                }
            });

            // Store the task handle
            let mut task_handle = healthchecker_task.write().await;
            *task_handle = Some(task);
        }

        app_state
    }

    /// Force an immediate health check (debug mode only)
    pub async fn check_health(&self) -> bool {
        let ok = sqlx::query("SELECT 1").execute(&self.db).await.is_ok();
        let mut h = self.health.write().await;
        h.set_database(ok);
        ok
    }
}

/// Create a custom span for HTTP requests with reduced verbosity
pub fn make_span<B>(request: &axum::http::Request<B>) -> tracing::Span {
    let path = request
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or_else(|| request.uri().path());

    if request.method() == axum::http::Method::GET {
        info_span!(
            "request",
            path = %path,
        )
    } else {
        info_span!(
            "request",
            method = %request.method(),
            path = %path,
        )
    }
}

/// Create the application router with all routes and middleware
pub fn create_router(app_state: AppState) -> Router {
    // Get static files directory from environment variable
    // Default to /app/static for production (Docker), or web/dist/client for local dev
    let static_dir = std::env::var("STATIC_FILES_DIR").unwrap_or_else(|_| {
        if std::path::Path::new("/app/static").exists() {
            "/app/static".to_string()
        } else {
            "web/dist/client".to_string()
        }
    });

    let static_path = PathBuf::from(&static_dir);
    let index_path = static_path.join("index.html");

    // Create API router with all backend routes
    let api_router = Router::new()
        .route(
            "/",
            get(|| async { "Pac-Man API Server. Visit /api/auth/github to start OAuth flow." }),
        )
        .route("/health", get(routes::health_handler))
        .route("/auth/providers", get(routes::list_providers_handler))
        .route("/auth/{provider}", get(routes::oauth_authorize_handler))
        .route("/auth/{provider}/callback", get(routes::oauth_callback_handler))
        .route("/logout", get(routes::logout_handler))
        .route("/profile", get(routes::profile_handler))
        .with_state(app_state)
        .layer(CookieLayer::default())
        .layer(axum::middleware::from_fn(inject_server_header));

    // Create main router with API routes nested under /api
    let router = Router::new().nest("/api", api_router);

    // Add static file serving if the directory exists
    let router = if static_path.exists() {
        tracing::info!(path = %static_dir, "Serving static files from directory");
        router.fallback_service(ServeDir::new(&static_path).not_found_service(ServeFile::new(&index_path)))
    } else {
        tracing::warn!(path = %static_dir, "Static files directory not found, serving API only");
        router
    };

    // Add tracing layer to the entire router
    router.layer(
        tower_http::trace::TraceLayer::new_for_http()
            .make_span_with(make_span)
            .on_request(|_request: &axum::http::Request<axum::body::Body>, _span: &tracing::Span| {
                // Disable request logging by doing nothing
            }),
    )
}

/// Inject the server header into responses
async fn inject_server_header(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    let mut res = next.run(req).await;
    res.headers_mut().insert(
        axum::http::header::SERVER,
        axum::http::HeaderValue::from_static(SERVER_HEADER_VALUE),
    );
    Ok(res)
}

// Constant value for the Server header: "<crate>/<version>"
const SERVER_HEADER_VALUE: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
