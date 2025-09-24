use jsonwebtoken::EncodingKey;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeVerifier, Scope, TokenResponse};
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use tracing::{trace, warn};

use crate::{
    auth::provider::{AuthUser, AuthorizeInfo, OAuthProvider},
    errors::ErrorResponse,
    session,
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
        warn!(status = %response.status(), endpoint = "/user/emails", "GitHub API returned an error");
        return Err(format!("GitHub API error: {}", response.status()).into());
    }

    let emails: Vec<GitHubEmail> = response.json().await?;
    Ok(emails)
}

pub struct GitHubProvider {
    pub client: super::OAuthClient,
    pub http: reqwest::Client,
}

impl GitHubProvider {
    pub fn new(client: super::OAuthClient, http: reqwest::Client) -> Arc<Self> {
        Arc::new(Self { client, http })
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

    async fn authorize(&self, encoding_key: &EncodingKey) -> Result<AuthorizeInfo, ErrorResponse> {
        let (pkce_challenge, pkce_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_state) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .add_scope(Scope::new("user:email".to_string()))
            .add_scope(Scope::new("read:user".to_string()))
            .url();

        // Store PKCE verifier and CSRF state in session
        let session_token = session::create_pkce_session(pkce_verifier.secret(), csrf_state.secret(), encoding_key);

        trace!(state = %csrf_state.secret(), "Generated OAuth authorization URL");
        Ok(AuthorizeInfo {
            authorize_url,
            session_token,
        })
    }

    async fn exchange_code_for_token(&self, code: &str, verifier: &str) -> Result<String, ErrorResponse> {
        let token = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier.to_string()))
            .request_async(&self.http)
            .await
            .map_err(|e| {
                warn!(error = %e, "Token exchange with GitHub failed");
                ErrorResponse::bad_gateway("token_exchange_failed", Some(e.to_string()))
            })?;

        Ok(token.access_token().secret().to_string())
    }

    async fn fetch_user_from_token(&self, access_token: &str) -> Result<AuthUser, ErrorResponse> {
        let user = fetch_github_user(&self.http, access_token).await.map_err(|e| {
            warn!(error = %e, "Failed to fetch GitHub user profile");
            ErrorResponse::bad_gateway("github_api_error", Some(format!("failed to fetch user: {}", e)))
        })?;

        let emails = fetch_github_emails(&self.http, access_token).await.map_err(|e| {
            warn!(error = %e, "Failed to fetch GitHub user emails");
            ErrorResponse::bad_gateway("github_api_error", Some(format!("failed to fetch emails: {}", e)))
        })?;

        let primary_email = emails.into_iter().find(|e| e.primary && e.verified);

        let (email, email_verified) = match primary_email {
            Some(e) => (Some(e.email), true),
            None => (user.email, false),
        };

        Ok(AuthUser {
            id: user.id.to_string(),
            username: user.login,
            name: user.name,
            email,
            email_verified,
            avatar_url: Some(user.avatar_url),
        })
    }
}
