use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_cookie::CookieManager;
use serde::Serialize;
use tracing::{debug, debug_span, info, instrument, trace, warn, Instrument};

use crate::data::user as user_repo;
use crate::image::ImageStorage;
use crate::{app::AppState, errors::ErrorResponse, session};

#[derive(Debug, serde::Deserialize)]
pub struct OAuthCallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Serialize)]
struct ProfilePayload {
    id: i64,
    email: Option<String>,
    providers: Vec<user_repo::ProviderPublic>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Handles the beginning of the OAuth authorization flow.
///
/// Requires the `provider` path parameter, which determines the OAuth provider to use.
#[instrument(skip_all, fields(provider = %provider))]
pub async fn oauth_authorize_handler(
    State(app_state): State<AppState>,
    Path(provider): Path<String>,
    cookie: CookieManager,
) -> axum::response::Response {
    let Some(prov) = app_state.auth.get(&provider) else {
        warn!(%provider, "Unknown OAuth provider");
        return ErrorResponse::bad_request("invalid_provider", Some(provider)).into_response();
    };
    trace!("Starting OAuth authorization");

    let auth_info = match prov.authorize(&app_state.jwt_encoding_key).await {
        Ok(info) => info,
        Err(e) => return e.into_response(),
    };

    session::set_session_cookie(&cookie, &auth_info.session_token);
    trace!("Redirecting to provider authorization page");
    Redirect::to(auth_info.authorize_url.as_str()).into_response()
}

/// Handles the callback from the OAuth provider after the user has authorized the app.
///
/// Requires the `provider` path parameter, which determines the OAuth provider to use for finishing the OAuth flow.
/// Requires the `code` and `state` query parameters, which are returned by the OAuth provider after the user has authorized the app.
pub async fn oauth_callback_handler(
    State(app_state): State<AppState>,
    Path(provider): Path<String>,
    Query(params): Query<OAuthCallbackParams>,
    cookie: CookieManager,
) -> axum::response::Response {
    // Check if database is configured - required for OAuth callback to work
    if !app_state.database_configured {
        warn!("OAuth callback attempted but database is not configured");
        return ErrorResponse::with_status(
            StatusCode::SERVICE_UNAVAILABLE,
            "database_not_configured",
            Some("Database is not configured. User authentication requires a database.".into()),
        )
        .into_response();
    }

    // Validate provider
    let Some(prov) = app_state.auth.get(&provider) else {
        warn!(%provider, "Unknown OAuth provider");
        return ErrorResponse::bad_request("invalid_provider", Some(provider)).into_response();
    };

    // Process callback-returned errors from provider
    if let Some(error) = params.error {
        warn!(%provider, error = %error, desc = ?params.error_description, "OAuth callback returned an error");
        return ErrorResponse::bad_request(error, params.error_description).into_response();
    }

    // Acquire required parameters
    let Some(code) = params.code.as_deref() else {
        return ErrorResponse::bad_request("invalid_request", Some("missing code".into())).into_response();
    };
    let Some(state) = params.state.as_deref() else {
        return ErrorResponse::bad_request("invalid_request", Some("missing state".into())).into_response();
    };

    debug_span!("oauth_callback_handler",  provider = %provider, code = %code, state = %state);

    // Handle callback from provider
    let user = match prov.handle_callback(code, state, &cookie, &app_state.jwt_decoding_key).await {
        Ok(u) => u,
        Err(e) => {
            warn!(%provider, "OAuth callback handling failed");
            return e.into_response();
        }
    };

    let linking_span = debug_span!("account_linking", provider_user_id = %user.id, provider_email = ?user.email, email_verified = %user.email_verified);
    let db_user_result = user_repo::find_or_create_user_for_oauth(&app_state.db, &provider, &user)
        .instrument(linking_span)
        .await;

    if let Err(e) = &db_user_result {
        warn!(error = %(e as &dyn std::error::Error), "Failed to process user linking/creation");
        return ErrorResponse::with_status(StatusCode::INTERNAL_SERVER_ERROR, "database_error", None).into_response();
    }

    // Create session token
    let session_token = session::create_jwt_for_user(&provider, &user, &app_state.jwt_encoding_key);
    session::set_session_cookie(&cookie, &session_token);
    info!(%provider, "Signed in successfully");

    spawn_avatar_processing(app_state.image_storage.as_ref(), &user.id, user.avatar_url.as_deref());

    (StatusCode::FOUND, Redirect::to("/api/profile")).into_response()
}

/// Handles the request to the profile endpoint.
///
/// Requires the `session` cookie to be present.
pub async fn profile_handler(State(app_state): State<AppState>, cookie: CookieManager) -> axum::response::Response {
    // Check if database is configured
    if !app_state.database_configured {
        return ErrorResponse::with_status(
            StatusCode::SERVICE_UNAVAILABLE,
            "database_not_configured",
            Some("Database is not configured. Profile lookup requires a database.".into()),
        )
        .into_response();
    }

    let Some(token_str) = session::get_session_token(&cookie) else {
        debug!("Missing session cookie");
        return ErrorResponse::unauthorized("missing session cookie").into_response();
    };
    let Some(claims) = session::decode_jwt(&token_str, &app_state.jwt_decoding_key) else {
        debug!("Invalid session token");
        return ErrorResponse::unauthorized("invalid session token").into_response();
    };
    // sub format: provider:provider_user_id
    let (prov, prov_user_id) = match claims.subject.split_once(':') {
        Some((p, id)) => (p, id),
        None => {
            debug!("Malformed session token subject");
            return ErrorResponse::unauthorized("invalid session token").into_response();
        }
    };
    match user_repo::find_user_by_provider_id(&app_state.db, prov, prov_user_id).await {
        Ok(Some(db_user)) => match user_repo::list_user_providers(&app_state.db, db_user.id).await {
            Ok(providers) => {
                let body = ProfilePayload {
                    id: db_user.id,
                    email: db_user.email.clone(),
                    providers,
                    created_at: db_user.created_at,
                    updated_at: db_user.updated_at,
                };
                axum::Json(body).into_response()
            }
            Err(e) => {
                warn!(error = %e, "Failed to list user providers");
                ErrorResponse::with_status(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    Some("could not fetch providers".into()),
                )
                .into_response()
            }
        },
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

pub async fn health_handler(
    State(app_state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> axum::response::Response {
    // Force health check in debug mode
    #[cfg(debug_assertions)]
    if params.contains_key("force") {
        app_state.check_health().await;
    }

    let health = app_state.health.read().await;
    let ok = health.ok();
    let status = if ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    // Include more details in the health response
    let body = serde_json::json!({
        "ok": ok,
        "database_configured": app_state.database_configured,
        "auth_providers": app_state.auth.len(),
        "image_storage_enabled": app_state.image_storage.is_some(),
    });

    (status, axum::Json(body)).into_response()
}

fn spawn_avatar_processing(image_storage: Option<&Arc<ImageStorage>>, user_id: &str, avatar_url: Option<&str>) {
    let Some(image_storage) = image_storage else { return };
    let Some(avatar_url) = avatar_url else { return };

    let image_storage = image_storage.clone();
    let user_id = user_id.to_string();
    let avatar_url = avatar_url.to_string();
    debug!(%user_id, %avatar_url, "Processing avatar");

    tokio::spawn(async move {
        match image_storage.process_avatar(&user_id, &avatar_url).await {
            Ok(avatar_urls) => {
                info!(
                    %user_id,
                    original_url = %avatar_urls.original_url,
                    mini_url = %avatar_urls.mini_url,
                    "Avatar processed successfully"
                );
            }
            Err(e) => {
                warn!(%user_id, %avatar_url, error = %e, "Failed to process avatar");
            }
        }
    });
}
