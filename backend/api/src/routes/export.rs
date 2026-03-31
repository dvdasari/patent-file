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
    let project = sqlx::query_as::<_, shared::models::Project>(
        "SELECT * FROM projects WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
    )
    .bind(project_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Project not found"))?;

    // Load all sections
    let sections = sqlx::query_as::<_, shared::models::PatentSection>(
        "SELECT * FROM patent_sections WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await?;

    if sections.len() < 8 {
        let existing: Vec<&str> = sections.iter().map(|s| s.section_type.as_str()).collect();
        let missing: Vec<&str> = shared::models::SECTION_TYPES
            .iter()
            .filter(|t| !existing.contains(t))
            .copied()
            .collect();

        return Err(AppError::bad_request(format!(
            "Missing sections: {}",
            missing.join(", ")
        )));
    }

    // Load applicant info
    let applicant = sqlx::query_as::<_, shared::models::ProjectApplicant>(
        "SELECT * FROM project_applicants WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_optional(&state.pool)
    .await?;

    // Build patent document
    let section_pairs: Vec<(String, String)> = sections
        .iter()
        .map(|s| (s.section_type.clone(), s.content.clone()))
        .collect();

    let applicant_info = applicant.map(|a| export::ApplicantInfo {
        applicant_name: a.applicant_name,
        applicant_address: a.applicant_address,
        applicant_nationality: a.applicant_nationality,
        inventor_name: a.inventor_name,
        inventor_address: a.inventor_address,
        inventor_nationality: a.inventor_nationality,
        agent_name: a.agent_name,
        agent_registration_no: a.agent_registration_no,
        assignee_name: a.assignee_name,
        priority_date: a.priority_date,
        priority_country: a.priority_country,
        priority_application_no: a.priority_application_no,
    });

    let patent_doc = export::PatentDocument::from_sections(
        &section_pairs,
        applicant_info,
        &project.patent_type,
    )
    .map_err(|e| AppError::bad_request(format!("Failed to build document: {e}")))?;

    // Generate the document
    let (data, content_type) = match req.format.as_str() {
        "pdf" => {
            let bytes = export::generate_pdf(&patent_doc)
                .map_err(|e| AppError::internal(format!("PDF generation failed: {e}")))?;
            (bytes, "application/pdf")
        }
        "docx" => {
            // DOCX not yet implemented — return a placeholder
            let placeholder = format!("Patent DOCX export for project {project_id} — DOCX generation coming soon.");
            (placeholder.into_bytes(), "application/vnd.openxmlformats-officedocument.wordprocessingml.document")
        }
        _ => unreachable!(),
    };

    let storage_key = format!("exports/{}/{}.{}", project_id, uuid::Uuid::new_v4(), req.format);

    state.storage.upload(&storage_key, &data, content_type).await?;

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
