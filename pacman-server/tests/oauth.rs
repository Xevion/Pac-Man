use std::{collections::HashMap, sync::Arc};

use pacman_server::auth::{
    provider::{MockOAuthProvider, OAuthProvider},
    AuthRegistry,
};
use pretty_assertions::assert_eq;
use time::Duration;

mod common;
use crate::common::{test_context, TestContext};

/// Test OAuth authorization flows
#[tokio::test]
async fn test_oauth_authorization_flows() {
    let TestContext { server, .. } = test_context().call().await;

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
    let TestContext { server, .. } = test_context().call().await;

    // Test OAuth callback with missing parameters (should fail gracefully)
    let response = server.get("/auth/github/callback").await;
    assert_eq!(response.status_code(), 400); // Bad request for missing code/state
}

/// Test OAuth authorization flow
#[tokio::test]
async fn test_oauth_authorization_flow() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_authorize().returning(|_| {
        Ok(pacman_server::auth::provider::AuthorizeInfo {
            authorize_url: "https://example.com".parse().unwrap(),
            session_token: "a_token".to_string(),
        })
    });

    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let TestContext { server, .. } = test_context().auth_registry(mock_registry).call().await;

    // Test that valid handlers redirect
    let response = server.get("/auth/mock").await;
    assert_eq!(response.status_code(), 303); // Redirect to GitHub OAuth

    // Test that unknown handlers return an error
    let response = server.get("/auth/unknown").await;
    assert_eq!(response.status_code(), 400); // Bad request for unknown provider

    // Test that session cookie is set
    let response = server.get("/auth/mock").await;
    assert_eq!(response.status_code(), 303);
    let cookies = {
        let cookies = response.cookies();
        cookies.iter().cloned().collect::<Vec<_>>()
    };
    assert_eq!(cookies.len(), 1);
    assert_eq!(cookies[0].name(), "session");
    assert_eq!(cookies[0].value(), "a_token");

    // Test that link parameter redirects and sets a link cookie
    let response = server.get("/auth/mock?link=true").await;
    assert_eq!(response.status_code(), 303);
    assert_eq!(response.maybe_cookie("link").is_some(), true);
    assert_eq!(response.maybe_cookie("link").unwrap().value(), "1");
}

/// Test OAuth callback validation
#[tokio::test]
async fn test_oauth_callback_validation() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_handle_callback()
        .times(0) // Should not be called
        .returning(|_, _, _, _| {
            Ok(pacman_server::auth::provider::AuthUser {
                id: "123".to_string(),
                username: "testuser".to_string(),
                name: None,
                email: Some("test@example.com".to_string()),
                avatar_url: None,
            })
        });
    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let TestContext { server, .. } = test_context().use_database(true).auth_registry(mock_registry).call().await;

    // Test that an unknown provider returns an error
    let response = server.get("/auth/unknown/callback?code=a&state=b").await;
    assert_eq!(response.status_code(), 400);

    // Test that a provider-returned error is handled
    let response = server.get("/auth/mock/callback?error=access_denied").await;
    assert_eq!(response.status_code(), 400);

    // Test that a missing code returns an error
    let response = server.get("/auth/mock/callback?state=b").await;
    assert_eq!(response.status_code(), 400);

    // Test that a missing state returns an error
    let response = server.get("/auth/mock/callback?code=a").await;
    assert_eq!(response.status_code(), 400);
}

/// Test OAuth callback processing
#[tokio::test]
async fn test_oauth_callback_processing() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_handle_callback().returning(|_, _, _, _| {
        Ok(pacman_server::auth::provider::AuthUser {
            id: "123".to_string(),
            username: "testuser".to_string(),
            name: None,
            email: Some("processing@example.com".to_string()),
            avatar_url: None,
        })
    });
    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let context = test_context().use_database(true).auth_registry(mock_registry).call().await;

    // Test that a successful callback redirects and sets a session cookie
    let response = context.server.get("/auth/mock/callback?code=a&state=b").await;
    assert_eq!(response.status_code(), 302);
    assert_eq!(response.headers().get("location").unwrap(), "/profile");
    assert!(response.maybe_cookie("session").is_some());
}

