use axum::extract::State;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::AuthUserWithRole;

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub user_id: uuid::Uuid,
    pub role: String,
}

#[derive(Serialize)]
pub struct UpdateRoleResponse {
    pub user_id: uuid::Uuid,
    pub role: String,
}

/// Admin-only endpoint to update a user's role.
pub async fn update_user_role(
    Extension(_auth): Extension<AuthUserWithRole>,
    State(pool): State<PgPool>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<UpdateRoleResponse>, AppError> {
    // Validate role value
    let valid_roles = ["inventor", "patent_agent", "admin"];
    if !valid_roles.contains(&req.role.as_str()) {
        return Err(AppError::bad_request(format!(
            "Invalid role '{}'. Must be one of: {}",
            req.role,
            valid_roles.join(", ")
        )));
    }

    // Safe: role is validated against allowlist above
    let query = format!(
        "UPDATE users SET role = '{}'::user_role WHERE id = $1 RETURNING id",
        req.role
    );
    let updated: Option<(uuid::Uuid,)> = sqlx::query_as(&query)
        .bind(req.user_id)
        .fetch_optional(&pool)
        .await?;

    match updated {
        Some((id,)) => Ok(Json(UpdateRoleResponse {
            user_id: id,
            role: req.role,
        })),
        None => Err(AppError::not_found("User not found")),
    }
}

#[derive(Serialize)]
pub struct UserListItem {
    pub id: uuid::Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
}

/// Admin-only endpoint to list all users with their roles.
pub async fn list_users(
    Extension(_auth): Extension<AuthUserWithRole>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<UserListItem>>, AppError> {
    let users = sqlx::query_as::<_, (uuid::Uuid, String, String, String)>(
        "SELECT id, email, full_name, role::text FROM users ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(
        users
            .into_iter()
            .map(|(id, email, full_name, role)| UserListItem {
                id,
                email,
                full_name,
                role,
            })
            .collect(),
    ))
}
