//! `nexus stats` — show home directory statistics.

use color_eyre::eyre;
use rusqlite::Connection;

pub fn run(conn: &Connection, json: bool) -> eyre::Result<()> {
    let stats = nexus_discovery::home_stats(conn)?;

    if json {
        nexus_core::output::print_json(&stats)?;
    } else {
        println!("Home Directory Statistics");
        println!("========================");
        nexus_core::output::print_row("Files:", &stats.total_files.to_string());
        nexus_core::output::print_row("Directories:", &stats.total_dirs.to_string());
        nexus_core::output::print_row(
            "Total Size:",
            &nexus_core::output::format_size(stats.total_size),
        );

        if !stats.by_category.is_empty() {
            println!("\nBy Category:");
            for cat in &stats.by_category {
                println!(
                    "  {:<15} {:>8} files  {:>10}",
                    cat.category.as_str(),
                    cat.file_count,
                    nexus_core::output::format_size(cat.total_size)
                );
            }
        }

        if let Some(last) = stats.last_scan {
            let dt = chrono::DateTime::from_timestamp(last, 0)
                .map(|d: chrono::DateTime<chrono::Utc>| d.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "unknown".into());
            nexus_core::output::print_row("Last Scan:", &dt);
        }
    }

    Ok(())
}
