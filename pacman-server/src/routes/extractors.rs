use axum::{
    extract::FromRequestParts,
    http::{self, StatusCode},
    response::IntoResponse,
};

use crate::{app::AppState, errors::ErrorResponse};

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
