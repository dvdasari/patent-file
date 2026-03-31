mod pdf;
mod typst_source;

pub use pdf::generate_pdf;
pub use typst_source::generate_typst_source;

use chrono::NaiveDate;

/// All data needed to render a patent specification document.
pub struct PatentDocument {
    pub title: String,
    pub field_of_invention: String,
    pub background: String,
    pub summary: String,
    pub detailed_description: String,
    pub claims: String,
    pub abstract_text: String,
    pub drawings_description: String,
    pub applicant: Option<ApplicantInfo>,
    pub patent_type: String,
}

pub struct ApplicantInfo {
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
}

impl PatentDocument {
    pub fn from_sections(
        sections: &[(String, String)],
        applicant: Option<ApplicantInfo>,
        patent_type: &str,
    ) -> anyhow::Result<Self> {
        let get = |key: &str| -> anyhow::Result<String> {
            sections
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .ok_or_else(|| anyhow::anyhow!("Missing section: {key}"))
        };

        Ok(Self {
            title: get("title")?,
            field_of_invention: get("field_of_invention")?,
            background: get("background")?,
            summary: get("summary")?,
            detailed_description: get("detailed_description")?,
            claims: get("claims")?,
            abstract_text: get("abstract")?,
            drawings_description: get("drawings_description")?,
            applicant,
            patent_type: patent_type.to_string(),
        })
    }
}
