use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;

use crate::app::AppState;

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
    #[cfg(debug_assertions)]
    if params.contains_key("force") {
        app_state.check_health().await;
    }

    let health = app_state.health.read().await;
    let ok = health.ok();
    let status = if ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    let body = serde_json::json!({
        "ok": ok,
        "database_configured": app_state.database_configured,
        "auth_providers": app_state.auth.len(),
        "image_storage_enabled": app_state.image_storage.is_some(),
    });

    (status, axum::Json(body)).into_response()
}
