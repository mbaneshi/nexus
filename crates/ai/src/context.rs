//! Build context from the database for LLM queries.

use nexus_core::Result;
use nexus_core::output::format_size;
use rusqlite::Connection;

/// Build a context string summarizing the user's home directory for an AI query.
pub fn build_context(conn: &Connection) -> Result<String> {
    let mut parts = Vec::new();

    // Overall stats
    let total_files: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM files",
            [],
            |row: &rusqlite::Row<'_>| row.get(0),
        )
        .unwrap_or(0);
    let total_size: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(size), 0) FROM files",
            [],
            |row: &rusqlite::Row<'_>| row.get(0),
        )
        .unwrap_or(0);

    parts.push(format!(
        "Home directory: {total_files} files, {} total",
        format_size(total_size as u64)
    ));

    // Category breakdown
    let mut stmt = conn.prepare(
        "SELECT category, COUNT(*), SUM(size) FROM files WHERE is_dir = 0 GROUP BY category ORDER BY SUM(size) DESC LIMIT 10",
    )?;
    let categories: Vec<(String, i64, i64)> = stmt
        .query_map([], |row: &rusqlite::Row<'_>| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r: std::result::Result<(String, i64, i64), _>| r.ok())
        .collect();

    if !categories.is_empty() {
        parts.push("\nBy category:".to_string());
        for (cat, count, size) in &categories {
            parts.push(format!(
                "  {cat}: {count} files ({})",
                format_size(*size as u64)
            ));
        }
    }

    // Config tools
    let tool_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM config_tools",
            [],
            |row: &rusqlite::Row<'_>| row.get(0),
        )
        .unwrap_or(0);
    if tool_count > 0 {
        parts.push(format!("\nConfig tools installed: {tool_count}"));
    }

    // Recent changes
    let mut stmt = conn
        .prepare("SELECT path, change_type FROM file_changes ORDER BY detected_at DESC LIMIT 10")?;
    let changes: Vec<(String, String)> = stmt
        .query_map([], |row: &rusqlite::Row<'_>| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r: std::result::Result<(String, String), _>| r.ok())
        .collect();

    if !changes.is_empty() {
        parts.push("\nRecent changes:".to_string());
        for (path, change_type) in &changes {
            parts.push(format!("  [{change_type}] {path}"));
        }
    }

    Ok(parts.join("\n"))
}
