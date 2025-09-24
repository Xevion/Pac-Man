use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_cookie::CookieManager;
use jsonwebtoken::{encode, Algorithm, Header};
use serde::Serialize;
use tracing::{debug, debug_span, info, instrument, trace, warn};

use crate::data::user as user_repo;
use crate::{app::AppState, errors::ErrorResponse, session};

#[derive(Debug, serde::Deserialize)]
pub struct OAuthCallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AuthorizeQuery {
    pub link: Option<bool>,
}

/// Handles the beginning of the OAuth authorization flow.
///
/// Requires the `provider` path parameter, which determines the OAuth provider to use.
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

    let is_linking = aq.link == Some(true);

    // Persist link intent using a short-lived cookie; callbacks won't carry our query params.
    if is_linking {
        cookie.add(
            axum_cookie::cookie::Cookie::builder("link", "1")
                .http_only(true)
                .same_site(axum_cookie::prelude::SameSite::Lax)
                .path("/")
                // TODO: Pick a reasonable max age that aligns with how long OAuth providers can successfully complete the flow.
                .max_age(std::time::Duration::from_secs(60 * 60))
                .build(),
        );
    }
    trace!(linking = %is_linking, "Starting OAuth authorization");

    // Try to acquire the existing session (PKCE session is ignored)
    let existing_session = match session::get_session_token(&cookie) {
        Some(token) => match session::decode_jwt(&token, &app_state.jwt_decoding_key) {
            Some(claims) if !session::is_pkce_session(&claims) => Some(claims),
            Some(_) => {
                debug!("existing session ignored; it is a PKCE session");
                None
            }
            None => {
                debug!("invalid session token");
                return ErrorResponse::unauthorized("invalid session token").into_response();
            }
        },
        None => {
            debug!("missing session cookie");
            return ErrorResponse::unauthorized("missing session cookie").into_response();
        }
    };

    // If linking is enabled, error if the session doesn't exist or is a PKCE session
    if is_linking && existing_session.is_none() {
        warn!("missing session cookie during linking flow, refusing");
        return ErrorResponse::unauthorized("missing session cookie").into_response();
    }

    let auth_info = match prov.authorize(&app_state.jwt_encoding_key).await {
        Ok(info) => info,
        Err(e) => return e.into_response(),
    };

    let final_token = if let Some(mut claims) = existing_session {
        // We have a user session and are linking. Merge PKCE info into it.
        if let Some(pkce_claims) = session::decode_jwt(&auth_info.session_token, &app_state.jwt_decoding_key) {
            claims.pkce_verifier = pkce_claims.pkce_verifier;
            claims.csrf_state = pkce_claims.csrf_state;

            // re-encode
            encode(&Header::new(Algorithm::HS256), &claims, &app_state.jwt_encoding_key).expect("jwt sign")
        } else {
            warn!("Failed to decode PKCE session token during linking flow");
            // Fallback to just using the PKCE token, which will break linking but not panic.
            auth_info.session_token
        }
    } else {
        // Not linking or no existing session, just use the new token.
        auth_info.session_token
    };

    session::set_session_cookie(&cookie, &final_token);
    trace!("Redirecting to provider authorization page");
    Redirect::to(auth_info.authorize_url.as_str()).into_response()
}

