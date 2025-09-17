use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_cookie::CookieManager;
use serde::Serialize;
use tracing::{debug, info, instrument, trace, warn};

use crate::data::user as user_repo;
use crate::{app::AppState, errors::ErrorResponse, session};

#[derive(Debug, serde::Deserialize)]
pub struct AuthQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AuthorizeQuery {
    pub link: Option<bool>,
}

#[instrument(skip_all, fields(provider = %provider))]
pub async fn oauth_authorize_handler(
    State(app_state): State<AppState>,
    Path(provider): Path<String>,
    Query(aq): Query<AuthorizeQuery>,
    cookie: CookieManager,
) -> axum::response::Response {
    let Some(prov) = app_state.auth.get(&provider) else {
        warn!(%provider, "Unknown OAuth provider");
        return ErrorResponse::bad_request("invalid_provider", Some(provider)).into_response();
    };
    trace!("Starting OAuth authorization");
    // Persist link intent using a short-lived cookie; callbacks won't carry our query params.
    if aq.link == Some(true) {
        cookie.add(
            axum_cookie::cookie::Cookie::builder("link", "1")
                .http_only(true)
                .same_site(axum_cookie::prelude::SameSite::Lax)
                .path("/")
                .build(),
        );
    }
    let resp = prov.authorize().await;
    trace!("Redirecting to provider authorization page");
    resp
}

#[instrument(skip_all, fields(provider = %provider))]
pub async fn oauth_callback_handler(
    State(app_state): State<AppState>,
    Path(provider): Path<String>,
    Query(params): Query<AuthQuery>,
    cookie: CookieManager,
) -> axum::response::Response {
    let Some(prov) = app_state.auth.get(&provider) else {
        warn!(%provider, "Unknown OAuth provider");
        return ErrorResponse::bad_request("invalid_provider", Some(provider)).into_response();
    };
    if let Some(error) = params.error {
        warn!(%provider, error = %error, desc = ?params.error_description, "OAuth callback returned an error");
        return ErrorResponse::bad_request(error, params.error_description).into_response();
    }
    let mut q = std::collections::HashMap::new();
    if let Some(v) = params.code {
        q.insert("code".to_string(), v);
    }
    if let Some(v) = params.state {
        q.insert("state".to_string(), v);
    }
    let user = match prov.handle_callback(&q).await {
        Ok(u) => u,
        Err(e) => {
            warn!(%provider, "OAuth callback handling failed");
            return e.into_response();
        }
    };
    // Linking or sign-in flow. Determine link intent from cookie (set at authorize time)
    let link_cookie = cookie.get("link").map(|c| c.value().to_string());
    if link_cookie.is_some() {
        cookie.remove("link");
    }
    let email = user.email.as_deref();
    let _db_user = if link_cookie.as_deref() == Some("1") {
        // Must be logged in already to link
        let Some(session_token) = session::get_session_token(&cookie) else {
            return ErrorResponse::bad_request("invalid_request", Some("must be signed in to link provider".into()))
                .into_response();
        };
        let Some(claims) = session::decode_jwt(&session_token, &app_state.jwt_decoding_key) else {
            return ErrorResponse::bad_request("invalid_request", Some("invalid session token".into())).into_response();
        };
        // Resolve current user from session
        let (cur_prov, cur_id) = claims.sub.split_once(':').unwrap_or(("", ""));
        let current_user = match user_repo::get_user_by_provider_id(&app_state.db, cur_prov, cur_id).await {
            Ok(Some(u)) => u,
            Ok(None) => {
                return ErrorResponse::bad_request("invalid_request", Some("current session user not found".into()))
                    .into_response();
            }
            Err(_) => {
                return ErrorResponse::with_status(StatusCode::INTERNAL_SERVER_ERROR, "database_error", None).into_response();
            }
        };

        // Link provider to current user
        if let Err(e) = user_repo::link_oauth_account(
            &app_state.db,
            current_user.id,
            &provider,
            &user.id,
            email,
            Some(&user.username),
            user.name.as_deref(),
            user.avatar_url.as_deref(),
        )
        .await
        {
            warn!(error = %e, %provider, "Failed to link OAuth account");
            return ErrorResponse::with_status(StatusCode::INTERNAL_SERVER_ERROR, "database_error", None).into_response();
        }
        current_user
    } else {
        // Normal sign-in: do NOT auto-link by email (security). If email exists, require linking flow.
        if let Some(e) = email {
            if let Ok(Some(existing)) = user_repo::get_user_by_email(&app_state.db, e).await {
                // Only block if the user already has at least one linked provider.
                // NOTE: We do not check whether providers are currently active. If a user has exactly one provider and it is inactive,
                // this may lock them out until the provider is reactivated or a manual admin link is performed.
                match user_repo::get_oauth_account_count_for_user(&app_state.db, existing.id).await {
                    Ok(count) if count > 0 => {
                        return ErrorResponse::bad_request(
                            "account_exists",
                            Some(format!(
                                "An account already exists for {}. Sign in with your existing provider, then visit /auth/{}?link=true to add this provider.",
                                e, provider
                            )),
                        )
                        .into_response();
                    }
                    Ok(_) => {
                        // No providers linked yet: safe to associate this provider
                        if let Err(e) = user_repo::link_oauth_account(
                            &app_state.db,
                            existing.id,
                            &provider,
                            &user.id,
                            email,
                            Some(&user.username),
                            user.name.as_deref(),
                            user.avatar_url.as_deref(),
                        )
                        .await
                        {
                            warn!(error = %e, %provider, "Failed to link OAuth account to existing user with no providers");
                            return ErrorResponse::with_status(StatusCode::INTERNAL_SERVER_ERROR, "database_error", None)
                                .into_response();
                        }
                        existing
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to count oauth accounts for user");
                        return ErrorResponse::with_status(StatusCode::INTERNAL_SERVER_ERROR, "database_error", None)
                            .into_response();
                    }
                }
            } else {
                // Create new user with email
                match user_repo::create_user(
                    &app_state.db,
                    &user.username,
                    user.name.as_deref(),
                    email,
                    user.avatar_url.as_deref(),
                    &provider,
                    &user.id,
                )
                .await
                {
                    Ok(u) => u,
                    Err(e) => {
                        warn!(error = %e, %provider, "Failed to create user");
                        return ErrorResponse::with_status(StatusCode::INTERNAL_SERVER_ERROR, "database_error", None)
                            .into_response();
                    }
                }
            }
        } else {
            // No email available: disallow sign-in for safety
            return ErrorResponse::bad_request(
                "invalid_request",
                Some("account has no email; sign in with a different provider".into()),
            )
            .into_response();
        }
    };
    let session_token = session::create_jwt_for_user(&provider, &user, &app_state.jwt_encoding_key);
    session::set_session_cookie(&cookie, &session_token);
    info!(%provider, "Signed in successfully");
    (StatusCode::FOUND, Redirect::to("/profile")).into_response()
}

