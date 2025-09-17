use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct User {
    pub id: i64,
    pub provider: String,
    pub provider_user_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn upsert_user(
    pool: &sqlx::PgPool,
    provider: &str,
    provider_user_id: &str,
    username: &str,
    display_name: Option<&str>,
    email: Option<&str>,
    avatar_url: Option<&str>,
) -> Result<User, sqlx::Error> {
    let rec = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (provider, provider_user_id, username, display_name, email, avatar_url)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (provider, provider_user_id)
        DO UPDATE SET
            username = EXCLUDED.username,
            display_name = EXCLUDED.display_name,
            email = EXCLUDED.email,
            avatar_url = EXCLUDED.avatar_url,
            updated_at = NOW()
        RETURNING id, provider, provider_user_id, username, display_name, email, avatar_url, created_at, updated_at
        "#,
    )
    .bind(provider)
    .bind(provider_user_id)
    .bind(username)
    .bind(display_name)
    .bind(email)
    .bind(avatar_url)
    .fetch_one(pool)
    .await?;

    Ok(rec)
}

pub async fn get_user_by_provider_id(
    pool: &sqlx::PgPool,
    provider: &str,
    provider_user_id: &str,
) -> Result<Option<User>, sqlx::Error> {
    let rec = sqlx::query_as::<_, User>(
        r#"
        SELECT id, provider, provider_user_id, username, display_name, email, avatar_url, created_at, updated_at
        FROM users
        WHERE provider = $1 AND provider_user_id = $2
        "#,
    )
    .bind(provider)
    .bind(provider_user_id)
    .fetch_optional(pool)
    .await?;
    Ok(rec)
}
