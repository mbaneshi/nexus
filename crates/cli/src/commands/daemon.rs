//! `nexus daemon` — manage the filesystem watcher daemon.

use crate::DaemonAction;
use color_eyre::eyre;
use nexus_core::config::Config;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub fn run(
    conn: &Connection,
    config: &Config,
    action: DaemonAction,
    json: bool,
) -> eyre::Result<()> {
    match action {
        DaemonAction::Start => start(config, json),
        DaemonAction::Stop => stop(json),
        DaemonAction::Status => status(json),
        DaemonAction::Run {
            watch,
            db_path,
            debounce,
        } => run_foreground(conn, watch, db_path, debounce),
    }
}

fn start(config: &Config, json: bool) -> eyre::Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_dir = home.join(".config");

    let mut watch_paths = config.watcher.watch_paths.clone();
    if watch_paths.is_empty() {
        watch_paths.push(config_dir);
    }

    let db_path = config
        .database
        .path
        .clone()
        .unwrap_or_else(nexus_core::db::default_db_path)
        .to_string_lossy()
        .to_string();

    let pid = nexus_watcher::start_daemon(&watch_paths, &db_path, config.watcher.debounce_secs)?;

    if json {
        println!("{}", serde_json::json!({"status": "started", "pid": pid}));
    } else {
        println!("Daemon started (PID {pid})");
    }
    Ok(())
}

fn stop(json: bool) -> eyre::Result<()> {
    let stopped = nexus_watcher::stop_daemon()?;
    if json {
        println!(
            "{}",
            serde_json::json!({"status": if stopped { "stopped" } else { "not_running" }})
        );
    } else if stopped {
        println!("Daemon stopped");
    } else {
        println!("Daemon was not running");
    }
    Ok(())
}

fn status(json: bool) -> eyre::Result<()> {
    let status = nexus_watcher::daemon_status();
    if json {
        let val = match &status {
            nexus_watcher::DaemonStatus::Running { pid } => {
                serde_json::json!({"running": true, "pid": pid})
            }
            nexus_watcher::DaemonStatus::Stopped => {
                serde_json::json!({"running": false})
            }
        };
        println!("{val}");
    } else {
        println!("Daemon: {status}");
    }
    Ok(())
}

fn run_foreground(
    conn: &Connection,
    watch_paths: Vec<String>,
    db_path: String,
    debounce: u64,
) -> eyre::Result<()> {
    let paths: Vec<std::path::PathBuf> = watch_paths.into_iter().map(Into::into).collect();

    let watcher_config = nexus_watcher::WatcherConfig {
        watch_paths: paths,
        debounce_secs: debounce,
    };

    // Open a separate DB connection for the watcher thread
    let db_conn = if db_path.is_empty() {
        nexus_core::db::open(&nexus_core::config::DatabaseConfig::default())?
    } else {
        nexus_core::db::open(&nexus_core::config::DatabaseConfig {
            path: Some(std::path::PathBuf::from(&db_path)),
            ..Default::default()
        })?
    };
    let db = Arc::new(Mutex::new(db_conn));

    eprintln!("Nexus daemon running (press Ctrl+C to stop)...");

    let _handle = nexus_watcher::watch(&watcher_config, {
        let db = Arc::clone(&db);
        move |change| {
            let path_str = change.path.to_string_lossy().to_string();
            tracing::info!(
                path = %path_str,
                change_type = %change.change_type.as_str(),
                "file change detected"
            );

            // Persist change to database
            if let Ok(conn) = db.lock() {
                if let Err(e) = nexus_core::db::record_change(&conn, &change) {
                    tracing::warn!(error = %e, "failed to record change");
                }

                // Auto-snapshot: if change is in a known config tool directory
                auto_snapshot_if_config(&conn, &path_str);
            }
        }
    })?;

    // Store PID
    let pid_path = nexus_watcher::pid_file_path();
    if let Some(parent) = pid_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&pid_path, std::process::id().to_string());

    // Block until signal
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .ok();

    match rx.recv() {
        Ok(()) => {}
        Err(_) => std::thread::park(),
    }

    let _ = std::fs::remove_file(&pid_path);
    let _ = std::fs::remove_file(nexus_watcher::socket_path());
    let _ = conn; // keep conn alive

    Ok(())
}

/// If a changed file belongs to a known config tool, auto-snapshot that tool.
fn auto_snapshot_if_config(conn: &Connection, path: &str) {
    // Look up if path belongs to a config tool
    let result: Option<(i64, String)> = conn
        .query_row(
            "SELECT id, config_dir FROM config_tools WHERE ?1 LIKE config_dir || '%'",
            [path],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    if let Some((tool_id, config_dir)) = result {
        let config_path = std::path::Path::new(&config_dir);
        if config_path.exists() {
            match nexus_configs::create_snapshot(conn, Some(tool_id), Some("auto"), config_path) {
                Ok(id) => {
                    tracing::info!(snapshot_id = id, tool_id = tool_id, "auto-snapshot created");
                }
                Err(e) => {
                    tracing::warn!(error = %e, "auto-snapshot failed");
                }
            }
        }
    }
}
