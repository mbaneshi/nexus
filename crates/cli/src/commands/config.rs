//! `nexus config` — manage config tools in ~/.config.

use crate::{ConfigAction, ProfileAction};
use color_eyre::eyre;
use rusqlite::Connection;

pub fn run(conn: &Connection, action: ConfigAction, json: bool) -> eyre::Result<()> {
    match action {
        ConfigAction::List => list_tools(conn, json),
        ConfigAction::Show { tool } => show_tool(conn, &tool, json),
        ConfigAction::Backup { tool, label } => {
            backup(conn, tool.as_deref(), label.as_deref(), json)
        }
        ConfigAction::Snapshots { tool } => list_snapshots(conn, tool.as_deref(), json),
        ConfigAction::Restore { id } => restore(conn, id, json),
        ConfigAction::Diff { tool } => diff_tool(conn, &tool, json),
        ConfigAction::Profile { action } => profile(conn, action, json),
        ConfigAction::Init => init_config(),
        ConfigAction::Path => {
            println!("{}", nexus_core::config::config_path().display());
            Ok(())
        }
    }
}

fn list_tools(conn: &Connection, json: bool) -> eyre::Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let tools = nexus_configs::discover_tools(&home)?;

    // Upsert tools into database
    for tool in &tools {
        conn.execute(
            "INSERT OR REPLACE INTO config_tools (name, config_dir, description, language)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                tool.name,
                tool.config_dir.to_string_lossy().as_ref(),
                tool.description,
                tool.language,
            ],
        )?;
    }

    if json {
        nexus_core::output::print_json(&tools)?;
    } else {
        println!("TOOL            FILES       SIZE DESCRIPTION");
        println!("{}", "-".repeat(65));
        for t in &tools {
            println!(
                "{:<15} {:>6} {:>10} {}",
                t.name,
                t.file_count,
                nexus_core::output::format_size(t.total_size),
                t.description
            );
        }
        println!("\n{} tools discovered", tools.len());
    }

    Ok(())
}

fn show_tool(conn: &Connection, tool_name: &str, json: bool) -> eyre::Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let tools = nexus_configs::discover_tools(&home)?;

    let tool = tools
        .iter()
        .find(|t| t.name == tool_name)
        .ok_or_else(|| eyre::eyre!("Tool '{}' not found", tool_name))?;

    let tool_id: i64 = conn
        .query_row(
            "SELECT id FROM config_tools WHERE name = ?1",
            [tool_name],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let files = nexus_configs::scan_tool_files(tool_id, tool_name, &tool.config_dir)?;

    if json {
        nexus_core::output::print_json(&files)?;
    } else {
        println!("Config files for: {tool_name}");
        println!("{}", "-".repeat(60));
        for f in &files {
            println!(
                "  {} ({}, {})",
                f.path.display(),
                nexus_core::output::format_size(f.size),
                f.language.as_deref().unwrap_or("unknown")
            );
        }
    }

    Ok(())
}

fn backup(
    conn: &Connection,
    tool_name: Option<&str>,
    label: Option<&str>,
    json: bool,
) -> eyre::Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let tools = nexus_configs::discover_tools(&home)?;

    let targets: Vec<_> = if let Some(name) = tool_name {
        tools.into_iter().filter(|t| t.name == name).collect()
    } else {
        tools
    };

    if targets.is_empty() {
        println!("No tools to backup.");
        return Ok(());
    }

    for tool in &targets {
        let tool_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM config_tools WHERE name = ?1",
                [&tool.name],
                |row| row.get(0),
            )
            .ok();

        let snap_id = nexus_configs::create_snapshot(conn, tool_id, label, &tool.config_dir)?;

        if json {
            println!(
                "{}",
                serde_json::json!({"tool": tool.name, "snapshot_id": snap_id})
            );
        } else {
            eprintln!("  Backed up {} -> snapshot #{}", tool.name, snap_id);
        }
    }

    Ok(())
}

fn list_snapshots(conn: &Connection, tool_name: Option<&str>, json: bool) -> eyre::Result<()> {
    let tool_id: Option<i64> = tool_name.and_then(|name| {
        conn.query_row(
            "SELECT id FROM config_tools WHERE name = ?1",
            [name],
            |row| row.get(0),
        )
        .ok()
    });

    let snapshots = nexus_configs::list_snapshots(conn, tool_id)?;

    if json {
        nexus_core::output::print_json(&snapshots)?;
    } else if snapshots.is_empty() {
        println!("No snapshots found.");
    } else {
        println!("ID     TOOL            CREATED                    FILES LABEL");
        println!("{}", "-".repeat(65));
        for s in &snapshots {
            let dt = chrono::DateTime::from_timestamp(s.created_at, 0)
                .map(|d: chrono::DateTime<chrono::Utc>| d.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "?".into());
            println!(
                "{:<6} {:<15} {:<25} {:>6} {}",
                s.id,
                s.tool_name.as_deref().unwrap_or("-"),
                dt,
                s.file_count,
                s.label.as_deref().unwrap_or("")
            );
        }
    }

    Ok(())
}

