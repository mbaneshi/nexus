//! Batched SQLite indexer — inserts files into the database in transactions.

use nexus_core::Result;
use nexus_core::models::FileEntry;
use rusqlite::Connection;

const BATCH_SIZE: usize = 5000;

/// Index a set of file entries into the database under a new scan.
pub fn index(conn: &Connection, root: &str, entries: &[FileEntry]) -> Result<i64> {
    let now = chrono::Utc::now().timestamp();

    // Create scan record
    conn.execute(
        "INSERT INTO scans (root_path, started_at, status) VALUES (?1, ?2, 'running')",
        rusqlite::params![root, now],
    )?;
    let scan_id = conn.last_insert_rowid();

    // Batch insert files
    let mut total_files: u64 = 0;
    let mut total_size: u64 = 0;

    for chunk in entries.chunks(BATCH_SIZE) {
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO files (path, name, extension, size, modified_at, created_at, is_dir, depth, parent_path, category, scan_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )?;

            for entry in chunk {
                stmt.execute(rusqlite::params![
                    entry.path.to_string_lossy().as_ref(),
                    entry.name,
                    entry.extension,
                    entry.size as i64,
                    entry.modified_at,
                    entry.created_at,
                    entry.is_dir as i32,
                    entry.depth,
                    entry.parent_path,
                    entry.category.as_str(),
                    scan_id,
                ])?;

                total_files += 1;
                total_size += entry.size;
            }
        }
        tx.commit()?;
    }

    // Update scan record
    let completed_at = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE scans SET completed_at = ?1, total_files = ?2, total_size = ?3, status = 'completed' WHERE id = ?4",
        rusqlite::params![completed_at, total_files as i64, total_size as i64, scan_id],
    )?;

    Ok(scan_id)
}
