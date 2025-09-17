use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    #[serde(skip_serializing)]
    status_code: Option<StatusCode>,
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ErrorResponse {
    pub fn status_code(&self) -> StatusCode {
        self.status_code.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn unauthorized(description: impl Into<String>) -> Self {
        Self {
            status_code: Some(StatusCode::UNAUTHORIZED),
            error: "unauthorized".into(),
            description: Some(description.into()),
        }
    }

    pub fn bad_request(error: impl Into<String>, description: impl Into<Option<String>>) -> Self {
        Self {
            status_code: Some(StatusCode::BAD_REQUEST),
            error: error.into(),
            description: description.into(),
        }
    }

    pub fn bad_gateway(error: impl Into<String>, description: impl Into<Option<String>>) -> Self {
        Self {
            status_code: Some(StatusCode::BAD_GATEWAY),
            error: error.into(),
            description: description.into(),
        }
    }

    pub fn with_status(status: StatusCode, error: impl Into<String>, description: impl Into<Option<String>>) -> Self {
        Self {
            status_code: Some(status),
            error: error.into(),
            description: description.into(),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), Json(self)).into_response()
    }
}
