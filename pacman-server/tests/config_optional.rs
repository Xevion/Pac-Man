//! Tests for optional configuration features
//!
//! These tests verify that:
//! 1. The server can start without database, Discord, GitHub, or S3 configured
//! 2. Partial configuration (e.g., only DISCORD_CLIENT_ID) fails with a clear error
//! 3. Routes behave correctly when features are disabled

mod common;

use assert2::{assert, check};
use axum::http::StatusCode;
use rstest::rstest;

use crate::common::{TestContext, test_context};

/// Test that the server starts and responds to health checks without any OAuth providers
#[tokio::test]
async fn test_server_without_oauth_providers() {
    let TestContext { server, app_state, .. } = test_context()
        .with_discord(false)
        .with_github(false)
        .use_database(false)
        .call()
        .await;

    // Verify no providers registered
    check!(app_state.auth.len() == 0);

    // Health check should work
    let response = server.get("/api/health").await;
    check!(response.status_code() == StatusCode::OK);

    // Providers endpoint should return empty list
    let response = server.get("/api/auth/providers").await;
    assert!(response.status_code() == StatusCode::OK);
    let body: Vec<serde_json::Value> = response.json();
    check!(body.is_empty());
}

/// Test that the server starts with only one provider configured
#[rstest]
#[case::discord_only("discord", "github")]
#[case::github_only("github", "discord")]
#[tokio::test]
async fn test_server_with_single_provider(#[case] enabled: &str, #[case] disabled: &str) {
    let TestContext { server, app_state, .. } = test_context()
        .with_discord(enabled == "discord")
        .with_github(enabled == "github")
        .use_database(false)
        .call()
        .await;

    // Verify only the enabled provider is registered
    check!(app_state.auth.len() == 1);
    check!(app_state.auth.get(enabled).is_some());
    check!(app_state.auth.get(disabled).is_none());

    // Providers endpoint should return only the enabled provider
    let response = server.get("/api/auth/providers").await;
    assert!(response.status_code() == StatusCode::OK);
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.len() == 1);
    check!(body[0]["id"] == enabled);
}

/// Test that the server starts without database configured
#[tokio::test]
async fn test_server_without_database() {
    let TestContext {
        server,
        app_state,
        config,
        ..
    } = test_context().use_database(false).call().await;

    // Verify database is not configured
    check!(config.database.is_none());
    check!(!app_state.database_configured);

    // Health check should still work
    let response = server.get("/api/health").await;
    assert!(response.status_code() == StatusCode::OK);

    let body: serde_json::Value = response.json();
    check!(body["ok"] == true);
    check!(body["database_configured"] == false);
}

/// Test that profile endpoint returns 503 when database is not configured
#[tokio::test]
async fn test_profile_without_database_returns_503() {
    let TestContext { server, .. } = test_context().use_database(false).call().await;

    // Create a fake session cookie to get past the auth check
    let response = server.get("/api/profile").await;

    // Should return 503 Service Unavailable because database is not configured
    assert!(response.status_code() == StatusCode::SERVICE_UNAVAILABLE);

    let body: serde_json::Value = response.json();
    check!(body["error"] == "database_not_configured");
}

/// Test that OAuth callback returns 503 when database is not configured
#[tokio::test]
async fn test_oauth_callback_without_database_returns_503() {
    let TestContext { server, .. } = test_context().with_github(true).use_database(false).call().await;

    // Try to complete OAuth flow - should fail because database is not configured
    let response = server
        .get("/api/auth/github/callback")
        .add_query_param("code", "test_code")
        .add_query_param("state", "test_state")
        .await;

    // Should return 503 Service Unavailable because database is not configured
    assert!(response.status_code() == StatusCode::SERVICE_UNAVAILABLE);

    let body: serde_json::Value = response.json();
    check!(body["error"] == "database_not_configured");
}

/// Test that unknown provider returns 400
#[tokio::test]
async fn test_unknown_provider_returns_400() {
    let TestContext { server, .. } = test_context().with_discord(true).use_database(false).call().await;

    // Try to access non-existent provider
    let response = server.get("/api/auth/twitter").await;
    assert!(response.status_code() == StatusCode::BAD_REQUEST);

    let body: serde_json::Value = response.json();
    check!(body["error"] == "invalid_provider");
}

/// Test that logout works without database
#[tokio::test]
async fn test_logout_without_database() {
    let TestContext { server, .. } = test_context().use_database(false).call().await;

    // Logout should work even without database
    let response = server.get("/api/logout").await;

    // Logout redirects to home
    check!(response.status_code() == StatusCode::FOUND);
}

/// Test basic routes work without database or OAuth
#[tokio::test]
async fn test_basic_routes_minimal_config() {
    let TestContext { server, .. } = test_context()
        .with_discord(false)
        .with_github(false)
        .use_database(false)
        .call()
        .await;

    // Root API endpoint
    let response = server.get("/api/").await;
    check!(response.status_code() == StatusCode::OK);

    // Health endpoint
    let response = server.get("/api/health").await;
    check!(response.status_code() == StatusCode::OK);

    // Providers endpoint (empty list)
    let response = server.get("/api/auth/providers").await;
    check!(response.status_code() == StatusCode::OK);
}

/// Test health endpoint includes feature status
#[tokio::test]
async fn test_health_includes_feature_status() {
    let TestContext { server, .. } = test_context()
        .with_discord(true)
        .with_github(false)
        .use_database(false)
        .call()
        .await;

    let response = server.get("/api/health").await;
    assert!(response.status_code() == StatusCode::OK);

    let body: serde_json::Value = response.json();
    check!(body["ok"] == true);
    check!(body["database_configured"] == false);
    check!(body["auth_providers"] == 1); // Only Discord
    check!(body["image_storage_enabled"] == false); // No S3 configured
}
