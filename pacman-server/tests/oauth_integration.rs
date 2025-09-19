use axum_test::TestServer;
use pretty_assertions::assert_eq;

mod common;
use common::{create_test_app_state, create_test_router, TestConfig};

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

/// Test health endpoint functionality
#[tokio::test]
async fn test_health_endpoint() {
    let server = setup_test_server().await;

    // Test health endpoint - wait for health checker to complete initial run
    tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;

    let mut health_ok = false;
    let start = tokio::time::Instant::now();
    let timeout = tokio::time::Duration::from_secs(3);
    while start.elapsed() < timeout {
        let response = server.get("/health").await;
        if response.status_code() == 200 {
            let health_json: serde_json::Value = response.json();
            if health_json["ok"] == true {
                health_ok = true;
                break;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    assert!(health_ok, "Health endpoint did not return ok=true within 3 seconds");
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