/// Test account linking flow
#[tokio::test]
async fn test_account_linking_flow() {
    let mut initial_provider_mock = MockOAuthProvider::new();
    initial_provider_mock.expect_handle_callback().returning(move |_, _, _, _| {
        Ok(pacman_server::auth::provider::AuthUser {
            id: "123".to_string(),
            username: "linkuser".to_string(),
            name: None,
            email: Some("link@example.com".to_string()),
            avatar_url: None,
        })
    });
    let initial_provider: Arc<dyn OAuthProvider> = Arc::new(initial_provider_mock);

    let mut link_provider_mock = MockOAuthProvider::new();
    link_provider_mock.expect_authorize().returning(|_| {
        Ok(pacman_server::auth::provider::AuthorizeInfo {
            authorize_url: "https://example.com".parse().unwrap(),
            session_token: "b_token".to_string(),
        })
    });
    link_provider_mock.expect_handle_callback().returning(|_, _, _, _| {
        Ok(pacman_server::auth::provider::AuthUser {
            id: "456".to_string(),
            username: "linkuser_new".to_string(),
            name: None,
            email: Some("link@example.com".to_string()),
            avatar_url: None,
        })
    });
    let link_provider: Arc<dyn OAuthProvider> = Arc::new(link_provider_mock);

    let registry = AuthRegistry {
        providers: HashMap::from([("mock_initial", initial_provider), ("mock_link", link_provider)]),
    };
    let context = test_context().use_database(true).auth_registry(registry).call().await;

    // 1. Create an initial user and provider link
    let user = pacman_server::data::user::create_user(
        &context.app_state.db,
        "linkuser",
        None,
        Some("link@example.com"),
        None,
        "mock_initial",
        "123",
    )
    .await
    .expect("Failed to create user");

    {
        let providers = pacman_server::data::user::list_user_providers(&context.app_state.db, user.id)
            .await
            .expect("Failed to list user's initial provider(s)");
        assert_eq!(providers.len(), 1, "User should have one provider");
        assert!(providers.iter().any(|p| p.provider == "mock_initial"));
    }

    // 2. Create a session for this user
    let session_cookie = {
        let response = context.server.get("/auth/mock_initial/callback?code=a&state=b").await;
        assert_eq!(response.status_code(), 302);
        assert!(response.maybe_cookie("session").is_some(), "Session cookie should be set");

        response.cookie("session").clone()
    };
    tracing::debug!(cookie = %session_cookie, "Session cookie acquired");

    // Begin linking flow
    let link_cookie = {
        let response = context
            .server
            .get("/auth/mock_link?link=true")
            .add_cookie(session_cookie.clone())
            .await;
        assert_eq!(response.status_code(), 303);
        assert_eq!(response.maybe_cookie("link").unwrap().value(), "1");

        response.cookie("link").clone()
    };
    tracing::debug!(cookie = %link_cookie, "Link cookie acquired");

    // 3. Perform the linking call
    let response = context
        .server
        .get("/auth/mock_link/callback?code=a&state=b")
        .add_cookie(link_cookie)
        .add_cookie(session_cookie.clone())
        .await;

    assert_eq!(response.status_code(), 303, "Post-linking response should be a redirect");
    assert_eq!(
        response.headers().get("location").unwrap(),
        "/profile",
        "Post-linking response should redirect to /profile"
    );

    let providers = pacman_server::data::user::list_user_providers(&context.app_state.db, user.id)
        .await
        .expect("Failed to list user's providers");
    assert_eq!(providers.len(), 2, "User should have two providers");
    assert!(providers.iter().any(|p| p.provider == "mock_initial"));
    assert!(providers.iter().any(|p| p.provider == "mock_link"));
}

/// Test new user registration
#[tokio::test]
async fn test_new_user_registration() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_handle_callback().returning(|_, _, _, _| {
        Ok(pacman_server::auth::provider::AuthUser {
            id: "123".to_string(),
            username: "testuser".to_string(),
            name: None,
            email: Some("newuser@example.com".to_string()),
            avatar_url: None,
        })
    });

    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let context = test_context().use_database(true).auth_registry(mock_registry).call().await;

    // Test that the OAuth callback handler creates a new user account when no existing user is found
    let response = context.server.get("/auth/mock/callback?code=a&state=b").await;
    assert_eq!(response.status_code(), 302);
    assert_eq!(response.headers().get("location").unwrap(), "/profile");
    let user = pacman_server::data::user::find_user_by_email(&context.app_state.db, "newuser@example.com")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.email, Some("newuser@example.com".to_string()));
}

