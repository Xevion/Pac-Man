use jsonwebtoken::EncodingKey;
use oauth2::Scope;
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use tracing::warn;

use crate::{
    auth::provider::{self, AuthUser, AuthorizeInfo, OAuthProvider},
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
        let scopes = [Scope::new("user:email".into()), Scope::new("read:user".into())];
        Ok(provider::authorize_with_pkce(&self.client, &scopes, encoding_key))
    }

    async fn exchange_code_for_token(&self, code: &str, verifier: &str) -> Result<String, ErrorResponse> {
        provider::exchange_code_with_pkce(&self.client, &self.http, code, verifier, self.label()).await
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
