//! AI integration via Claude API for natural language filesystem queries.
//!
//! Implements the `LlmPort` trait from `nexus-core` using the Claude API.

mod claude;
mod context;
mod queries;

pub use claude::ClaudeProvider;
pub use context::build_context;
pub use queries::{CONFIG_QUERY, FILESYSTEM_QUERY, QueryTemplate};
