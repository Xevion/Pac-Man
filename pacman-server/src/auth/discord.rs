use jsonwebtoken::EncodingKey;
use oauth2::Scope;
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use tracing::warn;

use crate::auth::provider::{self, AuthUser, AuthorizeInfo, OAuthProvider};
use crate::errors::ErrorResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub email: Option<String>,
    pub verified: Option<bool>,
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

    async fn authorize(&self, encoding_key: &EncodingKey) -> Result<AuthorizeInfo, ErrorResponse> {
        let scopes = [Scope::new("identify".into()), Scope::new("email".into())];
        Ok(provider::authorize_with_pkce(&self.client, &scopes, encoding_key))
    }

    async fn exchange_code_for_token(&self, code: &str, verifier: &str) -> Result<String, ErrorResponse> {
        provider::exchange_code_with_pkce(&self.client, &self.http, code, verifier, self.label()).await
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

        let (email, email_verified) = match (&user.email, user.verified) {
            (Some(e), Some(true)) => (Some(e.clone()), true),
            _ => (None, false),
        };

        Ok(AuthUser {
            id: user.id,
            username: user.username,
            name: user.global_name,
            email,
            email_verified,
            avatar_url,
        })
    }
}
