use axum::{routing::get, Router};
use axum_cookie::CookieLayer;
use dashmap::DashMap;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::data::pool::PgPool;
use crate::{auth::AuthRegistry, config::Config, image::ImageStorage, routes};

#[derive(Debug, Clone)]
pub struct Health {
    migrations: bool,
    database: bool,
}

impl Health {
    pub fn new() -> Self {
        Self {
            migrations: false,
            database: false,
        }
    }

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
    pub db: Arc<PgPool>,
    pub health: Arc<RwLock<Health>>,
    pub image_storage: Arc<ImageStorage>,
}

impl AppState {
    pub fn new(config: Config, auth: AuthRegistry, db: PgPool) -> Self {
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

        Self {
            auth: Arc::new(auth),
            sessions: Arc::new(DashMap::new()),
            jwt_encoding_key: Arc::new(EncodingKey::from_secret(jwt_secret.as_bytes())),
            jwt_decoding_key: Arc::new(DecodingKey::from_secret(jwt_secret.as_bytes())),
            db: Arc::new(db),
            health: Arc::new(RwLock::new(Health::new())),
            image_storage,
        }
    }
}

/// Create the application router with all routes and middleware
pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World! Visit /auth/github to start OAuth flow." }))
        .route("/health", get(routes::health_handler))
        .route("/auth/providers", get(routes::list_providers_handler))
        .route("/auth/{provider}", get(routes::oauth_authorize_handler))
        .route("/auth/{provider}/callback", get(routes::oauth_callback_handler))
        .route("/logout", get(routes::logout_handler))
        .route("/profile", get(routes::profile_handler))
        .with_state(app_state)
        .layer(CookieLayer::default())
        .layer(axum::middleware::from_fn(inject_server_header))
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
