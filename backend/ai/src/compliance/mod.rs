pub mod case_law;
pub mod claims;
pub mod section10;
pub mod section3;

use serde::Serialize;

/// Severity of a compliance warning
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A single compliance check result
#[derive(Debug, Clone, Serialize)]
pub struct ComplianceWarning {
    pub rule_id: String,
    pub severity: Severity,
    pub section_type: String,
    pub message: String,
    pub suggestion: String,
    pub citation: Option<String>,
}

/// Full compliance report for a patent specification
#[derive(Debug, Clone, Serialize)]
pub struct ComplianceReport {
    pub project_id: String,
    pub warnings: Vec<ComplianceWarning>,
    pub section10_passed: bool,
    pub section3_passed: bool,
    pub claims_passed: bool,
    pub form2_compliant: bool,
}

/// Input sections for compliance checking
pub struct PatentSections {
    pub title: String,
    pub field_of_invention: String,
    pub background: String,
    pub summary: String,
    pub detailed_description: String,
    pub claims: String,
    pub abstract_text: String,
    pub drawings_description: String,
    pub patent_type: String,
}

impl PatentSections {
    pub fn get_section(&self, name: &str) -> &str {
        match name {
            "title" => &self.title,
            "field_of_invention" => &self.field_of_invention,
            "background" => &self.background,
            "summary" => &self.summary,
            "detailed_description" => &self.detailed_description,
            "claims" => &self.claims,
            "abstract" => &self.abstract_text,
            "drawings_description" => &self.drawings_description,
            _ => "",
        }
    }
}

/// Run all compliance checks and produce a report
pub fn run_compliance_checks(
    project_id: uuid::Uuid,
    sections: &PatentSections,
) -> ComplianceReport {
    let mut warnings = Vec::new();

    // Section 10 checks (sufficiency of disclosure, best method, claims support)
    let s10_warnings = section10::check(sections);
    let section10_passed = !s10_warnings.iter().any(|w| w.severity == Severity::Error);
    warnings.extend(s10_warnings);

    // Section 3 exclusion screening
    let s3_warnings = section3::screen(sections);
    let section3_passed = !s3_warnings.iter().any(|w| w.severity == Severity::Error);
    warnings.extend(s3_warnings);

    // Indian claim drafting conventions
    let claims_warnings = claims::check(sections);
    let claims_passed = !claims_warnings.iter().any(|w| w.severity == Severity::Error);
    warnings.extend(claims_warnings);

    // Form 2 format compliance
    let form2_warnings = check_form2_compliance(sections);
    let form2_compliant = !form2_warnings.iter().any(|w| w.severity == Severity::Error);
    warnings.extend(form2_warnings);

    ComplianceReport {
        project_id: project_id.to_string(),
        warnings,
        section10_passed,
        section3_passed,
        claims_passed,
        form2_compliant,
    }
}

