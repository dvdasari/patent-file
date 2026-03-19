use anyhow::Result;
use tokio::sync::mpsc;

use crate::provider::{LlmProvider, Prompt};

pub struct AnthropicProvider {
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

impl LlmProvider for AnthropicProvider {
    fn generate_stream(&self, prompt: Prompt) -> Result<mpsc::Receiver<Result<String>>> {
        let (tx, rx) = mpsc::channel(64);
        let api_key = self.api_key.clone();

        tokio::spawn(async move {
            let client = reqwest::Client::new();

            let body = serde_json::json!({
                "model": "claude-sonnet-4-20250514",
                "max_tokens": 4096,
                "system": prompt.system,
                "messages": [{"role": "user", "content": prompt.user}],
                "stream": true,
            });

            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await;

            let response = match response {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Anthropic API error: {}", e))).await;
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                let _ = tx.send(Err(anyhow::anyhow!("Anthropic API {}: {}", status, text))).await;
                return;
            }

            // Parse SSE stream
            let mut stream = response.bytes_stream();
            use futures::StreamExt;
            let mut buffer = String::new();

            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(Err(anyhow::anyhow!("Stream error: {}", e))).await;
                        break;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                // Process complete SSE lines
                while let Some(pos) = buffer.find("\n\n") {
                    let event_block = buffer[..pos].to_string();
                    buffer = buffer[pos + 2..].to_string();

                    for line in event_block.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                return;
                            }
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                                if parsed["type"] == "content_block_delta" {
                                    if let Some(text) = parsed["delta"]["text"].as_str() {
                                        if tx.send(Ok(text.to_string())).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}
