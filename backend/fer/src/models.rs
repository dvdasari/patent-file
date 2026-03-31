use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFerRequest {
    pub fer_text: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub application_number: Option<String>,
    #[serde(default)]
    pub fer_date: Option<NaiveDate>,
    #[serde(default)]
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FerAnalysis {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub title: String,
    pub fer_text: String,
    pub application_number: Option<String>,
    pub fer_date: Option<NaiveDate>,
    pub examiner_name: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FerObjection {
    pub id: Uuid,
    pub analysis_id: Uuid,
    pub objection_number: i32,
    pub category: String,
    pub section_reference: Option<String>,
    pub summary: String,
    pub full_text: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FerResponse {
    pub id: Uuid,
    pub objection_id: Uuid,
    pub legal_arguments: String,
    pub claim_amendments: String,
    pub case_law_citations: String,
    pub status: String,
    pub user_edited_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResponseRequest {
    pub user_edited_text: String,
}

/// FER analysis with its objections and responses
#[derive(Debug, Clone, Serialize)]
pub struct FerAnalysisDetail {
    #[serde(flatten)]
    pub analysis: FerAnalysis,
    pub objections: Vec<ObjectionWithResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectionWithResponse {
    #[serde(flatten)]
    pub objection: FerObjection,
    pub response: Option<FerResponse>,
}

/// Parsed objection from the AI parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedObjection {
    pub objection_number: i32,
    pub category: String,
    pub section_reference: Option<String>,
    pub summary: String,
    pub full_text: String,
}

/// Parsed FER metadata from the AI parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFer {
    pub examiner_name: Option<String>,
    pub objections: Vec<ParsedObjection>,
}
