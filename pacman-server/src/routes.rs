use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_cookie::CookieManager;

use crate::{app::AppState, errors::ErrorResponse, session};

#[derive(Debug, serde::Deserialize)]
pub struct AuthQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

pub async fn oauth_authorize_handler(
    State(app_state): State<AppState>,
    Path(provider): Path<String>,
) -> axum::response::Response {
    let Some(prov) = app_state.auth.get(&provider) else {
        return ErrorResponse::bad_request("invalid_provider", Some(provider)).into_response();
    };
    prov.authorize().await
}

pub async fn oauth_callback_handler(
    State(app_state): State<AppState>,
    Path(provider): Path<String>,
    Query(params): Query<AuthQuery>,
    cookie: CookieManager,
) -> axum::response::Response {
    let Some(prov) = app_state.auth.get(&provider) else {
        return ErrorResponse::bad_request("invalid_provider", Some(provider)).into_response();
    };
    if let Some(error) = params.error {
        return ErrorResponse::bad_request(error, params.error_description).into_response();
    }
    let mut q = std::collections::HashMap::new();
    if let Some(v) = params.code {
        q.insert("code".to_string(), v);
    }
    if let Some(v) = params.state {
        q.insert("state".to_string(), v);
    }
    let user = match prov.handle_callback(&q).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };
    let session_token = session::create_jwt_for_user(&user, &app_state.jwt_encoding_key);
    app_state.sessions.insert(session_token.clone(), user);
    session::set_session_cookie(&cookie, &session_token);
    (StatusCode::FOUND, Redirect::to("/profile")).into_response()
}

pub async fn profile_handler(State(app_state): State<AppState>, cookie: CookieManager) -> axum::response::Response {
    let Some(token_str) = session::get_session_token(&cookie) else {
        return ErrorResponse::unauthorized("missing session cookie").into_response();
    };
    if !session::verify_jwt(&token_str, &app_state.jwt_decoding_key) {
        return ErrorResponse::unauthorized("invalid session token").into_response();
    }
    if let Some(user) = app_state.sessions.get(&token_str) {
        return axum::Json(&*user).into_response();
    }
    ErrorResponse::unauthorized("session not found").into_response()
}

pub async fn logout_handler(State(app_state): State<AppState>, cookie: CookieManager) -> axum::response::Response {
    if let Some(token_str) = session::get_session_token(&cookie) {
        // Remove from in-memory sessions if present
        app_state.sessions.remove(&token_str);
    }
    session::clear_session_cookie(&cookie);
    (StatusCode::FOUND, Redirect::to("/")).into_response()
}
