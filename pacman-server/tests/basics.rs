mod common;

use assert2::check;
use rstest::rstest;

use crate::common::{TestContext, test_context};

// A basic test of all the server's routes that aren't covered by other tests.
#[rstest]
#[case::api_root("/api/")]
#[case::auth_providers("/api/auth/providers")]
#[tokio::test]
async fn test_basic_routes(#[case] route: &str) {
    let TestContext { server, .. } = test_context().use_database(false).call().await;
    let response = server.get(route).await;
    check!(response.status_code() == 200);
}
