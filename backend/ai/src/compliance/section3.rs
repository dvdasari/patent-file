//! Section 3 of the Indian Patent Act 1970 — Non-Patentable Subject Matter
//!
//! Screens patent specifications for inventions that may fall under exclusions:
//!   - Section 3(d): Mere new form of a known substance (pharma evergreening)
//!   - Section 3(k): Mathematical or business method, computer program per se, algorithm
//!   - Section 3(e): Mere admixture / aggregation of known properties

use super::{ComplianceWarning, PatentSections, Severity};
use crate::compliance::case_law;

/// Screen for Section 3 exclusions
pub fn screen(sections: &PatentSections) -> Vec<ComplianceWarning> {
    let mut warnings = Vec::new();

    screen_section3d(sections, &mut warnings);
    screen_section3k(sections, &mut warnings);
    screen_section3e(sections, &mut warnings);

    warnings
}

/// Section 3(d): New form of known substance without enhanced efficacy
fn screen_section3d(sections: &PatentSections, warnings: &mut Vec<ComplianceWarning>) {
    let claims_lower = sections.claims.to_lowercase();
    let desc_lower = sections.detailed_description.to_lowercase();

    let pharma_indicators = [
        "polymorph",
        "salt form",
        "enantiomer",
        "prodrug",
        "metabolite",
        "crystalline form",
        "amorphous form",
        "hydrate",
        "solvate",
        "isomer",
        "new form",
        "new use",
        "second medical use",
        "pharmaceutical composition",
    ];

    let has_pharma_indicators = pharma_indicators
        .iter()
        .any(|ind| claims_lower.contains(ind) || desc_lower.contains(ind));

    if has_pharma_indicators {
        let efficacy_terms = [
            "enhanced efficacy",
            "improved bioavailability",
            "increased therapeutic",
            "superior efficacy",
            "synergistic effect",
            "significantly enhanced",
            "comparative study",
            "clinical trial",
        ];

        let has_efficacy_data = efficacy_terms
            .iter()
            .any(|term| desc_lower.contains(term));

        let severity = if has_efficacy_data {
            Severity::Info
        } else {
            Severity::Error
        };

        let case_ref = case_law::get_citation("section_3d");

        warnings.push(ComplianceWarning {
            rule_id: "S3D-NEW-FORM".into(),
            severity,
            section_type: "claims".into(),
            message: if has_efficacy_data {
                "Specification involves a new form of a known substance but includes efficacy data. \
                 Ensure the enhanced efficacy is clearly demonstrated with comparative data."
                    .into()
            } else {
                "Claims may relate to a new form of a known substance without demonstrating \
                 enhanced efficacy. This is likely non-patentable under Section 3(d)."
                    .into()
            },
            suggestion: "Include comparative experimental data demonstrating significantly enhanced \
                         efficacy of the new form over the known substance. Mere change in form \
                         without enhanced efficacy is excluded under Section 3(d)."
                .into(),
            citation: Some(case_ref),
        });
    }
}

/// Section 3(k): Computer programs per se, mathematical methods, business methods, algorithms
fn screen_section3k(sections: &PatentSections, warnings: &mut Vec<ComplianceWarning>) {
    let claims_lower = sections.claims.to_lowercase();
    let desc_lower = sections.detailed_description.to_lowercase();

    // Pure software/algorithm indicators
    let software_indicators = [
        "computer program",
        "software",
        "algorithm",
        "machine learning model",
        "neural network",
        "source code",
        "executable",
        "application programming interface",
        "api",
    ];

    let business_method_indicators = [
        "business method",
        "business process",
        "method of doing business",
        "financial method",
        "trading method",
        "marketing method",
    ];

    let mathematical_indicators = [
        "mathematical method",
        "mathematical formula",
        "mathematical model",
        "mathematical algorithm",
    ];

    let has_software = software_indicators
        .iter()
        .any(|ind| claims_lower.contains(ind));

    let has_business = business_method_indicators
        .iter()
        .any(|ind| claims_lower.contains(ind));

    let has_math = mathematical_indicators
        .iter()
        .any(|ind| claims_lower.contains(ind) || desc_lower.contains(ind));

    // Technical effect indicators that may save the claim
    let technical_effect_terms = [
        "technical effect",
        "hardware",
        "processor",
        "memory",
        "sensor",
        "physical",
        "industrial application",
        "technical problem",
        "apparatus",
        "device",
        "circuit",
        "embedded",
    ];

    let has_technical_effect = technical_effect_terms
        .iter()
        .any(|term| claims_lower.contains(term));

    if has_software {
        let case_ref = case_law::get_citation("section_3k_software");

        if has_technical_effect {
            warnings.push(ComplianceWarning {
                rule_id: "S3K-SOFTWARE-TECHNICAL".into(),
                severity: Severity::Info,
                section_type: "claims".into(),
                message: "Claims involve software/computer program elements but include technical \
                          effect indicators. Ensure claims are drafted to emphasize the technical \
                          contribution, not the program per se."
                    .into(),
                suggestion: "Draft claims to recite the technical problem solved and the hardware \
                             interaction. Avoid claiming the software in isolation. Use 'a system \
                             comprising a processor configured to...' rather than 'a program that...'."
                    .into(),
                citation: Some(case_ref),
            });
        } else {
            warnings.push(ComplianceWarning {
                rule_id: "S3K-SOFTWARE-PERSE".into(),
                severity: Severity::Error,
                section_type: "claims".into(),
                message: "Claims appear to recite a computer program per se without technical effect. \
                          This is non-patentable under Section 3(k)."
                    .into(),
                suggestion: "Reframe claims to emphasize the technical contribution and hardware \
                             interaction. CRI Guidelines (2017) require a novel hardware component \
                             or technical effect beyond the program itself."
                    .into(),
                citation: Some(case_ref),
            });
        }
    }

    if has_business {
        warnings.push(ComplianceWarning {
            rule_id: "S3K-BUSINESS-METHOD".into(),
            severity: Severity::Error,
            section_type: "claims".into(),
            message: "Claims appear to recite a business method, which is non-patentable under Section 3(k).".into(),
            suggestion: "If there is a technical implementation aspect, reframe claims to focus on \
                         the technical solution rather than the business process itself."
                .into(),
            citation: Some(case_law::get_citation("section_3k_business")),
        });
    }

    if has_math {
        warnings.push(ComplianceWarning {
            rule_id: "S3K-MATHEMATICAL".into(),
            severity: Severity::Warning,
            section_type: "claims".into(),
            message: "Claims or description reference mathematical methods. Mathematical methods \
                      per se are non-patentable under Section 3(k)."
                .into(),
            suggestion: "Ensure the claims focus on the practical application of the mathematical \
                         method, not the method itself. Demonstrate industrial applicability."
                .into(),
            citation: Some(case_law::get_citation("section_3k_math")),
        });
    }
}

