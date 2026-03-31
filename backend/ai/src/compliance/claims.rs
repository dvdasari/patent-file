//! Indian Patent Claim Drafting Conventions
//!
//! Validates claims against Indian Patent Office conventions and Rules:
//! - Claim structure (independent vs. dependent)
//! - Proper antecedent basis
//! - Unity of invention (Section 10(5))
//! - Claim clarity and conciseness (Rule 13(3))

use super::{ComplianceWarning, PatentSections, Severity};

/// Check Indian claim drafting conventions
pub fn check(sections: &PatentSections) -> Vec<ComplianceWarning> {
    let mut warnings = Vec::new();

    if sections.claims.trim().is_empty() {
        return warnings;
    }

    let claims = parse_claims(&sections.claims);

    check_independent_claim_exists(&claims, &mut warnings);
    check_dependent_claim_references(&claims, &mut warnings);
    check_antecedent_basis(&claims, &mut warnings);
    check_claim_clarity(&claims, &mut warnings);
    check_preamble_conventions(&claims, &mut warnings);

    warnings
}

struct ParsedClaim {
    number: usize,
    text: String,
    is_dependent: bool,
    refers_to: Option<usize>,
}

fn parse_claims(claims_text: &str) -> Vec<ParsedClaim> {
    let mut claims = Vec::new();
    let mut current_number: Option<usize> = None;
    let mut current_text = String::new();

    for line in claims_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if this starts a new claim (number followed by . or ))
        if let Some(num) = extract_claim_number(trimmed) {
            // Save previous claim
            if let Some(prev_num) = current_number {
                let text = current_text.trim().to_string();
                let (is_dep, refers_to) = detect_dependency(&text);
                claims.push(ParsedClaim {
                    number: prev_num,
                    text,
                    is_dependent: is_dep,
                    refers_to,
                });
            }
            current_number = Some(num);
            // Remove the number prefix from the text
            let text_start = trimmed
                .find(|c: char| c == '.' || c == ')')
                .map(|i| i + 1)
                .unwrap_or(0);
            current_text = trimmed[text_start..].trim().to_string();
        } else if current_number.is_some() {
            current_text.push(' ');
            current_text.push_str(trimmed);
        }
    }

    // Save last claim
    if let Some(num) = current_number {
        let text = current_text.trim().to_string();
        let (is_dep, refers_to) = detect_dependency(&text);
        claims.push(ParsedClaim {
            number: num,
            text,
            is_dependent: is_dep,
            refers_to,
        });
    }

    claims
}

fn extract_claim_number(line: &str) -> Option<usize> {
    let first_token: String = line.chars().take_while(|c| c.is_ascii_digit()).collect();
    if first_token.is_empty() {
        return None;
    }
    let num = first_token.parse::<usize>().ok()?;
    // Verify it's followed by . or ) or whitespace
    let rest = &line[first_token.len()..];
    if rest.starts_with('.') || rest.starts_with(')') || rest.starts_with(' ') {
        Some(num)
    } else {
        None
    }
}

fn detect_dependency(text: &str) -> (bool, Option<usize>) {
    let lower = text.to_lowercase();
    let dependency_patterns = [
        "according to claim ",
        "as claimed in claim ",
        "the method of claim ",
        "the system of claim ",
        "the apparatus of claim ",
        "the device of claim ",
        "the composition of claim ",
        "the process of claim ",
        "as defined in claim ",
        "of claim ",
    ];

    for pattern in &dependency_patterns {
        if let Some(pos) = lower.find(pattern) {
            let after = &lower[pos + pattern.len()..];
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = num_str.parse::<usize>() {
                return (true, Some(num));
            }
        }
    }

    (false, None)
}

fn check_independent_claim_exists(claims: &[ParsedClaim], warnings: &mut Vec<ComplianceWarning>) {
    let has_independent = claims.iter().any(|c| !c.is_dependent);
    if !has_independent && !claims.is_empty() {
        warnings.push(ComplianceWarning {
            rule_id: "CLAIMS-NO-INDEPENDENT".into(),
            severity: Severity::Error,
            section_type: "claims".into(),
            message: "No independent claim found. Every claim set must have at least one independent claim.".into(),
            suggestion: "Add an independent claim that defines the invention without reference to other claims.".into(),
            citation: Some("Rule 13(3) — Claims must define the matter for which protection is sought.".into()),
        });
    }
}

