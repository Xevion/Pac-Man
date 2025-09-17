use axum::{response::IntoResponse, response::Redirect};
use dashmap::DashMap;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse};
use serde::{Deserialize, Serialize};

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{trace, warn};

use crate::auth::provider::{AuthUser, OAuthProvider};
use crate::errors::ErrorResponse;

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
    pkce: DashMap<String, PkceRecord>,
    last_purge_at_secs: AtomicU32,
    pkce_additions: AtomicU32,
}

#[derive(Debug, Clone)]
struct PkceRecord {
    verifier: String,
    created_at: Instant,
}

impl DiscordProvider {
    pub fn new(client: super::OAuthClient, http: reqwest::Client) -> Arc<Self> {
        Arc::new(Self {
            client,
            http,
            pkce: DashMap::new(),
            last_purge_at_secs: AtomicU32::new(0),
            pkce_additions: AtomicU32::new(0),
        })
    }

    fn maybe_purge_stale_pkce_entries(&self) {
        // Purge when at least 5 minutes passed or more than 128 additions occurred
        const PURGE_INTERVAL_SECS: u32 = 5 * 60;
        const ADDITIONS_THRESHOLD: u32 = 128;

        let now_secs = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_secs() as u32,
            Err(_) => return,
        };

        let last = self.last_purge_at_secs.load(Ordering::Relaxed);
        let additions = self.pkce_additions.load(Ordering::Relaxed);
        if additions < ADDITIONS_THRESHOLD && now_secs.saturating_sub(last) < PURGE_INTERVAL_SECS {
            return;
        }

        const PKCE_TTL: Duration = Duration::from_secs(5 * 60);
        let now_inst = Instant::now();
        for entry in self.pkce.iter() {
            if now_inst.duration_since(entry.value().created_at) > PKCE_TTL {
                self.pkce.remove(entry.key());
            }
        }

        // Reset counters after purge
        self.pkce_additions.store(0, Ordering::Relaxed);
        self.last_purge_at_secs.store(now_secs, Ordering::Relaxed);
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

    async fn authorize(&self) -> axum::response::Response {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_state) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .add_scope(Scope::new("identify".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .url();
        trace!(state = %csrf_state.secret(), "Generated OAuth authorization URL");

        // Insert PKCE verifier with timestamp and purge when needed
        self.pkce.insert(
            csrf_state.secret().to_string(),
            PkceRecord {
                verifier: pkce_verifier.secret().to_string(),
                created_at: Instant::now(),
            },
        );
        self.pkce_additions.fetch_add(1, Ordering::Relaxed);
        self.maybe_purge_stale_pkce_entries();

        Redirect::to(authorize_url.as_str()).into_response()
    }

    async fn handle_callback(&self, query: &std::collections::HashMap<String, String>) -> Result<AuthUser, ErrorResponse> {
        if let Some(err) = query.get("error") {
            warn!(error = %err, desc = query.get("error_description").map(|s| s.as_str()), "OAuth callback contained an error");
            return Err(ErrorResponse::bad_request(
                err.clone(),
                query.get("error_description").cloned(),
            ));
        }
        let code = query
            .get("code")
            .cloned()
            .ok_or_else(|| ErrorResponse::bad_request("invalid_request", Some("missing code".into())))?;
        let state = query
            .get("state")
            .cloned()
            .ok_or_else(|| ErrorResponse::bad_request("invalid_request", Some("missing state".into())))?;
        let Some(rec) = self.pkce.remove(&state).map(|e| e.1) else {
            warn!("Missing PKCE verifier for state parameter");
            return Err(ErrorResponse::bad_request(
                "invalid_request",
                Some("missing pkce verifier for state".into()),
            ));
        };

        // Verify PKCE TTL
        if Instant::now().duration_since(rec.created_at) > Duration::from_secs(5 * 60) {
            warn!("PKCE verifier expired for state parameter");
            return Err(ErrorResponse::bad_request(
                "invalid_request",
                Some("expired pkce verifier for state".into()),
            ));
        }

        let token = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(PkceCodeVerifier::new(rec.verifier))
            .request_async(&self.http)
            .await
            .map_err(|e| {
                warn!(error = %e, "Token exchange with Discord failed");
                ErrorResponse::bad_gateway("token_exchange_failed", Some(e.to_string()))
            })?;

        let user = fetch_discord_user(&self.http, token.access_token().secret())
            .await
            .map_err(|e| {
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
