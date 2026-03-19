use axum::extract::{Path, State};
use axum::response::sse::{Event, Sse};
use axum::Extension;
use futures::stream::Stream;
use sqlx::PgPool;
use std::convert::Infallible;
use std::sync::Arc;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use ai::pipeline::{GenerationPipeline, SseEvent};
use ai::LlmProvider;

#[derive(Clone)]
pub struct GenerateState {
    pub pool: PgPool,
    pub provider: Arc<dyn LlmProvider>,
}

pub async fn generate(
    Extension(auth): Extension<AuthUser>,
    State(state): State<GenerateState>,
    Path(project_id): Path<uuid::Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    // Verify ownership
    let project = sqlx::query_as::<_, (String,)>(
        "SELECT status FROM projects WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
    )
    .bind(project_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Project not found"))?;

    if project.0 == "generating" {
        return Err(AppError::bad_request("Generation already in progress"));
    }

    // Set status to generating
    sqlx::query("UPDATE projects SET status = 'generating' WHERE id = $1")
        .bind(project_id)
        .execute(&state.pool)
        .await?;

    // Load interview responses
    let responses = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT question_key, response_text FROM interview_responses WHERE project_id = $1 ORDER BY step_number",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await?;

    let interview_context = responses
        .iter()
        .filter_map(|(key, val)| val.as_ref().map(|v| format!("{}: {}", key, v)))
        .collect::<Vec<_>>()
        .join("\n\n");

    // Load figure descriptions
    let figures = sqlx::query_as::<_, (String,)>(
        "SELECT description FROM figures WHERE project_id = $1 ORDER BY sort_order",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await?;

    let figure_descriptions = figures
        .iter()
        .enumerate()
        .map(|(i, (desc,))| format!("Figure {}: {}", i + 1, desc))
        .collect::<Vec<_>>()
        .join("\n");

    // Load existing sections (for recovery)
    let existing = sqlx::query_as::<_, (String, String)>(
        "SELECT section_type, content FROM patent_sections WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await?;

    // Start pipeline
    let mut rx = GenerationPipeline::run(
        project_id,
        interview_context,
        figure_descriptions,
        existing,
        state.provider.as_ref(),
    )?;

    let pool = state.pool.clone();

    let stream = async_stream::stream! {
        while let Some(event) = rx.recv().await {
            // Save completed sections to DB
            if let SseEvent::SectionComplete { ref section_type, ref content } = event {
                let _ = sqlx::query(
                    "INSERT INTO patent_sections (project_id, section_type, content)
                     VALUES ($1, $2, $3)
                     ON CONFLICT (project_id, section_type) DO UPDATE SET content = EXCLUDED.content"
                )
                .bind(project_id)
                .bind(section_type)
                .bind(content)
                .execute(&pool)
                .await;
            }

            if let SseEvent::GenerationComplete { .. } = event {
                let _ = sqlx::query("UPDATE projects SET status = 'review' WHERE id = $1")
                    .bind(project_id)
                    .execute(&pool)
                    .await;
            }

            if let SseEvent::Error { .. } = event {
                let _ = sqlx::query("UPDATE projects SET status = 'interview_complete' WHERE id = $1")
                    .bind(project_id)
                    .execute(&pool)
                    .await;
            }

            let json = serde_json::to_string(&event).unwrap_or_default();
            yield Ok(Event::default().data(json));
        }
    };

    Ok(Sse::new(stream))
}
