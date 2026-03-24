//! `nexus config` — manage config tools in ~/.config.

use crate::ConfigAction;
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
        println!("Config files for: {}", tool_name);
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
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
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

fn init_config() -> eyre::Result<()> {
    let path = nexus_core::config::init()?;
    println!("Config initialized at: {}", path.display());
    Ok(())
}
