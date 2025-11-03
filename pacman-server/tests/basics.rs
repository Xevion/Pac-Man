mod common;

use pretty_assertions::assert_eq;

use crate::common::{test_context, TestContext};

// A basic test of all the server's routes that aren't covered by other tests.
#[tokio::test]
async fn test_basic_routes() {
    let routes = vec!["/api/", "/api/auth/providers"];

    for route in routes {
        let TestContext { server, .. } = test_context().use_database(false).call().await;
        let response = server.get(route).await;
        assert_eq!(response.status_code(), 200);
    }
}
