use axum::extract::{Extension, Path, State};
use axum::Json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

use search::models::{PriorArtResult, PriorArtSearch, SearchReport, SearchRequest};
use search::SearchEngine;

#[derive(Clone)]
pub struct SearchState {
    pub pool: sqlx::PgPool,
    pub engine: Arc<SearchEngine>,
    pub storage: Arc<dyn storage::StorageClient>,
}

/// POST /api/search — Execute a new prior art search
pub async fn create_search(
    Extension(auth): Extension<AuthUser>,
    State(state): State<SearchState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<PriorArtSearchResponse>, AppError> {
    if req.query.trim().is_empty() {
        return Err(AppError::bad_request("Search query is required"));
    }

    // Create the search record
    let search: PriorArtSearch = sqlx::query_as(
        "INSERT INTO prior_art_searches (user_id, project_id, query_text, ipc_classification, applicant_filter, date_from, date_to, include_npl, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'searching')
         RETURNING *",
    )
    .bind(auth.user_id)
    .bind(req.project_id)
    .bind(&req.query)
    .bind(&req.ipc_classification)
    .bind(&req.applicant)
    .bind(req.date_from)
    .bind(req.date_to)
    .bind(req.include_npl)
    .fetch_one(&state.pool)
    .await?;

    let search_id = search.id;
    let pool = state.pool.clone();
    let engine = state.engine.clone();
    let query = req.query.clone();
    let ipc = req.ipc_classification.clone();
    let applicant = req.applicant.clone();
    let date_from = req.date_from;
    let date_to = req.date_to;
    let include_npl = req.include_npl;

    // Run the search asynchronously
    tokio::spawn(async move {
        let result = engine
            .search(
                &query,
                ipc.as_deref(),
                applicant.as_deref(),
                date_from,
                date_to,
                include_npl,
            )
            .await;

        match result {
            Ok(assessed_results) => {
                let count = assessed_results.len() as i32;

                // Insert results
                for (rank, assessed) in assessed_results.iter().enumerate() {
                    let r = &assessed.raw;
                    let _ = sqlx::query(
                        "INSERT INTO prior_art_results (search_id, source, external_id, title, applicant, filing_date, publication_date, ipc_codes, abstract_text, url, similarity_score, novelty_assessment, relevance_rank)
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
                    )
                    .bind(search_id)
                    .bind(&r.source)
                    .bind(&r.external_id)
                    .bind(&r.title)
                    .bind(&r.applicant)
                    .bind(r.filing_date)
                    .bind(r.publication_date)
                    .bind(&r.ipc_codes)
                    .bind(&r.abstract_text)
                    .bind(&r.url)
                    .bind(assessed.similarity_score)
                    .bind(&assessed.novelty_assessment)
                    .bind((rank + 1) as i32)
                    .execute(&pool)
                    .await;
                }

                // Update search status
                let _ = sqlx::query(
                    "UPDATE prior_art_searches SET status = 'complete', result_count = $2 WHERE id = $1",
                )
                .bind(search_id)
                .bind(count)
                .execute(&pool)
                .await;

                tracing::info!("Search {search_id} complete with {count} results");
            }
            Err(e) => {
                tracing::error!("Search {search_id} failed: {e}");
                let _ = sqlx::query(
                    "UPDATE prior_art_searches SET status = 'failed' WHERE id = $1",
                )
                .bind(search_id)
                .execute(&pool)
                .await;
            }
        }
    });

    Ok(Json(PriorArtSearchResponse {
        search,
        results: vec![],
    }))
}

/// GET /api/searches — List user's searches
pub async fn list_searches(
    Extension(auth): Extension<AuthUser>,
    State(state): State<SearchState>,
) -> Result<Json<Vec<PriorArtSearch>>, AppError> {
    let searches: Vec<PriorArtSearch> = sqlx::query_as(
        "SELECT * FROM prior_art_searches WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50",
    )
    .bind(auth.user_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(searches))
}

/// GET /api/searches/:id — Get search with results
pub async fn get_search(
    Extension(auth): Extension<AuthUser>,
    State(state): State<SearchState>,
    Path(search_id): Path<Uuid>,
) -> Result<Json<PriorArtSearchResponse>, AppError> {
    let search: PriorArtSearch = sqlx::query_as(
        "SELECT * FROM prior_art_searches WHERE id = $1 AND user_id = $2",
    )
    .bind(search_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Search not found"))?;

    let results: Vec<PriorArtResult> = sqlx::query_as(
        "SELECT * FROM prior_art_results WHERE search_id = $1 ORDER BY relevance_rank ASC",
    )
    .bind(search_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PriorArtSearchResponse { search, results }))
}

/// POST /api/searches/:id/report — Generate PDF report
pub async fn generate_report(
    Extension(auth): Extension<AuthUser>,
    State(state): State<SearchState>,
    Path(search_id): Path<Uuid>,
) -> Result<Json<SearchReport>, AppError> {
    let search: PriorArtSearch = sqlx::query_as(
        "SELECT * FROM prior_art_searches WHERE id = $1 AND user_id = $2",
    )
    .bind(search_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Search not found"))?;

    if search.status != "complete" {
        return Err(AppError::bad_request(
            "Search must be complete before generating a report",
        ));
    }

    let results: Vec<PriorArtResult> = sqlx::query_as(
        "SELECT * FROM prior_art_results WHERE search_id = $1 ORDER BY relevance_rank ASC",
    )
    .bind(search_id)
    .fetch_all(&state.pool)
    .await?;

    let pdf_bytes =
        search::report::generate_search_report(&search, &results).map_err(|e| {
            tracing::error!("PDF generation failed: {e}");
            AppError::internal("Failed to generate PDF report")
        })?;

    let file_size = pdf_bytes.len() as i64;
    let storage_path = format!("search-reports/{}/{}.pdf", auth.user_id, search_id);

    state
        .storage
        .upload(&storage_path, &pdf_bytes, "application/pdf")
        .await
        .map_err(|e| {
            tracing::error!("Report upload failed: {e}");
            AppError::internal("Failed to save report")
        })?;

    let report: SearchReport = sqlx::query_as(
        "INSERT INTO search_reports (search_id, format, storage_path, file_size_bytes)
         VALUES ($1, 'pdf', $2, $3)
         RETURNING *",
    )
    .bind(search_id)
    .bind(&storage_path)
    .bind(file_size)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(report))
}

/// GET /api/search-reports/:id/download — Download report
pub async fn download_report(
    Extension(auth): Extension<AuthUser>,
    State(state): State<SearchState>,
    Path(report_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let report: SearchReport = sqlx::query_as(
        "SELECT sr.* FROM search_reports sr
         JOIN prior_art_searches s ON s.id = sr.search_id
         WHERE sr.id = $1 AND s.user_id = $2",
    )
    .bind(report_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Report not found"))?;

    let url = state
        .storage
        .download_url(&report.storage_path, 3600)
        .await
        .map_err(|e| {
            tracing::error!("Failed to generate download URL: {e}");
            AppError::internal("Failed to generate download link")
        })?;

    Ok(Json(serde_json::json!({ "url": url })))
}

#[derive(serde::Serialize)]
pub struct PriorArtSearchResponse {
    pub search: PriorArtSearch,
    pub results: Vec<PriorArtResult>,
}