fn restore(conn: &Connection, snapshot_id: i64, json: bool) -> eyre::Result<()> {
    let count = nexus_configs::restore_snapshot(conn, snapshot_id)?;

    if json {
        println!(
            "{}",
            serde_json::json!({"snapshot_id": snapshot_id, "files_restored": count})
        );
    } else {
        println!("Restored {count} files from snapshot #{snapshot_id}");
    }

    Ok(())
}

fn diff_tool(conn: &Connection, tool_name: &str, json: bool) -> eyre::Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let tools = nexus_configs::discover_tools(&home)?;

    let tool = tools
        .iter()
        .find(|t| t.name == tool_name)
        .ok_or_else(|| eyre::eyre!("Tool '{}' not found", tool_name))?;

    // Find the latest snapshot for this tool
    let tool_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM config_tools WHERE name = ?1",
            [tool_name],
            |row| row.get(0),
        )
        .ok();

    let snapshots = nexus_configs::list_snapshots(conn, tool_id)?;
    let latest = snapshots.first().ok_or_else(|| {
        eyre::eyre!(
            "No snapshots found for '{}'. Run `nexus config backup {}` first.",
            tool_name,
            tool_name
        )
    })?;

    let diffs = nexus_configs::diff_snapshot(conn, latest.id, &tool.config_dir)?;

    if json {
        nexus_core::output::print_json(&diffs)?;
    } else if diffs.is_empty() {
        println!("No changes since snapshot #{} for {tool_name}", latest.id);
    } else {
        println!("Changes since snapshot #{} for {tool_name}:", latest.id);
        println!("{}", "-".repeat(60));
        for d in &diffs {
            let size_info = match (&d.old_size, &d.new_size) {
                (Some(old), Some(new)) => format!(
                    "{} -> {}",
                    nexus_core::output::format_size(*old),
                    nexus_core::output::format_size(*new)
                ),
                (None, Some(new)) => format!("+{}", nexus_core::output::format_size(*new)),
                (Some(old), None) => format!("-{}", nexus_core::output::format_size(*old)),
                (None, None) => String::new(),
            };
            let marker = match d.status {
                nexus_configs::DiffStatus::Added => "+",
                nexus_configs::DiffStatus::Removed => "-",
                nexus_configs::DiffStatus::Modified => "~",
                nexus_configs::DiffStatus::Unchanged => " ",
            };
            println!("  {marker} {} ({size_info})", d.path);
        }
        println!("\n{} file(s) changed", diffs.len());
    }

    Ok(())
}

fn profile(conn: &Connection, action: ProfileAction, json: bool) -> eyre::Result<()> {
    match action {
        ProfileAction::Save { name, description } => {
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let tools = nexus_configs::discover_tools(&home)?;

            // Ensure tools are in DB
            for tool in &tools {
                conn.execute(
                    "INSERT OR REPLACE INTO config_tools (name, config_dir, description, language)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        tool.name,
                        tool.config_dir.to_string_lossy().as_ref(),
                        tool.description,
                        tool.language,
                    ],
                )?;
            }

            let tool_data: Vec<(i64, String, std::path::PathBuf)> = tools
                .iter()
                .filter_map(|t| {
                    conn.query_row(
                        "SELECT id FROM config_tools WHERE name = ?1",
                        [&t.name],
                        |row| row.get::<_, i64>(0),
                    )
                    .ok()
                    .map(|id| (id, t.name.clone(), t.config_dir.clone()))
                })
                .collect();

            let profile_id =
                nexus_configs::save_profile(conn, &name, description.as_deref(), &tool_data)?;

            if json {
                println!(
                    "{}",
                    serde_json::json!({"profile_id": profile_id, "name": name, "tools": tool_data.len()})
                );
            } else {
                println!("Profile '{}' saved ({} tools)", name, tool_data.len());
            }
            Ok(())
        }
        ProfileAction::List => {
            let profiles = nexus_configs::list_profiles(conn)?;
            if json {
                nexus_core::output::print_json(&profiles)?;
            } else if profiles.is_empty() {
                println!("No profiles saved.");
            } else {
                println!("NAME             TOOLS  CREATED");
                println!("{}", "-".repeat(50));
                for p in &profiles {
                    let dt = chrono::DateTime::from_timestamp(p.created_at, 0)
                        .map(|d: chrono::DateTime<chrono::Utc>| {
                            d.format("%Y-%m-%d %H:%M:%S").to_string()
                        })
                        .unwrap_or_else(|| "?".into());
                    println!("{:<16} {:>5}  {}", p.name, p.snapshot_count, dt);
                }
            }
            Ok(())
        }
        ProfileAction::Apply { name } => {
            let count = nexus_configs::apply_profile(conn, &name)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"profile": name, "files_restored": count})
                );
            } else {
                println!("Applied profile '{name}': {count} files restored");
            }
            Ok(())
        }
        ProfileAction::Delete { name } => {
            let deleted = nexus_configs::delete_profile(conn, &name)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"profile": name, "deleted": deleted})
                );
            } else if deleted {
                println!("Deleted profile '{name}'");
            } else {
                println!("Profile '{name}' not found");
            }
            Ok(())
        }
    }
}

fn init_config() -> eyre::Result<()> {
    let path = nexus_core::config::init()?;
    println!("Config initialized at: {}", path.display());
    Ok(())
}
