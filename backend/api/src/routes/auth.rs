use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::response::AppendHeaders;
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::{
    hash_token, issue_access_token, issue_refresh_token, validate_token, ISS_REFRESH,
};

#[derive(Debug, Clone)]
pub struct AuthState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub full_name: String,
    pub has_active_subscription: bool,
}

pub async fn login(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> Result<(AppendHeaders<[(http::HeaderName, String); 2]>, Json<LoginResponse>), AppError> {
    let user = sqlx::query_as::<_, (uuid::Uuid, String, String, String)>(
        "SELECT id, email, full_name, password_hash FROM users WHERE email = $1",
    )
    .bind(&req.email)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::unauthorized("Invalid email or password"))?;

    let (user_id, email, full_name, password_hash) = user;

    // Verify password
    let parsed_hash = argon2::PasswordHash::new(&password_hash)
        .map_err(|_| AppError::internal("Password hash error"))?;
    argon2::PasswordVerifier::verify_password(
        &argon2::Argon2::default(),
        req.password.as_bytes(),
        &parsed_hash,
    )
    .map_err(|_| AppError::unauthorized("Invalid email or password"))?;

    // Issue tokens
    let access_token = issue_access_token(&user_id, &state.jwt_secret)?;
    let refresh_token = issue_refresh_token(&user_id, &state.jwt_secret)?;

    // Store session with hashed refresh token
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

    // Check subscription
    let has_active_subscription: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM subscriptions WHERE user_id = $1 AND status = 'active' AND current_period_end > now())"
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(false);

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
        Json(LoginResponse {
            id: user_id,
            email,
            full_name,
            has_active_subscription,
        }),
    ))
}

pub async fn refresh(
    State(state): State<AuthState>,
    headers: http::HeaderMap,
) -> Result<AppendHeaders<[(http::HeaderName, String); 2]>, AppError> {
    // Extract refresh token from cookie
    let refresh_token = headers
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find_map(|c| c.trim().strip_prefix("refresh_token="))
                .map(|s| s.to_string())
        })
        .ok_or_else(|| AppError::unauthorized("No refresh token"))?;

    // Validate JWT and ensure it's a refresh token
    let claims = validate_token(&refresh_token, &state.jwt_secret)
        .map_err(|_| AppError::unauthorized("Invalid refresh token"))?;
    if !claims.is_refresh_token() {
        return Err(AppError::unauthorized("Not a refresh token"));
    }
    let user_id = claims.user_id()?;

    // Look up session by hash
    let refresh_hash = hash_token(&refresh_token);
    let session = sqlx::query_as::<_, (uuid::Uuid,)>(
        "SELECT id FROM sessions WHERE refresh_token_hash = $1 AND user_id = $2 AND NOT revoked AND expires_at > now()"
    )
    .bind(&refresh_hash)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::unauthorized("Session not found or revoked"))?;

    // Revoke old session
    sqlx::query("UPDATE sessions SET revoked = true WHERE id = $1")
        .bind(session.0)
        .execute(&state.pool)
        .await?;

    // Issue new tokens
    let new_access = issue_access_token(&user_id, &state.jwt_secret)?;
    let new_refresh = issue_refresh_token(&user_id, &state.jwt_secret)?;

    // Create new session
    let new_hash = hash_token(&new_refresh);
    let expires_at = Utc::now() + chrono::Duration::days(7);
    sqlx::query(
        "INSERT INTO sessions (user_id, refresh_token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&new_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    let access_cookie = format!(
        "access_token={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=86400",
        new_access
    );
    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly; Path=/api/auth; SameSite=Lax; Max-Age=604800",
        new_refresh
    );

    Ok(AppendHeaders([
        (SET_COOKIE, access_cookie),
        (SET_COOKIE, refresh_cookie),
    ]))
}

pub async fn logout(
    State(state): State<AuthState>,
    headers: http::HeaderMap,
) -> Result<AppendHeaders<[(http::HeaderName, String); 2]>, AppError> {
    // Revoke session if refresh token present
    if let Some(refresh_token) = headers
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find_map(|c| c.trim().strip_prefix("refresh_token="))
                .map(|s| s.to_string())
        })
    {
        let refresh_hash = hash_token(&refresh_token);
        let _ = sqlx::query(
            "UPDATE sessions SET revoked = true WHERE refresh_token_hash = $1 AND NOT revoked",
        )
        .bind(&refresh_hash)
        .execute(&state.pool)
        .await;
    }

    // Clear cookies
    let clear_access =
        "access_token=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0".to_string();
    let clear_refresh =
        "refresh_token=; HttpOnly; Path=/api/auth; SameSite=Lax; Max-Age=0".to_string();

    Ok(AppendHeaders([
        (SET_COOKIE, clear_access),
        (SET_COOKIE, clear_refresh),
    ]))
}
