mod common;

#[cfg(feature = "postgres-tests")]
use crate::common::{test_context, TestContext};

/// Test health endpoint with PostgreSQL (requires postgres-tests feature)
#[tokio::test]
#[cfg(feature = "postgres-tests")]
async fn test_health_endpoint() {
    let TestContext { server, container, .. } = test_context().use_database(true).call().await;

    // First, verify health endpoint works when database is healthy
    let response = server.get("/api/health").await;
    assert_eq!(response.status_code(), 200);
    let health_json: serde_json::Value = response.json();
    assert_eq!(health_json["ok"], true);

    // Now kill the database container to simulate database failure
    drop(container);

    // Now verify health endpoint reports bad health
    let response = server.get("/api/health?force").await;
    assert_eq!(response.status_code(), 503); // SERVICE_UNAVAILABLE
    let health_json: serde_json::Value = response.json();
    assert_eq!(health_json["ok"], false);
}
