//! `nexus ask` — AI-powered filesystem queries.

use color_eyre::eyre;
use nexus_core::config::Config;
use nexus_core::ports::LlmPort;
use rusqlite::Connection;

pub fn run(conn: &Connection, config: &Config, question: &str) -> eyre::Result<()> {
    let provider = nexus_ai::ClaudeProvider::from_config(&config.ai)
        .ok_or_else(|| eyre::eyre!(
            "No API key configured. Set ANTHROPIC_API_KEY or add api_key to ~/.config/nexus/config.toml"
        ))?;

    let context = nexus_ai::build_context(conn)?;

    let prompt =
        format!("Context about the user's filesystem:\n{context}\n\nUser question: {question}");

    let rt = tokio::runtime::Runtime::new()?;
    let response =
        rt.block_on(provider.generate(&prompt, Some(nexus_ai::FILESYSTEM_QUERY.system_prompt)))?;

    println!("{response}");

    // Store query in history
    conn.execute(
        "INSERT INTO ai_queries (query, context, response, model) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![question, context, response, config.ai.model],
    )?;

    Ok(())
}
