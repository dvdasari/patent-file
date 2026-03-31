use axum::extract::{Extension, Path, State};
use axum::response::sse::{Event, Sse};
use axum::Json;
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

use fer::models::{
    CreateFerRequest, FerAnalysis, FerAnalysisDetail, FerObjection, FerResponse,
    ObjectionWithResponse, UpdateResponseRequest,
};
use fer::pipeline::{FerResponsePipeline, FerSseEvent};
use ai::LlmProvider;

#[derive(Clone)]
pub struct FerState {
    pub pool: sqlx::PgPool,
    pub provider: Arc<dyn LlmProvider>,
}

/// POST /api/fer — Create a new FER analysis and start parsing
pub async fn create_fer(
    Extension(auth): Extension<AuthUser>,
    State(state): State<FerState>,
    Json(req): Json<CreateFerRequest>,
) -> Result<Json<FerAnalysis>, AppError> {
    if req.fer_text.trim().is_empty() {
        return Err(AppError::bad_request("FER text is required"));
    }

    let title = req
        .title
        .unwrap_or_else(|| "Untitled FER Analysis".to_string());

    let analysis: FerAnalysis = sqlx::query_as(
        "INSERT INTO fer_analyses (user_id, project_id, title, fer_text, application_number, fer_date, status)
         VALUES ($1, $2, $3, $4, $5, $6, 'parsing')
         RETURNING *",
    )
    .bind(auth.user_id)
    .bind(req.project_id)
    .bind(&title)
    .bind(&req.fer_text)
    .bind(&req.application_number)
    .bind(req.fer_date)
    .fetch_one(&state.pool)
    .await?;

    let analysis_id = analysis.id;
    let pool = state.pool.clone();
    let provider = state.provider.clone();
    let fer_text = req.fer_text.clone();

    // Parse FER asynchronously
    tokio::spawn(async move {
        match fer::parser::parse_fer(provider.as_ref(), &fer_text).await {
            Ok(parsed) => {
                // Update examiner name if found
                if let Some(ref name) = parsed.examiner_name {
                    let _ = sqlx::query(
                        "UPDATE fer_analyses SET examiner_name = $2 WHERE id = $1",
                    )
                    .bind(analysis_id)
                    .bind(name)
                    .execute(&pool)
                    .await;
                }

                // Insert parsed objections and create empty response records
                for obj in &parsed.objections {
                    let result = sqlx::query_as::<_, (Uuid,)>(
                        "INSERT INTO fer_objections (analysis_id, objection_number, category, section_reference, summary, full_text)
                         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
                    )
                    .bind(analysis_id)
                    .bind(obj.objection_number)
                    .bind(&obj.category)
                    .bind(&obj.section_reference)
                    .bind(&obj.summary)
                    .bind(&obj.full_text)
                    .fetch_one(&pool)
                    .await;

                    if let Ok((objection_id,)) = result {
                        let _ = sqlx::query(
                            "INSERT INTO fer_responses (objection_id, status) VALUES ($1, 'pending')",
                        )
                        .bind(objection_id)
                        .execute(&pool)
                        .await;
                    }
                }

                let _ = sqlx::query("UPDATE fer_analyses SET status = 'parsed' WHERE id = $1")
                    .bind(analysis_id)
                    .execute(&pool)
                    .await;

                tracing::info!(
                    "FER {analysis_id} parsed: {} objections found",
                    parsed.objections.len()
                );
            }
            Err(e) => {
                tracing::error!("FER {analysis_id} parsing failed: {e}");
                let _ = sqlx::query("UPDATE fer_analyses SET status = 'failed' WHERE id = $1")
                    .bind(analysis_id)
                    .execute(&pool)
                    .await;
            }
        }
    });

    Ok(Json(analysis))
}

/// GET /api/fer — List user's FER analyses
pub async fn list_fer(
    Extension(auth): Extension<AuthUser>,
    State(state): State<FerState>,
) -> Result<Json<Vec<FerAnalysis>>, AppError> {
    let analyses: Vec<FerAnalysis> = sqlx::query_as(
        "SELECT * FROM fer_analyses WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50",
    )
    .bind(auth.user_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(analyses))
}

