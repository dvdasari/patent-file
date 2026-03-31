use anyhow::{Context, Result};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub port: u16,
    pub ai_provider: String,
    // Storage
    pub storage_backend: String,
    pub storage_local_path: Option<String>,
    // Cloudflare R2 (only required when storage_backend = "r2")
    pub r2_account_id: Option<String>,
    pub r2_access_key_id: Option<String>,
    pub r2_secret_access_key: Option<String>,
    pub r2_bucket_name: Option<String>,
    pub r2_public_url: Option<String>,
    // Razorpay
    pub razorpay_key_id: String,
    pub razorpay_key_secret: String,
    pub razorpay_webhook_secret: String,
    pub razorpay_plan_id: String,
    // CORS
    pub allowed_origin: String,
    // Patent search
    pub lens_api_key: Option<String>,
    pub patent_search_provider: String,
    // OAuth
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub linkedin_client_id: Option<String>,
    pub linkedin_client_secret: Option<String>,
    pub oauth_redirect_base_url: String,
}

impl AppConfig {
    /// Build config from a map (testable without env mutation).
    pub fn from_map(vars: &HashMap<String, String>) -> Result<Self> {
        let get = |key: &str| -> Option<String> { vars.get(key).cloned() };
        let require = |key: &str| -> Result<String> {
            get(key).context(format!("{key} must be set"))
        };

        let ai_provider = get("AI_PROVIDER").unwrap_or_else(|| "anthropic".to_string());

        Ok(Self {
            database_url: require("DATABASE_URL")?,
            jwt_secret: require("JWT_SECRET")?,
            anthropic_api_key: get("ANTHROPIC_API_KEY"),
            openai_api_key: get("OPENAI_API_KEY"),
            port: get("PORT")
                .unwrap_or_else(|| "5012".to_string())
                .parse()
                .context("PORT must be a valid u16")?,
            ai_provider,
            storage_backend: get("STORAGE_BACKEND").unwrap_or_else(|| "local".to_string()),
            storage_local_path: get("STORAGE_LOCAL_PATH"),
            r2_account_id: get("R2_ACCOUNT_ID"),
            r2_access_key_id: get("R2_ACCESS_KEY_ID"),
            r2_secret_access_key: get("R2_SECRET_ACCESS_KEY"),
            r2_bucket_name: get("R2_BUCKET_NAME"),
            r2_public_url: get("R2_PUBLIC_URL"),
            razorpay_key_id: require("RAZORPAY_KEY_ID")?,
            razorpay_key_secret: require("RAZORPAY_KEY_SECRET")?,
            razorpay_webhook_secret: require("RAZORPAY_WEBHOOK_SECRET")?,
            razorpay_plan_id: require("RAZORPAY_PLAN_ID")?,
            allowed_origin: get("ALLOWED_ORIGIN")
                .unwrap_or_else(|| "http://localhost:3000".to_string()),
            lens_api_key: get("LENS_API_KEY"),
            patent_search_provider: get("PATENT_SEARCH_PROVIDER")
                .unwrap_or_else(|| "mock".to_string()),
            google_client_id: get("GOOGLE_CLIENT_ID"),
            google_client_secret: get("GOOGLE_CLIENT_SECRET"),
            linkedin_client_id: get("LINKEDIN_CLIENT_ID"),
            linkedin_client_secret: get("LINKEDIN_CLIENT_SECRET"),
            oauth_redirect_base_url: get("OAUTH_REDIRECT_BASE_URL")
                .unwrap_or_else(|| "http://localhost:5012".to_string()),
        })
    }

    /// Convenience: read from real environment variables.
    pub fn from_env() -> Result<Self> {
        let vars: HashMap<String, String> = std::env::vars().collect();
        Self::from_map(&vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn required_vars() -> HashMap<String, String> {
        HashMap::from([
            ("DATABASE_URL".into(), "postgresql://test".into()),
            (
                "JWT_SECRET".into(),
                "test-secret-64-chars-long-enough-for-hmac".into(),
            ),
            ("STORAGE_BACKEND".into(), "local".into()),
            ("RAZORPAY_KEY_ID".into(), "rzp_test".into()),
            ("RAZORPAY_KEY_SECRET".into(), "test".into()),
            ("RAZORPAY_WEBHOOK_SECRET".into(), "test".into()),
            ("RAZORPAY_PLAN_ID".into(), "plan_test".into()),
        ])
    }

    #[test]
    fn test_default_port() {
        let config = AppConfig::from_map(&required_vars()).unwrap();
        assert_eq!(config.port, 5012);
    }

    #[test]
    fn test_default_ai_provider() {
        let config = AppConfig::from_map(&required_vars()).unwrap();
        assert_eq!(config.ai_provider, "anthropic");
    }

    #[test]
    fn test_anthropic_key_optional_with_mock() {
        let mut vars = required_vars();
        vars.insert("AI_PROVIDER".into(), "mock".into());
        let config = AppConfig::from_map(&vars).unwrap();
        assert!(config.anthropic_api_key.is_none());
    }

    #[test]
    fn test_missing_required_var() {
        let vars = HashMap::new();
        assert!(AppConfig::from_map(&vars).is_err());
    }
}
