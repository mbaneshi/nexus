//! SQLite database management — WAL mode, FTS5, migrations.
//!
//! All database access in nexus goes through this module.

use crate::config::DatabaseConfig;
use crate::error::Result;
use rusqlite::Connection;
use std::path::PathBuf;

/// Current schema version.
const SCHEMA_VERSION: i32 = 1;

/// Default database path: `~/.local/share/nexus/nexus.db`.
pub fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nexus")
        .join("nexus.db")
}

/// Open or create the nexus database with WAL pragmas.
pub fn open(config: &DatabaseConfig) -> Result<Connection> {
    let path = config.path.clone().unwrap_or_else(default_db_path);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&path)?;
    configure(&conn, config)?;
    migrate(&conn)?;
    Ok(conn)
}

/// Open an in-memory database (for testing).
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    let config = DatabaseConfig::default();
    configure(&conn, &config)?;
    migrate(&conn)?;
    Ok(conn)
}

/// Apply performance pragmas.
fn configure(conn: &Connection, config: &DatabaseConfig) -> Result<()> {
    conn.execute_batch(&format!(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -{cache_kb};
         PRAGMA mmap_size = {mmap_bytes};
         PRAGMA foreign_keys = ON;
         PRAGMA temp_store = MEMORY;",
        cache_kb = config.cache_mb * 1024,
        mmap_bytes = (config.mmap_mb as u64) * 1024 * 1024,
    ))?;
    Ok(())
}

/// Run versioned migrations.
fn migrate(conn: &Connection) -> Result<()> {
    let version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap_or(0);

    if version < 1 {
        conn.execute_batch(SCHEMA_V1)?;
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    }

    Ok(())
}

const SCHEMA_V1: &str = r#"
-- Scans
CREATE TABLE IF NOT EXISTS scans (
    id           INTEGER PRIMARY KEY,
    root_path    TEXT NOT NULL,
    started_at   INTEGER NOT NULL,
    completed_at INTEGER,
    total_files  INTEGER,
    total_size   INTEGER,
    status       TEXT NOT NULL DEFAULT 'running'
);

-- Files
CREATE TABLE IF NOT EXISTS files (
    id          INTEGER PRIMARY KEY,
    path        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    extension   TEXT,
    size        INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    created_at  INTEGER,
    is_dir      INTEGER NOT NULL DEFAULT 0,
    depth       INTEGER NOT NULL,
    parent_path TEXT,
    category    TEXT,
    scan_id     INTEGER NOT NULL REFERENCES scans(id),
    indexed_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

-- FTS5 index
CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
    name, extension, path,
    content='files', content_rowid='id',
    tokenize='unicode61 remove_diacritics 2'
);

-- FTS triggers
CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON files BEGIN
    INSERT INTO files_fts(rowid, name, extension, path)
    VALUES (new.id, new.name, new.extension, new.path);
END;

CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
    INSERT INTO files_fts(files_fts, rowid, name, extension, path)
    VALUES ('delete', old.id, old.name, old.extension, old.path);
END;

CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON files BEGIN
    INSERT INTO files_fts(files_fts, rowid, name, extension, path)
    VALUES ('delete', old.id, old.name, old.extension, old.path);
    INSERT INTO files_fts(rowid, name, extension, path)
    VALUES (new.id, new.name, new.extension, new.path);
END;

