use axum::{response::IntoResponse, response::Redirect};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeVerifier, Scope, TokenResponse};
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use tracing::{trace, warn};

use crate::{
    auth::{
        pkce::PkceManager,
        provider::{AuthUser, OAuthProvider},
    },
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
    pub client: super::OAuthClient,
    pub http: reqwest::Client,
    pkce: PkceManager,
}

impl GitHubProvider {
    pub fn new(client: super::OAuthClient, http: reqwest::Client) -> Arc<Self> {
        Arc::new(Self {
            client,
            http,
            pkce: PkceManager::new(),
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
        let (pkce_challenge, verifier) = self.pkce.generate_challenge();
        let (authorize_url, csrf_state) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .add_scope(Scope::new("user:email".to_string()))
            .add_scope(Scope::new("read:user".to_string()))
            .url();
        // store verifier keyed by the returned state
        self.pkce.store_verifier(csrf_state.secret(), verifier);
        trace!(state = %csrf_state.secret(), "Generated OAuth authorization URL");
        Redirect::to(authorize_url.as_str()).into_response()
    }

    async fn handle_callback(&self, code: &str, state: &str) -> Result<AuthUser, ErrorResponse> {
        let Some(verifier) = self.pkce.take_verifier(state) else {
            warn!(%state, "Missing or expired PKCE verifier for state parameter");
            return Err(ErrorResponse::bad_request(
                "invalid_request",
                Some("missing or expired pkce verifier for state".into()),
            ));
        };

        let token = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier))
            .request_async(&self.http)
            .await
            .map_err(|e| {
                warn!(error = %e, %state, "Token exchange with GitHub failed");
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

impl GitHubProvider {}

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
