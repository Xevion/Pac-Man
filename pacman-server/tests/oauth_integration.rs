use axum_test::TestServer;
use mockall::predicate::*;
use pretty_assertions::assert_eq;

mod common;
use common::{create_test_app_state, create_test_router, TestConfig};
// OAuth provider imports removed as they're not used in these health tests

/// Common setup function for all tests
async fn setup_test_server() -> TestServer {
    let test_config = TestConfig::new().await;
    let app_state = create_test_app_state(&test_config).await;
    let router = create_test_router(app_state);
    TestServer::new(router).unwrap()
}

/// Test basic endpoints functionality
#[tokio::test]
async fn test_basic_endpoints() {
    let server = setup_test_server().await;

    // Test root endpoint
    let response = server.get("/").await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.text(), "Hello, World! Visit /auth/github to start OAuth flow.");
}

/// Test health endpoint functionality with real database connectivity
#[tokio::test]
async fn test_health_endpoint() {
    let test_config = TestConfig::new().await;
    let app_state = create_test_app_state(&test_config).await;

    let router = create_test_router(app_state.clone());
    let server = TestServer::new(router).unwrap();

    // First, verify health endpoint works when database is healthy
    let response = server.get("/health").await;
    assert_eq!(response.status_code(), 200);
    let health_json: serde_json::Value = response.json();
    assert_eq!(health_json["ok"], true);

    // Now kill the database container to simulate database failure
    drop(test_config.container);

    // Now verify health endpoint reports bad health
    let response = server.get("/health?force").await;
    assert_eq!(response.status_code(), 503); // SERVICE_UNAVAILABLE
    let health_json: serde_json::Value = response.json();
    assert_eq!(health_json["ok"], false);
}

/// Test OAuth provider listing and configuration
#[tokio::test]
async fn test_oauth_provider_configuration() {
    let server = setup_test_server().await;

    // Test providers list endpoint
    let response = server.get("/auth/providers").await;
    assert_eq!(response.status_code(), 200);
    let providers: Vec<serde_json::Value> = response.json();
    assert_eq!(providers.len(), 2); // Should have GitHub and Discord providers

    // Verify provider structure
    let provider_ids: Vec<&str> = providers.iter().map(|p| p["id"].as_str().unwrap()).collect();
    assert!(provider_ids.contains(&"github"));
    assert!(provider_ids.contains(&"discord"));

    // Verify provider details
    for provider in providers {
        let id = provider["id"].as_str().unwrap();
        let name = provider["name"].as_str().unwrap();
        let active = provider["active"].as_bool().unwrap();

        assert!(active, "Provider {} should be active", id);

        match id {
            "github" => assert_eq!(name, "GitHub"),
            "discord" => assert_eq!(name, "Discord"),
            _ => panic!("Unknown provider: {}", id),
        }
    }
}

/// Test OAuth authorization flows
#[tokio::test]
async fn test_oauth_authorization_flows() {
    let server = setup_test_server().await;

    // Test OAuth authorize endpoint (should redirect)
    let response = server.get("/auth/github").await;
    assert_eq!(response.status_code(), 303); // Redirect to GitHub OAuth

    // Test OAuth authorize endpoint for Discord
    let response = server.get("/auth/discord").await;
    assert_eq!(response.status_code(), 303); // Redirect to Discord OAuth

    // Test unknown provider
    let response = server.get("/auth/unknown").await;
    assert_eq!(response.status_code(), 400); // Bad request for unknown provider
}

/// Test OAuth callback handling
#[tokio::test]
async fn test_oauth_callback_handling() {
    let server = setup_test_server().await;

    // Test OAuth callback with missing parameters (should fail gracefully)
    let response = server.get("/auth/github/callback").await;
    assert_eq!(response.status_code(), 400); // Bad request for missing code/state
}

/// Test session management endpoints
#[tokio::test]
async fn test_session_management() {
    let server = setup_test_server().await;

    // Test logout endpoint (should redirect)
    let response = server.get("/logout").await;
    assert_eq!(response.status_code(), 302); // Redirect to home

    // Test profile endpoint without session (should be unauthorized)
    let response = server.get("/profile").await;
    assert_eq!(response.status_code(), 401); // Unauthorized without session
}

