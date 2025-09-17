use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_cookie::CookieManager;
use serde::Serialize;
use tracing::{debug, info, instrument, trace, warn};

use crate::data::user as user_repo;
use crate::{app::AppState, errors::ErrorResponse, session};

#[derive(Debug, serde::Deserialize)]
pub struct AuthQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[instrument(skip_all, fields(provider = %provider))]
pub async fn oauth_authorize_handler(
    State(app_state): State<AppState>,
    Path(provider): Path<String>,
) -> axum::response::Response {
    let Some(prov) = app_state.auth.get(&provider) else {
        warn!(%provider, "Unknown OAuth provider");
        return ErrorResponse::bad_request("invalid_provider", Some(provider)).into_response();
    };
    trace!("Starting OAuth authorization");
    let resp = prov.authorize().await;
    trace!("Redirecting to provider authorization page");
    resp
}

#[instrument(skip_all, fields(provider = %provider))]
pub async fn oauth_callback_handler(
    State(app_state): State<AppState>,
    Path(provider): Path<String>,
    Query(params): Query<AuthQuery>,
    cookie: CookieManager,
) -> axum::response::Response {
    let Some(prov) = app_state.auth.get(&provider) else {
        warn!(%provider, "Unknown OAuth provider");
        return ErrorResponse::bad_request("invalid_provider", Some(provider)).into_response();
    };
    if let Some(error) = params.error {
        warn!(%provider, error = %error, desc = ?params.error_description, "OAuth callback returned an error");
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
        Err(e) => {
            warn!(%provider, "OAuth callback handling failed");
            return e.into_response();
        }
    };
    // Persist or update in database
    match user_repo::upsert_user(
        &app_state.db,
        &provider,
        &user.id,
        &user.username,
        user.name.as_deref(),
        user.email.as_deref(),
        user.avatar_url.as_deref(),
    )
    .await
    {
        Ok(_db_user) => {}
        Err(e) => {
            warn!(error = %e, provider = %provider, "Failed to upsert user in database");
        }
    }
    let session_token = session::create_jwt_for_user(&provider, &user, &app_state.jwt_encoding_key);
    session::set_session_cookie(&cookie, &session_token);
    info!(%provider, "Signed in successfully");
    (StatusCode::FOUND, Redirect::to("/profile")).into_response()
}

#[instrument(skip_all)]
pub async fn profile_handler(State(app_state): State<AppState>, cookie: CookieManager) -> axum::response::Response {
    let Some(token_str) = session::get_session_token(&cookie) else {
        debug!("Missing session cookie");
        return ErrorResponse::unauthorized("missing session cookie").into_response();
    };
    let Some(claims) = session::decode_jwt(&token_str, &app_state.jwt_decoding_key) else {
        debug!("Invalid session token");
        return ErrorResponse::unauthorized("invalid session token").into_response();
    };
    // sub format: provider:provider_user_id
    let (prov, prov_user_id) = match claims.sub.split_once(':') {
        Some((p, id)) => (p, id),
        None => {
            debug!("Malformed session token subject");
            return ErrorResponse::unauthorized("invalid session token").into_response();
        }
    };
    match user_repo::get_user_by_provider_id(&app_state.db, prov, prov_user_id).await {
        Ok(Some(db_user)) => axum::Json(db_user).into_response(),
        Ok(None) => {
            debug!("User not found for session");
            ErrorResponse::unauthorized("session not found").into_response()
        }
        Err(e) => {
            warn!(error = %e, "Failed to fetch user for session");
            ErrorResponse::with_status(
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                Some("could not fetch user".into()),
            )
            .into_response()
        }
    }
}

pub async fn logout_handler(State(app_state): State<AppState>, cookie: CookieManager) -> axum::response::Response {
    if let Some(token_str) = session::get_session_token(&cookie) {
        // Remove from in-memory sessions if present
        app_state.sessions.remove(&token_str);
    }
    session::clear_session_cookie(&cookie);
    info!("Signed out successfully");
    (StatusCode::FOUND, Redirect::to("/")).into_response()
}

#[derive(Serialize)]
struct ProviderInfo {
    id: &'static str,
    name: &'static str,
    active: bool,
}

pub async fn list_providers_handler(State(app_state): State<AppState>) -> axum::response::Response {
    let providers: Vec<ProviderInfo> = app_state
        .auth
        .values()
        .map(|provider| ProviderInfo {
            id: provider.id(),
            name: provider.label(),
            active: provider.active(),
        })
        .collect();
    axum::Json(providers).into_response()
}
