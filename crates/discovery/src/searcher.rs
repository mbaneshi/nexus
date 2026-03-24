//! FTS5 search and statistics queries.

use nexus_core::Result;
use nexus_core::models::{CategoryStats, FileCategory, HomeStats, SearchQuery, SearchResult};
use rusqlite::Connection;

/// Full-text search over indexed files.
pub fn search(conn: &Connection, query: &SearchQuery) -> Result<Vec<SearchResult>> {
    let mut sql = String::from(
        "SELECT f.path, f.name, f.size, f.category, f.modified_at, fts.rank
         FROM files_fts fts
         JOIN files f ON f.id = fts.rowid
         WHERE files_fts MATCH ?1",
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(query.text.clone())];
    let mut param_idx = 2;

    if let Some(ref cat) = query.category {
        sql.push_str(&format!(" AND f.category = ?{param_idx}"));
        params.push(Box::new(cat.as_str().to_string()));
        param_idx += 1;
    }

    if let Some(min) = query.min_size {
        sql.push_str(&format!(" AND f.size >= ?{param_idx}"));
        params.push(Box::new(min as i64));
        param_idx += 1;
    }

    if let Some(max) = query.max_size {
        sql.push_str(&format!(" AND f.size <= ?{param_idx}"));
        params.push(Box::new(max as i64));
        let _ = param_idx + 1;
    }

    sql.push_str(" ORDER BY fts.rank LIMIT ?");
    params.push(Box::new(query.limit as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let results = stmt
        .query_map(param_refs.as_slice(), |row: &rusqlite::Row<'_>| {
            Ok(SearchResult {
                path: std::path::PathBuf::from(row.get::<_, String>(0)?),
                name: row.get(1)?,
                size: row.get::<_, i64>(2)? as u64,
                category: FileCategory::from_str_lossy(&row.get::<_, String>(3)?),
                modified_at: row.get(4)?,
                rank: row.get(5)?,
            })
        })?
        .filter_map(|r: std::result::Result<SearchResult, _>| r.ok())
        .collect();

    Ok(results)
}

/// Get summary statistics for the home directory.
pub fn home_stats(conn: &Connection) -> Result<HomeStats> {
    let total_files: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM files WHERE is_dir = 0",
            [],
            |row: &rusqlite::Row<'_>| row.get(0),
        )
        .unwrap_or(0);

    let total_dirs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM files WHERE is_dir = 1",
            [],
            |row: &rusqlite::Row<'_>| row.get(0),
        )
        .unwrap_or(0);

    let total_size: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(size), 0) FROM files",
            [],
            |row: &rusqlite::Row<'_>| row.get(0),
        )
        .unwrap_or(0);

    let last_scan: Option<i64> = conn
        .query_row(
            "SELECT MAX(completed_at) FROM scans WHERE status = 'completed'",
            [],
            |row: &rusqlite::Row<'_>| row.get(0),
        )
        .ok()
        .flatten();

    let mut stmt = conn.prepare(
        "SELECT category, COUNT(*), COALESCE(SUM(size), 0) FROM files WHERE is_dir = 0 GROUP BY category ORDER BY SUM(size) DESC",
    )?;

    let by_category: Vec<CategoryStats> = stmt
        .query_map([], |row: &rusqlite::Row<'_>| {
            Ok(CategoryStats {
                category: FileCategory::from_str_lossy(&row.get::<_, String>(0)?),
                file_count: row.get::<_, i64>(1)? as u64,
                total_size: row.get::<_, i64>(2)? as u64,
            })
        })?
        .filter_map(|r: std::result::Result<CategoryStats, _>| r.ok())
        .collect();

    Ok(HomeStats {
        total_files: total_files as u64,
        total_dirs: total_dirs as u64,
        total_size: total_size as u64,
        by_category,
        last_scan,
    })
}
