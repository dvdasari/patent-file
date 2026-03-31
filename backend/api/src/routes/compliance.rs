use axum::extract::{Path, State};
use axum::Json;
use sqlx::PgPool;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use axum::Extension;

use ai::{run_compliance_checks, ComplianceReport, PatentSections};

/// POST /api/projects/:id/compliance-check
///
/// Runs all Indian Patent Act compliance checks on the project's current sections
/// and returns a detailed report with warnings, citations, and suggestions.
pub async fn check_compliance(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path(project_id): Path<uuid::Uuid>,
) -> Result<Json<ComplianceReport>, AppError> {
    // Verify ownership
    let project = sqlx::query_as::<_, (String, String)>(
        "SELECT status, patent_type FROM projects WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
    )
    .bind(project_id)
    .bind(auth.user_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("Project not found"))?;

    let patent_type = project.1;

    // Load all sections
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT section_type, content FROM patent_sections WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_all(&pool)
    .await?;

    if rows.is_empty() {
        return Err(AppError::bad_request(
            "No patent sections found. Generate the specification first.",
        ));
    }

    let mut sections = PatentSections {
        title: String::new(),
        field_of_invention: String::new(),
        background: String::new(),
        summary: String::new(),
        detailed_description: String::new(),
        claims: String::new(),
        abstract_text: String::new(),
        drawings_description: String::new(),
        patent_type,
    };

    for (section_type, content) in &rows {
        match section_type.as_str() {
            "title" => sections.title = content.clone(),
            "field_of_invention" => sections.field_of_invention = content.clone(),
            "background" => sections.background = content.clone(),
            "summary" => sections.summary = content.clone(),
            "detailed_description" => sections.detailed_description = content.clone(),
            "claims" => sections.claims = content.clone(),
            "abstract" => sections.abstract_text = content.clone(),
            "drawings_description" => sections.drawings_description = content.clone(),
            _ => {}
        }
    }

    // Run compliance checks
    let report = run_compliance_checks(project_id, &sections);

    // Persist the compliance check result
    let report_json = serde_json::to_value(&report).unwrap_or_default();
    let total_warnings = report.warnings.len() as i32;
    let total_errors = report
        .warnings
        .iter()
        .filter(|w| w.severity == ai::compliance::Severity::Error)
        .count() as i32;

    sqlx::query(
        "INSERT INTO compliance_checks \
         (project_id, total_warnings, total_errors, section10_passed, section3_passed, \
          claims_passed, form2_compliant, report_json) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(project_id)
    .bind(total_warnings)
    .bind(total_errors)
    .bind(report.section10_passed)
    .bind(report.section3_passed)
    .bind(report.claims_passed)
    .bind(report.form2_compliant)
    .bind(&report_json)
    .execute(&pool)
    .await?;

    Ok(Json(report))
}

/// GET /api/projects/:id/compliance-checks
///
/// Returns the history of compliance check runs for a project.
pub async fn list_compliance_checks(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Path(project_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<shared::models::ComplianceCheck>>, AppError> {
    // Verify ownership
    sqlx::query_as::<_, (uuid::Uuid,)>(
        "SELECT id FROM projects WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
    )
    .bind(project_id)
    .bind(auth.user_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("Project not found"))?;

    let checks = sqlx::query_as::<_, shared::models::ComplianceCheck>(
        "SELECT * FROM compliance_checks WHERE project_id = $1 ORDER BY run_at DESC LIMIT 20",
    )
    .bind(project_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(checks))
}

/// GET /api/case-law/search?section=3(d)&keyword=efficacy
///
/// Search the Indian patent case law citation database.
pub async fn search_case_law(
    axum::extract::Query(params): axum::extract::Query<CaseLawSearchParams>,
) -> Json<Vec<&'static ai::compliance::case_law::CaseLawEntry>> {
    let mut results = Vec::new();

    if let Some(ref section) = params.section {
        results.extend(ai::compliance::case_law::search_by_section(section));
    }

    if let Some(ref keyword) = params.keyword {
        let keyword_results = ai::compliance::case_law::search_by_keyword(keyword);
        if results.is_empty() {
            results = keyword_results;
        } else {
            // Intersect: keep only entries that match both
            let keyword_ids: Vec<&str> = keyword_results.iter().map(|e| e.id).collect();
            results.retain(|e| keyword_ids.contains(&e.id));
        }
    }

    if params.section.is_none() && params.keyword.is_none() {
        results = ai::compliance::case_law::CASE_LAW_DB.iter().collect();
    }

    Json(results)
}

#[derive(serde::Deserialize)]
pub struct CaseLawSearchParams {
    pub section: Option<String>,
    pub keyword: Option<String>,
}
