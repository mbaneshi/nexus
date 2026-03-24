//! Backup and restore config snapshots.

use flate2::Compression;
use flate2::write::GzEncoder;
use nexus_core::Result;
use nexus_core::models::ConfigSnapshot;
use rusqlite::Connection;
use std::io::Write;
use std::path::Path;

/// Create a snapshot of a tool's config files, storing compressed content in the database.
pub fn create_snapshot(
    conn: &Connection,
    tool_id: Option<i64>,
    label: Option<&str>,
    config_dir: &Path,
) -> Result<i64> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "INSERT INTO config_snapshots (tool_id, label) VALUES (?1, ?2)",
        rusqlite::params![tool_id, label],
    )?;
    let snapshot_id = tx.last_insert_rowid();

    // Walk all files in the config dir and store them
    store_dir_recursive(&tx, snapshot_id, config_dir)?;

    tx.commit()?;
    Ok(snapshot_id)
}

fn store_dir_recursive(tx: &Connection, snapshot_id: i64, dir: &Path) -> Result<()> {
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
            store_dir_recursive(tx, snapshot_id, &path)?;
        } else if meta.is_file() {
            let content = std::fs::read(&path)?;
            let hash = blake3::hash(&content).to_hex().to_string();

            // Compress with gzip
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&content)?;
            let compressed = encoder.finish()?;

            tx.execute(
                "INSERT INTO snapshot_files (snapshot_id, path, content, content_hash, size) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    snapshot_id,
                    path.to_string_lossy().as_ref(),
                    compressed,
                    hash,
                    meta.len() as i64,
                ],
            )?;
        }
    }

    Ok(())
}

/// List all snapshots, optionally filtered by tool.
pub fn list_snapshots(conn: &Connection, tool_id: Option<i64>) -> Result<Vec<ConfigSnapshot>> {
    let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(tid) = tool_id {
        (
            "SELECT s.id, s.tool_id, t.name, s.label, s.created_at,
                    (SELECT COUNT(*) FROM snapshot_files WHERE snapshot_id = s.id)
             FROM config_snapshots s
             LEFT JOIN config_tools t ON t.id = s.tool_id
             WHERE s.tool_id = ?1
             ORDER BY s.created_at DESC",
            vec![Box::new(tid)],
        )
    } else {
        (
            "SELECT s.id, s.tool_id, t.name, s.label, s.created_at,
                    (SELECT COUNT(*) FROM snapshot_files WHERE snapshot_id = s.id)
             FROM config_snapshots s
             LEFT JOIN config_tools t ON t.id = s.tool_id
             ORDER BY s.created_at DESC",
            vec![],
        )
    };

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(sql)?;
    let snapshots = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(ConfigSnapshot {
                id: row.get(0)?,
                tool_id: row.get(1)?,
                tool_name: row.get(2)?,
                label: row.get(3)?,
                created_at: row.get(4)?,
                file_count: row.get::<_, i64>(5)? as u32,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(snapshots)
}

/// Restore files from a snapshot to their original paths.
pub fn restore_snapshot(conn: &Connection, snapshot_id: i64) -> Result<u32> {
    let mut stmt =
        conn.prepare("SELECT path, content, size FROM snapshot_files WHERE snapshot_id = ?1")?;

    let files: Vec<(String, Vec<u8>, i64)> = stmt
        .query_map([snapshot_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut restored = 0u32;
    for (path_str, compressed, _size) in &files {
        let path = std::path::Path::new(path_str);

        // Decompress
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut content = Vec::new();
        decoder.read_to_end(&mut content)?;

        // Ensure parent dir exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, &content)?;
        restored += 1;
    }

    Ok(restored)
}