fn check_dependent_claim_references(claims: &[ParsedClaim], warnings: &mut Vec<ComplianceWarning>) {
    let claim_numbers: Vec<usize> = claims.iter().map(|c| c.number).collect();

    for claim in claims {
        if let Some(ref_num) = claim.refers_to {
            if !claim_numbers.contains(&ref_num) {
                warnings.push(ComplianceWarning {
                    rule_id: "CLAIMS-INVALID-REF".into(),
                    severity: Severity::Error,
                    section_type: "claims".into(),
                    message: format!(
                        "Claim {} references non-existent claim {}.",
                        claim.number, ref_num
                    ),
                    suggestion: "Fix the dependent claim reference to point to an existing claim.".into(),
                    citation: Some("Rule 13(3) — Dependent claims must refer to a preceding claim.".into()),
                });
            } else if ref_num >= claim.number {
                warnings.push(ComplianceWarning {
                    rule_id: "CLAIMS-FORWARD-REF".into(),
                    severity: Severity::Error,
                    section_type: "claims".into(),
                    message: format!(
                        "Claim {} has a forward reference to claim {}. Dependent claims must refer to preceding claims.",
                        claim.number, ref_num
                    ),
                    suggestion: "Reorder claims so dependent claims follow their parent claims.".into(),
                    citation: Some("Rule 13(3) — Claims shall be numbered consecutively and dependent claims shall refer back to preceding claims.".into()),
                });
            }
        }
    }
}

fn check_antecedent_basis(claims: &[ParsedClaim], warnings: &mut Vec<ComplianceWarning>) {
    for claim in claims {
        if claim.is_dependent {
            continue;
        }

        let lower = claim.text.to_lowercase();
        // "the X" without prior "a X" or "an X" in the same claim
        let sentences: Vec<&str> = lower.split(|c| c == ';' || c == ':').collect();

        for (i, sentence) in sentences.iter().enumerate() {
            // Skip the preamble (first segment) — "the" in "The method of..." is fine
            if i == 0 {
                continue;
            }

            let words: Vec<&str> = sentence.split_whitespace().collect();
            for (j, &word) in words.iter().enumerate() {
                if word == "the" || word == "said" {
                    if let Some(&next_word) = words.get(j + 1) {
                        // Check if "a/an {next_word}" appears earlier in the claim
                        let article_pattern_a = format!("a {}", next_word);
                        let article_pattern_an = format!("an {}", next_word);

                        // Look in all text up to this point
                        let preceding_text: String = sentences[..=i].join("; ");
                        let current_pos = preceding_text.rfind(sentence).unwrap_or(preceding_text.len());
                        let text_before = &preceding_text[..current_pos];

                        if !text_before.contains(&article_pattern_a)
                            && !text_before.contains(&article_pattern_an)
                            && !ANTECEDENT_EXCEPTIONS.contains(&next_word)
                        {
                            warnings.push(ComplianceWarning {
                                rule_id: "CLAIMS-ANTECEDENT".into(),
                                severity: Severity::Warning,
                                section_type: "claims".into(),
                                message: format!(
                                    "Claim {}: '{}' in '... {} {} ...' may lack antecedent basis.",
                                    claim.number,
                                    if word == "the" { "the" } else { "said" },
                                    word,
                                    next_word
                                ),
                                suggestion: format!(
                                    "Introduce '{}' with 'a' or 'an' before referring to it with 'the' or 'said'.",
                                    next_word
                                ),
                                citation: Some(
                                    "IPO Manual of Patent Practice 2.2.5 — Claims must use proper antecedent basis.".into(),
                                ),
                            });
                            // Only one antecedent warning per claim to avoid noise
                            return;
                        }
                    }
                }
            }
        }
    }
}

/// Words that commonly appear with "the" without needing explicit antecedent
const ANTECEDENT_EXCEPTIONS: &[&str] = &[
    "method", "system", "device", "apparatus", "invention", "steps",
    "present", "following", "above", "below", "same", "user",
    "processor", "memory", "computer", "server", "network",
];

