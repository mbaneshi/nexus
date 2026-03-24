//! API route handlers.

use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

/// Build all API routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/stats", get(stats))
        .route("/search", get(search))
        .route("/config/tools", get(config_tools))
        .route("/config/snapshots", get(config_snapshots))
        .route("/config/backup", post(config_backup))
        .route("/config/restore/{id}", post(config_restore))
        .route("/config/{tool}/diff", get(config_diff))
        .route("/daemon/status", get(daemon_status))
        .route("/changes", get(recent_changes))
}

/// JSON error response.
fn api_error(status: StatusCode, msg: String) -> Response {
    (status, Json(serde_json::json!({"error": msg}))).into_response()
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok", "service": "nexus"}))
}

async fn stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();

    let stats = nexus_discovery::home_stats(&db).ok();

    match stats {
        Some(s) => Json(serde_json::json!({
            "total_files": s.total_files,
            "total_dirs": s.total_dirs,
            "total_size": s.total_size,
            "by_category": s.by_category,
            "last_scan": s.last_scan,
            "config_tools": db.query_row(
                "SELECT COUNT(*) FROM config_tools", [],
                |row: &rusqlite::Row<'_>| row.get::<_, i64>(0)
            ).unwrap_or(0),
        })),
        None => Json(serde_json::json!({
            "total_files": 0,
            "total_dirs": 0,
            "total_size": 0,
            "by_category": [],
            "last_scan": null,
            "config_tools": 0,
        })),
    }
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    category: Option<String>,
    limit: Option<usize>,
}

async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Response {
    let db = state.db.lock().unwrap();

    let category = params
        .category
        .as_deref()
        .map(nexus_core::models::FileCategory::from_str_lossy);

    let query = nexus_core::models::SearchQuery {
        text: params.q,
        category,
        limit: params.limit.unwrap_or(50),
        ..Default::default()
    };

    match nexus_discovery::search(&db, &query) {
        Ok(results) => Json(serde_json::json!({
            "count": results.len(),
            "results": results,
        }))
        .into_response(),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
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

async fn config_snapshots(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();

    match nexus_configs::list_snapshots(&db, None) {
        Ok(snapshots) => Json(serde_json::json!(snapshots)),
        Err(_) => Json(serde_json::json!([])),
    }
}

#[derive(Deserialize)]
struct BackupParams {
    tool: Option<String>,
    label: Option<String>,
}

async fn config_backup(
    State(state): State<AppState>,
    Json(params): Json<BackupParams>,
) -> Response {
    let db = state.db.lock().unwrap();

    let (tool_id, config_dir) = if let Some(ref tool_name) = params.tool {
        let result: Option<(i64, String)> = db
            .query_row(
                "SELECT id, config_dir FROM config_tools WHERE name = ?1",
                [tool_name],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        match result {
            Some((id, dir)) => (Some(id), std::path::PathBuf::from(dir)),
            None => {
                return api_error(
                    StatusCode::NOT_FOUND,
                    format!("tool '{tool_name}' not found"),
                );
            }
        }
    } else {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        (None, home.join(".config"))
    };

    match nexus_configs::create_snapshot(&db, tool_id, params.label.as_deref(), &config_dir) {
        Ok(id) => Json(serde_json::json!({"snapshot_id": id})).into_response(),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn config_restore(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Response {
    let db = state.db.lock().unwrap();

    match nexus_configs::restore_snapshot(&db, id) {
        Ok(count) => Json(serde_json::json!({"restored_files": count})).into_response(),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn config_diff(
    State(state): State<AppState>,
    Path(tool_name): Path<String>,
) -> Response {
    let db = state.db.lock().unwrap();

    let tool: Option<(i64, String)> = db
        .query_row(
            "SELECT id, config_dir FROM config_tools WHERE name = ?1",
            [&tool_name],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    let (tool_id, config_dir) = match tool {
        Some(t) => t,
        None => {
            return api_error(
                StatusCode::NOT_FOUND,
                format!("tool '{tool_name}' not found"),
            );
        }
    };

    let snapshots = nexus_configs::list_snapshots(&db, Some(tool_id)).unwrap_or_default();
    let latest = match snapshots.first() {
        Some(s) => s,
        None => {
            return api_error(
                StatusCode::NOT_FOUND,
                format!("no snapshots for '{tool_name}'"),
            );
        }
    };

    match nexus_configs::diff_snapshot(&db, latest.id, std::path::Path::new(&config_dir)) {
        Ok(diffs) => Json(serde_json::json!({
            "tool": tool_name,
            "snapshot_id": latest.id,
            "changes": diffs,
        }))
        .into_response(),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn daemon_status() -> Json<serde_json::Value> {
    let status = nexus_watcher::daemon_status();
    match status {
        nexus_watcher::DaemonStatus::Running { pid } => {
            Json(serde_json::json!({"running": true, "pid": pid}))
        }
        nexus_watcher::DaemonStatus::Stopped => {
            Json(serde_json::json!({"running": false}))
        }
    }
}

#[derive(Deserialize)]
struct ChangesParams {
    limit: Option<usize>,
}

async fn recent_changes(
    State(state): State<AppState>,
    Query(params): Query<ChangesParams>,
) -> Json<serde_json::Value> {
    let db = state.db.lock().unwrap();
    let limit = params.limit.unwrap_or(100) as i64;

    let mut stmt = db
        .prepare(
            "SELECT path, change_type, detected_at, new_size
             FROM file_changes
             ORDER BY detected_at DESC
             LIMIT ?1",
        )
        .ok();

    let changes: Vec<serde_json::Value> = stmt
        .as_mut()
        .and_then(|s| {
            s.query_map([limit], |row: &rusqlite::Row<'_>| {
                Ok(serde_json::json!({
                    "path": row.get::<_, String>(0)?,
                    "change_type": row.get::<_, String>(1)?,
                    "detected_at": row.get::<_, i64>(2)?,
                    "new_size": row.get::<_, Option<i64>>(3)?,
                }))
            })
            .ok()
        })
        .map(|r| r.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    Json(serde_json::json!({
        "count": changes.len(),
        "changes": changes,
    }))
}
