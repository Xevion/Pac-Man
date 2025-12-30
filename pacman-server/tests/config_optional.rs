//! Tests for optional configuration features
//!
//! These tests verify that:
//! 1. The server can start without database, Discord, GitHub, or S3 configured
//! 2. Partial configuration (e.g., only DISCORD_CLIENT_ID) fails with a clear error
//! 3. Routes behave correctly when features are disabled

mod common;

use axum::http::StatusCode;
use pretty_assertions::assert_eq;

use crate::common::{test_context, TestContext};

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
    assert_eq!(app_state.auth.len(), 0);

    // Health check should work
    let response = server.get("/api/health").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // Providers endpoint should return empty list
    let response = server.get("/api/auth/providers").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

/// Test that the server starts with only Discord configured
#[tokio::test]
async fn test_server_with_discord_only() {
    let TestContext { server, app_state, .. } = test_context()
        .with_discord(true)
        .with_github(false)
        .use_database(false)
        .call()
        .await;

    // Verify only Discord is registered
    assert_eq!(app_state.auth.len(), 1);
    assert!(app_state.auth.get("discord").is_some());
    assert!(app_state.auth.get("github").is_none());

    // Providers endpoint should return only Discord
    let response = server.get("/api/auth/providers").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["id"], "discord");
}

/// Test that the server starts with only GitHub configured
#[tokio::test]
async fn test_server_with_github_only() {
    let TestContext { server, app_state, .. } = test_context()
        .with_discord(false)
        .with_github(true)
        .use_database(false)
        .call()
        .await;

    // Verify only GitHub is registered
    assert_eq!(app_state.auth.len(), 1);
    assert!(app_state.auth.get("github").is_some());
    assert!(app_state.auth.get("discord").is_none());

    // Providers endpoint should return only GitHub
    let response = server.get("/api/auth/providers").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["id"], "github");
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
    assert!(config.database.is_none());
    assert!(!app_state.database_configured);

    // Health check should still work
    let response = server.get("/api/health").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["ok"], true);
    assert_eq!(body["database_configured"], false);
}

/// Test that profile endpoint returns 503 when database is not configured
#[tokio::test]
async fn test_profile_without_database_returns_503() {
    let TestContext { server, .. } = test_context().use_database(false).call().await;

    // Create a fake session cookie to get past the auth check
    let response = server.get("/api/profile").await;

    // Should return 503 Service Unavailable because database is not configured
    assert_eq!(response.status_code(), StatusCode::SERVICE_UNAVAILABLE);

    let body: serde_json::Value = response.json();
    assert_eq!(body["error"], "database_not_configured");
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
    assert_eq!(response.status_code(), StatusCode::SERVICE_UNAVAILABLE);

    let body: serde_json::Value = response.json();
    assert_eq!(body["error"], "database_not_configured");
}

/// Test that unknown provider returns 400
#[tokio::test]
async fn test_unknown_provider_returns_400() {
    let TestContext { server, .. } = test_context().with_discord(true).use_database(false).call().await;

    // Try to access non-existent provider
    let response = server.get("/api/auth/twitter").await;
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = response.json();
    assert_eq!(body["error"], "invalid_provider");
}

/// Test that logout works without database
#[tokio::test]
async fn test_logout_without_database() {
    let TestContext { server, .. } = test_context().use_database(false).call().await;

    // Logout should work even without database
    let response = server.get("/api/logout").await;

    // Logout redirects to home
    assert_eq!(response.status_code(), StatusCode::FOUND);
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
    assert_eq!(response.status_code(), StatusCode::OK);

    // Health endpoint
    let response = server.get("/api/health").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // Providers endpoint (empty list)
    let response = server.get("/api/auth/providers").await;
    assert_eq!(response.status_code(), StatusCode::OK);
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
    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["ok"], true);
    assert_eq!(body["database_configured"], false);
    assert_eq!(body["auth_providers"], 1); // Only Discord
    assert_eq!(body["image_storage_enabled"], false); // No S3 configured
}
