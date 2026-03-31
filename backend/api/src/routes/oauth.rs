use axum::extract::{Path, Query, State};
use axum::http::header::SET_COOKIE;
use axum::response::{AppendHeaders, IntoResponse, Redirect};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::{hash_token, issue_access_token, issue_refresh_token};

#[derive(Clone)]
pub struct OAuthState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub linkedin_client_id: Option<String>,
    pub linkedin_client_secret: Option<String>,
    pub redirect_base_url: String,
    pub frontend_url: String,
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: String,
    name: String,
}

#[derive(Deserialize)]
struct LinkedInTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct LinkedInUserInfo {
    sub: String,
    email: String,
    name: String,
}

/// Redirect to OAuth provider's authorization page.
pub async fn oauth_redirect(
    State(state): State<OAuthState>,
    Path(provider): Path<String>,
) -> Result<Redirect, AppError> {
    let redirect_uri = format!(
        "{}/api/auth/oauth/{}/callback",
        state.redirect_base_url, provider
    );

    let url = match provider.as_str() {
        "google" => {
            let client_id = state
                .google_client_id
                .as_deref()
                .ok_or_else(|| AppError::internal("Google OAuth not configured"))?;
            format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&access_type=offline",
                urlencoding::encode(client_id),
                urlencoding::encode(&redirect_uri),
            )
        }
        "linkedin" => {
            let client_id = state
                .linkedin_client_id
                .as_deref()
                .ok_or_else(|| AppError::internal("LinkedIn OAuth not configured"))?;
            format!(
                "https://www.linkedin.com/oauth/v2/authorization?client_id={}&redirect_uri={}&response_type=code&scope=openid%20profile%20email",
                urlencoding::encode(client_id),
                urlencoding::encode(&redirect_uri),
            )
        }
        _ => return Err(AppError::bad_request("Unsupported OAuth provider")),
    };

    Ok(Redirect::temporary(&url))
}

/// Handle OAuth callback, create/link user, issue tokens, redirect to frontend.
/// On error, redirects to login page with error query param instead of returning JSON.
pub async fn oauth_callback(
    State(state): State<OAuthState>,
    Path(provider): Path<String>,
    Query(query): Query<CallbackQuery>,
) -> axum::response::Response {
    match oauth_callback_inner(&state, &provider, &query).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("OAuth callback error ({}): {}", provider, e.message);
            let error_url = format!(
                "{}/login?error={}",
                state.frontend_url,
                urlencoding::encode(&e.message)
            );
            Redirect::temporary(&error_url).into_response()
        }
    }
}

async fn oauth_callback_inner(
    state: &OAuthState,
    provider: &str,
    query: &CallbackQuery,
) -> Result<axum::response::Response, AppError> {
    let redirect_uri = format!(
        "{}/api/auth/oauth/{}/callback",
        state.redirect_base_url, provider
    );

    let (oauth_id, email, full_name) = match provider {
        "google" => fetch_google_user(state, &query.code, &redirect_uri).await?,
        "linkedin" => fetch_linkedin_user(state, &query.code, &redirect_uri).await?,
        _ => return Err(AppError::bad_request("Unsupported OAuth provider")),
    };

    // Find or create user, linking to existing email accounts
    let user_id = find_or_create_oauth_user(&state.pool, provider, &oauth_id, &email, &full_name)
        .await?;

    // Issue tokens (same as email/password login)
    let access_token = issue_access_token(&user_id, &state.jwt_secret)?;
    let refresh_token = issue_refresh_token(&user_id, &state.jwt_secret)?;

    let refresh_hash = hash_token(&refresh_token);
    let expires_at = Utc::now() + chrono::Duration::days(7);
    sqlx::query(
        "INSERT INTO sessions (user_id, refresh_token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&refresh_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    let access_cookie = format!(
        "access_token={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=86400",
        access_token
    );
    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly; Path=/api/auth; SameSite=Lax; Max-Age=604800",
        refresh_token
    );

    Ok((
        AppendHeaders([
            (SET_COOKIE, access_cookie),
            (SET_COOKIE, refresh_cookie),
        ]),
        Redirect::temporary(&format!("{}/projects", state.frontend_url)),
    )
        .into_response())
}

