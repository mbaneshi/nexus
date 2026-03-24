//! Diff config files between current state and a snapshot.

use nexus_core::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

/// A diff entry showing what changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    pub path: String,
    pub status: DiffStatus,
    pub old_size: Option<u64>,
    pub new_size: Option<u64>,
}

/// Status of a file in the diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffStatus {
    Added,
    Removed,
    Modified,
    Unchanged,
}

impl DiffStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Removed => "removed",
            Self::Modified => "modified",
            Self::Unchanged => "unchanged",
        }
    }
}

impl std::fmt::Display for DiffStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Diff current config files against a snapshot.
/// Returns only changed files (added, removed, modified).
pub fn diff_snapshot(
    conn: &Connection,
    snapshot_id: i64,
    config_dir: &Path,
) -> Result<Vec<DiffEntry>> {
    // Get snapshot files with their hashes
    let mut stmt =
        conn.prepare("SELECT path, content_hash, size FROM snapshot_files WHERE snapshot_id = ?1")?;
    let snapshot_files: HashMap<String, (String, i64)> = stmt
        .query_map([snapshot_id], |row: &rusqlite::Row<'_>| {
            Ok((
                row.get::<_, String>(0)?,
                (row.get::<_, String>(1)?, row.get::<_, i64>(2)?),
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Scan current files
    let current_files = scan_current_hashes(config_dir);

    let mut diffs = Vec::new();

    // Check for modified and removed files
    for (path, (old_hash, old_size)) in &snapshot_files {
        match current_files.get(path) {
            Some((new_hash, new_size)) => {
                if old_hash != new_hash {
                    diffs.push(DiffEntry {
                        path: path.clone(),
                        status: DiffStatus::Modified,
                        old_size: Some(*old_size as u64),
                        new_size: Some(*new_size),
                    });
                }
            }
            None => {
                diffs.push(DiffEntry {
                    path: path.clone(),
                    status: DiffStatus::Removed,
                    old_size: Some(*old_size as u64),
                    new_size: None,
                });
            }
        }
    }

    // Check for added files
    for (path, (_, new_size)) in &current_files {
        if !snapshot_files.contains_key(path) {
            diffs.push(DiffEntry {
                path: path.clone(),
                status: DiffStatus::Added,
                old_size: None,
                new_size: Some(*new_size),
            });
        }
    }

    diffs.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(diffs)
}

/// Diff two snapshots against each other.
pub fn diff_snapshots(conn: &Connection, old_id: i64, new_id: i64) -> Result<Vec<DiffEntry>> {
    let old_files = snapshot_hashes(conn, old_id)?;
    let new_files = snapshot_hashes(conn, new_id)?;

    let mut diffs = Vec::new();

    for (path, (old_hash, old_size)) in &old_files {
        match new_files.get(path) {
            Some((new_hash, new_size)) => {
                if old_hash != new_hash {
                    diffs.push(DiffEntry {
                        path: path.clone(),
                        status: DiffStatus::Modified,
                        old_size: Some(*old_size as u64),
                        new_size: Some(*new_size as u64),
                    });
                }
            }
            None => {
                diffs.push(DiffEntry {
                    path: path.clone(),
                    status: DiffStatus::Removed,
                    old_size: Some(*old_size as u64),
                    new_size: None,
                });
            }
        }
    }

    for (path, (_, new_size)) in &new_files {
        if !old_files.contains_key(path) {
            diffs.push(DiffEntry {
                path: path.clone(),
                status: DiffStatus::Added,
                old_size: None,
                new_size: Some(*new_size as u64),
            });
        }
    }

    diffs.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(diffs)
}

/// Show the actual content diff of a file between snapshot and current.
pub fn file_content_diff(
    conn: &Connection,
    snapshot_id: i64,
    file_path: &str,
) -> Result<Option<(String, String)>> {
    let mut stmt =
        conn.prepare("SELECT content FROM snapshot_files WHERE snapshot_id = ?1 AND path = ?2")?;

    let compressed: Option<Vec<u8>> = stmt
        .query_row(
            rusqlite::params![snapshot_id, file_path],
            |row: &rusqlite::Row<'_>| row.get(0),
        )
        .ok();

    let old_content = match compressed {
        Some(data) => {
            let mut decoder = flate2::read::GzDecoder::new(data.as_slice());
            let mut content = String::new();
            decoder.read_to_string(&mut content).unwrap_or_default();
            content
        }
        None => return Ok(None),
    };

    let new_content = std::fs::read_to_string(file_path).unwrap_or_default();

    Ok(Some((old_content, new_content)))
}

fn scan_current_hashes(dir: &Path) -> HashMap<String, (String, u64)> {
    let mut files = HashMap::new();
    scan_hashes_recursive(dir, &mut files);
    files
}

fn scan_hashes_recursive(dir: &Path, files: &mut HashMap<String, (String, u64)>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.is_dir() {
            scan_hashes_recursive(&path, files);
        } else if meta.is_file() {
            let content = std::fs::read(&path).unwrap_or_default();
            let hash = blake3::hash(&content).to_hex().to_string();
            files.insert(path.to_string_lossy().to_string(), (hash, meta.len()));
        }
    }
}

fn snapshot_hashes(conn: &Connection, snapshot_id: i64) -> Result<HashMap<String, (String, i64)>> {
    let mut stmt =
        conn.prepare("SELECT path, content_hash, size FROM snapshot_files WHERE snapshot_id = ?1")?;
    let map = stmt
        .query_map([snapshot_id], |row: &rusqlite::Row<'_>| {
            Ok((
                row.get::<_, String>(0)?,
                (row.get::<_, String>(1)?, row.get::<_, i64>(2)?),
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(map)
}