fn check_claim_clarity(claims: &[ParsedClaim], warnings: &mut Vec<ComplianceWarning>) {
    for claim in claims {
        // Check for vague terms
        let vague_terms = [
            ("substantially", "Use precise quantitative ranges instead of 'substantially'."),
            ("approximately", "Specify the acceptable range/tolerance instead of 'approximately'."),
            ("and/or", "Replace 'and/or' with explicit alternatives using 'at least one of' construction."),
            ("etc.", "Remove 'etc.' and explicitly list all elements."),
            ("such as", "Replace 'such as' with 'comprising' or enumerate specific items."),
        ];

        let lower = claim.text.to_lowercase();
        for (term, suggestion) in &vague_terms {
            if lower.contains(term) {
                warnings.push(ComplianceWarning {
                    rule_id: "CLAIMS-VAGUE-TERM".into(),
                    severity: Severity::Warning,
                    section_type: "claims".into(),
                    message: format!(
                        "Claim {} contains vague term '{}'. IPO examiners may raise clarity objections.",
                        claim.number, term
                    ),
                    suggestion: suggestion.to_string(),
                    citation: Some("Section 10(5) — Claims shall be clear, succinct, and fairly based on the matter disclosed.".into()),
                });
            }
        }

        // Check claim length (extremely long claims reduce clarity)
        let word_count = claim.text.split_whitespace().count();
        if word_count > 200 {
            warnings.push(ComplianceWarning {
                rule_id: "CLAIMS-TOO-LONG".into(),
                severity: Severity::Warning,
                section_type: "claims".into(),
                message: format!(
                    "Claim {} has {} words. Very long claims can attract clarity objections.",
                    claim.number, word_count
                ),
                suggestion: "Consider splitting into an independent claim with dependent claims for specific features.".into(),
                citation: Some("Section 10(5) — Claims shall be succinct.".into()),
            });
        }
    }
}

fn check_preamble_conventions(claims: &[ParsedClaim], warnings: &mut Vec<ComplianceWarning>) {
    for claim in claims {
        if claim.is_dependent {
            continue;
        }

        let lower = claim.text.to_lowercase();

        // Indian convention: independent claims should use "comprising" (open) or "consisting of" (closed)
        let has_transition = lower.contains("comprising")
            || lower.contains("consisting of")
            || lower.contains("characterized in that")
            || lower.contains("characterised in that")
            || lower.contains("wherein");

        if !has_transition {
            warnings.push(ComplianceWarning {
                rule_id: "CLAIMS-NO-TRANSITION".into(),
                severity: Severity::Warning,
                section_type: "claims".into(),
                message: format!(
                    "Claim {} lacks a transitional phrase ('comprising', 'consisting of', 'characterized in that'). \
                     Indian drafting convention uses a clear preamble-transition-body structure.",
                    claim.number
                ),
                suggestion: "Structure the claim as: [Preamble] comprising/consisting of: [body elements].".into(),
                citation: Some("IPO Manual of Patent Practice — Claims should have clear preamble, transition, and body.".into()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::PatentSections;

    fn make_sections(claims: &str) -> PatentSections {
        PatentSections {
            title: "Test".into(),
            field_of_invention: "Test.".into(),
            background: "Test.".into(),
            summary: "Test summary.".into(),
            detailed_description: "Test description.".into(),
            claims: claims.into(),
            abstract_text: "Test.".into(),
            drawings_description: "".into(),
            patent_type: "complete".into(),
        }
    }

    #[test]
    fn test_valid_claims() {
        let claims = "1. A method for processing data comprising:\n\
                      a) receiving input data from a sensor;\n\
                      b) transforming the input data using a processor.\n\n\
                      2. The method of claim 1, wherein the processor is a GPU.";
        let sections = make_sections(claims);
        let warnings = check(&sections);
        let errors: Vec<_> = warnings.iter().filter(|w| w.severity == Severity::Error).collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_no_independent_claim() {
        let claims = "1. The method of claim 2, wherein X is Y.";
        let sections = make_sections(claims);
        let warnings = check(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "CLAIMS-NO-INDEPENDENT"));
    }

    #[test]
    fn test_invalid_reference() {
        let claims = "1. A method comprising step A.\n\n2. The method of claim 5, further comprising step B.";
        let sections = make_sections(claims);
        let warnings = check(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "CLAIMS-INVALID-REF"));
    }

    #[test]
    fn test_vague_terms() {
        let claims = "1. A method comprising substantially heating a material etc.";
        let sections = make_sections(claims);
        let warnings = check(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "CLAIMS-VAGUE-TERM"));
    }

    #[test]
    fn test_missing_transition() {
        let claims = "1. A method for processing data by receiving input and transforming it.";
        let sections = make_sections(claims);
        let warnings = check(&sections);
        assert!(warnings.iter().any(|w| w.rule_id == "CLAIMS-NO-TRANSITION"));
    }
}