/// Handles the callback from the OAuth provider after the user has authorized the app.
///
/// Requires the `provider` path parameter, which determines the OAuth provider to use for finishing the OAuth flow.
/// Requires the `code` and `state` query parameters, which are returned by the OAuth provider after the user has authorized the app.
pub async fn oauth_callback_handler(
    State(app_state): State<AppState>,
    Path(provider): Path<String>,
    Query(params): Query<OAuthCallbackParams>,
    cookie: CookieManager,
) -> axum::response::Response {
    // Validate provider
    let Some(prov) = app_state.auth.get(&provider) else {
        warn!(%provider, "Unknown OAuth provider");
        return ErrorResponse::bad_request("invalid_provider", Some(provider)).into_response();
    };

    // Process callback-returned errors from provider
    if let Some(error) = params.error {
        warn!(%provider, error = %error, desc = ?params.error_description, "OAuth callback returned an error");
        return ErrorResponse::bad_request(error, params.error_description).into_response();
    }

    // Acquire required parameters
    let Some(code) = params.code.as_deref() else {
        return ErrorResponse::bad_request("invalid_request", Some("missing code".into())).into_response();
    };
    let Some(state) = params.state.as_deref() else {
        return ErrorResponse::bad_request("invalid_request", Some("missing state".into())).into_response();
    };

    debug_span!("oauth_callback_handler",  provider = %provider, code = %code, state = %state);

    // Handle callback from provider
    let user = match prov.handle_callback(code, state, &cookie, &app_state.jwt_decoding_key).await {
        Ok(u) => u,
        Err(e) => {
            warn!(%provider, "OAuth callback handling failed");
            return e.into_response();
        }
    };

    debug!(cookies = ?cookie.cookie().iter().collect::<Vec<_>>(), "Cookies");

    // Linking or sign-in flow. Determine link intent from cookie (set at authorize time)
    let link_cookie = cookie.get("link").map(|c| c.value().to_string());
    if link_cookie.is_some() {
        cookie.remove("link");
    }
    let email = user.email.as_deref();

    // Determine linking intent with a valid session
    if link_cookie.as_deref() == Some("1") {
        debug!("Link intent present");

        if let Some(claims) =
            session::get_session_token(&cookie).and_then(|t| session::decode_jwt(&t, &app_state.jwt_decoding_key))
        {
            // Perform linking with current session user
            let (cur_prov, cur_id) = claims.subject.split_once(':').unwrap_or(("", ""));
            let current_user = match user_repo::find_user_by_provider_id(&app_state.db, cur_prov, cur_id).await {
                Ok(Some(u)) => u,
                Ok(None) => {
                    warn!("Current session user not found; proceeding as normal sign-in");
                    return ErrorResponse::bad_request("invalid_request", Some("current session user not found".into()))
                        .into_response();
                }
                Err(_) => {
                    return ErrorResponse::with_status(StatusCode::INTERNAL_SERVER_ERROR, "database_error", None).into_response();
                }
            };
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
            return (StatusCode::FOUND, Redirect::to("/profile")).into_response();
        } else {
            warn!(%provider, "Link intent present but session missing/invalid; proceeding as normal sign-in");
        }
    }

    // Normal sign-in: do NOT auto-link by email (security). If email exists, require linking flow.
    if let Some(e) = email {
        if let Ok(Some(existing)) = user_repo::find_user_by_email(&app_state.db, e).await {
            // Only block if the user already has at least one linked provider.
            // NOTE: We do not check whether providers are currently active. If a user has exactly one provider and it is inactive,
            // this may lock them out until the provider is reactivated or a manual admin link is performed.
            match user_repo::get_oauth_account_count_for_user(&app_state.db, existing.id).await {
                Ok(count) if count > 0 => {
                    // Check if the "new" provider is already linked to the user
                    match user_repo::find_user_by_provider_id(&app_state.db, &provider, &user.id).await {
                        Ok(Some(_)) => {
                            debug!(
                                %provider,
                                %existing.id,
                                "Provider already linked to user, signing in normally");
                        }
                        Ok(None) => {
                            debug!(
                                %provider,
                                %existing.id,
                                "Provider not linked to user, failing"
                            );
                            return ErrorResponse::bad_request(
                                "account_exists",
                                Some(format!(
                                    "An account already exists for {}. Sign in with your existing provider, then visit /auth/{}?link=true to add this provider.",
                                    e, provider
                                )),
                            )
                            .into_response();
                        }
                        Err(e) => {
                            warn!(error = %e, %provider, "Failed to find user by provider ID");
                            return ErrorResponse::with_status(StatusCode::INTERNAL_SERVER_ERROR, "database_error", None)
                                .into_response();
                        }
                    }
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
                }
                Err(e) => {
                    warn!(error = %e, "Failed to count oauth accounts for user");
                    return ErrorResponse::with_status(StatusCode::INTERNAL_SERVER_ERROR, "database_error", None).into_response();
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
                    return ErrorResponse::with_status(StatusCode::INTERNAL_SERVER_ERROR, "database_error", None).into_response();
                }
            };
        }
    } else {
        // No email available: disallow sign-in for safety
        return ErrorResponse::bad_request(
            "invalid_request",
            Some("account has no email; sign in with a different provider".into()),
        )
        .into_response();
    }

    // Create session token
    let session_token = session::create_jwt_for_user(&provider, &user, &app_state.jwt_encoding_key);
    session::set_session_cookie(&cookie, &session_token);
    info!(%provider, "Signed in successfully");

    // Process avatar asynchronously (don't block the response)
    if let Some(avatar_url) = user.avatar_url.as_deref() {
        let image_storage = app_state.image_storage.clone();
        let user_public_id = user.id.clone();
        let avatar_url = avatar_url.to_string();
        debug!(%user_public_id, %avatar_url, "Processing avatar");

        tokio::spawn(async move {
            match image_storage.process_avatar(&user_public_id, &avatar_url).await {
                Ok(avatar_urls) => {
                    info!(
                        user_id = %user_public_id,
                        original_url = %avatar_urls.original_url,
                        mini_url = %avatar_urls.mini_url,
                        "Avatar processed successfully"
                    );
                }
                Err(e) => {
                    warn!(
                        user_id = %user_public_id,
                        avatar_url = %avatar_url,
                        error = %e,
                        "Failed to process avatar"
                    );
                }
            }
        });
    }

    (StatusCode::FOUND, Redirect::to("/profile")).into_response()
}

/// Handles the request to the profile endpoint.
///
/// Requires the `session` cookie to be present.
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
    let (prov, prov_user_id) = match claims.subject.split_once(':') {
        Some((p, id)) => (p, id),
        None => {
            debug!("Malformed session token subject");
            return ErrorResponse::unauthorized("invalid session token").into_response();
        }
    };
    match user_repo::find_user_by_provider_id(&app_state.db, prov, prov_user_id).await {
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

pub async fn health_handler(
    State(app_state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> axum::response::Response {
    // Force health check in debug mode
    #[cfg(debug_assertions)]
    if params.contains_key("force") {
        app_state.check_health().await;
    }

    let ok = app_state.health.read().await.ok();
    let status = if ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    let body = serde_json::json!({ "ok": ok });
    (status, axum::Json(body)).into_response()
}
