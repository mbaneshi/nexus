//! Named config profiles for machine provisioning.
//!
//! A profile is a named collection of config snapshots — one per tool.
//! Save a profile on your current machine, apply it on a new one.

use nexus_core::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// A named config profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub snapshot_count: u32,
}

/// Ensure the profiles table exists (additive migration).
pub fn ensure_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS config_profiles (
            id          INTEGER PRIMARY KEY,
            name        TEXT NOT NULL UNIQUE,
            description TEXT,
            created_at  INTEGER NOT NULL DEFAULT (unixepoch())
        );
        CREATE TABLE IF NOT EXISTS profile_snapshots (
            id          INTEGER PRIMARY KEY,
            profile_id  INTEGER NOT NULL REFERENCES config_profiles(id) ON DELETE CASCADE,
            snapshot_id INTEGER NOT NULL REFERENCES config_snapshots(id),
            tool_name   TEXT NOT NULL
        );",
    )?;
    Ok(())
}

/// Save the current state of all tools as a named profile.
/// Creates a fresh snapshot for each tool and links them to the profile.
pub fn save_profile(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
    tools: &[(i64, String, std::path::PathBuf)], // (tool_id, tool_name, config_dir)
) -> Result<i64> {
    ensure_table(conn)?;

    let tx = conn.unchecked_transaction()?;

    // Delete existing profile with same name if it exists
    tx.execute("DELETE FROM config_profiles WHERE name = ?1", [name])?;

    tx.execute(
        "INSERT INTO config_profiles (name, description) VALUES (?1, ?2)",
        rusqlite::params![name, description],
    )?;
    let profile_id = tx.last_insert_rowid();

    tx.commit()?;

    for (tool_id, tool_name, config_dir) in tools {
        // Create snapshot for this tool (each manages its own transaction)
        let snap_id = super::backup::create_snapshot(conn, Some(*tool_id), Some(name), config_dir)?;

        conn.execute(
            "INSERT INTO profile_snapshots (profile_id, snapshot_id, tool_name) VALUES (?1, ?2, ?3)",
            rusqlite::params![profile_id, snap_id, tool_name],
        )?;
    }
    Ok(profile_id)
}

/// List all saved profiles.
pub fn list_profiles(conn: &Connection) -> Result<Vec<Profile>> {
    ensure_table(conn)?;

    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.description, p.created_at,
                (SELECT COUNT(*) FROM profile_snapshots WHERE profile_id = p.id)
         FROM config_profiles p
         ORDER BY p.created_at DESC",
    )?;

    let profiles = stmt
        .query_map([], |row: &rusqlite::Row<'_>| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                snapshot_count: row.get::<_, i64>(4)? as u32,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(profiles)
}

/// Apply a profile — restore all snapshots linked to the profile.
pub fn apply_profile(conn: &Connection, profile_name: &str) -> Result<u32> {
    ensure_table(conn)?;

    let mut stmt = conn.prepare(
        "SELECT ps.snapshot_id, ps.tool_name
         FROM profile_snapshots ps
         JOIN config_profiles p ON p.id = ps.profile_id
         WHERE p.name = ?1",
    )?;

    let snapshots: Vec<(i64, String)> = stmt
        .query_map([profile_name], |row: &rusqlite::Row<'_>| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if snapshots.is_empty() {
        return Err(nexus_core::NexusError::Config(format!(
            "Profile '{profile_name}' not found or empty"
        )));
    }

    let mut total_restored = 0u32;
    for (snapshot_id, _tool_name) in &snapshots {
        let count = super::backup::restore_snapshot(conn, *snapshot_id)?;
        total_restored += count;
    }

    Ok(total_restored)
}

/// Delete a profile and its linked snapshots.
pub fn delete_profile(conn: &Connection, profile_name: &str) -> Result<bool> {
    ensure_table(conn)?;

    let rows = conn.execute(
        "DELETE FROM config_profiles WHERE name = ?1",
        [profile_name],
    )?;
    Ok(rows > 0)
}
