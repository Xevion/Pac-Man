use std::{collections::HashMap, sync::Arc};

use pacman_server::{
    auth::{
        provider::{AuthUser, MockOAuthProvider, OAuthProvider},
        AuthRegistry,
    },
    data::user as user_repo,
    session,
};
use pretty_assertions::assert_eq;
use time::Duration;

mod common;
use crate::common::{test_context, TestContext};

/// Test the basic authorization redirect flow
#[tokio::test]
async fn test_oauth_authorization_redirect() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_authorize().returning(|encoding_key| {
        Ok(pacman_server::auth::provider::AuthorizeInfo {
            authorize_url: "https://example.com/auth".parse().unwrap(),
            session_token: session::create_pkce_session("verifier", "state", encoding_key),
        })
    });

    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let TestContext { server, app_state, .. } = test_context().auth_registry(mock_registry).call().await;

    let response = server.get("/api/auth/mock").await;
    assert_eq!(response.status_code(), 303);
    assert_eq!(response.headers().get("location").unwrap(), "https://example.com/auth");

    let session_cookie = response.cookie("session");
    let claims = session::decode_jwt(session_cookie.value(), &app_state.jwt_decoding_key).unwrap();
    assert!(session::is_pkce_session(&claims), "A PKCE session should be set");
}

/// Test new user registration via OAuth callback
#[tokio::test]
async fn test_new_user_registration() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_handle_callback().returning(|_, _, _, _| {
        Ok(AuthUser {
            id: "newuser123".to_string(),
            username: "new_user".to_string(),
            name: None,
            email: Some("new@example.com".to_string()),
            email_verified: true,
            avatar_url: None,
        })
    });

    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let context = test_context().use_database(true).auth_registry(mock_registry).call().await;

    let response = context.server.get("/api/auth/mock/callback?code=a&state=b").await;
    assert_eq!(response.status_code(), 302);
    assert_eq!(response.headers().get("location").unwrap(), "/api/profile");

    // Verify user and oauth_account were created
    let user = user_repo::find_user_by_email(&context.app_state.db, "new@example.com")
        .await
        .unwrap()
        .expect("User should be created");
    assert_eq!(user.email, Some("new@example.com".to_string()));

    let providers = user_repo::list_user_providers(&context.app_state.db, user.id).await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].provider, "mock");
    assert_eq!(providers[0].provider_user_id, "newuser123");
}

/// Test sign-in for an existing user with an already-linked provider
#[tokio::test]
async fn test_existing_user_signin() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_handle_callback().returning(|_, _, _, _| {
        Ok(AuthUser {
            id: "existing123".to_string(),
            username: "existing_user".to_string(),
            name: None,
            email: Some("existing@example.com".to_string()),
            email_verified: true,
            avatar_url: None,
        })
    });

    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let context = test_context().use_database(true).auth_registry(mock_registry).call().await;

    // Pre-create the user and link
    let user = user_repo::create_user(&context.app_state.db, Some("existing@example.com"))
        .await
        .unwrap();
    user_repo::link_oauth_account(
        &context.app_state.db,
        user.id,
        "mock",
        "existing123",
        Some("existing@example.com"),
        Some("existing_user"),
        None,
        None,
    )
    .await
    .unwrap();

    let response = context.server.get("/api/auth/mock/callback?code=a&state=b").await;
    assert_eq!(response.status_code(), 302, "Should sign in successfully");
    assert_eq!(response.headers().get("location").unwrap(), "/api/profile");

    // Verify no new user was created
    let users = sqlx::query("SELECT * FROM users")
        .fetch_all(&context.app_state.db)
        .await
        .unwrap();
    assert_eq!(users.len(), 1, "No new user should be created");
}

