use axum::extract::{Path, State};
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

#[derive(Deserialize)]
pub struct UpdateSectionRequest {
    pub content: String,
}

pub async fn update_section(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path((project_id, section_type)): Path<(uuid::Uuid, String)>,
    Json(req): Json<UpdateSectionRequest>,
) -> Result<Json<shared::models::PatentSection>, AppError> {
    if !shared::models::is_valid_section_type(&section_type) {
        return Err(AppError::bad_request(format!("Invalid section type: {}", section_type)));
    }

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

    // Get current section (to save version)
    let current = sqlx::query_as::<_, (uuid::Uuid, String, i32)>(
        "SELECT id, content, edit_count FROM patent_sections WHERE project_id = $1 AND section_type = $2",
    )
    .bind(project_id)
    .bind(&section_type)
    .fetch_optional(&pool)
    .await?;

    if let Some((section_id, old_content, edit_count)) = current {
        // Save current as version
        let next_version: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version_number), 0) + 1 FROM section_versions WHERE section_id = $1",
        )
        .bind(section_id)
        .fetch_one(&pool)
        .await?;

        sqlx::query(
            "INSERT INTO section_versions (section_id, content, version_number, source) VALUES ($1, $2, $3, 'manual')",
        )
        .bind(section_id)
        .bind(&old_content)
        .bind(next_version)
        .execute(&pool)
        .await?;

        // Update section
        let section = sqlx::query_as::<_, shared::models::PatentSection>(
            "UPDATE patent_sections SET content = $1, ai_generated = false, edit_count = $2
             WHERE id = $3 RETURNING *",
        )
        .bind(&req.content)
        .bind(edit_count + 1)
        .bind(section_id)
        .fetch_one(&pool)
        .await?;

        Ok(Json(section))
    } else {
        // Create new section (manual creation)
        let section = sqlx::query_as::<_, shared::models::PatentSection>(
            "INSERT INTO patent_sections (project_id, section_type, content, ai_generated)
             VALUES ($1, $2, $3, false) RETURNING *",
        )
        .bind(project_id)
        .bind(&section_type)
        .bind(&req.content)
        .fetch_one(&pool)
        .await?;

        Ok(Json(section))
    }
}

pub async fn list_versions(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path((project_id, section_type)): Path<(uuid::Uuid, String)>,
) -> Result<Json<Vec<shared::models::SectionVersion>>, AppError> {
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

    let section_id: uuid::Uuid = sqlx::query_scalar(
        "SELECT id FROM patent_sections WHERE project_id = $1 AND section_type = $2",
    )
    .bind(project_id)
    .bind(&section_type)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("Section not found"))?;

    let versions = sqlx::query_as::<_, shared::models::SectionVersion>(
        "SELECT * FROM section_versions WHERE section_id = $1 ORDER BY version_number DESC",
    )
    .bind(section_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(versions))
}

pub async fn restore_version(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path((project_id, section_type, version_number)): Path<(uuid::Uuid, String, i32)>,
) -> Result<Json<shared::models::PatentSection>, AppError> {
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

    let section = sqlx::query_as::<_, (uuid::Uuid, String, i32)>(
        "SELECT id, content, edit_count FROM patent_sections WHERE project_id = $1 AND section_type = $2",
    )
    .bind(project_id)
    .bind(&section_type)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("Section not found"))?;

    let (section_id, current_content, edit_count) = section;

    // Get the version to restore
    let version_content: String = sqlx::query_scalar(
        "SELECT content FROM section_versions WHERE section_id = $1 AND version_number = $2",
    )
    .bind(section_id)
    .bind(version_number)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("Version not found"))?;

    // Save current content as a new version before restoring
    let next_version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_number), 0) + 1 FROM section_versions WHERE section_id = $1",
    )
    .bind(section_id)
    .fetch_one(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO section_versions (section_id, content, version_number, source) VALUES ($1, $2, $3, 'manual')",
    )
    .bind(section_id)
    .bind(&current_content)
    .bind(next_version)
    .execute(&pool)
    .await?;

    // Restore
    let restored = sqlx::query_as::<_, shared::models::PatentSection>(
        "UPDATE patent_sections SET content = $1, edit_count = $2 WHERE id = $3 RETURNING *",
    )
    .bind(&version_content)
    .bind(edit_count + 1)
    .bind(section_id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(restored))
}
