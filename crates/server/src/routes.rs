//! API route handlers.

use crate::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

/// Build all API routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/stats", get(stats))
        .route("/config/tools", get(config_tools))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok", "service": "nexus"}))
}

async fn stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();

    let total_files: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM files",
            [],
            |row: &rusqlite::Row<'_>| row.get(0),
        )
        .unwrap_or(0);
    let total_size: i64 = db
        .query_row(
            "SELECT COALESCE(SUM(size), 0) FROM files",
            [],
            |row: &rusqlite::Row<'_>| row.get(0),
        )
        .unwrap_or(0);
    let tool_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM config_tools",
            [],
            |row: &rusqlite::Row<'_>| row.get(0),
        )
        .unwrap_or(0);

    Json(serde_json::json!({
        "total_files": total_files,
        "total_size": total_size,
        "config_tools": tool_count,
    }))
}

async fn config_tools(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();

    let mut stmt = db
        .prepare("SELECT name, config_dir, description, language FROM config_tools ORDER BY name")
        .unwrap();

    let tools: Vec<serde_json::Value> = stmt
        .query_map([], |row: &rusqlite::Row<'_>| {
            Ok(serde_json::json!({
                "name": row.get::<_, String>(0)?,
                "config_dir": row.get::<_, String>(1)?,
                "description": row.get::<_, String>(2).unwrap_or_default(),
                "language": row.get::<_, String>(3).unwrap_or_default(),
            }))
        })
        .unwrap()
        .filter_map(|r: std::result::Result<serde_json::Value, _>| r.ok())
        .collect();

    Json(serde_json::json!(tools))
}
