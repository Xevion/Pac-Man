mod common;
use crate::common::test_context;
use cookie::Cookie;
use pacman_server::{data::user as user_repo, session};

use pretty_assertions::assert_eq;

#[tokio::test]
async fn test_session_management() {
    let context = test_context().use_database(true).call().await;

    // 1. Create a user and link a provider account
    let user = user_repo::create_user(&context.app_state.db, Some("test@example.com"))
        .await
        .unwrap();
    let provider_account = user_repo::link_oauth_account(
        &context.app_state.db,
        user.id,
        "test_provider",
        "123",
        Some("test@example.com"),
        Some("testuser"),
        None,
        None,
    )
    .await
    .unwrap();

    // 2. Create a session token for the user
    let auth_user = pacman_server::auth::provider::AuthUser {
        id: provider_account.provider_user_id,
        username: provider_account.username.unwrap(),
        name: provider_account.display_name,
        email: user.email,
        email_verified: true,
        avatar_url: provider_account.avatar_url,
    };
    let token = session::create_jwt_for_user("test_provider", &auth_user, &context.app_state.jwt_encoding_key);

    // 3. Make a request to the protected route WITH the session, expect success
    let response = context
        .server
        .get("/api/profile")
        .add_cookie(Cookie::new(session::SESSION_COOKIE_NAME, token))
        .await;
    assert_eq!(response.status_code(), 200);

    // 4. Sign out
    let response = context.server.get("/api/logout").await;
    assert_eq!(response.status_code(), 302); // Redirect after logout

    // 5. Make a request to the protected route without a session, expect failure
    let response = context.server.get("/api/profile").await;
    assert_eq!(response.status_code(), 401); // Unauthorized without session
}
