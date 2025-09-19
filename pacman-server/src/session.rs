use std::time::{SystemTime, UNIX_EPOCH};

use axum_cookie::{cookie::Cookie, prelude::SameSite, CookieManager};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::auth::provider::AuthUser;
use tracing::{trace, warn};

pub const SESSION_COOKIE_NAME: &str = "session";
pub const JWT_TTL_SECS: u64 = 60 * 60; // 1 hour

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String, // format: "{provider}:{provider_user_id}"
    pub name: Option<String>,
    pub iat: usize,
    pub exp: usize,
    // PKCE flow fields - only present during OAuth flow
    #[serde(rename = "ver", skip_serializing_if = "Option::is_none")]
    pub pkce_verifier: Option<String>,
    #[serde(rename = "st", skip_serializing_if = "Option::is_none")]
    pub csrf_state: Option<String>,
}

pub fn create_jwt_for_user(provider: &str, user: &AuthUser, encoding_key: &EncodingKey) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs() as usize;
    let claims = Claims {
        sub: format!("{}:{}", provider, user.id),
        name: user.name.clone(),
        iat: now,
        exp: now + JWT_TTL_SECS as usize,
        pkce_verifier: None,
        csrf_state: None,
    };
    let token = encode(&Header::new(Algorithm::HS256), &claims, encoding_key).expect("jwt sign");
    trace!(sub = %claims.sub, exp = claims.exp, "Created session JWT");
    token
}

/// Creates a temporary session for PKCE flow with verifier and CSRF state
pub fn create_pkce_session(pkce_verifier: &str, csrf_state: &str, encoding_key: &EncodingKey) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs() as usize;
    let claims = Claims {
        sub: "pkce_flow".to_string(), // Special marker for PKCE flow
        name: None,
        iat: now,
        exp: now + JWT_TTL_SECS as usize,
        pkce_verifier: Some(pkce_verifier.to_string()),
        csrf_state: Some(csrf_state.to_string()),
    };
    let token = encode(&Header::new(Algorithm::HS256), &claims, encoding_key).expect("jwt sign");
    trace!(csrf_state = %csrf_state, "Created PKCE session JWT");
    token
}

/// Checks if a session is a PKCE flow session
pub fn is_pkce_session(claims: &Claims) -> bool {
    claims.sub == "pkce_flow" && claims.pkce_verifier.is_some() && claims.csrf_state.is_some()
}

pub fn decode_jwt(token: &str, decoding_key: &DecodingKey) -> Option<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 30;
    match decode::<Claims>(token, decoding_key, &validation) {
        Ok(data) => Some(data.claims),
        Err(e) => {
            warn!(error = %e, "Session JWT verification failed");
            None
        }
    }
}

pub fn set_session_cookie(cookie: &CookieManager, token: &str) {
    cookie.add(
        Cookie::builder(SESSION_COOKIE_NAME, token.to_string())
            .http_only(true)
            .secure(!cfg!(debug_assertions))
            .path("/")
            .same_site(SameSite::Lax)
            .build(),
    );
}

pub fn clear_session_cookie(cookie: &CookieManager) {
    cookie.remove(SESSION_COOKIE_NAME);
}

pub fn get_session_token(cookie: &CookieManager) -> Option<String> {
    cookie.get(SESSION_COOKIE_NAME).map(|c| c.value().to_string())
}
