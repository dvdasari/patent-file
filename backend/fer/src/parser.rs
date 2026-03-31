use anyhow::Result;
use tracing;

use crate::models::ParsedFer;
use crate::prompts::build_parse_prompt;
use ai::LlmProvider;

/// Parse a FER text using AI to extract structured objections
pub async fn parse_fer(provider: &dyn LlmProvider, fer_text: &str) -> Result<ParsedFer> {
    let prompt = build_parse_prompt(fer_text);
    let mut rx = provider.generate_stream(prompt)?;

    let mut full_response = String::new();
    while let Some(chunk) = rx.recv().await {
        match chunk {
            Ok(text) => full_response.push_str(&text),
            Err(e) => {
                tracing::error!("FER parse stream error: {e}");
                return Err(e);
            }
        }
    }

    // Extract JSON from response (may be wrapped in ```json ... ```)
    let json_str = extract_json(&full_response);

    let parsed: ParsedFer = serde_json::from_str(json_str).map_err(|e| {
        tracing::error!("Failed to parse FER JSON: {e}\nRaw response: {full_response}");
        anyhow::anyhow!("Failed to parse FER analysis result: {e}")
    })?;

    Ok(parsed)
}

/// Extract JSON from a response that may contain markdown code fences
fn extract_json(text: &str) -> &str {
    let trimmed = text.trim();

    // Try to find ```json ... ``` block
    if let Some(start) = trimmed.find("```json") {
        let after_fence = &trimmed[start + 7..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }

    // Try ``` ... ``` block
    if let Some(start) = trimmed.find("```") {
        let after_fence = &trimmed[start + 3..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }

    // Assume the whole thing is JSON
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_plain() {
        let input = r#"{"examiner_name": "Test"}"#;
        assert_eq!(extract_json(input), input);
    }

    #[test]
    fn test_extract_json_fenced() {
        let input = "```json\n{\"examiner_name\": \"Test\"}\n```";
        assert_eq!(extract_json(input), r#"{"examiner_name": "Test"}"#);
    }

    #[test]
    fn test_extract_json_fenced_no_lang() {
        let input = "```\n{\"examiner_name\": \"Test\"}\n```";
        assert_eq!(extract_json(input), r#"{"examiner_name": "Test"}"#);
    }
}