/// Check Form 2 specification format compliance
fn check_form2_compliance(sections: &PatentSections) -> Vec<ComplianceWarning> {
    let mut warnings = Vec::new();

    // Title length (IPO recommends concise titles, typically under 15 words)
    let title_words: Vec<&str> = sections.title.split_whitespace().collect();
    if title_words.len() > 15 {
        warnings.push(ComplianceWarning {
            rule_id: "FORM2-TITLE-LENGTH".into(),
            severity: Severity::Warning,
            section_type: "title".into(),
            message: format!(
                "Title has {} words. IPO Form 2 recommends concise titles (under 15 words).",
                title_words.len()
            ),
            suggestion: "Shorten the title to clearly identify the invention in fewer words.".into(),
            citation: Some("Rule 13(1)(a) — The title shall sufficiently indicate the subject matter.".into()),
        });
    }

    if sections.title.is_empty() {
        warnings.push(ComplianceWarning {
            rule_id: "FORM2-TITLE-MISSING".into(),
            severity: Severity::Error,
            section_type: "title".into(),
            message: "Title is required for Form 2 specification.".into(),
            suggestion: "Add a title that clearly identifies the invention.".into(),
            citation: Some("Section 10(1) — Every specification shall describe the invention.".into()),
        });
    }

    // Abstract word count (IPO requires max 150 words)
    let abstract_words: Vec<&str> = sections.abstract_text.split_whitespace().collect();
    if abstract_words.len() > 150 {
        warnings.push(ComplianceWarning {
            rule_id: "FORM2-ABSTRACT-LENGTH".into(),
            severity: Severity::Error,
            section_type: "abstract".into(),
            message: format!(
                "Abstract has {} words. IPO Form 2 requires abstracts of 150 words or fewer.",
                abstract_words.len()
            ),
            suggestion: "Reduce the abstract to 150 words while retaining the technical essence.".into(),
            citation: Some("Rule 13(7) — The abstract shall not exceed 150 words.".into()),
        });
    }

    // Required sections must be non-empty
    let required_sections = [
        ("field_of_invention", "Field of Invention"),
        ("background", "Background"),
        ("summary", "Summary"),
        ("detailed_description", "Detailed Description"),
        ("claims", "Claims"),
        ("abstract", "Abstract"),
    ];

    for (key, label) in &required_sections {
        if sections.get_section(key).trim().is_empty() {
            warnings.push(ComplianceWarning {
                rule_id: format!("FORM2-MISSING-{}", key.to_uppercase()),
                severity: Severity::Error,
                section_type: key.to_string(),
                message: format!("{} section is required for a complete specification.", label),
                suggestion: format!("Add the {} section to complete Form 2 specification.", label),
                citation: Some("Section 10(4) — Every complete specification shall include claims, description, and abstract.".into()),
            });
        }
    }

    // Complete specification needs claims (provisional does not)
    if sections.patent_type == "complete" && sections.claims.trim().is_empty() {
        warnings.push(ComplianceWarning {
            rule_id: "FORM2-CLAIMS-REQUIRED".into(),
            severity: Severity::Error,
            section_type: "claims".into(),
            message: "Claims are mandatory for a complete specification.".into(),
            suggestion: "Add at least one independent claim defining the scope of protection.".into(),
            citation: Some("Section 10(4)(c) — A complete specification shall end with a claim or claims.".into()),
        });
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sections(overrides: &[(&str, &str)]) -> PatentSections {
        let mut s = PatentSections {
            title: "Method and System for Automated Patent Drafting".into(),
            field_of_invention: "The present invention relates to artificial intelligence.".into(),
            background: "Prior art systems are limited in scope.".into(),
            summary: "The invention provides a method for automated patent drafting.".into(),
            detailed_description: "The system comprises a processor configured to analyze patent claims and generate specifications. The processor includes a neural network trained on Indian patent data. The best mode of carrying out the invention involves using a transformer architecture with attention mechanisms specifically tuned for legal text generation.".into(),
            claims: "1. A method for automated patent drafting comprising:\na) receiving an invention disclosure;\nb) generating a patent specification using a trained model;\nc) validating the specification against Indian Patent Act requirements.".into(),
            abstract_text: "A method and system for automated patent drafting using AI.".into(),
            drawings_description: "Figure 1 shows the system architecture.".into(),
            patent_type: "complete".into(),
        };
        for (k, v) in overrides {
            match *k {
                "title" => s.title = v.to_string(),
                "abstract" => s.abstract_text = v.to_string(),
                "claims" => s.claims = v.to_string(),
                "detailed_description" => s.detailed_description = v.to_string(),
                "patent_type" => s.patent_type = v.to_string(),
                _ => {}
            }
        }
        s
    }

    #[test]
    fn test_form2_abstract_too_long() {
        let long_abstract = (0..200).map(|i| format!("word{}", i)).collect::<Vec<_>>().join(" ");
        let sections = make_sections(&[("abstract", &long_abstract)]);
        let warnings = check_form2_compliance(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "FORM2-ABSTRACT-LENGTH"));
    }

    #[test]
    fn test_form2_missing_title() {
        let sections = make_sections(&[("title", "")]);
        let warnings = check_form2_compliance(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "FORM2-TITLE-MISSING"));
    }

    #[test]
    fn test_full_compliance_report() {
        let sections = make_sections(&[]);
        let report = run_compliance_checks(uuid::Uuid::new_v4(), &sections);
        assert!(report.form2_compliant);
    }
}
