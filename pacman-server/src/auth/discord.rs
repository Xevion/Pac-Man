use axum::{response::IntoResponse, response::Redirect};
use axum_cookie::CookieManager;
use jsonwebtoken::EncodingKey;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeVerifier, Scope, TokenResponse};
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use tracing::{trace, warn};

use crate::auth::provider::{AuthUser, OAuthProvider};
use crate::errors::ErrorResponse;
use crate::session;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
}

pub async fn fetch_discord_user(
    http_client: &reqwest::Client,
    access_token: &str,
) -> Result<DiscordUser, Box<dyn std::error::Error + Send + Sync>> {
    let response = http_client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", crate::config::USER_AGENT)
        .send()
        .await?;

    if !response.status().is_success() {
        warn!(status = %response.status(), endpoint = "/users/@me", "Discord API returned an error");
        return Err(format!("Discord API error: {}", response.status()).into());
    }

    let user: DiscordUser = response.json().await?;
    Ok(user)
}

pub struct DiscordProvider {
    pub client: super::OAuthClient,
    pub http: reqwest::Client,
}

impl DiscordProvider {
    pub fn new(client: super::OAuthClient, http: reqwest::Client) -> Arc<Self> {
        Arc::new(Self { client, http })
    }

    fn avatar_url_for(user_id: &str, avatar_hash: &str) -> String {
        let ext = if avatar_hash.starts_with("a_") { "gif" } else { "png" };
        format!("https://cdn.discordapp.com/avatars/{}/{}.{}", user_id, avatar_hash, ext)
    }
}

#[async_trait::async_trait]
impl OAuthProvider for DiscordProvider {
    fn id(&self) -> &'static str {
        "discord"
    }
    fn label(&self) -> &'static str {
        "Discord"
    }

    async fn authorize(&self, cookie: &CookieManager, encoding_key: &EncodingKey) -> axum::response::Response {
        let (pkce_challenge, pkce_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_state) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .add_scope(Scope::new("identify".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .url();

        // Store PKCE verifier and CSRF state in session
        let session_token = session::create_pkce_session(pkce_verifier.secret(), csrf_state.secret(), encoding_key);
        session::set_session_cookie(cookie, &session_token);

        trace!(state = %csrf_state.secret(), "Generated OAuth authorization URL");
        Redirect::to(authorize_url.as_str()).into_response()
    }

    async fn exchange_code_for_token(&self, code: &str, verifier: &str) -> Result<String, ErrorResponse> {
        let token = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier.to_string()))
            .request_async(&self.http)
            .await
            .map_err(|e| {
                warn!(error = %e, "Token exchange with Discord failed");
                ErrorResponse::bad_gateway("token_exchange_failed", Some(e.to_string()))
            })?;

        Ok(token.access_token().secret().to_string())
    }

    async fn fetch_user_from_token(&self, access_token: &str) -> Result<AuthUser, ErrorResponse> {
        let user = fetch_discord_user(&self.http, access_token).await.map_err(|e| {
            warn!(error = %e, "Failed to fetch Discord user profile");
            ErrorResponse::bad_gateway("discord_api_error", Some(format!("failed to fetch user: {}", e)))
        })?;

        let avatar_url = match (&user.id, &user.avatar) {
            (id, Some(hash)) => Some(Self::avatar_url_for(id, hash)),
            _ => None,
        };

        Ok(AuthUser {
            id: user.id,
            username: user.username,
            name: user.global_name,
            email: user.email,
            avatar_url,
        })
    }
}
