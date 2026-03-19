use axum::{extract::Request, extract::State, http::StatusCode, middleware::Next, response::Response};
use sqlx::PgPool;

use super::auth::AuthUser;

pub async fn subscription_middleware(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_user = request
        .extensions()
        .get::<AuthUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?
        .clone();

    let has_active_sub: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM subscriptions WHERE user_id = $1 AND status = 'active' AND current_period_end > now())"
    )
    .bind(auth_user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !has_active_sub {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
