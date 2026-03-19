use axum::extract::{Path, State};
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

#[derive(Deserialize)]
pub struct UpsertApplicantRequest {
    pub applicant_name: String,
    pub applicant_address: String,
    pub applicant_nationality: Option<String>,
    pub inventor_name: String,
    pub inventor_address: String,
    pub inventor_nationality: Option<String>,
    pub agent_name: Option<String>,
    pub agent_registration_no: Option<String>,
    pub assignee_name: Option<String>,
    pub priority_date: Option<String>,
    pub priority_country: Option<String>,
    pub priority_application_no: Option<String>,
}

pub async fn upsert_applicant(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path(project_id): Path<uuid::Uuid>,
    Json(req): Json<UpsertApplicantRequest>,
) -> Result<Json<shared::models::ProjectApplicant>, AppError> {
    // Verify project ownership
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

    let nationality = req.applicant_nationality.unwrap_or_else(|| "Indian".to_string());
    let inv_nationality = req.inventor_nationality.unwrap_or_else(|| "Indian".to_string());

    let priority_date: Option<chrono::NaiveDate> = req
        .priority_date
        .as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let applicant = sqlx::query_as::<_, shared::models::ProjectApplicant>(
        "INSERT INTO project_applicants (
            project_id, applicant_name, applicant_address, applicant_nationality,
            inventor_name, inventor_address, inventor_nationality,
            agent_name, agent_registration_no, assignee_name,
            priority_date, priority_country, priority_application_no
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
         ON CONFLICT (project_id) DO UPDATE SET
            applicant_name = EXCLUDED.applicant_name,
            applicant_address = EXCLUDED.applicant_address,
            applicant_nationality = EXCLUDED.applicant_nationality,
            inventor_name = EXCLUDED.inventor_name,
            inventor_address = EXCLUDED.inventor_address,
            inventor_nationality = EXCLUDED.inventor_nationality,
            agent_name = EXCLUDED.agent_name,
            agent_registration_no = EXCLUDED.agent_registration_no,
            assignee_name = EXCLUDED.assignee_name,
            priority_date = EXCLUDED.priority_date,
            priority_country = EXCLUDED.priority_country,
            priority_application_no = EXCLUDED.priority_application_no
         RETURNING *",
    )
    .bind(project_id)
    .bind(&req.applicant_name)
    .bind(&req.applicant_address)
    .bind(&nationality)
    .bind(&req.inventor_name)
    .bind(&req.inventor_address)
    .bind(&inv_nationality)
    .bind(&req.agent_name)
    .bind(&req.agent_registration_no)
    .bind(&req.assignee_name)
    .bind(priority_date)
    .bind(&req.priority_country)
    .bind(&req.priority_application_no)
    .fetch_one(&pool)
    .await?;

    Ok(Json(applicant))
}

pub async fn get_applicant(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path(project_id): Path<uuid::Uuid>,
) -> Result<Json<shared::models::ProjectApplicant>, AppError> {
    // Verify project ownership
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

    let applicant = sqlx::query_as::<_, shared::models::ProjectApplicant>(
        "SELECT * FROM project_applicants WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("Applicant details not found"))?;

    Ok(Json(applicant))
}