/// GET /api/fer/:id — Get FER analysis with objections and responses
pub async fn get_fer(
    Extension(auth): Extension<AuthUser>,
    State(state): State<FerState>,
    Path(analysis_id): Path<Uuid>,
) -> Result<Json<FerAnalysisDetail>, AppError> {
    let analysis: FerAnalysis = sqlx::query_as(
        "SELECT * FROM fer_analyses WHERE id = $1 AND user_id = $2",
    )
    .bind(analysis_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("FER analysis not found"))?;

    let objections: Vec<FerObjection> = sqlx::query_as(
        "SELECT * FROM fer_objections WHERE analysis_id = $1 ORDER BY objection_number ASC",
    )
    .bind(analysis_id)
    .fetch_all(&state.pool)
    .await?;

    let mut objections_with_responses = Vec::new();
    for obj in objections {
        let response: Option<FerResponse> = sqlx::query_as(
            "SELECT * FROM fer_responses WHERE objection_id = $1",
        )
        .bind(obj.id)
        .fetch_optional(&state.pool)
        .await?;

        objections_with_responses.push(ObjectionWithResponse {
            objection: obj,
            response,
        });
    }

    Ok(Json(FerAnalysisDetail {
        analysis,
        objections: objections_with_responses,
    }))
}

/// POST /api/fer/:id/generate — Generate responses for all objections (SSE stream)
pub async fn generate_responses(
    Extension(auth): Extension<AuthUser>,
    State(state): State<FerState>,
    Path(analysis_id): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let analysis: FerAnalysis = sqlx::query_as(
        "SELECT * FROM fer_analyses WHERE id = $1 AND user_id = $2",
    )
    .bind(analysis_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("FER analysis not found"))?;

    if analysis.status != "parsed" && analysis.status != "complete" {
        return Err(AppError::bad_request(
            "FER must be parsed before generating responses",
        ));
    }

    // Update status
    sqlx::query("UPDATE fer_analyses SET status = 'generating' WHERE id = $1")
        .bind(analysis_id)
        .execute(&state.pool)
        .await?;

    let objections: Vec<FerObjection> = sqlx::query_as(
        "SELECT * FROM fer_objections WHERE analysis_id = $1 ORDER BY objection_number ASC",
    )
    .bind(analysis_id)
    .fetch_all(&state.pool)
    .await?;

    let mut rx = FerResponsePipeline::run(
        analysis_id,
        &analysis.fer_text,
        objections,
        state.provider.as_ref(),
    )?;

    let pool = state.pool.clone();

    let stream = async_stream::stream! {
        while let Some(event) = rx.recv().await {
            // Save completed responses to DB
            if let FerSseEvent::ObjectionComplete {
                ref objection_id,
                ref legal_arguments,
                ref claim_amendments,
                ref case_law_citations,
            } = event
            {
                if let Ok(obj_uuid) = Uuid::parse_str(objection_id) {
                    let _ = sqlx::query(
                        "UPDATE fer_responses SET legal_arguments = $2, claim_amendments = $3, case_law_citations = $4, status = 'complete'
                         WHERE objection_id = $1",
                    )
                    .bind(obj_uuid)
                    .bind(legal_arguments)
                    .bind(claim_amendments)
                    .bind(case_law_citations)
                    .execute(&pool)
                    .await;
                }
            }

            if let FerSseEvent::GenerationComplete { ref analysis_id, .. } = event {
                if let Ok(a_uuid) = Uuid::parse_str(analysis_id) {
                    let _ = sqlx::query("UPDATE fer_analyses SET status = 'complete' WHERE id = $1")
                        .bind(a_uuid)
                        .execute(&pool)
                        .await;
                }
            }

            if let FerSseEvent::Error { .. } = event {
                let _ = sqlx::query("UPDATE fer_analyses SET status = 'parsed' WHERE id = $1")
                    .bind(analysis_id)
                    .execute(&pool)
                    .await;
            }

            let json = serde_json::to_string(&event).unwrap_or_default();
            yield Ok(Event::default().data(json));
        }
    };

    Ok(Sse::new(stream))
}

/// PATCH /api/fer/responses/:id — Update a response (user edits)
pub async fn update_response(
    Extension(auth): Extension<AuthUser>,
    State(state): State<FerState>,
    Path(response_id): Path<Uuid>,
    Json(req): Json<UpdateResponseRequest>,
) -> Result<Json<FerResponse>, AppError> {
    // Verify ownership through the chain
    let response: FerResponse = sqlx::query_as(
        "SELECT r.* FROM fer_responses r
         JOIN fer_objections o ON o.id = r.objection_id
         JOIN fer_analyses a ON a.id = o.analysis_id
         WHERE r.id = $1 AND a.user_id = $2",
    )
    .bind(response_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Response not found"))?;

    let updated: FerResponse = sqlx::query_as(
        "UPDATE fer_responses SET user_edited_text = $2, status = 'edited', updated_at = now()
         WHERE id = $1
         RETURNING *",
    )
    .bind(response.id)
    .bind(&req.user_edited_text)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(updated))
}

