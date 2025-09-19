mod common;

use pretty_assertions::assert_eq;

use crate::common::{test_context, TestContext};

/// Test health endpoint functionality with real database connectivity
#[tokio::test]
async fn test_health_endpoint() {
    let TestContext { server, container, .. } = test_context().use_database(true).call().await;

    // First, verify health endpoint works when database is healthy
    let response = server.get("/health").await;
    assert_eq!(response.status_code(), 200);
    let health_json: serde_json::Value = response.json();
    assert_eq!(health_json["ok"], true);

    // Now kill the database container to simulate database failure
    drop(container);

    // Now verify health endpoint reports bad health
    let response = server.get("/health?force").await;
    assert_eq!(response.status_code(), 503); // SERVICE_UNAVAILABLE
    let health_json: serde_json::Value = response.json();
    assert_eq!(health_json["ok"], false);
}
