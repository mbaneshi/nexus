//! Daemon management — start/stop/status via PID file and Unix socket IPC.

use nexus_core::Result;
use std::path::PathBuf;

/// Path to the daemon PID file.
pub fn pid_file_path() -> PathBuf {
    dirs::runtime_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("nexus")
        .join("nexus.pid")
}

/// Path to the daemon Unix socket.
pub fn socket_path() -> PathBuf {
    dirs::runtime_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("nexus")
        .join("nexus.sock")
}

/// Check if the daemon is currently running.
pub fn is_daemon_running() -> bool {
    let pid_path = pid_file_path();
    if !pid_path.exists() {
        return false;
    }

    let pid_str = match std::fs::read_to_string(&pid_path) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let pid: u32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Check if process is alive
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// Get daemon status info.
pub fn daemon_status() -> DaemonStatus {
    let pid_path = pid_file_path();
    if !pid_path.exists() {
        return DaemonStatus::Stopped;
    }

    let pid_str = match std::fs::read_to_string(&pid_path) {
        Ok(s) => s,
        Err(_) => return DaemonStatus::Stopped,
    };

    let pid: u32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => return DaemonStatus::Stopped,
    };

    if unsafe { libc::kill(pid as i32, 0) == 0 } {
        DaemonStatus::Running { pid }
    } else {
        // Stale PID file
        let _ = std::fs::remove_file(&pid_path);
        DaemonStatus::Stopped
    }
}

/// Start the daemon in the background.
pub fn start_daemon(watch_paths: &[PathBuf], db_path: &str, debounce_secs: u64) -> Result<u32> {
    if is_daemon_running() {
        return Err(nexus_core::NexusError::Internal(
            "Daemon is already running".to_string(),
        ));
    }

    let pid_path = pid_file_path();
    if let Some(parent) = pid_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Build arguments for re-invoking ourselves
    let exe = std::env::current_exe()
        .map_err(|e| nexus_core::NexusError::Internal(format!("cannot find exe: {e}")))?;

    let mut cmd = std::process::Command::new(exe);
    cmd.arg("daemon").arg("run");

    for path in watch_paths {
        cmd.arg("--watch").arg(path);
    }
    cmd.arg("--db-path").arg(db_path);
    cmd.arg("--debounce").arg(debounce_secs.to_string());

    // Detach from terminal
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    let child = cmd
        .spawn()
        .map_err(|e| nexus_core::NexusError::Internal(format!("failed to spawn daemon: {e}")))?;

    let pid = child.id();
    std::fs::write(&pid_path, pid.to_string())?;

    Ok(pid)
}

/// Stop the running daemon.
pub fn stop_daemon() -> Result<bool> {
    let pid_path = pid_file_path();
    if !pid_path.exists() {
        return Ok(false);
    }

    let pid_str = std::fs::read_to_string(&pid_path)?;
    let pid: i32 = pid_str
        .trim()
        .parse()
        .map_err(|e| nexus_core::NexusError::Internal(format!("invalid PID: {e}")))?;

    // Send SIGTERM
    let result = unsafe { libc::kill(pid, libc::SIGTERM) };
    let _ = std::fs::remove_file(&pid_path);
    let _ = std::fs::remove_file(socket_path());

    Ok(result == 0)
}

/// Daemon status.
#[derive(Debug)]
pub enum DaemonStatus {
    Running { pid: u32 },
    Stopped,
}

impl std::fmt::Display for DaemonStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonStatus::Running { pid } => write!(f, "running (PID {pid})"),
            DaemonStatus::Stopped => write!(f, "stopped"),
        }
    }
}
