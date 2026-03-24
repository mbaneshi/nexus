//! `nexus scan` — index home directory.

use color_eyre::eyre::{self, WrapErr};
use nexus_core::config::Config;
use nexus_core::models::FileEntry;
use rusqlite::Connection;

pub fn run(conn: &Connection, config: &Config, root: Option<&str>, json: bool) -> eyre::Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let root_path = root
        .map(std::path::PathBuf::from)
        .or_else(|| config.scan.root.clone())
        .unwrap_or(home);

    let root_str = root_path.to_string_lossy().to_string();
    eprintln!("Scanning {}...", root_str);

    let mut entries: Vec<FileEntry> = Vec::new();

    nexus_discovery::scan(
        &root_path,
        &config.scan.excludes,
        |progress| {
            eprint!(
                "\r  {} files, {} ...",
                progress.files_scanned,
                nexus_core::output::format_size(progress.total_size),
            );
        },
        |entry| {
            entries.push(entry);
        },
    )
    .wrap_err("scan failed")?;

    eprintln!("\r  Scanned {} entries, indexing...", entries.len());

    let scan_id = nexus_discovery::index(conn, &root_str, &entries).wrap_err("indexing failed")?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "scan_id": scan_id,
                "total_entries": entries.len(),
            })
        );
    } else {
        eprintln!(
            "  Done! Scan ID: {scan_id}, {} entries indexed.",
            entries.len()
        );
    }

    Ok(())
}