/// Test that verifies database operations work correctly
#[tokio::test]
async fn test_database_operations() {
    let server = setup_test_server().await;

    // Act: Test health endpoint to verify database connectivity
    let response = server.get("/health").await;

    // Assert: Health should be OK, indicating database is connected and migrations ran
    assert_eq!(response.status_code(), 200);
    let health_json: serde_json::Value = response.json();
    assert_eq!(health_json["ok"], true);
}

/// Test OAuth authorization flow
#[tokio::test]
async fn test_oauth_authorization_flow() {
    let _server = setup_test_server().await;

    // TODO: Test that the OAuth authorize handler redirects to the provider's authorization page for valid providers
    // TODO: Test that the OAuth authorize handler returns an error for unknown providers
    // TODO: Test that the OAuth authorize handler sets a link cookie when the link parameter is true
}

/// Test OAuth callback validation
#[tokio::test]
async fn test_oauth_callback_validation() {
    let _server = setup_test_server().await;

    // TODO: Test that the OAuth callback handler validates the provider exists before processing
    // TODO: Test that the OAuth callback handler returns an error when the provider returns an OAuth error
    // TODO: Test that the OAuth callback handler returns an error when the authorization code is missing
    // TODO: Test that the OAuth callback handler returns an error when the state parameter is missing
}

/// Test OAuth callback processing
#[tokio::test]
async fn test_oauth_callback_processing() {
    let _server = setup_test_server().await;

    // TODO: Test that the OAuth callback handler exchanges the authorization code for user information successfully
    // TODO: Test that the OAuth callback handler handles provider callback errors gracefully
    // TODO: Test that the OAuth callback handler creates a session token after successful authentication
    // TODO: Test that the OAuth callback handler sets a session cookie after successful authentication
    // TODO: Test that the OAuth callback handler redirects to the profile page after successful authentication
}

/// Test account linking flow
#[tokio::test]
async fn test_account_linking_flow() {
    let _server = setup_test_server().await;

    // TODO: Test that the OAuth callback handler links a new provider to an existing user when link intent is present and session is valid
    // TODO: Test that the OAuth callback handler redirects to profile after successful account linking
    // TODO: Test that the OAuth callback handler falls back to normal sign-in when link intent is present but no valid session exists
}

/// Test new user registration
#[tokio::test]
async fn test_new_user_registration() {
    let _server = setup_test_server().await;

    // TODO: Test that the OAuth callback handler creates a new user account when no existing user is found
    // TODO: Test that the OAuth callback handler requires an email address for all sign-ins
    // TODO: Test that the OAuth callback handler rejects sign-in attempts when no email is available
}

/// Test existing user sign-in
#[tokio::test]
async fn test_existing_user_sign_in() {
    let _server = setup_test_server().await;

    // TODO: Test that the OAuth callback handler allows sign-in when the provider is already linked to an existing user
    // TODO: Test that the OAuth callback handler requires explicit linking when a user with the same email exists and has other providers linked
    // TODO: Test that the OAuth callback handler auto-links a provider when a user exists but has no other providers linked
}

/// Test avatar processing
#[tokio::test]
async fn test_avatar_processing() {
    let _server = setup_test_server().await;

    // TODO: Test that the OAuth callback handler processes user avatars asynchronously without blocking the response
    // TODO: Test that the OAuth callback handler handles avatar processing errors gracefully
}

/// Test profile access
#[tokio::test]
async fn test_profile_access() {
    let _server = setup_test_server().await;

    // TODO: Test that the profile handler returns user information when a valid session exists
    // TODO: Test that the profile handler returns an error when no session cookie is present
    // TODO: Test that the profile handler returns an error when the session token is invalid
    // TODO: Test that the profile handler includes linked providers in the response
    // TODO: Test that the profile handler returns an error when the user is not found in the database
}

/// Test logout functionality
#[tokio::test]
async fn test_logout_functionality() {
    let _server = setup_test_server().await;

    // TODO: Test that the logout handler clears the session if a session was there
    // TODO: Test that the logout handler removes the session from memory storage
    // TODO: Test that the logout handler clears the session cookie
    // TODO: Test that the logout handler redirects to the home page after logout
}

/// Test provider configuration
#[tokio::test]
async fn test_provider_configuration() {
    let _server = setup_test_server().await;

    // TODO: Test that the providers list handler returns all configured OAuth providers
    // TODO: Test that the providers list handler includes provider status (active/inactive)
}
