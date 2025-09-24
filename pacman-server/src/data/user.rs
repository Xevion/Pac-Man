use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct User {
    pub id: i64,
    pub email: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn find_user_by_email(pool: &sqlx::PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
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
    pool: &sqlx::PgPool,
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
        DO UPDATE SET email = EXCLUDED.email, username = EXCLUDED.username, display_name = EXCLUDED.display_name, avatar_url = EXCLUDED.avatar_url, user_id = EXCLUDED.user_id, updated_at = NOW()
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

pub async fn create_user(pool: &sqlx::PgPool, email: Option<&str>) -> Result<User, sqlx::Error> {
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
    pool: &sqlx::PgPool,
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

pub async fn list_user_providers(pool: &sqlx::PgPool, user_id: i64) -> Result<Vec<ProviderPublic>, sqlx::Error> {
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
