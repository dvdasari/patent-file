use axum::extract::{Path, State};
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

#[derive(Deserialize)]
pub struct InterviewResponseInput {
    pub step_number: i32,
    pub question_key: String,
    pub question_text: String,
    pub response_text: Option<String>,
}

#[derive(Deserialize)]
pub struct SaveInterviewRequest {
    pub responses: Vec<InterviewResponseInput>,
}

pub async fn save_interview(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path(project_id): Path<uuid::Uuid>,
    Json(req): Json<SaveInterviewRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
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

    for r in &req.responses {
        sqlx::query(
            "INSERT INTO interview_responses (project_id, step_number, question_key, question_text, response_text)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (project_id, question_key) DO UPDATE SET
                step_number = EXCLUDED.step_number,
                question_text = EXCLUDED.question_text,
                response_text = EXCLUDED.response_text",
        )
        .bind(project_id)
        .bind(r.step_number)
        .bind(&r.question_key)
        .bind(&r.question_text)
        .bind(&r.response_text)
        .execute(&pool)
        .await?;
    }

    Ok(Json(serde_json::json!({
        "saved": req.responses.len(),
    })))
}

pub async fn get_interview(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path(project_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<shared::models::InterviewResponse>>, AppError> {
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

    let responses = sqlx::query_as::<_, shared::models::InterviewResponse>(
        "SELECT * FROM interview_responses WHERE project_id = $1 ORDER BY step_number, question_key",
    )
    .bind(project_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(responses))
}