/// Test implicit account linking via a shared verified email
#[tokio::test]
async fn test_implicit_account_linking() {
    // 1. User signs in with 'provider-a'
    let mut mock_a = MockOAuthProvider::new();
    mock_a.expect_handle_callback().returning(|_, _, _, _| {
        Ok(AuthUser {
            id: "user_a_123".to_string(),
            username: "user_a".to_string(),
            name: None,
            email: Some("shared@example.com".to_string()),
            email_verified: true,
            avatar_url: None,
        })
    });

    // 2. Later, the same user signs in with 'provider-b'
    let mut mock_b = MockOAuthProvider::new();
    mock_b.expect_handle_callback().returning(|_, _, _, _| {
        Ok(AuthUser {
            id: "user_b_456".to_string(),
            username: "user_b".to_string(),
            name: None,
            email: Some("shared@example.com".to_string()),
            email_verified: true,
            avatar_url: None,
        })
    });

    let provider_a: Arc<dyn OAuthProvider> = Arc::new(mock_a);
    let provider_b: Arc<dyn OAuthProvider> = Arc::new(mock_b);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("provider-a", provider_a), ("provider-b", provider_b)]),
    };

    let context = test_context().use_database(true).auth_registry(mock_registry).call().await;

    // Action 1: Sign in with provider-a, creating the initial user
    let response1 = context.server.get("/api/auth/provider-a/callback?code=a&state=b").await;
    assert_eq!(response1.status_code(), 302);

    let user = user_repo::find_user_by_email(&context.app_state.db, "shared@example.com")
        .await
        .unwrap()
        .unwrap();
    let providers1 = user_repo::list_user_providers(&context.app_state.db, user.id).await.unwrap();
    assert_eq!(providers1.len(), 1);
    assert_eq!(providers1[0].provider, "provider-a");

    // Action 2: Sign in with provider-b
    let response2 = context.server.get("/api/auth/provider-b/callback?code=a&state=b").await;
    assert_eq!(response2.status_code(), 302);

    // Assertions: No new user, but a new provider link
    let users = sqlx::query("SELECT * FROM users")
        .fetch_all(&context.app_state.db)
        .await
        .unwrap();
    assert_eq!(users.len(), 1, "A new user should NOT have been created");

    let providers2 = user_repo::list_user_providers(&context.app_state.db, user.id).await.unwrap();
    assert_eq!(providers2.len(), 2, "A new provider should have been linked");
    assert!(providers2.iter().any(|p| p.provider == "provider-a"));
    assert!(providers2.iter().any(|p| p.provider == "provider-b"));
}

/// Test that an unverified email does NOT link accounts
#[tokio::test]
async fn test_unverified_email_creates_new_account() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_handle_callback().returning(|_, _, _, _| {
        Ok(AuthUser {
            id: "unverified123".to_string(),
            username: "unverified_user".to_string(),
            name: None,
            email: Some("unverified@example.com".to_string()),
            email_verified: false,
            avatar_url: None,
        })
    });

    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let context = test_context().use_database(true).auth_registry(mock_registry).call().await;

    // Pre-create a user with the same email, but they will not be linked.
    user_repo::create_user(&context.app_state.db, Some("unverified@example.com"))
        .await
        .unwrap();

    let response = context.server.get("/api/auth/mock/callback?code=a&state=b").await;
    assert_eq!(response.status_code(), 302);

    // Should create a second user because the email wasn't trusted for linking
    let users = sqlx::query("SELECT * FROM users")
        .fetch_all(&context.app_state.db)
        .await
        .unwrap();
    assert_eq!(users.len(), 2, "A new user should be created for the unverified email");
}

/// Test logout functionality
#[tokio::test]
async fn test_logout_functionality() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_handle_callback().returning(|_, _, _, _| {
        Ok(AuthUser {
            id: "123".to_string(),
            username: "testuser".to_string(),
            name: None,
            email: Some("test@example.com".to_string()),
            email_verified: true,
            avatar_url: None,
        })
    });
    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let context = test_context().use_database(true).auth_registry(mock_registry).call().await;

    // Sign in to establish a session
    let response = context.server.get("/api/auth/mock/callback?code=a&state=b").await;
    assert_eq!(response.status_code(), 302);

    // Test that the logout handler clears the session cookie and redirects
    let response = context.server.get("/api/logout").await;

    assert_eq!(response.status_code(), 302);
    assert!(response.headers().contains_key("location"));

    let cookie = response.cookie("session");
    assert_eq!(cookie.value(), "removed");
    assert_eq!(cookie.max_age(), Some(Duration::ZERO));
}