-- Config tools
CREATE TABLE IF NOT EXISTS config_tools (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    config_dir  TEXT NOT NULL,
    description TEXT,
    language    TEXT,
    discovered_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Config files
CREATE TABLE IF NOT EXISTS config_files (
    id          INTEGER PRIMARY KEY,
    tool_id     INTEGER NOT NULL REFERENCES config_tools(id),
    path        TEXT NOT NULL UNIQUE,
    content_hash TEXT,
    size        INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    language    TEXT,
    indexed_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Config snapshots
CREATE TABLE IF NOT EXISTS config_snapshots (
    id          INTEGER PRIMARY KEY,
    tool_id     INTEGER,
    label       TEXT,
    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    commit_hash TEXT
);

-- Snapshot file contents
CREATE TABLE IF NOT EXISTS snapshot_files (
    id          INTEGER PRIMARY KEY,
    snapshot_id INTEGER NOT NULL REFERENCES config_snapshots(id) ON DELETE CASCADE,
    path        TEXT NOT NULL,
    content     BLOB NOT NULL,
    content_hash TEXT NOT NULL,
    size        INTEGER NOT NULL
);

-- Change tracking
CREATE TABLE IF NOT EXISTS file_changes (
    id          INTEGER PRIMARY KEY,
    path        TEXT NOT NULL,
    change_type TEXT NOT NULL,
    detected_at INTEGER NOT NULL DEFAULT (unixepoch()),
    old_size    INTEGER,
    new_size    INTEGER
);

-- AI query history
CREATE TABLE IF NOT EXISTS ai_queries (
    id          INTEGER PRIMARY KEY,
    query       TEXT NOT NULL,
    context     TEXT,
    response    TEXT,
    model       TEXT,
    created_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_files_category ON files(category);
CREATE INDEX IF NOT EXISTS idx_files_size ON files(size);
CREATE INDEX IF NOT EXISTS idx_files_modified ON files(modified_at);
CREATE INDEX IF NOT EXISTS idx_files_parent ON files(parent_path);
CREATE INDEX IF NOT EXISTS idx_files_scan ON files(scan_id);
CREATE INDEX IF NOT EXISTS idx_config_files_tool ON config_files(tool_id);
CREATE INDEX IF NOT EXISTS idx_changes_path ON file_changes(path);
CREATE INDEX IF NOT EXISTS idx_changes_detected ON file_changes(detected_at);
CREATE INDEX IF NOT EXISTS idx_snapshots_tool ON config_snapshots(tool_id);
"#;

/// Record a file change in the database.
pub fn record_change(conn: &Connection, change: &crate::models::FileChange) -> Result<()> {
    conn.execute(
        "INSERT INTO file_changes (path, change_type, detected_at, old_size, new_size)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            change.path.to_string_lossy().as_ref(),
            change.change_type.as_str(),
            change.detected_at,
            change.old_size.map(|s| s as i64),
            change.new_size.map(|s| s as i64),
        ],
    )?;
    Ok(())
}

/// List recent file changes from the database.
pub fn list_changes(conn: &Connection, limit: usize) -> Result<Vec<crate::models::FileChange>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, change_type, detected_at, old_size, new_size
         FROM file_changes ORDER BY detected_at DESC LIMIT ?1",
    )?;

    let changes = stmt
        .query_map([limit as i64], |row: &rusqlite::Row<'_>| {
            let change_type_str: String = row.get(2)?;
            let change_type = match change_type_str.as_str() {
                "created" => crate::models::ChangeType::Created,
                "modified" => crate::models::ChangeType::Modified,
                "deleted" => crate::models::ChangeType::Deleted,
                _ => crate::models::ChangeType::Modified,
            };
            Ok(crate::models::FileChange {
                id: row.get(0)?,
                path: std::path::PathBuf::from(row.get::<_, String>(1)?),
                change_type,
                detected_at: row.get(3)?,
                old_size: row.get::<_, Option<i64>>(4)?.map(|s| s as u64),
                new_size: row.get::<_, Option<i64>>(5)?.map(|s| s as u64),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory() {
        let conn = Connection::open_in_memory().unwrap();
        configure(
            &conn,
            &DatabaseConfig {
                path: None,
                cache_mb: 64,
                mmap_mb: 256,
            },
        )
        .unwrap();
        migrate(&conn).unwrap();

        // Verify tables exist
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='files'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn migration_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        configure(&conn, &DatabaseConfig::default()).unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap(); // second call should be a no-op

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn record_and_list_changes() {
        let conn = super::open_in_memory().unwrap();

        let change1 = crate::models::FileChange {
            id: None,
            path: std::path::PathBuf::from("/home/user/.config/nvim/init.lua"),
            change_type: crate::models::ChangeType::Modified,
            detected_at: 1000,
            old_size: Some(100),
            new_size: Some(200),
        };
        let change2 = crate::models::FileChange {
            id: None,
            path: std::path::PathBuf::from("/home/user/.config/fish/config.fish"),
            change_type: crate::models::ChangeType::Created,
            detected_at: 2000,
            old_size: None,
            new_size: Some(50),
        };

        record_change(&conn, &change1).unwrap();
        record_change(&conn, &change2).unwrap();

        let changes = list_changes(&conn, 10).unwrap();
        assert_eq!(changes.len(), 2);
        // Most recent first
        assert_eq!(changes[0].detected_at, 2000);
        assert_eq!(changes[0].change_type, crate::models::ChangeType::Created);
        assert_eq!(changes[1].detected_at, 1000);
        assert_eq!(changes[1].new_size, Some(200));
    }

    #[test]
    fn list_changes_respects_limit() {
        let conn = super::open_in_memory().unwrap();

        for i in 0..10 {
            let change = crate::models::FileChange {
                id: None,
                path: std::path::PathBuf::from(format!("/tmp/file{i}.txt")),
                change_type: crate::models::ChangeType::Modified,
                detected_at: i,
                old_size: None,
                new_size: None,
            };
            record_change(&conn, &change).unwrap();
        }

        let changes = list_changes(&conn, 3).unwrap();
        assert_eq!(changes.len(), 3);
    }

    #[test]
    fn all_tables_created() {
        let conn = super::open_in_memory().unwrap();

        let expected = [
            "scans",
            "files",
            "config_tools",
            "config_files",
            "config_snapshots",
            "snapshot_files",
            "file_changes",
            "ai_queries",
        ];

        for table in &expected {
            let count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "table '{table}' should exist");
        }
    }
}
