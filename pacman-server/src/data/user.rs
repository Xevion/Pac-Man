use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use tracing::debug;

use crate::auth::provider::AuthUser;

use super::pool::PgPool;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct User {
    pub id: i64,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct OAuthAccount {
    pub id: i64,
    pub user_id: i64,
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, created_at, updated_at
        FROM users WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn link_oauth_account(
    pool: &PgPool,
    user_id: i64,
    provider: &str,
    provider_user_id: &str,
    email: Option<&str>,
    username: Option<&str>,
    display_name: Option<&str>,
    avatar_url: Option<&str>,
) -> Result<OAuthAccount, sqlx::Error> {
    sqlx::query_as::<_, OAuthAccount>(
        r#"
        INSERT INTO oauth_accounts (user_id, provider, provider_user_id, email, username, display_name, avatar_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (provider, provider_user_id)
        DO UPDATE SET email = EXCLUDED.email, username = EXCLUDED.username, display_name = EXCLUDED.display_name, avatar_url = EXCLUDED.avatar_url, user_id = EXCLUDED.user_id, updated_at = CURRENT_TIMESTAMP
        RETURNING id, user_id, provider, provider_user_id, email, username, display_name, avatar_url, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(provider)
    .bind(provider_user_id)
    .bind(email)
    .bind(username)
    .bind(display_name)
    .bind(avatar_url)
    .fetch_one(pool)
    .await
}

pub async fn create_user(pool: &PgPool, email: Option<&str>) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email)
        VALUES ($1)
        ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email
        RETURNING id, email, created_at, updated_at
        "#,
    )
    .bind(email)
    .fetch_one(pool)
    .await
}

pub async fn find_user_by_provider_id(
    pool: &PgPool,
    provider: &str,
    provider_user_id: &str,
) -> Result<Option<User>, sqlx::Error> {
    let rec = sqlx::query_as::<_, User>(
        r#"
        SELECT u.id, u.email, u.created_at, u.updated_at
        FROM users u
        JOIN oauth_accounts oa ON oa.user_id = u.id
        WHERE oa.provider = $1 AND oa.provider_user_id = $2
        "#,
    )
    .bind(provider)
    .bind(provider_user_id)
    .fetch_optional(pool)
    .await?;
    Ok(rec)
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProviderPublic {
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

pub async fn list_user_providers(pool: &PgPool, user_id: i64) -> Result<Vec<ProviderPublic>, sqlx::Error> {
    let recs = sqlx::query_as::<_, ProviderPublic>(
        r#"
        SELECT provider, provider_user_id, email, username, display_name, avatar_url
        FROM oauth_accounts
        WHERE user_id = $1
        ORDER BY provider
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(recs)
}

/// Find an existing user or create a new one, then link the OAuth provider account.
///
/// Linking strategy:
/// 1. If a user already exists with this provider+ID, return them.
/// 2. If the email is verified, try to find a user with that email and link.
/// 3. Otherwise, create a new user.
pub async fn find_or_create_user_for_oauth(pool: &PgPool, provider: &str, auth_user: &AuthUser) -> Result<User, sqlx::Error> {
    // 1. Check if we already have this specific provider account linked
    if let Some(user) = find_user_by_provider_id(pool, provider, &auth_user.id).await? {
        debug!(user_id = %user.id, "Found existing user by provider ID");
        return Ok(user);
    }

    // 2. If not, try to find an existing user by verified email to link to
    let user_to_link = if auth_user.email_verified {
        if let Some(email) = auth_user.email.as_deref() {
            if let Some(existing_user) = find_user_by_email(pool, email).await? {
                debug!(user_id = %existing_user.id, "Found existing user by email, linking new provider");
                existing_user
            } else {
                debug!("No user found by email, creating a new one");
                create_user(pool, Some(email)).await?
            }
        } else {
            create_user(pool, None).await?
        }
    } else {
        debug!("No verified email, creating a new user");
        create_user(pool, None).await?
    };

    // 3. Link the new provider account to our user record (whether old or new)
    link_oauth_account(
        pool,
        user_to_link.id,
        provider,
        &auth_user.id,
        auth_user.email.as_deref(),
        Some(&auth_user.username),
        auth_user.name.as_deref(),
        auth_user.avatar_url.as_deref(),
    )
    .await?;

    Ok(user_to_link)
}
