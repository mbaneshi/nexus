//! Scan individual tool config directories for files.

use nexus_core::Result;
use nexus_core::models::ConfigFile;
use std::path::Path;

/// Scan all files in a tool's config directory.
pub fn scan_tool_files(
    tool_id: i64,
    tool_name: &str,
    config_dir: &Path,
) -> Result<Vec<ConfigFile>> {
    let mut files = Vec::new();
    scan_recursive(tool_id, tool_name, config_dir, &mut files)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn scan_recursive(
    tool_id: i64,
    tool_name: &str,
    dir: &Path,
    files: &mut Vec<ConfigFile>,
) -> Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.is_dir() {
            scan_recursive(tool_id, tool_name, &path, files)?;
        } else if meta.is_file() {
            let content = std::fs::read(&path).unwrap_or_default();
            let hash = blake3::hash(&content).to_hex().to_string();

            let language = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_string());

            let modified_at = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            files.push(ConfigFile {
                id: None,
                tool_id,
                tool_name: tool_name.to_string(),
                path,
                content_hash: hash,
                size: meta.len(),
                modified_at,
                language,
            });
        }
    }

    Ok(())
}
