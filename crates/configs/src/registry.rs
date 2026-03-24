//! Discover which config tools are installed on this machine.

use nexus_core::Result;
use nexus_core::models::{ConfigTool, KNOWN_TOOLS};
use std::path::Path;

/// Discover all known tools that have config directories present.
pub fn discover_tools(home: &Path) -> Result<Vec<ConfigTool>> {
    let mut tools = Vec::new();

    for def in KNOWN_TOOLS {
        let config_dir = home.join(def.config_path);
        if !config_dir.exists() {
            continue;
        }

        let (total_size, file_count, last_modified) = dir_stats(&config_dir);

        tools.push(ConfigTool {
            id: None,
            name: def.name.to_string(),
            config_dir,
            description: def.description.to_string(),
            language: def.language.to_string(),
            total_size,
            file_count,
            last_modified,
        });
    }

    tools.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(tools)
}

/// Compute size, file count, and last modified for a directory.
fn dir_stats(dir: &Path) -> (u64, u32, i64) {
    let mut total_size = 0u64;
    let mut file_count = 0u32;
    let mut last_modified = 0i64;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total_size += meta.len();
                    file_count += 1;
                    if let Ok(modified) = meta.modified() {
                        let ts = modified
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        if ts > last_modified {
                            last_modified = ts;
                        }
                    }
                }
            }
        }
    }

    (total_size, file_count, last_modified)
}