/// Test OAuth callback handler rejects sign-in attempts when no email is available
#[tokio::test]
async fn test_oauth_callback_no_email() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_handle_callback().returning(|_, _, _, _| {
        Ok(pacman_server::auth::provider::AuthUser {
            id: "456".to_string(),
            username: "noemailuser".to_string(),
            name: None,
            email: None,
            avatar_url: None,
        })
    });

    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let context = test_context().use_database(true).auth_registry(mock_registry).call().await;

    // Test that the OAuth callback handler rejects sign-in attempts when no email is available
    let response = context.server.get("/auth/mock/callback?code=a&state=b").await;
    assert_eq!(response.status_code(), 400);
}

/// Test existing user sign-in with new provider fails
#[tokio::test]
async fn test_existing_user_sign_in_new_provider_fails() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_handle_callback().returning(move |_, _, _, _| {
        Ok(pacman_server::auth::provider::AuthUser {
            id: "456".to_string(), // Different provider ID
            username: "existinguser_newprovider".to_string(),
            name: None,
            email: Some("existing@example.com".to_string()),
            avatar_url: None,
        })
    });
    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock_new", provider)]),
    };

    let context = test_context().use_database(true).auth_registry(mock_registry).call().await;

    // Create a user with one linked provider
    pacman_server::data::user::create_user(
        &context.app_state.db,
        "existinguser",
        None,
        Some("existing@example.com"),
        None,
        "mock",
        "123",
    )
    .await
    .unwrap();

    // A user with the email exists, but has one provider. If they sign in with a *new* provider, it should fail.
    let response = context.server.get("/auth/mock_new/callback?code=a&state=b").await;
    assert_eq!(response.status_code(), 400); // Should fail and ask to link explicitly.
}

/// Test existing user sign-in with existing provider succeeds
#[tokio::test]
async fn test_existing_user_sign_in_existing_provider_succeeds() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_handle_callback().returning(move |_, _, _, _| {
        Ok(pacman_server::auth::provider::AuthUser {
            id: "123".to_string(), // Same provider ID as created user
            username: "existinguser".to_string(),
            name: None,
            email: Some("existing@example.com".to_string()),
            avatar_url: None,
        })
    });
    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let context = test_context().use_database(true).auth_registry(mock_registry).call().await;

    // Create a user with one linked provider
    pacman_server::data::user::create_user(
        &context.app_state.db,
        "existinguser",
        None,
        Some("existing@example.com"),
        None,
        "mock",
        "123",
    )
    .await
    .unwrap();

    // Test sign-in with an existing linked provider.
    let response = context.server.get("/auth/mock/callback?code=a&state=b").await;
    assert_eq!(response.status_code(), 302); // Should sign in successfully
    assert_eq!(response.headers().get("location").unwrap(), "/profile");
}

/// Test logout functionality
#[tokio::test]
async fn test_logout_functionality() {
    let mut mock = MockOAuthProvider::new();
    mock.expect_handle_callback().returning(|_, _, _, _| {
        Ok(pacman_server::auth::provider::AuthUser {
            id: "123".to_string(),
            username: "testuser".to_string(),
            name: None,
            email: Some("test@example.com".to_string()),
            avatar_url: None,
        })
    });
    let provider: Arc<dyn OAuthProvider> = Arc::new(mock);
    let mock_registry = AuthRegistry {
        providers: HashMap::from([("mock", provider)]),
    };

    let context = test_context().use_database(true).auth_registry(mock_registry).call().await;

    // Sign in to establish a session
    let response = context.server.get("/auth/mock/callback?code=a&state=b").await;
    assert_eq!(response.status_code(), 302);
    let session_cookie = response.cookie("session").clone();

    // Test that the logout handler clears the session cookie and redirects
    let response = context
        .server
        .get("/logout")
        .add_cookie(cookie::Cookie::new(
            session_cookie.name().to_string(),
            session_cookie.value().to_string(),
        ))
        .await;

    // Redirect assertions
    assert_eq!(response.status_code(), 302);
    assert!(
        response.headers().contains_key("location"),
        "Response redirect should have a location header"
    );

    // Cookie assertions
    assert_eq!(
        response.cookie("session").value(),
        "removed",
        "Session cookie should be removed"
    );
    assert_eq!(
        response.cookie("session").max_age(),
        Some(Duration::ZERO),
        "Session cookie should have a max age of 0"
    );
}
