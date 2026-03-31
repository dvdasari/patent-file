use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub status: String,
    pub jurisdiction: String,
    pub patent_type: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectApplicant {
    pub id: Uuid,
    pub project_id: Uuid,
    pub applicant_name: String,
    pub applicant_address: String,
    pub applicant_nationality: String,
    pub inventor_name: String,
    pub inventor_address: String,
    pub inventor_nationality: String,
    pub agent_name: Option<String>,
    pub agent_registration_no: Option<String>,
    pub assignee_name: Option<String>,
    pub priority_date: Option<NaiveDate>,
    pub priority_country: Option<String>,
    pub priority_application_no: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InterviewResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub step_number: i32,
    pub question_key: String,
    pub question_text: String,
    pub response_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PatentSection {
    pub id: Uuid,
    pub project_id: Uuid,
    pub section_type: String,
    pub content: String,
    pub ai_generated: bool,
    pub edit_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SectionVersion {
    pub id: Uuid,
    pub section_id: Uuid,
    pub content: String,
    pub version_number: i32,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Figure {
    pub id: Uuid,
    pub project_id: Uuid,
    pub sort_order: i32,
    pub description: String,
    pub storage_path: String,
    pub file_name: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Export {
    pub id: Uuid,
    pub project_id: Uuid,
    pub format: String,
    pub storage_path: String,
    pub file_size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub razorpay_customer_id: String,
    pub razorpay_subscription_id: String,
    pub plan_id: String,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RateLimit {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action_type: String,
    pub window_start: DateTime<Utc>,
    pub request_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Deadline {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_date: NaiveDate,
    pub status: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Valid section types for IPO Form 2
pub const SECTION_TYPES: &[&str] = &[
    "title",
    "field_of_invention",
    "background",
    "summary",
    "detailed_description",
    "claims",
    "abstract",
    "drawings_description",
];

pub fn is_valid_section_type(s: &str) -> bool {
    SECTION_TYPES.contains(&s)
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceCheck {
    pub id: Uuid,
    pub project_id: Uuid,
    pub run_at: DateTime<Utc>,
    pub total_warnings: i32,
    pub total_errors: i32,
    pub section10_passed: bool,
    pub section3_passed: bool,
    pub claims_passed: bool,
    pub form2_compliant: bool,
    pub report_json: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_section_types() {
        assert!(is_valid_section_type("title"));
        assert!(is_valid_section_type("claims"));
        assert!(is_valid_section_type("abstract"));
        assert!(!is_valid_section_type("invalid"));
        assert!(!is_valid_section_type(""));
    }

    #[test]
    fn test_section_types_count() {
        assert_eq!(SECTION_TYPES.len(), 8);
    }
}
