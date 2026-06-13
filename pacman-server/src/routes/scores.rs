use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::data::score::{self as score_repo, LeaderboardPeriod};
use crate::{app::AppState, errors::ErrorResponse};

use super::extractors::{AuthenticatedUser, RequireDatabase};

const DEFAULT_LIMIT: i64 = 10;
const MAX_LIMIT: i64 = 100;
/// Generous ceiling for a single submission; real anti-cheat lands with the game client.
const MAX_SCORE: i64 = 100_000_000;
const MAX_LEVEL_COUNT: i32 = 256;

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    period: Option<String>,
    limit: Option<i64>,
}

#[derive(Serialize)]
struct LeaderboardEntryResponse {
    rank: i64,
    user_id: i64,
    name: Option<String>,
    avatar: Option<String>,
    score: i64,
    level_count: i32,
    duration_ms: Option<i32>,
    submitted_at: DateTime<Utc>,
}

/// `GET /api/scores` &mdash; public ranked leaderboard, best score per user.
pub async fn list_scores_handler(
    State(app_state): State<AppState>,
    _db: RequireDatabase,
    Query(query): Query<LeaderboardQuery>,
) -> axum::response::Response {
    let period = match query.period.as_deref() {
        Some("monthly") => LeaderboardPeriod::Monthly,
        _ => LeaderboardPeriod::Global,
    };
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);

    match score_repo::top_scores(&app_state.db, period, limit).await {
        Ok(entries) => {
            let response: Vec<LeaderboardEntryResponse> = entries
                .into_iter()
                .enumerate()
                .map(|(index, entry)| LeaderboardEntryResponse {
                    rank: index as i64 + 1,
                    user_id: entry.user_id,
                    name: entry.name,
                    avatar: entry.avatar,
                    score: entry.score,
                    level_count: entry.level_count,
                    duration_ms: entry.duration_ms,
                    submitted_at: entry.submitted_at,
                })
                .collect();
            Json(response).into_response()
        }
        Err(e) => {
            warn!(error = %e, "Failed to fetch leaderboard");
            ErrorResponse::with_status(
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                Some("could not fetch leaderboard".into()),
            )
            .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct SubmitScoreRequest {
    score: i64,
    level_count: i32,
    duration_ms: Option<i32>,
}

/// `POST /api/scores` &mdash; submit a score for the authenticated user.
pub async fn submit_score_handler(
    State(app_state): State<AppState>,
    _db: RequireDatabase,
    user: AuthenticatedUser,
    Json(body): Json<SubmitScoreRequest>,
) -> axum::response::Response {
    if !(0..=MAX_SCORE).contains(&body.score) {
        return ErrorResponse::bad_request("invalid_score", Some("score is out of range".into())).into_response();
    }
    if !(1..=MAX_LEVEL_COUNT).contains(&body.level_count) {
        return ErrorResponse::bad_request("invalid_level_count", Some("level_count is out of range".into())).into_response();
    }
    if body.duration_ms.is_some_and(|d| d < 0) {
        return ErrorResponse::bad_request("invalid_duration", Some("duration_ms must be non-negative".into())).into_response();
    }

    match score_repo::insert_score(&app_state.db, user.user_id, body.score, body.level_count, body.duration_ms).await {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => {
            warn!(error = %e, "Failed to insert score");
            ErrorResponse::with_status(
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                Some("could not submit score".into()),
            )
            .into_response()
        }
    }
}
