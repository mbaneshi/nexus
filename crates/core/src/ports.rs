//! Port traits — abstractions for external boundaries.
//!
//! Only two ports: LLM access and config storage.
//! Everything else uses concrete types (SQLite, filesystem).

use crate::error::Result;

/// Port for LLM access (Claude API, Ollama, etc.).
#[async_trait::async_trait]
pub trait LlmPort: Send + Sync {
    /// Generate a text response from a prompt.
    async fn generate(&self, prompt: &str, system: Option<&str>) -> Result<String>;
}
