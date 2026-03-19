use axum::extract::State;
use axum::{Extension, Json};
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

pub async fn create_subscription(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    // TODO: Call Razorpay API to create subscription
    // For now, return a placeholder subscription ID
    Ok(Json(serde_json::json!({
        "subscription_id": format!("sub_placeholder_{}", auth.user_id),
        "message": "Razorpay integration pending — use seed-user for local dev"
    })))
}

pub async fn subscription_status(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let sub = sqlx::query_as::<_, shared::models::Subscription>(
        "SELECT * FROM subscriptions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(auth.user_id)
    .fetch_optional(&pool)
    .await?;

    match sub {
        Some(s) => Ok(Json(serde_json::json!({
            "status": s.status,
            "plan_id": s.plan_id,
            "current_period_start": s.current_period_start,
            "current_period_end": s.current_period_end,
        }))),
        None => Ok(Json(serde_json::json!({
            "status": "none",
        }))),
    }
}
