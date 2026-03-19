use axum::extract::{Multipart, Path, State};
use axum::{Extension, Json};
use sqlx::PgPool;
use std::sync::Arc;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use storage::StorageClient;

#[derive(Clone)]
pub struct FiguresState {
    pub pool: PgPool,
    pub storage: Arc<dyn StorageClient>,
}

pub async fn upload_figure(
    Extension(auth): Extension<AuthUser>,
    State(state): State<FiguresState>,
    Path(project_id): Path<uuid::Uuid>,
    mut multipart: Multipart,
) -> Result<Json<shared::models::Figure>, AppError> {
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

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = String::new();
    let mut content_type = String::from("application/octet-stream");
    let mut description = String::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::bad_request(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().unwrap_or("upload").to_string();
                content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
                file_data = Some(field.bytes().await.map_err(|e| AppError::bad_request(e.to_string()))?.to_vec());
            }
            "description" => {
                description = field.text().await.map_err(|e| AppError::bad_request(e.to_string()))?;
            }
            _ => {}
        }
    }

    let data = file_data.ok_or_else(|| AppError::bad_request("No file uploaded"))?;
    if description.is_empty() {
        return Err(AppError::bad_request("Description is required"));
    }

    let file_size = data.len() as i64;
    let storage_key = format!("figures/{}/{}", project_id, file_name);

    state.storage.upload(&storage_key, &data, &content_type).await?;

    // Get next sort order
    let max_order: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(sort_order) FROM figures WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_one(&state.pool)
    .await?;
    let sort_order = max_order.unwrap_or(-1) + 1;

    let figure = sqlx::query_as::<_, shared::models::Figure>(
        "INSERT INTO figures (project_id, sort_order, description, storage_path, file_name, content_type, file_size_bytes)
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
    )
    .bind(project_id)
    .bind(sort_order)
    .bind(&description)
    .bind(&storage_key)
    .bind(&file_name)
    .bind(&content_type)
    .bind(file_size)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(figure))
}

pub async fn list_figures(
    Extension(auth): Extension<AuthUser>,
    State(state): State<FiguresState>,
    Path(project_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<shared::models::Figure>>, AppError> {
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

    let figures = sqlx::query_as::<_, shared::models::Figure>(
        "SELECT * FROM figures WHERE project_id = $1 ORDER BY sort_order",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(figures))
}

pub async fn delete_figure(
    Extension(auth): Extension<AuthUser>,
    State(state): State<FiguresState>,
    Path((project_id, figure_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
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

    let figure = sqlx::query_as::<_, (String,)>(
        "DELETE FROM figures WHERE id = $1 AND project_id = $2 RETURNING storage_path",
    )
    .bind(figure_id)
    .bind(project_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Figure not found"))?;

    // Delete from storage (best-effort)
    let _ = state.storage.delete(&figure.0).await;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
