//! Claude API adapter implementing LlmPort.

use nexus_core::Result;
use nexus_core::ports::LlmPort;

/// Claude API provider.
pub struct ClaudeProvider {
    api_key: String,
    model: String,
    max_tokens: u32,
    client: reqwest::Client,
}

impl ClaudeProvider {
    /// Create a new Claude provider.
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Self {
        Self {
            api_key,
            model,
            max_tokens,
            client: reqwest::Client::new(),
        }
    }

    /// Create from nexus AI config, falling back to ANTHROPIC_API_KEY env var.
    pub fn from_config(config: &nexus_core::config::AiConfig) -> Option<Self> {
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())?;

        Some(Self::new(api_key, config.model.clone(), config.max_tokens))
    }
}

#[async_trait::async_trait]
impl LlmPort for ClaudeProvider {
    async fn generate(&self, prompt: &str, system: Option<&str>) -> Result<String> {
        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });

        if let Some(sys) = system {
            body["system"] = serde_json::Value::String(sys.to_string());
        }

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| nexus_core::NexusError::Ai(e.to_string()))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| nexus_core::NexusError::Ai(e.to_string()))?;

        let text = json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(text)
    }
}