#[instrument(skip_all)]
pub async fn profile_handler(State(app_state): State<AppState>, cookie: CookieManager) -> axum::response::Response {
    let Some(token_str) = session::get_session_token(&cookie) else {
        debug!("Missing session cookie");
        return ErrorResponse::unauthorized("missing session cookie").into_response();
    };
    let Some(claims) = session::decode_jwt(&token_str, &app_state.jwt_decoding_key) else {
        debug!("Invalid session token");
        return ErrorResponse::unauthorized("invalid session token").into_response();
    };
    // sub format: provider:provider_user_id
    let (prov, prov_user_id) = match claims.sub.split_once(':') {
        Some((p, id)) => (p, id),
        None => {
            debug!("Malformed session token subject");
            return ErrorResponse::unauthorized("invalid session token").into_response();
        }
    };
    match user_repo::get_user_by_provider_id(&app_state.db, prov, prov_user_id).await {
        Ok(Some(db_user)) => {
            // Include linked providers in the profile payload
            match user_repo::list_user_providers(&app_state.db, db_user.id).await {
                Ok(providers) => {
                    #[derive(Serialize)]
                    struct ProfilePayload<T> {
                        id: i64,
                        email: Option<String>,
                        providers: Vec<T>,
                        created_at: chrono::DateTime<chrono::Utc>,
                        updated_at: chrono::DateTime<chrono::Utc>,
                    }
                    let body = ProfilePayload {
                        id: db_user.id,
                        email: db_user.email.clone(),
                        providers,
                        created_at: db_user.created_at,
                        updated_at: db_user.updated_at,
                    };
                    axum::Json(body).into_response()
                }
                Err(e) => {
                    warn!(error = %e, "Failed to list user providers");
                    ErrorResponse::with_status(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "database_error",
                        Some("could not fetch providers".into()),
                    )
                    .into_response()
                }
            }
        }
        Ok(None) => {
            debug!("User not found for session");
            ErrorResponse::unauthorized("session not found").into_response()
        }
        Err(e) => {
            warn!(error = %e, "Failed to fetch user for session");
            ErrorResponse::with_status(
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                Some("could not fetch user".into()),
            )
            .into_response()
        }
    }
}

pub async fn logout_handler(State(app_state): State<AppState>, cookie: CookieManager) -> axum::response::Response {
    if let Some(token_str) = session::get_session_token(&cookie) {
        // Remove from in-memory sessions if present
        app_state.sessions.remove(&token_str);
    }
    session::clear_session_cookie(&cookie);
    info!("Signed out successfully");
    (StatusCode::FOUND, Redirect::to("/")).into_response()
}

#[derive(Serialize)]
struct ProviderInfo {
    id: &'static str,
    name: &'static str,
    active: bool,
}

pub async fn list_providers_handler(State(app_state): State<AppState>) -> axum::response::Response {
    let providers: Vec<ProviderInfo> = app_state
        .auth
        .values()
        .map(|provider| ProviderInfo {
            id: provider.id(),
            name: provider.label(),
            active: provider.active(),
        })
        .collect();
    axum::Json(providers).into_response()
}
