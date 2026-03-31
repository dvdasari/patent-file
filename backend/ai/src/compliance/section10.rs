//! Section 10 of the Indian Patent Act 1970 — Specification Requirements
//!
//! Section 10(4) requires a complete specification to:
//!   (a) fully and particularly describe the invention and its operation/use, and
//!       the method by which it is to be performed;
//!   (b) disclose the best method of performing the invention known to the applicant;
//!   (c) end with a claim or claims defining the scope of the invention;
//!   (d) be accompanied by an abstract.

use super::{ComplianceWarning, PatentSections, Severity};

/// Minimum word counts indicating sufficient disclosure per section
const MIN_DETAILED_DESCRIPTION_WORDS: usize = 200;
const MIN_CLAIMS_WORDS: usize = 30;
const MIN_SUMMARY_WORDS: usize = 30;

/// Run Section 10 compliance checks
pub fn check(sections: &PatentSections) -> Vec<ComplianceWarning> {
    let mut warnings = Vec::new();

    check_sufficiency_of_disclosure(sections, &mut warnings);
    check_best_method(sections, &mut warnings);
    check_claims_support(sections, &mut warnings);
    check_abstract_requirement(sections, &mut warnings);

    warnings
}

/// Section 10(4)(a): Full and particular description of the invention
fn check_sufficiency_of_disclosure(sections: &PatentSections, warnings: &mut Vec<ComplianceWarning>) {
    let desc_words = sections.detailed_description.split_whitespace().count();

    if desc_words < MIN_DETAILED_DESCRIPTION_WORDS {
        warnings.push(ComplianceWarning {
            rule_id: "S10-DISCLOSURE-INSUFFICIENT".into(),
            severity: Severity::Error,
            section_type: "detailed_description".into(),
            message: format!(
                "Detailed description has only {} words. Section 10(4)(a) requires the invention to be \
                 fully and particularly described so that a person skilled in the art can perform it.",
                desc_words
            ),
            suggestion: "Expand the detailed description to include: specific implementation steps, \
                         technical parameters, materials used, and working examples."
                .into(),
            citation: Some(
                "Section 10(4)(a) — Fully and particularly describe the invention and its operation. \
                 See also Bishwanath Prasad v. Hindustan Metal Industries (1979) — insufficiency of \
                 description is a valid ground for revocation."
                    .into(),
            ),
        });
    }

    let summary_words = sections.summary.split_whitespace().count();
    if summary_words < MIN_SUMMARY_WORDS {
        warnings.push(ComplianceWarning {
            rule_id: "S10-SUMMARY-THIN".into(),
            severity: Severity::Warning,
            section_type: "summary".into(),
            message: format!(
                "Summary has only {} words. It may not adequately convey the invention's contribution.",
                summary_words
            ),
            suggestion: "Expand the summary to describe the problem solved, the key technical features, \
                         and the advantages of the invention."
                .into(),
            citation: Some("Section 10(4)(a) — The specification must describe the invention.".into()),
        });
    }

    // Check that claims are referenced/supported by the detailed description
    if !sections.detailed_description.is_empty() && !sections.claims.is_empty() {
        check_claims_description_coherence(sections, warnings);
    }
}

/// Check that key terms in claims appear in the detailed description
fn check_claims_description_coherence(sections: &PatentSections, warnings: &mut Vec<ComplianceWarning>) {
    let description_lower = sections.detailed_description.to_lowercase();

    // Extract key noun phrases from independent claims (lines starting with a number)
    let claim_lines: Vec<&str> = sections
        .claims
        .lines()
        .filter(|l| {
            let trimmed = l.trim();
            trimmed.starts_with(|c: char| c.is_ascii_digit())
        })
        .collect();

    for line in claim_lines {
        // Extract significant words (>5 chars, skip common patent terms)
        let significant_words: Vec<&str> = line
            .split_whitespace()
            .filter(|w| {
                let w_lower = w.to_lowercase();
                w.len() > 5
                    && !COMMON_PATENT_WORDS.contains(&w_lower.as_str())
            })
            .collect();

        let unsupported: Vec<&&str> = significant_words
            .iter()
            .filter(|w| !description_lower.contains(&w.to_lowercase()))
            .collect();

        if unsupported.len() > 2 {
            warnings.push(ComplianceWarning {
                rule_id: "S10-CLAIMS-UNSUPPORTED".into(),
                severity: Severity::Warning,
                section_type: "claims".into(),
                message: format!(
                    "Claim terms may lack support in the detailed description: {}",
                    unsupported.iter().take(5).map(|w| format!("\"{}\"", w)).collect::<Vec<_>>().join(", ")
                ),
                suggestion: "Ensure every element recited in the claims is described in the specification. \
                             Add description for any missing claim elements."
                    .into(),
                citation: Some(
                    "Section 10(4)(c) read with 10(4)(a) — Claims must be fairly based on the matter \
                     disclosed in the specification. See Raj Prakash v. Mangat Ram (1978)."
                        .into(),
                ),
            });
        }
    }
}

