use axum::extract::{Path, State};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectSummary {
    pub id: uuid::Uuid,
    pub title: String,
    pub status: String,
    pub jurisdiction: String,
    pub patent_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub title: String,
    pub patent_type: Option<String>,
    pub jurisdiction: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateProjectRequest {
    pub title: Option<String>,
    pub patent_type: Option<String>,
}

pub async fn list_projects(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<ProjectSummary>>, AppError> {
    let projects = sqlx::query_as::<_, ProjectSummary>(
        "SELECT id, title, status, jurisdiction, patent_type, created_at, updated_at
         FROM projects WHERE user_id = $1 AND deleted_at IS NULL
         ORDER BY updated_at DESC",
    )
    .bind(auth.user_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(projects))
}

pub async fn create_project(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<ProjectSummary>, AppError> {
    let patent_type = req.patent_type.unwrap_or_else(|| "complete".to_string());
    let jurisdiction = req.jurisdiction.unwrap_or_else(|| "IPO".to_string());

    let project = sqlx::query_as::<_, ProjectSummary>(
        "INSERT INTO projects (user_id, title, patent_type, jurisdiction)
         VALUES ($1, $2, $3, $4)
         RETURNING id, title, status, jurisdiction, patent_type, created_at, updated_at",
    )
    .bind(auth.user_id)
    .bind(&req.title)
    .bind(&patent_type)
    .bind(&jurisdiction)
    .fetch_one(&pool)
    .await?;

    Ok(Json(project))
}

pub async fn get_project(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path(project_id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project = sqlx::query_as::<_, ProjectSummary>(
        "SELECT id, title, status, jurisdiction, patent_type, created_at, updated_at
         FROM projects WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
    )
    .bind(project_id)
    .bind(auth.user_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("Project not found"))?;

    // Fetch sections if any
    let sections = sqlx::query_as::<_, shared::models::PatentSection>(
        "SELECT id, project_id, section_type, content, ai_generated, edit_count, created_at, updated_at
         FROM patent_sections WHERE project_id = $1
         ORDER BY ARRAY_POSITION(ARRAY['title','field_of_invention','background','summary','detailed_description','claims','abstract','drawings_description'], section_type)",
    )
    .bind(project_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(serde_json::json!({
        "project": project,
        "sections": sections,
    })))
}

pub async fn update_project(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path(project_id): Path<uuid::Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectSummary>, AppError> {
    // Verify ownership
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL)",
    )
    .bind(project_id)
    .bind(auth.user_id)
    .fetch_one(&pool)
    .await?;

    if !exists {
        return Err(AppError::not_found("Project not found"));
    }

    if let Some(title) = &req.title {
        sqlx::query("UPDATE projects SET title = $1 WHERE id = $2")
            .bind(title)
            .bind(project_id)
            .execute(&pool)
            .await?;
    }
    if let Some(patent_type) = &req.patent_type {
        sqlx::query("UPDATE projects SET patent_type = $1 WHERE id = $2")
            .bind(patent_type)
            .bind(project_id)
            .execute(&pool)
            .await?;
    }

    let project = sqlx::query_as::<_, ProjectSummary>(
        "SELECT id, title, status, jurisdiction, patent_type, created_at, updated_at
         FROM projects WHERE id = $1",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(project))
}

pub async fn delete_project(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path(project_id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query(
        "UPDATE projects SET deleted_at = now() WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
    )
    .bind(project_id)
    .bind(auth.user_id)
    .execute(&pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found("Project not found"));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}
