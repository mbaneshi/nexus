//! Server route integration tests.

use crate::{AppState, api_router};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

fn test_state() -> AppState {
    let conn = nexus_core::db::open_in_memory().unwrap();
    AppState {
        db: Arc::new(Mutex::new(conn)),
    }
}

async fn get(app: axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

async fn post_json(
    app: axum::Router,
    uri: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

#[tokio::test]
async fn health_endpoint() {
    let app = api_router(test_state());
    let (status, json) = get(app, "/api/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "nexus");
}

#[tokio::test]
async fn stats_endpoint_empty_db() {
    let app = api_router(test_state());
    let (status, json) = get(app, "/api/stats").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["total_files"], 0);
    assert_eq!(json["total_dirs"], 0);
    assert_eq!(json["config_tools"], 0);
}

#[tokio::test]
async fn config_tools_endpoint_empty() {
    let app = api_router(test_state());
    let (status, json) = get(app, "/api/config/tools").await;
    assert_eq!(status, StatusCode::OK);
    assert!(json.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn config_snapshots_endpoint_empty() {
    let app = api_router(test_state());
    let (status, json) = get(app, "/api/config/snapshots").await;
    assert_eq!(status, StatusCode::OK);
    assert!(json.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn daemon_status_endpoint() {
    let app = api_router(test_state());
    let (status, json) = get(app, "/api/daemon/status").await;
    assert_eq!(status, StatusCode::OK);
    // Daemon shouldn't be running in tests
    assert!(json.get("running").is_some());
}

#[tokio::test]
async fn changes_endpoint_empty() {
    let app = api_router(test_state());
    let (status, json) = get(app, "/api/changes").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 0);
    assert!(json["changes"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn changes_endpoint_with_data() {
    let state = test_state();
    {
        let conn = state.db.lock().unwrap();
        let change = nexus_core::models::FileChange {
            id: None,
            path: std::path::PathBuf::from("/tmp/test.txt"),
            change_type: nexus_core::models::ChangeType::Created,
            detected_at: 1000,
            old_size: None,
            new_size: Some(42),
        };
        nexus_core::db::record_change(&conn, &change).unwrap();
    }

    let app = api_router(state);
    let (status, json) = get(app, "/api/changes").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 1);
    assert_eq!(json["changes"][0]["change_type"], "created");
    assert_eq!(json["changes"][0]["new_size"], 42);
}

#[tokio::test]
async fn search_endpoint_empty_db() {
    let app = api_router(test_state());
    let (status, json) = get(app, "/api/search?q=rust").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["count"], 0);
}

#[tokio::test]
async fn config_diff_unknown_tool() {
    let app = api_router(test_state());
    let (status, json) = get(app, "/api/config/nonexistent/diff").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn config_restore_nonexistent() {
    let app = api_router(test_state());
    let (status, json) = post_json(app, "/api/config/restore/999", serde_json::json!({})).await;
    // No snapshot files to restore, returns 0
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["restored_files"], 0);
}
