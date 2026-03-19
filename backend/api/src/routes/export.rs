use axum::extract::{Path, State};
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use storage::StorageClient;

#[derive(Clone)]
pub struct ExportState {
    pub pool: PgPool,
    pub storage: Arc<dyn StorageClient>,
}

#[derive(Deserialize)]
pub struct CreateExportRequest {
    pub format: String,
}

pub async fn create_export(
    Extension(auth): Extension<AuthUser>,
    State(state): State<ExportState>,
    Path(project_id): Path<uuid::Uuid>,
    Json(req): Json<CreateExportRequest>,
) -> Result<Json<shared::models::Export>, AppError> {
    if req.format != "pdf" && req.format != "docx" {
        return Err(AppError::bad_request("Format must be 'pdf' or 'docx'"));
    }

    // Verify ownership
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL)",
    )
    .bind(project_id)
    .bind(auth.user_id)
    .fetch_one(&state.pool)
    .await?;

    if !exists {
        return Err(AppError::not_found("Project not found"));
    }

    // Check all required sections exist
    let section_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM patent_sections WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_one(&state.pool)
    .await?;

    if section_count < 8 {
        let existing: Vec<String> = sqlx::query_scalar(
            "SELECT section_type FROM patent_sections WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await?;

        let all_types = shared::models::SECTION_TYPES;
        let missing: Vec<&str> = all_types
            .iter()
            .filter(|t| !existing.contains(&t.to_string()))
            .copied()
            .collect();

        return Err(AppError::bad_request(format!(
            "Missing sections: {}",
            missing.join(", ")
        )));
    }

    // Generate export (placeholder — real implementation uses typst/docx-rs)
    let content = format!("Patent export in {} format for project {}", req.format, project_id);
    let storage_key = format!("exports/{}/{}.{}", project_id, uuid::Uuid::new_v4(), req.format);
    let data = content.as_bytes();

    state.storage.upload(&storage_key, data, "application/octet-stream").await?;

    let export = sqlx::query_as::<_, shared::models::Export>(
        "INSERT INTO exports (project_id, format, storage_path, file_size_bytes) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(project_id)
    .bind(&req.format)
    .bind(&storage_key)
    .bind(data.len() as i64)
    .fetch_one(&state.pool)
    .await?;

    // Update project status
    sqlx::query("UPDATE projects SET status = 'exported' WHERE id = $1")
        .bind(project_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(export))
}

pub async fn list_exports(
    Extension(auth): Extension<AuthUser>,
    State(state): State<ExportState>,
    Path(project_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<shared::models::Export>>, AppError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL)",
    )
    .bind(project_id)
    .bind(auth.user_id)
    .fetch_one(&state.pool)
    .await?;

    if !exists {
        return Err(AppError::not_found("Project not found"));
    }

    let exports = sqlx::query_as::<_, shared::models::Export>(
        "SELECT * FROM exports WHERE project_id = $1 ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(exports))
}

pub async fn get_download_url(
    Extension(auth): Extension<AuthUser>,
    State(state): State<ExportState>,
    Path(export_id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let export = sqlx::query_as::<_, (uuid::Uuid, String)>(
        "SELECT e.project_id, e.storage_path FROM exports e
         JOIN projects p ON p.id = e.project_id
         WHERE e.id = $1 AND p.user_id = $2 AND p.deleted_at IS NULL",
    )
    .bind(export_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Export not found"))?;

    let url = state.storage.download_url(&export.1, 3600).await?;
    Ok(Json(serde_json::json!({ "url": url })))
}
