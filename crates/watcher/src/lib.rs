//! Filesystem watcher daemon — monitors home directory and ~/.config for changes.
//!
//! Uses the `notify` crate for OS-level filesystem events with debouncing.
//! Supports daemon mode with Unix socket IPC for start/stop/status.

mod daemon;
mod watcher;

pub use daemon::{
    DaemonStatus, daemon_status, is_daemon_running, pid_file_path, socket_path, start_daemon,
    stop_daemon,
};
pub use watcher::{WatchHandle, WatcherConfig, watch};
