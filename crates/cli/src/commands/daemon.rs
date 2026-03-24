//! `nexus daemon` — manage the filesystem watcher daemon.

use crate::DaemonAction;
use color_eyre::eyre;
use nexus_core::config::Config;
use rusqlite::Connection;

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
    _db_path: String,
    debounce: u64,
) -> eyre::Result<()> {
    let paths: Vec<std::path::PathBuf> = watch_paths.into_iter().map(Into::into).collect();

    let watcher_config = nexus_watcher::WatcherConfig {
        watch_paths: paths,
        debounce_secs: debounce,
    };

    eprintln!("Nexus daemon running (press Ctrl+C to stop)...");

    let _handle = nexus_watcher::watch(&watcher_config, move |change| {
        // Log changes to database
        let path_str = change.path.to_string_lossy().to_string();
        tracing::info!(
            path = %path_str,
            change_type = %change.change_type.as_str(),
            "file change detected"
        );
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

    // If ctrlc handler didn't work, just park the thread
    match rx.recv() {
        Ok(()) => {}
        Err(_) => std::thread::park(),
    }

    let _ = std::fs::remove_file(&pid_path);
    let _ = std::fs::remove_file(nexus_watcher::socket_path());
    let _ = conn; // keep conn alive

    Ok(())
}
