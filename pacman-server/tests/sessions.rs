mod common;
use crate::common::{test_context, TestContext};
use cookie::Cookie;
use pacman_server::session;

use pretty_assertions::assert_eq;

#[tokio::test]
async fn test_session_management() {
    let context = test_context().use_database(true).call().await;

    // 1. Create a user
    let user =
        pacman_server::data::user::create_user(&context.app_state.db, "testuser", None, None, None, "test_provider", "123")
            .await
            .unwrap();

    // 2. Create a session token for the user
    let provider_account = pacman_server::data::user::list_user_providers(&context.app_state.db, user.id)
        .await
        .unwrap()
        .into_iter()
        .find(|p| p.provider == "test_provider")
        .unwrap();

    let auth_user = pacman_server::auth::provider::AuthUser {
        id: provider_account.provider_user_id,
        username: provider_account.username.unwrap(),
        name: provider_account.display_name,
        email: user.email,
        avatar_url: provider_account.avatar_url,
    };
    let token = session::create_jwt_for_user("test_provider", &auth_user, &context.app_state.jwt_encoding_key);

    // 3. Make a request to the protected route WITH the session, expect success
    let response = context
        .server
        .get("/profile")
        .add_cookie(Cookie::new(session::SESSION_COOKIE_NAME, token))
        .await;
    assert_eq!(response.status_code(), 200);

    // 4. Sign out
    let response = context.server.get("/logout").await;
    assert_eq!(response.status_code(), 302); // Redirect after logout

    // 5. Make a request to the protected route without a session, expect failure
    let response = context.server.get("/profile").await;
    assert_eq!(response.status_code(), 401); // Unauthorized without session
}