/// Exchange Google auth code for user info.
async fn fetch_google_user(
    state: &OAuthState,
    code: &str,
    redirect_uri: &str,
) -> Result<(String, String, String), AppError> {
    let client_id = state
        .google_client_id
        .as_deref()
        .ok_or_else(|| AppError::internal("Google OAuth not configured"))?;
    let client_secret = state
        .google_client_secret
        .as_deref()
        .ok_or_else(|| AppError::internal("Google OAuth not configured"))?;

    let http = reqwest::Client::new();

    // Exchange code for token
    let token_res = http
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| AppError::internal(format!("Google token exchange failed: {e}")))?;

    if !token_res.status().is_success() {
        let body = token_res.text().await.unwrap_or_default();
        tracing::error!("Google token exchange error: {}", body);
        return Err(AppError::unauthorized("Google authentication failed"));
    }

    let token: GoogleTokenResponse = token_res
        .json()
        .await
        .map_err(|e| AppError::internal(format!("Failed to parse Google token: {e}")))?;

    // Fetch user info
    let user_res = http
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(&token.access_token)
        .send()
        .await
        .map_err(|e| AppError::internal(format!("Google userinfo failed: {e}")))?;

    if !user_res.status().is_success() {
        return Err(AppError::unauthorized("Failed to fetch Google profile"));
    }

    let info: GoogleUserInfo = user_res
        .json()
        .await
        .map_err(|e| AppError::internal(format!("Failed to parse Google userinfo: {e}")))?;

    Ok((info.sub, info.email, info.name))
}

/// Exchange LinkedIn auth code for user info.
async fn fetch_linkedin_user(
    state: &OAuthState,
    code: &str,
    redirect_uri: &str,
) -> Result<(String, String, String), AppError> {
    let client_id = state
        .linkedin_client_id
        .as_deref()
        .ok_or_else(|| AppError::internal("LinkedIn OAuth not configured"))?;
    let client_secret = state
        .linkedin_client_secret
        .as_deref()
        .ok_or_else(|| AppError::internal("LinkedIn OAuth not configured"))?;

    let http = reqwest::Client::new();

    // Exchange code for token
    let token_res = http
        .post("https://www.linkedin.com/oauth/v2/accessToken")
        .form(&[
            ("code", code),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| AppError::internal(format!("LinkedIn token exchange failed: {e}")))?;

    if !token_res.status().is_success() {
        let body = token_res.text().await.unwrap_or_default();
        tracing::error!("LinkedIn token exchange error: {}", body);
        return Err(AppError::unauthorized("LinkedIn authentication failed"));
    }

    let token: LinkedInTokenResponse = token_res
        .json()
        .await
        .map_err(|e| AppError::internal(format!("Failed to parse LinkedIn token: {e}")))?;

    // Fetch user info via OpenID Connect userinfo endpoint
    let user_res = http
        .get("https://api.linkedin.com/v2/userinfo")
        .bearer_auth(&token.access_token)
        .send()
        .await
        .map_err(|e| AppError::internal(format!("LinkedIn userinfo failed: {e}")))?;

    if !user_res.status().is_success() {
        return Err(AppError::unauthorized("Failed to fetch LinkedIn profile"));
    }

    let info: LinkedInUserInfo = user_res
        .json()
        .await
        .map_err(|e| AppError::internal(format!("Failed to parse LinkedIn userinfo: {e}")))?;

    Ok((info.sub, info.email, info.name))
}

/// Find an existing user by OAuth provider+id or by email, or create a new one.
/// Handles account linking: if user exists by email, adds OAuth columns.
async fn find_or_create_oauth_user(
    pool: &PgPool,
    provider: &str,
    oauth_id: &str,
    email: &str,
    full_name: &str,
) -> Result<uuid::Uuid, AppError> {
    // 1. Check if a user already exists with this OAuth provider + id
    let existing_oauth: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM users WHERE oauth_provider = $1 AND oauth_provider_id = $2",
    )
    .bind(provider)
    .bind(oauth_id)
    .fetch_optional(pool)
    .await?;

    if let Some((id,)) = existing_oauth {
        return Ok(id);
    }

    // 2. Check if a user exists with this email (account linking)
    let existing_email: Option<(uuid::Uuid, Option<String>)> = sqlx::query_as(
        "SELECT id, oauth_provider FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    if let Some((id, existing_provider)) = existing_email {
        if existing_provider.is_some() {
            // Already linked to a different OAuth provider — we don't overwrite
            // Just log them in via the existing account
            return Ok(id);
        }
        // Link OAuth to existing email/password user
        sqlx::query(
            "UPDATE users SET oauth_provider = $1, oauth_provider_id = $2 WHERE id = $3",
        )
        .bind(provider)
        .bind(oauth_id)
        .bind(id)
        .execute(pool)
        .await?;
        return Ok(id);
    }

    // 3. Create new user (no password, OAuth only)
    let user_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO users (email, full_name, oauth_provider, oauth_provider_id) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(email)
    .bind(full_name)
    .bind(provider)
    .bind(oauth_id)
    .fetch_one(pool)
    .await?;

    Ok(user_id)
}
