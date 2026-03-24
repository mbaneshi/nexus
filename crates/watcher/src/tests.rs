//! Watcher and daemon tests.

use super::*;
use std::sync::{Arc, Mutex};

#[test]
fn watch_detects_file_creation() {
    let dir = tempfile::tempdir().unwrap();
    let changes: Arc<Mutex<Vec<nexus_core::models::FileChange>>> =
        Arc::new(Mutex::new(Vec::new()));

    let config = WatcherConfig {
        watch_paths: vec![dir.path().to_path_buf()],
        debounce_secs: 0, // no debounce for testing
    };

    let changes_clone = Arc::clone(&changes);
    let _handle = watch(&config, move |change| {
        changes_clone.lock().unwrap().push(change);
    })
    .unwrap();

    // Wait for watcher to start
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Create a file
    let file_path = dir.path().join("test.txt");
    std::fs::write(&file_path, "hello").unwrap();

    // Poll for change detection (up to 2 seconds)
    let mut detected = false;
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if !changes.lock().unwrap().is_empty() {
            detected = true;
            break;
        }
    }

    assert!(detected, "should detect at least one change");
}

#[test]
fn watch_detects_modification() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("modify.txt");
    std::fs::write(&file_path, "initial").unwrap();

    let changes: Arc<Mutex<Vec<nexus_core::models::FileChange>>> =
        Arc::new(Mutex::new(Vec::new()));

    let config = WatcherConfig {
        watch_paths: vec![dir.path().to_path_buf()],
        debounce_secs: 0,
    };

    let changes_clone = Arc::clone(&changes);
    let _handle = watch(&config, move |change| {
        changes_clone.lock().unwrap().push(change);
    })
    .unwrap();

    // Small delay to ensure watcher is set up
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Modify the file
    std::fs::write(&file_path, "modified content").unwrap();

    std::thread::sleep(std::time::Duration::from_millis(500));

    let recorded = changes.lock().unwrap();
    assert!(!recorded.is_empty(), "should detect modification");
}

#[test]
fn watch_nonexistent_path_ignored() {
    let config = WatcherConfig {
        watch_paths: vec![std::path::PathBuf::from("/nonexistent/path/12345")],
        debounce_secs: 1,
    };

    // Should succeed even with nonexistent path (just skips it)
    let _handle = watch(&config, |_| {}).unwrap();
}

#[test]
fn daemon_status_returns_stopped() {
    // Clean any stale PID file
    let pid_path = pid_file_path();
    let _ = std::fs::remove_file(&pid_path);

    let status = daemon_status();
    assert!(
        matches!(status, DaemonStatus::Stopped),
        "daemon should be stopped"
    );
    assert!(!is_daemon_running());
}
