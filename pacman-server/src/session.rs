use std::time::{SystemTime, UNIX_EPOCH};

use axum_cookie::{cookie::Cookie, prelude::SameSite, CookieManager};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::auth::provider::AuthUser;

pub const SESSION_COOKIE_NAME: &str = "session";
pub const JWT_TTL_SECS: u64 = 60 * 60; // 1 hour

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,
    pub name: Option<String>,
    pub iat: usize,
    pub exp: usize,
}

pub fn create_jwt_for_user(user: &AuthUser, encoding_key: &EncodingKey) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs() as usize;
    let claims = Claims {
        sub: user.username.clone(),
        name: user.name.clone(),
        iat: now,
        exp: now + JWT_TTL_SECS as usize,
    };
    encode(&Header::new(Algorithm::HS256), &claims, encoding_key).expect("jwt sign")
}

pub fn verify_jwt(token: &str, decoding_key: &DecodingKey) -> bool {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 30;
    match decode::<Claims>(token, decoding_key, &validation) {
        Ok(_) => true,
        Err(e) => {
            println!("JWT verification failed: {:?}", e);
            false
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
