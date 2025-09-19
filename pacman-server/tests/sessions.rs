mod common;
use crate::common::{test_context, TestContext};

use pretty_assertions::assert_eq;

/// Test session management endpoints
#[tokio::test]
async fn test_session_management() {
    let TestContext { server, .. } = test_context().use_database(true).call().await;

    // Test logout endpoint (should redirect)
    let response = server.get("/logout").await;
    assert_eq!(response.status_code(), 302); // Redirect to home

    // Test profile endpoint without session (should be unauthorized)
    let response = server.get("/profile").await;
    assert_eq!(response.status_code(), 401); // Unauthorized without session
}
