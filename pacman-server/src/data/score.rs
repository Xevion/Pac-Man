use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::Serialize;
use sqlx::FromRow;

use super::pool::PgPool;

/// Which slice of history a leaderboard query covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderboardPeriod {
    /// All scores ever submitted.
    Global,
    /// Scores submitted within the current calendar month (UTC).
    Monthly,
}

/// A single ranked leaderboard row: a user's best qualifying score, joined with
/// their display name and avatar from their most recently updated OAuth account.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct LeaderboardEntry {
    pub user_id: i64,
    pub score: i64,
    pub level_count: i32,
    pub duration_ms: Option<i32>,
    pub submitted_at: DateTime<Utc>,
    pub name: Option<String>,
    pub avatar: Option<String>,
}

/// Record a score submission for a user.
pub async fn insert_score(
    pool: &PgPool,
    user_id: i64,
    score: i64,
    level_count: i32,
    duration_ms: Option<i32>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO scores (user_id, score, level_count, duration_ms)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(user_id)
    .bind(score)
    .bind(level_count)
    .bind(duration_ms)
    .execute(pool)
    .await?;
    Ok(())
}

/// Fetch the leaderboard: each user's single best score for the period, ranked descending.
///
/// Ties on score break toward the earlier submission. Name and avatar come from the
/// user's most recently updated OAuth account (a left join, so users without one still rank).
pub async fn top_scores(pool: &PgPool, period: LeaderboardPeriod, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    let since = match period {
        LeaderboardPeriod::Global => None,
        LeaderboardPeriod::Monthly => {
            let now = Utc::now();
            Some(
                Utc.with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
                    .single()
                    .expect("first of month is a valid instant"),
            )
        }
    };

    sqlx::query_as::<_, LeaderboardEntry>(
        r#"
        SELECT user_id, score, level_count, duration_ms, submitted_at, name, avatar
        FROM (
            SELECT DISTINCT ON (s.user_id)
                s.user_id                                AS user_id,
                s.score                                  AS score,
                s.level_count                            AS level_count,
                s.duration_ms                            AS duration_ms,
                s.created_at                             AS submitted_at,
                COALESCE(oa.display_name, oa.username)   AS name,
                oa.avatar_url                            AS avatar
            FROM scores s
            LEFT JOIN LATERAL (
                SELECT display_name, username, avatar_url
                FROM oauth_accounts
                WHERE user_id = s.user_id
                ORDER BY updated_at DESC
                LIMIT 1
            ) oa ON true
            WHERE ($1::timestamptz IS NULL OR s.created_at >= $1)
            ORDER BY s.user_id, s.score DESC, s.created_at ASC
        ) ranked
        ORDER BY score DESC, submitted_at ASC
        LIMIT $2
        "#,
    )
    .bind(since)
    .bind(limit)
    .fetch_all(pool)
    .await
}
