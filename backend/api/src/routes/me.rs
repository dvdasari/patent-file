use axum::extract::State;
use axum::{Extension, Json};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

#[derive(Serialize)]
pub struct MeResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub has_active_subscription: bool,
}

pub async fn get_me(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
) -> Result<Json<MeResponse>, AppError> {
    let row = sqlx::query_as::<_, (uuid::Uuid, String, String, String)>(
        "SELECT id, email, full_name, role::text FROM users WHERE id = $1",
    )
    .bind(auth.user_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("User not found"))?;

    let has_active_subscription: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM subscriptions WHERE user_id = $1 AND status = 'active' AND current_period_end > now())"
    )
    .bind(auth.user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    Ok(Json(MeResponse {
        id: row.0,
        email: row.1,
        full_name: row.2,
        role: row.3,
        has_active_subscription,
    }))
}
