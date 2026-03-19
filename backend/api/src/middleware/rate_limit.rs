use axum::{extract::Request, extract::State, http::StatusCode, middleware::Next, response::Response};
use chrono::Utc;
use sqlx::PgPool;

use super::auth::AuthUser;

pub struct RateLimitConfig {
    pub action_type: String,
    pub max_requests: i32,
}

pub fn generate_rate_limiter(action: &str, max: i32) -> RateLimitConfig {
    RateLimitConfig {
        action_type: action.to_string(),
        max_requests: max,
    }
}

pub async fn check_rate_limit(
    pool: &PgPool,
    user_id: uuid::Uuid,
    action_type: &str,
    max_requests: i32,
) -> Result<(), StatusCode> {
    let window_start = Utc::now()
        .date_naive()
        .and_hms_opt(Utc::now().time().hour().into(), 0, 0)
        .unwrap()
        .and_utc();

    let count: Option<i32> = sqlx::query_scalar(
        "SELECT request_count FROM rate_limits WHERE user_id = $1 AND action_type = $2 AND window_start = $3"
    )
    .bind(user_id)
    .bind(action_type)
    .bind(window_start)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(c) = count {
        if c >= max_requests {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    // Upsert increment
    sqlx::query(
        "INSERT INTO rate_limits (user_id, action_type, window_start, request_count)
         VALUES ($1, $2, $3, 1)
         ON CONFLICT (user_id, action_type, window_start)
         DO UPDATE SET request_count = rate_limits.request_count + 1"
    )
    .bind(user_id)
    .bind(action_type)
    .bind(window_start)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

use chrono::Timelike;

pub async fn rate_limit_generate(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_user = request.extensions().get::<AuthUser>().ok_or(StatusCode::UNAUTHORIZED)?.clone();
    check_rate_limit(&pool, auth_user.user_id, "generate", 5).await?;
    Ok(next.run(request).await)
}

pub async fn rate_limit_regenerate(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_user = request.extensions().get::<AuthUser>().ok_or(StatusCode::UNAUTHORIZED)?.clone();
    check_rate_limit(&pool, auth_user.user_id, "regenerate", 20).await?;
    Ok(next.run(request).await)
}

pub async fn rate_limit_export(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_user = request.extensions().get::<AuthUser>().ok_or(StatusCode::UNAUTHORIZED)?.clone();
    check_rate_limit(&pool, auth_user.user_id, "export", 10).await?;
    Ok(next.run(request).await)
}
