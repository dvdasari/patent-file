use axum::extract::{Path, State};
use axum::{Extension, Json};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

#[derive(Deserialize)]
pub struct CreateDeadlineRequest {
    pub title: String,
    pub description: Option<String>,
    pub due_date: NaiveDate,
}

#[derive(Deserialize)]
pub struct UpdateDeadlineRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub status: Option<String>,
}

pub async fn create_deadline(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path(project_id): Path<uuid::Uuid>,
    Json(req): Json<CreateDeadlineRequest>,
) -> Result<Json<shared::models::Deadline>, AppError> {
    verify_project_ownership(&pool, project_id, auth.user_id).await?;

    let deadline = sqlx::query_as::<_, shared::models::Deadline>(
        "INSERT INTO deadlines (project_id, title, description, due_date) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(project_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(req.due_date)
    .fetch_one(&pool)
    .await?;

    Ok(Json(deadline))
}

pub async fn list_deadlines(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path(project_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<shared::models::Deadline>>, AppError> {
    verify_project_ownership(&pool, project_id, auth.user_id).await?;

    let deadlines = sqlx::query_as::<_, shared::models::Deadline>(
        "SELECT * FROM deadlines WHERE project_id = $1 ORDER BY due_date ASC",
    )
    .bind(project_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(deadlines))
}

pub async fn update_deadline(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path((project_id, deadline_id)): Path<(uuid::Uuid, uuid::Uuid)>,
    Json(req): Json<UpdateDeadlineRequest>,
) -> Result<Json<shared::models::Deadline>, AppError> {
    verify_project_ownership(&pool, project_id, auth.user_id).await?;

    if let Some(ref status) = req.status {
        if !["upcoming", "overdue", "completed"].contains(&status.as_str()) {
            return Err(AppError::bad_request(
                "Status must be 'upcoming', 'overdue', or 'completed'",
            ));
        }
    }

    let completed_at = match req.status.as_deref() {
        Some("completed") => Some(Utc::now()),
        Some(_) => None, // Clear completed_at when moving back to non-completed
        None => {
            // Keep existing value — fetch current
            sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>(
                "SELECT completed_at FROM deadlines WHERE id = $1 AND project_id = $2",
            )
            .bind(deadline_id)
            .bind(project_id)
            .fetch_optional(&pool)
            .await?
            .flatten()
        }
    };

    let deadline = sqlx::query_as::<_, shared::models::Deadline>(
        "UPDATE deadlines SET
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            due_date = COALESCE($3, due_date),
            status = COALESCE($4, status),
            completed_at = $5,
            updated_at = now()
         WHERE id = $6 AND project_id = $7
         RETURNING *",
    )
    .bind(req.title.as_deref())
    .bind(req.description.as_deref())
    .bind(req.due_date)
    .bind(req.status.as_deref())
    .bind(completed_at)
    .bind(deadline_id)
    .bind(project_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("Deadline not found"))?;

    Ok(Json(deadline))
}

pub async fn delete_deadline(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path((project_id, deadline_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_project_ownership(&pool, project_id, auth.user_id).await?;

    let rows = sqlx::query("DELETE FROM deadlines WHERE id = $1 AND project_id = $2")
        .bind(deadline_id)
        .bind(project_id)
        .execute(&pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::not_found("Deadline not found"));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn verify_project_ownership(
    pool: &PgPool,
    project_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<(), AppError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL)",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if !exists {
        return Err(AppError::not_found("Project not found"));
    }

    Ok(())
}