/// Common patent terms to exclude from coherence checking
const COMMON_PATENT_WORDS: &[&str] = &[
    "comprising", "method", "system", "device", "apparatus", "wherein",
    "according", "present", "invention", "providing", "receiving",
    "generating", "processing", "configured", "includes", "having",
    "further", "claims", "claimed", "thereof", "therein", "whereby",
];

/// Section 10(4)(b): Best method of performing the invention
fn check_best_method(sections: &PatentSections, warnings: &mut Vec<ComplianceWarning>) {
    let desc_lower = sections.detailed_description.to_lowercase();

    let best_method_indicators = [
        "best mode",
        "preferred embodiment",
        "preferred method",
        "best method",
        "preferred implementation",
        "most effective",
        "optimal",
        "recommended approach",
    ];

    let has_best_method = best_method_indicators
        .iter()
        .any(|indicator| desc_lower.contains(indicator));

    if !has_best_method && sections.patent_type == "complete" {
        warnings.push(ComplianceWarning {
            rule_id: "S10-BEST-METHOD-MISSING".into(),
            severity: Severity::Warning,
            section_type: "detailed_description".into(),
            message: "No best method/preferred embodiment disclosure found. Section 10(4)(b) requires \
                      the applicant to disclose the best method of performing the invention."
                .into(),
            suggestion: "Add a section describing the preferred/best mode of performing the invention, \
                         including specific parameters, materials, and conditions."
                .into(),
            citation: Some(
                "Section 10(4)(b) — The specification shall disclose the best method of performing \
                 the invention which is known to the applicant and for which he is entitled to \
                 claim protection."
                    .into(),
            ),
        });
    }
}

/// Section 10(4)(c): Claims support
fn check_claims_support(sections: &PatentSections, warnings: &mut Vec<ComplianceWarning>) {
    let claims_words = sections.claims.split_whitespace().count();

    if claims_words < MIN_CLAIMS_WORDS && !sections.claims.is_empty() {
        warnings.push(ComplianceWarning {
            rule_id: "S10-CLAIMS-THIN".into(),
            severity: Severity::Warning,
            section_type: "claims".into(),
            message: format!(
                "Claims section has only {} words, which may not adequately define the scope of protection.",
                claims_words
            ),
            suggestion: "Expand claims to include at least one independent claim and dependent claims \
                         covering key embodiments."
                .into(),
            citation: Some(
                "Section 10(4)(c) — Every complete specification shall end with a claim or claims \
                 defining the scope of the invention for which protection is claimed."
                    .into(),
            ),
        });
    }

    // Check claim numbering
    let claim_numbers: Vec<usize> = sections
        .claims
        .lines()
        .filter_map(|l| {
            let trimmed = l.trim();
            trimmed
                .split(|c: char| c == '.' || c == ' ' || c == ')')
                .next()
                .and_then(|n| n.parse::<usize>().ok())
        })
        .collect();

    if !claim_numbers.is_empty() {
        // Check for sequential numbering
        for (i, &num) in claim_numbers.iter().enumerate() {
            if num != i + 1 {
                warnings.push(ComplianceWarning {
                    rule_id: "S10-CLAIMS-NUMBERING".into(),
                    severity: Severity::Warning,
                    section_type: "claims".into(),
                    message: format!(
                        "Claim numbering is not sequential. Expected claim {}, found claim {}.",
                        i + 1,
                        num
                    ),
                    suggestion: "Renumber claims sequentially starting from 1.".into(),
                    citation: Some("Rule 13(3) — Claims shall be numbered consecutively in Arabic numerals.".into()),
                });
                break;
            }
        }
    }
}

