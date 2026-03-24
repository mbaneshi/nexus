//! `nexus changes` — show recent filesystem changes.

use color_eyre::eyre;
use rusqlite::Connection;

pub fn run(conn: &Connection, limit: usize, json: bool) -> eyre::Result<()> {
    let changes = nexus_core::db::list_changes(conn, limit)?;

    if json {
        nexus_core::output::print_json(&changes)?;
        return Ok(());
    }

    if changes.is_empty() {
        println!("No changes recorded. Start the daemon with `nexus daemon start`.");
        return Ok(());
    }

    let header_path = "Path";
    println!(
        "{:<10} {:<10} {:<10} {header_path}",
        "Time", "Type", "Size"
    );
    println!("{}", "-".repeat(70));

    for change in &changes {
        let time = chrono::DateTime::from_timestamp(change.detected_at, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "?".to_string());

        let size = change
            .new_size
            .map(nexus_core::output::format_size)
            .unwrap_or_else(|| "-".to_string());

        let path = shorten_path(&change.path.to_string_lossy());

        println!(
            "{:<10} {:<10} {:<10} {}",
            time,
            change.change_type.as_str(),
            size,
            path
        );
    }

    println!("\n{} changes shown", changes.len());
    Ok(())
}

fn shorten_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if let Some(rest) = path.strip_prefix(home_str.as_ref()) {
            return format!("~{rest}");
        }
    }
    path.to_string()
}
