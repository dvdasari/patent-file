use anyhow::Result;
use serde::Serialize;
use tokio::sync::mpsc;

use crate::models::FerObjection;
use crate::prompts::build_response_prompt;
use ai::LlmProvider;

/// SSE events for FER response generation
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum FerSseEvent {
    #[serde(rename = "objection_start")]
    ObjectionStart {
        objection_id: String,
        objection_number: i32,
        index: usize,
        total: usize,
    },
    #[serde(rename = "content_delta")]
    ContentDelta {
        objection_id: String,
        delta: String,
    },
    #[serde(rename = "objection_complete")]
    ObjectionComplete {
        objection_id: String,
        legal_arguments: String,
        claim_amendments: String,
        case_law_citations: String,
    },
    #[serde(rename = "generation_complete")]
    GenerationComplete {
        analysis_id: String,
        responses_generated: usize,
    },
    #[serde(rename = "error")]
    Error {
        objection_id: String,
        message: String,
    },
}

pub struct FerResponsePipeline;

impl FerResponsePipeline {
    pub fn run(
        analysis_id: uuid::Uuid,
        fer_text: &str,
        objections: Vec<FerObjection>,
        provider: &dyn LlmProvider,
    ) -> Result<mpsc::Receiver<FerSseEvent>> {
        let (tx, rx) = mpsc::channel(64);
        let total = objections.len();

        // Pre-build all prompts and streams
        let mut streams: Vec<(FerObjection, mpsc::Receiver<Result<String>>)> = Vec::new();
        for objection in &objections {
            let prompt = build_response_prompt(
                &objection.full_text,
                &objection.category,
                objection.section_reference.as_deref(),
                fer_text,
            );
            let stream = provider.generate_stream(prompt)?;
            streams.push((objection.clone(), stream));
        }

        tokio::spawn(async move {
            let mut generated_count = 0;

            for (index, (objection, mut stream)) in streams.into_iter().enumerate() {
                let obj_id = objection.id.to_string();

                let _ = tx
                    .send(FerSseEvent::ObjectionStart {
                        objection_id: obj_id.clone(),
                        objection_number: objection.objection_number,
                        index,
                        total,
                    })
                    .await;

                let mut full_content = String::new();
                let mut had_error = false;

                while let Some(chunk_result) = stream.recv().await {
                    match chunk_result {
                        Ok(delta) => {
                            full_content.push_str(&delta);
                            let _ = tx
                                .send(FerSseEvent::ContentDelta {
                                    objection_id: obj_id.clone(),
                                    delta,
                                })
                                .await;
                        }
                        Err(e) => {
                            let _ = tx
                                .send(FerSseEvent::Error {
                                    objection_id: obj_id.clone(),
                                    message: e.to_string(),
                                })
                                .await;
                            had_error = true;
                            break;
                        }
                    }
                }

                if had_error {
                    break;
                }

                // Parse the three sections from the AI response
                let (legal, amendments, citations) = parse_response_sections(&full_content);

                let _ = tx
                    .send(FerSseEvent::ObjectionComplete {
                        objection_id: obj_id,
                        legal_arguments: legal,
                        claim_amendments: amendments,
                        case_law_citations: citations,
                    })
                    .await;

                generated_count += 1;
            }

            let _ = tx
                .send(FerSseEvent::GenerationComplete {
                    analysis_id: analysis_id.to_string(),
                    responses_generated: generated_count,
                })
                .await;
        });

        Ok(rx)
    }
}

/// Parse AI response into three sections based on markdown headings (public for reuse in routes)
pub fn parse_response_sections_public(content: &str) -> (String, String, String) {
    parse_response_sections(content)
}

fn parse_response_sections(content: &str) -> (String, String, String) {
    let mut legal = String::new();
    let mut amendments = String::new();
    let mut citations = String::new();

    let mut current_section: Option<&str> = None;

    for line in content.lines() {
        let lower = line.to_lowercase();
        if lower.contains("## legal argument") {
            current_section = Some("legal");
            continue;
        } else if lower.contains("## suggested claim") || lower.contains("## claim amendment") {
            current_section = Some("amendments");
            continue;
        } else if lower.contains("## indian case law") || lower.contains("## case law") || lower.contains("## citation") {
            current_section = Some("citations");
            continue;
        }

        match current_section {
            Some("legal") => {
                legal.push_str(line);
                legal.push('\n');
            }
            Some("amendments") => {
                amendments.push_str(line);
                amendments.push('\n');
            }
            Some("citations") => {
                citations.push_str(line);
                citations.push('\n');
            }
            _ => {
                // Content before any heading goes to legal arguments
                if !line.trim().is_empty() {
                    legal.push_str(line);
                    legal.push('\n');
                }
            }
        }
    }

    (
        legal.trim().to_string(),
        amendments.trim().to_string(),
        citations.trim().to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_sections() {
        let content = r#"## Legal Arguments
The applicant submits that the claimed invention satisfies novelty.

## Suggested Claim Amendments
Amend claim 1 to specify the dosage form.

## Indian Case Law & Citations
Novartis v. Union of India (2013) — landmark Section 3(d) case.
"#;

        let (legal, amendments, citations) = parse_response_sections(content);
        assert!(legal.contains("novelty"));
        assert!(amendments.contains("dosage form"));
        assert!(citations.contains("Novartis"));
    }
}