/// Section 10(4)(d): Abstract requirement
fn check_abstract_requirement(sections: &PatentSections, warnings: &mut Vec<ComplianceWarning>) {
    let abstract_words = sections.abstract_text.split_whitespace().count();

    if abstract_words < 10 && !sections.abstract_text.is_empty() {
        warnings.push(ComplianceWarning {
            rule_id: "S10-ABSTRACT-THIN".into(),
            severity: Severity::Warning,
            section_type: "abstract".into(),
            message: format!(
                "Abstract has only {} words. It should provide a meaningful technical summary.",
                abstract_words
            ),
            suggestion: "The abstract should concisely state the technical problem, the gist of the \
                         solution, and the principal use of the invention."
                .into(),
            citation: Some("Section 10(4)(d) read with Rule 13(7).".into()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::PatentSections;

    fn make_complete_sections() -> PatentSections {
        PatentSections {
            title: "Method for Automated Patent Drafting".into(),
            field_of_invention: "The present invention relates to artificial intelligence applied to legal document generation.".into(),
            background: "Prior art systems lack the ability to generate patent specifications compliant with Indian law.".into(),
            summary: "The invention provides an automated system and method for generating patent specifications that comply with the Indian Patent Act 1970. The system uses machine learning to analyze invention disclosures and produce complete specifications.".into(),
            detailed_description: {
                let mut desc = String::new();
                desc.push_str("The present invention will now be described in detail with reference to the accompanying drawings. ");
                desc.push_str("The preferred embodiment of the system comprises a processor, memory, and a neural network model. ");
                desc.push_str("The processor is configured to receive invention disclosure data from a user interface. ");
                desc.push_str("The neural network is trained on a corpus of Indian patent specifications. ");
                desc.push_str("In the best mode of carrying out the invention, the system uses a transformer architecture. ");
                desc.push_str("The transformer includes multi-head attention mechanisms tuned for legal text. ");
                desc.push_str("The system processes the disclosure through multiple stages: entity extraction, claim generation, and specification assembly. ");
                desc.push_str("Entity extraction identifies key technical features, problem statements, and prior art references. ");
                desc.push_str("Claim generation uses a specialized decoder trained on Indian patent claims format. ");
                desc.push_str("The specification assembly module combines all sections into Form 2 compliant output. ");
                desc.push_str("The system validates each section against the requirements of Section 10 of the Indian Patent Act. ");
                desc.push_str("Temperature parameters between 0.3 and 0.7 are used for different sections. ");
                desc.push_str("The training data includes over 500,000 Indian patent specifications from the IPO database. ");
                desc.push_str("Validation accuracy exceeds 95% on held-out test sets for compliance checking. ");
                desc
            },
            claims: "1. A method for automated patent drafting comprising:\na) receiving an invention disclosure from a user;\nb) extracting technical features using a neural network;\nc) generating patent claims based on extracted features;\nd) assembling a complete specification in Form 2 format;\ne) validating the specification against Indian Patent Act Section 10 requirements.\n\n2. The method of claim 1, wherein the neural network is a transformer architecture with multi-head attention.\n\n3. The method of claim 1, further comprising displaying compliance warnings to the user before filing.".into(),
            abstract_text: "A method and system for automated patent drafting using artificial intelligence, specifically designed for Indian Patent Office compliance.".into(),
            drawings_description: "Figure 1 shows the system architecture. Figure 2 illustrates the processing pipeline.".into(),
            patent_type: "complete".into(),
        }
    }

    #[test]
    fn test_compliant_specification_passes() {
        let sections = make_complete_sections();
        let warnings = check(&sections);
        let errors: Vec<_> = warnings.iter().filter(|w| w.severity == Severity::Error).collect();
        assert!(errors.is_empty(), "Expected no errors but got: {:?}", errors);
    }

    #[test]
    fn test_insufficient_description() {
        let mut sections = make_complete_sections();
        sections.detailed_description = "Short description.".into();
        let warnings = check(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "S10-DISCLOSURE-INSUFFICIENT"));
    }

    #[test]
    fn test_missing_best_method() {
        let mut sections = make_complete_sections();
        sections.detailed_description = sections.detailed_description.replace("best mode", "another way").replace("preferred embodiment", "one approach");
        let warnings = check(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "S10-BEST-METHOD-MISSING"));
    }

    #[test]
    fn test_thin_claims() {
        let mut sections = make_complete_sections();
        sections.claims = "1. A method.".into();
        let warnings = check(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "S10-CLAIMS-THIN"));
    }
}
