use anyhow::Result;
use tokio::sync::mpsc;

pub struct Prompt {
    pub system: String,
    pub user: String,
}

pub trait LlmProvider: Send + Sync {
    fn generate_stream(&self, prompt: Prompt) -> Result<mpsc::Receiver<Result<String>>>;
}

pub fn create_provider(
    provider_name: &str,
    anthropic_api_key: Option<&str>,
) -> Result<Box<dyn LlmProvider>> {
    match provider_name {
        "mock" => Ok(Box::new(super::mock::MockProvider::new())),
        "anthropic" => {
            let key = anthropic_api_key.ok_or_else(|| anyhow::anyhow!("ANTHROPIC_API_KEY required for anthropic provider"))?;
            Ok(Box::new(super::anthropic::AnthropicProvider::new(key)))
        }
        other => anyhow::bail!("Unknown AI provider: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mock_provider() {
        let p = create_provider("mock", None);
        assert!(p.is_ok());
    }

    #[test]
    fn test_create_anthropic_requires_key() {
        let p = create_provider("anthropic", None);
        assert!(p.is_err());
    }

    #[test]
    fn test_unknown_provider() {
        let p = create_provider("unknown", None);
        assert!(p.is_err());
    }
}
