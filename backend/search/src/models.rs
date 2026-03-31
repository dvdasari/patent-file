use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default)]
    pub ipc_classification: Option<String>,
    #[serde(default)]
    pub applicant: Option<String>,
    #[serde(default)]
    pub date_from: Option<NaiveDate>,
    #[serde(default)]
    pub date_to: Option<NaiveDate>,
    #[serde(default)]
    pub include_npl: bool,
    #[serde(default)]
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PriorArtSearch {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub query_text: String,
    pub ipc_classification: Option<String>,
    pub applicant_filter: Option<String>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub include_npl: bool,
    pub status: String,
    pub result_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PriorArtResult {
    pub id: Uuid,
    pub search_id: Uuid,
    pub source: String,
    pub external_id: Option<String>,
    pub title: String,
    pub applicant: Option<String>,
    pub filing_date: Option<NaiveDate>,
    pub publication_date: Option<NaiveDate>,
    pub ipc_codes: Option<String>,
    pub abstract_text: Option<String>,
    pub url: Option<String>,
    pub similarity_score: f32,
    pub novelty_assessment: Option<String>,
    pub relevance_rank: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchReport {
    pub id: Uuid,
    pub search_id: Uuid,
    pub format: String,
    pub storage_path: String,
    pub file_size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

/// Raw result from a patent data source before AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPatentResult {
    pub source: String,
    pub external_id: Option<String>,
    pub title: String,
    pub applicant: Option<String>,
    pub filing_date: Option<NaiveDate>,
    pub publication_date: Option<NaiveDate>,
    pub ipc_codes: Option<String>,
    pub abstract_text: Option<String>,
    pub url: Option<String>,
}
