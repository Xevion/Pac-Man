use async_trait::async_trait;
use axum_cookie::CookieManager;
use jsonwebtoken::{DecodingKey, EncodingKey};
use mockall::automock;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse};
use serde::Serialize;
use tracing::{trace, warn};

use crate::errors::ErrorResponse;
use crate::session;

use super::OAuthClient;

// A user object returned from the provider after authentication.
#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    // A unique identifier for the user, from the provider.
    pub id: String,
    // A username from the provider. Generally unique, a handle for the user.
    pub username: String,

    // A display name for the user. Not always available.
    pub name: Option<String>,
    // An email address for the user. Not always available.
    pub email: Option<String>,
    // Whether the email address has been verified by the provider.
    pub email_verified: bool,
    // An avatar URL for the user. Not always available.
    pub avatar_url: Option<String>,
}

// Information required to begin an OAuth authorization flow.
#[derive(Debug)]
pub struct AuthorizeInfo {
    // The URL to redirect the user to for authorization.
    pub authorize_url: oauth2::url::Url,
    // A session token to be stored in the user's session cookie.
    pub session_token: String,
}

#[automock]
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    // Builds the necessary information to redirect the user to the provider's authorization page.
    // This generally also includes beginning a PKCE flow (proof key for code exchange).
    // The returned session token should be stored in the user's session cookie.
    async fn authorize(&self, encoding_key: &EncodingKey) -> Result<AuthorizeInfo, ErrorResponse>;

    // Handles the callback from the provider after the user has authorized the app.
    // This generally also includes completing the PKCE flow (proof key for code exchange).
    // The cookie manager is used to retrieve the PKCE verifier from the session.
    async fn handle_callback(
        &self,
        code: &str,
        state: &str,
        cookie: &CookieManager,
        decoding_key: &DecodingKey,
    ) -> Result<AuthUser, ErrorResponse> {
        // Common PKCE session validation and token exchange logic
        let verifier = self.validate_pkce_session(cookie, state, decoding_key).await?;
        let access_token = self.exchange_code_for_token(code, &verifier).await?;
        let user = self.fetch_user_from_token(&access_token).await?;
        Ok(user)
    }

    // Validates the PKCE session and returns the verifier
    async fn validate_pkce_session(
        &self,
        cookie: &CookieManager,
        state: &str,
        decoding_key: &DecodingKey,
    ) -> Result<String, ErrorResponse> {
        // Get the session token and verify it's a PKCE session
        let Some(session_token) = session::get_session_token(cookie) else {
            warn!(%state, "Missing session cookie during OAuth callback");
            return Err(ErrorResponse::bad_request(
                "invalid_request",
                Some("missing session cookie".into()),
            ));
        };

        let Some(claims) = session::decode_jwt(&session_token, decoding_key) else {
            warn!(%state, "Invalid session token during OAuth callback");
            return Err(ErrorResponse::bad_request(
                "invalid_request",
                Some("invalid session token".into()),
            ));
        };

        // Verify this is a PKCE session and the state matches
        if !session::is_pkce_session(&claims) {
            warn!(%state, "Session is not a PKCE session");
            return Err(ErrorResponse::bad_request(
                "invalid_request",
                Some("invalid session type".into()),
            ));
        }

        if claims.csrf_state.as_deref() != Some(state) {
            warn!(%state, "CSRF state mismatch during OAuth callback");
            return Err(ErrorResponse::bad_request(
                "invalid_request",
                Some("state parameter mismatch".into()),
            ));
        }

        let Some(verifier) = claims.pkce_verifier else {
            warn!(%state, "Missing PKCE verifier in session");
            return Err(ErrorResponse::bad_request(
                "invalid_request",
                Some("missing pkce verifier".into()),
            ));
        };

        Ok(verifier)
    }

    // Exchanges the authorization code for an access token using PKCE
    async fn exchange_code_for_token(&self, code: &str, verifier: &str) -> Result<String, ErrorResponse>;

    // Fetches user information from the provider using the access token
    async fn fetch_user_from_token(&self, access_token: &str) -> Result<AuthUser, ErrorResponse>;

    // The provider's unique identifier (e.g. "discord")
    fn id(&self) -> &'static str;

    // The provider's display name (e.g. "Discord")
    fn label(&self) -> &'static str;

    // Whether the provider is active (defaults to true for now)
    fn active(&self) -> bool {
        true
    }
}

/// Builds an OAuth authorization URL with PKCE and stores the verifier in a session token.
///
/// Shared by all providers -- only the scopes differ.
pub fn authorize_with_pkce(client: &OAuthClient, scopes: &[Scope], encoding_key: &EncodingKey) -> AuthorizeInfo {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let mut request = client.authorize_url(CsrfToken::new_random).set_pkce_challenge(pkce_challenge);
    for scope in scopes {
        request = request.add_scope(scope.clone());
    }
    let (authorize_url, csrf_state) = request.url();
    let session_token = session::create_pkce_session(pkce_verifier.secret(), csrf_state.secret(), encoding_key);
    trace!(state = %csrf_state.secret(), "Generated OAuth authorization URL");
    AuthorizeInfo {
        authorize_url,
        session_token,
    }
}

/// Exchanges an authorization code for an access token using PKCE.
///
/// Shared by all providers -- the only difference is the error label.
pub async fn exchange_code_with_pkce(
    client: &OAuthClient,
    http: &reqwest::Client,
    code: &str,
    verifier: &str,
    provider_label: &str,
) -> Result<String, ErrorResponse> {
    let token = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(PkceCodeVerifier::new(verifier.to_string()))
        .request_async(http)
        .await
        .map_err(|e| {
            warn!(error = %e, "Token exchange with {} failed", provider_label);
            ErrorResponse::bad_gateway("token_exchange_failed", Some(e.to_string()))
        })?;
    Ok(token.access_token().secret().to_string())
}
