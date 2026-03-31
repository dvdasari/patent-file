//! AI-powered novelty assessment using the LLM provider.
//!
//! Takes search results and produces similarity scores + plain-language
//! novelty assessments for each result relative to the invention query.

use anyhow::Result;
use tokio::sync::mpsc;

use ai::{LlmProvider, Prompt};

use crate::models::RawPatentResult;

/// Assessed result with AI-generated scores and analysis.
#[derive(Debug, Clone)]
pub struct AssessedResult {
    pub raw: RawPatentResult,
    pub similarity_score: f32,
    pub novelty_assessment: String,
}

/// Analyze a batch of search results against the user's invention query
/// using the AI provider for relevance ranking and novelty assessment.
pub async fn assess_results(
    provider: &dyn LlmProvider,
    query: &str,
    results: Vec<RawPatentResult>,
) -> Result<Vec<AssessedResult>> {
    if results.is_empty() {
        return Ok(Vec::new());
    }

    // Build a compact summary of all results for the AI to evaluate
    let mut results_summary = String::new();
    for (i, r) in results.iter().enumerate() {
        results_summary.push_str(&format!(
            "\n--- Result #{} ---\nTitle: {}\nSource: {}\n",
            i + 1,
            r.title,
            r.source
        ));
        if let Some(ref ext_id) = r.external_id {
            results_summary.push_str(&format!("ID: {ext_id}\n"));
        }
        if let Some(ref applicant) = r.applicant {
            results_summary.push_str(&format!("Applicant: {applicant}\n"));
        }
        if let Some(ref ipc) = r.ipc_codes {
            results_summary.push_str(&format!("IPC: {ipc}\n"));
        }
        if let Some(ref abs) = r.abstract_text {
            // Truncate long abstracts
            let trimmed = if abs.len() > 500 { &abs[..500] } else { abs };
            results_summary.push_str(&format!("Abstract: {trimmed}\n"));
        }
    }

    let system = format!(
        "You are an Indian patent prior art analyst. Your task is to evaluate prior art search \
         results against an invention description and produce a JSON array of assessments.\n\n\
         For each result, provide:\n\
         1. A similarity_score from 0.0 (completely unrelated) to 1.0 (identical invention)\n\
         2. A novelty_assessment: a 2-3 sentence plain-language explanation of how the prior art \
            relates to the invention, what overlaps exist, and whether it threatens novelty under \
            Section 2(1)(j) of the Indian Patents Act.\n\n\
         IMPORTANT: Respond ONLY with a JSON array, no markdown, no explanation. Example:\n\
         [{{\"index\": 1, \"similarity_score\": 0.75, \"novelty_assessment\": \"This patent describes...\"}}]\n\n\
         Consider the Indian Patents Act standards for novelty (Section 2(1)(j)), \
         inventive step (Section 2(1)(ja)), and industrial applicability (Section 2(1)(ac))."
    );

    let user = format!(
        "## Invention Under Analysis\n{query}\n\n## Prior Art Results to Evaluate\n{results_summary}"
    );

    let prompt = Prompt { system, user };

    // Collect the full AI response
    let rx = provider.generate_stream(prompt)?;
    let full_response = collect_stream(rx).await?;

    // Parse the JSON response
    parse_assessments(&full_response, results)
}

async fn collect_stream(mut rx: mpsc::Receiver<Result<String>>) -> Result<String> {
    let mut full = String::new();
    while let Some(chunk) = rx.recv().await {
        match chunk {
            Ok(text) => full.push_str(&text),
            Err(e) => {
                tracing::warn!("AI stream error during novelty assessment: {e}");
                break;
            }
        }
    }
    Ok(full)
}

fn parse_assessments(
    ai_response: &str,
    results: Vec<RawPatentResult>,
) -> Result<Vec<AssessedResult>> {
    // Try to find JSON array in the response
    let json_str = extract_json_array(ai_response);

    let assessments: Vec<AiAssessment> = serde_json::from_str(&json_str).unwrap_or_else(|e| {
        tracing::warn!("Failed to parse AI assessment JSON: {e}. Falling back to defaults.");
        Vec::new()
    });

    let mut assessed: Vec<AssessedResult> = results
        .into_iter()
        .enumerate()
        .map(|(i, raw)| {
            let matching = assessments
                .iter()
                .find(|a| a.index == (i + 1) as i32);

            AssessedResult {
                raw,
                similarity_score: matching.map(|a| a.similarity_score.clamp(0.0, 1.0)).unwrap_or(0.0),
                novelty_assessment: matching
                    .map(|a| a.novelty_assessment.clone())
                    .unwrap_or_else(|| "Assessment not available.".to_string()),
            }
        })
        .collect();

    // Sort by similarity score descending
    assessed.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));

    Ok(assessed)
}

fn extract_json_array(text: &str) -> String {
    // Find the first '[' and last ']' to extract the JSON array
    if let (Some(start), Some(end)) = (text.find('['), text.rfind(']')) {
        if start < end {
            return text[start..=end].to_string();
        }
    }
    "[]".to_string()
}

#[derive(Debug, serde::Deserialize)]
struct AiAssessment {
    index: i32,
    similarity_score: f32,
    novelty_assessment: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_array() {
        let input = r#"Here is the assessment:
[{"index": 1, "similarity_score": 0.8, "novelty_assessment": "Highly similar."}]
That's all."#;
        let result = extract_json_array(input);
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));

        let parsed: Vec<AiAssessment> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].index, 1);
    }

    #[test]
    fn test_extract_json_array_missing() {
        assert_eq!(extract_json_array("no json here"), "[]");
    }
}
