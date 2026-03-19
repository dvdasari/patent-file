use anyhow::Result;
use serde::Serialize;
use tokio::sync::mpsc;

use crate::prompts::{build_prompt, SECTION_ORDER};
use crate::provider::LlmProvider;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum SseEvent {
    #[serde(rename = "section_start")]
    SectionStart {
        section_type: String,
        index: usize,
        total: usize,
    },
    #[serde(rename = "content_delta")]
    ContentDelta {
        section_type: String,
        delta: String,
    },
    #[serde(rename = "section_complete")]
    SectionComplete {
        section_type: String,
        content: String,
    },
    #[serde(rename = "generation_complete")]
    GenerationComplete {
        project_id: String,
        sections_generated: usize,
    },
    #[serde(rename = "error")]
    Error {
        section_type: String,
        message: String,
        recoverable: bool,
    },
}

pub struct GenerationPipeline;

impl GenerationPipeline {
    pub fn run(
        project_id: uuid::Uuid,
        interview_context: String,
        figure_descriptions: String,
        existing_sections: Vec<(String, String)>,
        provider: &dyn LlmProvider,
    ) -> Result<mpsc::Receiver<SseEvent>> {
        let (tx, rx) = mpsc::channel(64);

        // Determine which sections need generating
        let existing_types: Vec<&str> = existing_sections.iter().map(|(t, _)| t.as_str()).collect();
        let sections_to_generate: Vec<&str> = SECTION_ORDER
            .iter()
            .filter(|s| !existing_types.contains(s))
            .copied()
            .collect();

        let total = sections_to_generate.len();

        // Build previous sections context
        let mut prev_context = String::new();
        for (t, c) in &existing_sections {
            prev_context.push_str(&format!("### {}\n{}\n\n", t.replace('_', " "), c));
        }

        // Start generation streams for each section
        let mut streams: Vec<(&str, mpsc::Receiver<Result<String>>)> = Vec::new();
        for section_type in &sections_to_generate {
            let prompt = build_prompt(section_type, &interview_context, &prev_context, &figure_descriptions);
            let stream = provider.generate_stream(prompt)?;
            streams.push((section_type, stream));
        }

        tokio::spawn(async move {
            let mut generated_count = 0;

            for (index, (section_type, mut stream)) in streams.into_iter().enumerate() {
                let section_type_str = section_type.to_string();

                // Emit section_start
                let _ = tx.send(SseEvent::SectionStart {
                    section_type: section_type_str.clone(),
                    index,
                    total,
                }).await;

                let mut full_content = String::new();
                let mut had_error = false;

                while let Some(chunk_result) = stream.recv().await {
                    match chunk_result {
                        Ok(delta) => {
                            full_content.push_str(&delta);
                            let _ = tx.send(SseEvent::ContentDelta {
                                section_type: section_type_str.clone(),
                                delta,
                            }).await;
                        }
                        Err(e) => {
                            let _ = tx.send(SseEvent::Error {
                                section_type: section_type_str.clone(),
                                message: e.to_string(),
                                recoverable: true,
                            }).await;
                            had_error = true;
                            break;
                        }
                    }
                }

                if had_error {
                    break;
                }

                // Emit section_complete
                let _ = tx.send(SseEvent::SectionComplete {
                    section_type: section_type_str,
                    content: full_content,
                }).await;

                generated_count += 1;
            }

            // Emit generation_complete
            let _ = tx.send(SseEvent::GenerationComplete {
                project_id: project_id.to_string(),
                sections_generated: generated_count,
            }).await;
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockProvider;

    #[tokio::test]
    async fn test_pipeline_generates_all_sections() {
        let provider = MockProvider::new();
        let project_id = uuid::Uuid::new_v4();

        let mut rx = GenerationPipeline::run(
            project_id,
            "Test invention".to_string(),
            "".to_string(),
            vec![],
            &provider,
        ).unwrap();

        let mut section_starts = 0;
        let mut section_completes = 0;
        let mut generation_complete = false;

        while let Some(event) = rx.recv().await {
            match event {
                SseEvent::SectionStart { .. } => section_starts += 1,
                SseEvent::SectionComplete { .. } => section_completes += 1,
                SseEvent::GenerationComplete { sections_generated, .. } => {
                    assert_eq!(sections_generated, 8);
                    generation_complete = true;
                }
                _ => {}
            }
        }

        assert_eq!(section_starts, 8);
        assert_eq!(section_completes, 8);
        assert!(generation_complete);
    }

    #[tokio::test]
    async fn test_pipeline_skips_existing_sections() {
        let provider = MockProvider::new();
        let project_id = uuid::Uuid::new_v4();

        let existing = vec![
            ("title".to_string(), "Existing Title".to_string()),
            ("field_of_invention".to_string(), "Existing Field".to_string()),
        ];

        let mut rx = GenerationPipeline::run(
            project_id,
            "Test".to_string(),
            "".to_string(),
            existing,
            &provider,
        ).unwrap();

        let mut generated = 0;
        while let Some(event) = rx.recv().await {
            if let SseEvent::GenerationComplete { sections_generated, .. } = event {
                generated = sections_generated;
            }
        }

        assert_eq!(generated, 6); // 8 - 2 existing
    }
}
