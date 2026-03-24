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
}
