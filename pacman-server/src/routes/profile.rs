use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_cookie::CookieManager;
use serde::Serialize;
use tracing::{debug, warn};

use crate::data::user as user_repo;
use crate::{app::AppState, errors::ErrorResponse, session};

use super::extractors::RequireDatabase;

#[derive(Serialize)]
struct ProfilePayload {
    id: i64,
    email: Option<String>,
    providers: Vec<user_repo::ProviderPublic>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Handles the request to the profile endpoint.
///
/// Requires the `session` cookie to be present.
pub async fn profile_handler(
    State(app_state): State<AppState>,
    cookie: CookieManager,
    _db: RequireDatabase,
) -> axum::response::Response {
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
