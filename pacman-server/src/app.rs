use dashmap::DashMap;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::sync::Arc;

use crate::data::pool::PgPool;
use crate::{auth::AuthRegistry, config::Config};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub auth: Arc<AuthRegistry>,
    pub sessions: Arc<DashMap<String, crate::auth::provider::AuthUser>>,
    pub jwt_encoding_key: Arc<EncodingKey>,
    pub jwt_decoding_key: Arc<DecodingKey>,
    pub db: Arc<PgPool>,
}

impl AppState {
    pub fn new(config: Config, auth: AuthRegistry, db: PgPool) -> Self {
        let jwt_secret = config.jwt_secret.clone();

        Self {
            config: Arc::new(config),
            auth: Arc::new(auth),
            sessions: Arc::new(DashMap::new()),
            jwt_encoding_key: Arc::new(EncodingKey::from_secret(jwt_secret.as_bytes())),
            jwt_decoding_key: Arc::new(DecodingKey::from_secret(jwt_secret.as_bytes())),
            db: Arc::new(db),
        }
    }
}
