use axum::{
    extract::FromRequestParts,
    http::{self, StatusCode},
    response::IntoResponse,
};
use axum_cookie::CookieManager;
use tracing::{debug, warn};

use crate::{app::AppState, data::user as user_repo, errors::ErrorResponse, session};

/// Axum extractor that rejects requests with 503 when no database is configured.
pub struct RequireDatabase;

impl FromRequestParts<AppState> for RequireDatabase {
    type Rejection = axum::response::Response;

    async fn from_request_parts(_parts: &mut http::request::Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        if state.database_configured {
            Ok(RequireDatabase)
        } else {
            Err(ErrorResponse::with_status(
                StatusCode::SERVICE_UNAVAILABLE,
                "database_not_configured",
                Some("Database is not configured. This endpoint requires a database.".into()),
            )
            .into_response())
        }
    }
}

/// Axum extractor that resolves the authenticated user from the session cookie.
///
/// Rejects with 401 when the cookie is missing, the token is invalid, or no user
/// matches it. Requires a database to perform the user lookup.
pub struct AuthenticatedUser {
    pub user_id: i64,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = axum::response::Response;

    async fn from_request_parts(parts: &mut http::request::Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let cookie = CookieManager::from_request_parts(parts, state)
            .await
            .map_err(|_| ErrorResponse::unauthorized("missing session cookie").into_response())?;

        let Some(token) = session::get_session_token(&cookie) else {
            debug!("Missing session cookie");
            return Err(ErrorResponse::unauthorized("missing session cookie").into_response());
        };
        let Some(claims) = session::decode_jwt(&token, &state.jwt_decoding_key) else {
            debug!("Invalid session token");
            return Err(ErrorResponse::unauthorized("invalid session token").into_response());
        };
        // sub format: provider:provider_user_id
        let Some((provider, provider_user_id)) = claims.subject.split_once(':') else {
            debug!("Malformed session token subject");
            return Err(ErrorResponse::unauthorized("invalid session token").into_response());
        };

        match user_repo::find_user_by_provider_id(&state.db, provider, provider_user_id).await {
            Ok(Some(user)) => Ok(AuthenticatedUser { user_id: user.id }),
            Ok(None) => {
                debug!("User not found for session");
                Err(ErrorResponse::unauthorized("session not found").into_response())
            }
            Err(e) => {
                warn!(error = %e, "Failed to look up user for session");
                Err(ErrorResponse::with_status(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    Some("could not verify session".into()),
                )
                .into_response())
            }
        }
    }
}
