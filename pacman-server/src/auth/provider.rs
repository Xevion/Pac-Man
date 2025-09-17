use std::collections::HashMap;

use async_trait::async_trait;
use serde::Serialize;

use crate::errors::ErrorResponse;

#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

#[async_trait]
pub trait OAuthProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn active(&self) -> bool {
        true
    }

    async fn authorize(&self) -> axum::response::Response;

    async fn handle_callback(&self, query: &HashMap<String, String>) -> Result<AuthUser, ErrorResponse>;
}
