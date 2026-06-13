mod common;

#[cfg(feature = "postgres-tests")]
mod scores {
    use crate::common::{test_context, TestContext};
    use cookie::Cookie;
    use pacman_server::{auth::provider::AuthUser, data::user as user_repo, session};
    use pretty_assertions::assert_eq;
    use serde_json::{json, Value};

    /// Create a user with a linked OAuth account and return a session cookie for them.
    async fn authenticated_cookie(context: &TestContext, provider_user_id: &str, display_name: &str) -> Cookie<'static> {
        let user = user_repo::create_user(&context.app_state.db, Some(&format!("{provider_user_id}@example.com")))
            .await
            .unwrap();
        let account = user_repo::link_oauth_account(
            &context.app_state.db,
            user.id,
            "test_provider",
            provider_user_id,
            user.email.as_deref(),
            Some("testuser"),
            Some(display_name),
            Some("https://example.com/avatar.png"),
        )
        .await
        .unwrap();

        let auth_user = AuthUser {
            id: account.provider_user_id,
            username: account.username.unwrap(),
            name: account.display_name,
            email: user.email,
            email_verified: true,
            avatar_url: account.avatar_url,
        };
        let token = session::create_jwt_for_user("test_provider", &auth_user, &context.app_state.jwt_encoding_key);
        Cookie::new(session::SESSION_COOKIE_NAME, token)
    }

    #[tokio::test]
    async fn empty_leaderboard_returns_empty_array() {
        let context = test_context().use_database(true).call().await;

        let response = context.server.get("/api/scores").await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        assert_eq!(body.as_array().expect("array response").len(), 0);
    }

    #[tokio::test]
    async fn submit_requires_authentication() {
        let context = test_context().use_database(true).call().await;

        let response = context
            .server
            .post("/api/scores")
            .json(&json!({ "score": 1000, "level_count": 1, "duration_ms": 5000 }))
            .await;

        assert_eq!(response.status_code(), 401);
    }

    #[tokio::test]
    async fn submitted_score_appears_on_leaderboard() {
        let context = test_context().use_database(true).call().await;
        let cookie = authenticated_cookie(&context, "100", "Wakka Wakka").await;

        let submit = context
            .server
            .post("/api/scores")
            .add_cookie(cookie)
            .json(&json!({ "score": 12345, "level_count": 3, "duration_ms": 60000 }))
            .await;
        assert_eq!(submit.status_code(), 201);

        let board = context.server.get("/api/scores").await;
        assert_eq!(board.status_code(), 200);
        let entries = board.json::<Value>();
        let entries = entries.as_array().expect("array response");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["score"], 12345);
        assert_eq!(entries[0]["level_count"], 3);
        assert_eq!(entries[0]["name"], "Wakka Wakka");
        assert_eq!(entries[0]["rank"], 1);
    }

    #[tokio::test]
    async fn leaderboard_shows_only_best_score_per_user() {
        let context = test_context().use_database(true).call().await;
        let cookie = authenticated_cookie(&context, "200", "Repeat Player").await;

        for score in [1000, 5000, 2500] {
            let response = context
                .server
                .post("/api/scores")
                .add_cookie(cookie.clone())
                .json(&json!({ "score": score, "level_count": 1, "duration_ms": 1000 }))
                .await;
            assert_eq!(response.status_code(), 201);
        }

        let board = context.server.get("/api/scores").await;
        let entries = board.json::<Value>();
        let entries = entries.as_array().expect("array response");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["score"], 5000);
    }

    #[tokio::test]
    async fn rejects_negative_score() {
        let context = test_context().use_database(true).call().await;
        let cookie = authenticated_cookie(&context, "300", "Cheater").await;

        let response = context
            .server
            .post("/api/scores")
            .add_cookie(cookie)
            .json(&json!({ "score": -500, "level_count": 1, "duration_ms": 1000 }))
            .await;

        assert_eq!(response.status_code(), 400);
    }
}
