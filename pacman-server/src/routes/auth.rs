use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_cookie::CookieManager;
use tracing::{debug_span, info, instrument, trace, warn, Instrument};

use crate::data::user as user_repo;
use crate::image::ImageStorage;
use crate::{app::AppState, errors::ErrorResponse, session};

use super::extractors::RequireDatabase;

#[derive(Debug, serde::Deserialize)]
pub struct OAuthCallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
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
    _db: RequireDatabase,
) -> axum::response::Response {
    let Some(prov) = app_state.auth.get(&provider) else {
        warn!(%provider, "Unknown OAuth provider");
        return ErrorResponse::bad_request("invalid_provider", Some(provider)).into_response();
    };

    if let Some(error) = params.error {
        warn!(%provider, error = %error, desc = ?params.error_description, "OAuth callback returned an error");
        return ErrorResponse::bad_request(error, params.error_description).into_response();
    }

    let Some(code) = params.code.as_deref() else {
        return ErrorResponse::bad_request("invalid_request", Some("missing code".into())).into_response();
    };
    let Some(state) = params.state.as_deref() else {
        return ErrorResponse::bad_request("invalid_request", Some("missing state".into())).into_response();
    };

    debug_span!("oauth_callback_handler",  provider = %provider, code = %code, state = %state);

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

    let session_token = session::create_jwt_for_user(&provider, &user, &app_state.jwt_encoding_key);
    session::set_session_cookie(&cookie, &session_token);
    info!(%provider, "Signed in successfully");

    spawn_avatar_processing(app_state.image_storage.as_ref(), &user.id, user.avatar_url.as_deref());

    (StatusCode::FOUND, Redirect::to("/api/profile")).into_response()
}

pub async fn logout_handler(State(_app_state): State<AppState>, cookie: CookieManager) -> axum::response::Response {
    session::clear_session_cookie(&cookie);
    info!("Signed out successfully");
    (StatusCode::FOUND, Redirect::to("/")).into_response()
}

fn spawn_avatar_processing(image_storage: Option<&Arc<ImageStorage>>, user_id: &str, avatar_url: Option<&str>) {
    let Some(image_storage) = image_storage else { return };
    let Some(avatar_url) = avatar_url else { return };

    let image_storage = image_storage.clone();
    let user_id = user_id.to_string();
    let avatar_url = avatar_url.to_string();
    tracing::debug!(%user_id, %avatar_url, "Processing avatar");

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
