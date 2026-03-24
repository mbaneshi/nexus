//! Filesystem watcher daemon — monitors home directory and ~/.config for changes.
//!
//! Uses the `notify` crate for OS-level filesystem events with debouncing.

use nexus_core::Result;
use nexus_core::models::{ChangeType, FileChange};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

/// Configuration for the watcher daemon.
pub struct WatcherConfig {
    pub watch_paths: Vec<PathBuf>,
    pub debounce_secs: u64,
}

/// Start watching filesystem paths, calling `on_change` for each detected change.
pub fn watch(
    config: &WatcherConfig,
    on_change: impl Fn(FileChange) + Send + 'static,
) -> Result<WatchHandle> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })
    .map_err(|e| nexus_core::NexusError::Internal(format!("watcher init failed: {e}")))?;

    for path in &config.watch_paths {
        if path.exists() {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|e| nexus_core::NexusError::Internal(format!("watch failed: {e}")))?;
        }
    }

    let debounce = Duration::from_secs(config.debounce_secs);

    let handle = std::thread::spawn(move || {
        let mut last_events: std::collections::HashMap<PathBuf, std::time::Instant> =
            std::collections::HashMap::new();

        while let Ok(event) = rx.recv() {
            let change_type = match event.kind {
                EventKind::Create(_) => ChangeType::Created,
                EventKind::Modify(_) => ChangeType::Modified,
                EventKind::Remove(_) => ChangeType::Deleted,
                _ => continue,
            };

            for path in event.paths {
                let now = std::time::Instant::now();
                if let Some(last) = last_events.get(&path) {
                    if now.duration_since(*last) < debounce {
                        continue;
                    }
                }
                last_events.insert(path.clone(), now);

                let size = path.metadata().map(|m| m.len()).ok();

                on_change(FileChange {
                    id: None,
                    path,
                    change_type: change_type.clone(),
                    detected_at: chrono::Utc::now().timestamp(),
                    old_size: None,
                    new_size: size,
                });
            }
        }
    });

    Ok(WatchHandle {
        _watcher: watcher,
        _thread: handle,
    })
}

/// Handle to a running watcher — keeps the watcher alive while held.
pub struct WatchHandle {
    _watcher: notify::RecommendedWatcher,
    _thread: std::thread::JoinHandle<()>,
}
