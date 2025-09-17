use axum::{response::IntoResponse, response::Redirect};
use dashmap::DashMap;
use oauth2::{basic::BasicClient, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse};
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{trace, warn};

use crate::{
    auth::provider::{AuthUser, OAuthProvider},
    errors::ErrorResponse,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
    pub visibility: Option<String>,
}

/// Fetch user information from GitHub API
pub async fn fetch_github_user(
    http_client: &reqwest::Client,
    access_token: &str,
) -> Result<GitHubUser, Box<dyn std::error::Error + Send + Sync>> {
    let response = http_client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", crate::config::USER_AGENT)
        .send()
        .await?;

    if !response.status().is_success() {
        warn!(status = %response.status(), endpoint = "/user", "GitHub API returned an error");
        return Err(format!("GitHub API error: {}", response.status()).into());
    }

    let user: GitHubUser = response.json().await?;
    Ok(user)
}

pub struct GitHubProvider {
    pub client: BasicClient<
        oauth2::EndpointSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointSet,
    >,
    pub http: reqwest::Client,
    pkce: DashMap<String, (String, Instant)>,
}

impl GitHubProvider {
    pub fn new(
        client: BasicClient<
            oauth2::EndpointSet,
            oauth2::EndpointNotSet,
            oauth2::EndpointNotSet,
            oauth2::EndpointNotSet,
            oauth2::EndpointSet,
        >,
        http: reqwest::Client,
    ) -> Arc<Self> {
        Arc::new(Self {
            client,
            http,
            pkce: DashMap::new(),
        })
    }
}

#[async_trait::async_trait]
impl OAuthProvider for GitHubProvider {
    fn id(&self) -> &'static str {
        "github"
    }
    fn label(&self) -> &'static str {
        "GitHub"
    }

    async fn authorize(&self) -> axum::response::Response {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_state) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .add_scope(Scope::new("user:email".to_string()))
            .add_scope(Scope::new("read:user".to_string()))
            .url();
        // Insert PKCE verifier with timestamp and purge stale entries
        let now = Instant::now();
        self.pkce
            .insert(csrf_state.secret().to_string(), (pkce_verifier.secret().to_string(), now));
        // Best-effort cleanup to avoid unbounded growth
        const PKCE_TTL: Duration = Duration::from_secs(5 * 60);
        for entry in self.pkce.iter() {
            if now.duration_since(entry.value().1) > PKCE_TTL {
                self.pkce.remove(entry.key());
            }
        }
        trace!(state = %csrf_state.secret(), "Generated OAuth authorization URL");
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
        let Some((verifier, created_at)) = self.pkce.remove(&state).map(|e| e.1) else {
            warn!("Missing PKCE verifier for state parameter");
            return Err(ErrorResponse::bad_request(
                "invalid_request",
                Some("missing pkce verifier for state".into()),
            ));
        };
        // Verify PKCE TTL
        if Instant::now().duration_since(created_at) > Duration::from_secs(5 * 60) {
            warn!("PKCE verifier expired for state parameter");
            return Err(ErrorResponse::bad_request(
                "invalid_request",
                Some("expired pkce verifier for state".into()),
            ));
        }

        let token = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier))
            .request_async(&self.http)
            .await
            .map_err(|e| {
                warn!(error = %e, "Token exchange with GitHub failed");
                ErrorResponse::bad_gateway("token_exchange_failed", Some(e.to_string()))
            })?;

        let user = fetch_github_user(&self.http, token.access_token().secret())
            .await
            .map_err(|e| {
                warn!(error = %e, "Failed to fetch GitHub user profile");
                ErrorResponse::bad_gateway("github_api_error", Some(format!("failed to fetch user: {}", e)))
            })?;
        let _emails = fetch_github_emails(&self.http, token.access_token().secret())
            .await
            .map_err(|e| {
                warn!(error = %e, "Failed to fetch GitHub user emails");
                ErrorResponse::bad_gateway("github_api_error", Some(format!("failed to fetch emails: {}", e)))
            })?;

        Ok(AuthUser {
            id: user.id.to_string(),
            username: user.login,
            name: user.name,
            email: user.email,
            avatar_url: Some(user.avatar_url),
        })
    }
}

/// Fetch user emails from GitHub API
pub async fn fetch_github_emails(
    http_client: &reqwest::Client,
    access_token: &str,
) -> Result<Vec<GitHubEmail>, Box<dyn std::error::Error + Send + Sync>> {
    let response = http_client
        .get("https://api.github.com/user/emails")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", crate::config::USER_AGENT)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()).into());
    }

    let emails: Vec<GitHubEmail> = response.json().await?;
    Ok(emails)
}