/// Section 3(e): Mere admixture or aggregation resulting in aggregation of properties
fn screen_section3e(sections: &PatentSections, warnings: &mut Vec<ComplianceWarning>) {
    let claims_lower = sections.claims.to_lowercase();
    let desc_lower = sections.detailed_description.to_lowercase();

    let aggregation_indicators = [
        "combination of",
        "admixture",
        "mixture of known",
        "composition comprising known",
        "blend of",
        "combining known",
    ];

    let has_aggregation = aggregation_indicators
        .iter()
        .any(|ind| claims_lower.contains(ind) || desc_lower.contains(ind));

    if has_aggregation {
        let synergy_terms = [
            "synergistic",
            "unexpected",
            "surprising effect",
            "non-obvious combination",
            "enhanced beyond",
            "more than additive",
            "superior to individual",
        ];

        let has_synergy = synergy_terms
            .iter()
            .any(|term| desc_lower.contains(term));

        let severity = if has_synergy {
            Severity::Info
        } else {
            Severity::Warning
        };

        warnings.push(ComplianceWarning {
            rule_id: "S3E-AGGREGATION".into(),
            severity,
            section_type: "claims".into(),
            message: if has_synergy {
                "Specification claims a combination and includes synergistic effect data. \
                 Ensure the synergy is clearly demonstrated with comparative evidence."
                    .into()
            } else {
                "Claims may involve a mere admixture or aggregation of known components. \
                 Under Section 3(e), this is non-patentable unless a synergistic effect is shown."
                    .into()
            },
            suggestion: "Include experimental data demonstrating that the combination produces an \
                         effect beyond the sum of individual components (synergistic effect)."
                .into(),
            citation: Some(case_law::get_citation("section_3e")),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::PatentSections;

    fn base_sections() -> PatentSections {
        PatentSections {
            title: "Test".into(),
            field_of_invention: "Test field.".into(),
            background: "Test background.".into(),
            summary: "Test summary of the invention.".into(),
            detailed_description: "A detailed technical description of the invention and its working.".into(),
            claims: "1. A system comprising a processor and memory for data processing.".into(),
            abstract_text: "Test abstract.".into(),
            drawings_description: "Figure 1.".into(),
            patent_type: "complete".into(),
        }
    }

    #[test]
    fn test_no_flags_on_clean_spec() {
        let sections = base_sections();
        let warnings = screen(&sections);
        assert!(warnings.is_empty(), "Expected no warnings but got: {:?}", warnings.len());
    }

    #[test]
    fn test_section3d_polymorph_no_efficacy() {
        let mut sections = base_sections();
        sections.claims = "1. A crystalline form of compound X being a polymorph.".into();
        let warnings = screen(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "S3D-NEW-FORM" && w.severity == Severity::Error));
    }

    #[test]
    fn test_section3d_with_efficacy() {
        let mut sections = base_sections();
        sections.claims = "1. A crystalline form of compound X being a polymorph.".into();
        sections.detailed_description = "The polymorph shows enhanced efficacy with improved bioavailability compared to the known form.".into();
        let warnings = screen(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "S3D-NEW-FORM" && w.severity == Severity::Info));
    }

    #[test]
    fn test_section3k_software_no_technical_effect() {
        let mut sections = base_sections();
        sections.claims = "1. A computer program for sorting data using an algorithm.".into();
        let warnings = screen(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "S3K-SOFTWARE-PERSE"));
    }

    #[test]
    fn test_section3k_software_with_technical_effect() {
        let mut sections = base_sections();
        sections.claims = "1. A system comprising a processor configured to execute a computer program for real-time sensor data processing.".into();
        let warnings = screen(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "S3K-SOFTWARE-TECHNICAL" && w.severity == Severity::Info));
    }

    #[test]
    fn test_section3k_business_method() {
        let mut sections = base_sections();
        sections.claims = "1. A business method for optimizing supply chain logistics.".into();
        let warnings = screen(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "S3K-BUSINESS-METHOD"));
    }

    #[test]
    fn test_section3e_aggregation() {
        let mut sections = base_sections();
        sections.claims = "1. A composition comprising known compounds A and B.".into();
        sections.detailed_description = "The combination of known components A and B.".into();
        let warnings = screen(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "S3E-AGGREGATION"));
    }
}