/// POST /api/fer/responses/:id/accept — Accept a response
pub async fn accept_response(
    Extension(auth): Extension<AuthUser>,
    State(state): State<FerState>,
    Path(response_id): Path<Uuid>,
) -> Result<Json<FerResponse>, AppError> {
    let updated: FerResponse = sqlx::query_as(
        "UPDATE fer_responses SET status = 'accepted', updated_at = now()
         WHERE id = $1 AND id IN (
             SELECT r.id FROM fer_responses r
             JOIN fer_objections o ON o.id = r.objection_id
             JOIN fer_analyses a ON a.id = o.analysis_id
             WHERE a.user_id = $2
         )
         RETURNING *",
    )
    .bind(response_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Response not found"))?;

    Ok(Json(updated))
}

/// POST /api/fer/responses/:id/regenerate — Regenerate a single response (SSE stream)
pub async fn regenerate_response(
    Extension(auth): Extension<AuthUser>,
    State(state): State<FerState>,
    Path(response_id): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    // Get the response with its objection and analysis context
    let row = sqlx::query_as::<_, (Uuid, String, String, Option<String>, String)>(
        "SELECT o.id, o.full_text, o.category, o.section_reference, a.fer_text
         FROM fer_responses r
         JOIN fer_objections o ON o.id = r.objection_id
         JOIN fer_analyses a ON a.id = o.analysis_id
         WHERE r.id = $1 AND a.user_id = $2",
    )
    .bind(response_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Response not found"))?;

    let (objection_id, full_text, category, section_ref, fer_text) = row;

    // Reset response status
    sqlx::query("UPDATE fer_responses SET status = 'generating', legal_arguments = '', claim_amendments = '', case_law_citations = '', user_edited_text = NULL WHERE id = $1")
        .bind(response_id)
        .execute(&state.pool)
        .await?;

    let prompt = fer::prompts::build_response_prompt(
        &full_text,
        &category,
        section_ref.as_deref(),
        &fer_text,
    );
    let mut stream_rx = state.provider.generate_stream(prompt)?;

    let pool = state.pool.clone();
    let obj_id_str = objection_id.to_string();

    let stream = async_stream::stream! {
        let mut full_content = String::new();

        // Start event
        let start = FerSseEvent::ObjectionStart {
            objection_id: obj_id_str.clone(),
            objection_number: 0,
            index: 0,
            total: 1,
        };
        yield Ok(Event::default().data(serde_json::to_string(&start).unwrap_or_default()));

        while let Some(chunk_result) = stream_rx.recv().await {
            match chunk_result {
                Ok(delta) => {
                    full_content.push_str(&delta);
                    let evt = FerSseEvent::ContentDelta {
                        objection_id: obj_id_str.clone(),
                        delta,
                    };
                    yield Ok(Event::default().data(serde_json::to_string(&evt).unwrap_or_default()));
                }
                Err(e) => {
                    let evt = FerSseEvent::Error {
                        objection_id: obj_id_str.clone(),
                        message: e.to_string(),
                    };
                    yield Ok(Event::default().data(serde_json::to_string(&evt).unwrap_or_default()));
                    break;
                }
            }
        }

        let (legal, amendments, citations) = fer::pipeline::parse_response_sections_public(&full_content);

        // Save to DB
        let _ = sqlx::query(
            "UPDATE fer_responses SET legal_arguments = $2, claim_amendments = $3, case_law_citations = $4, status = 'complete'
             WHERE objection_id = $1",
        )
        .bind(objection_id)
        .bind(&legal)
        .bind(&amendments)
        .bind(&citations)
        .execute(&pool)
        .await;

        let complete = FerSseEvent::ObjectionComplete {
            objection_id: obj_id_str,
            legal_arguments: legal,
            claim_amendments: amendments,
            case_law_citations: citations,
        };
        yield Ok(Event::default().data(serde_json::to_string(&complete).unwrap_or_default()));
    };

    Ok(Sse::new(stream))
}
