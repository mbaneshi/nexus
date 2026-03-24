//! Export and import config files as portable tar.gz archives.

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use nexus_core::Result;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Export config directories to a tar.gz archive.
///
/// Paths inside the archive are relative to the home directory (e.g. `.config/nvim/init.lua`).
pub fn export_configs(
    home: &Path,
    tool_dirs: &[(String, PathBuf)],
    output: &Path,
) -> Result<u32> {
    let file = std::fs::File::create(output)?;
    let gz = GzEncoder::new(file, Compression::default());
    let mut archive = tar::Builder::new(gz);

    let mut file_count = 0u32;

    for (_tool_name, config_dir) in tool_dirs {
        if !config_dir.exists() {
            continue;
        }
        file_count += add_dir_recursive(&mut archive, config_dir, home)?;
    }

    let gz = archive
        .into_inner()
        .map_err(|e| nexus_core::NexusError::Io(std::io::Error::other(e)))?;
    gz.finish()?;

    Ok(file_count)
}

fn add_dir_recursive<W: std::io::Write>(
    archive: &mut tar::Builder<W>,
    dir: &Path,
    home: &Path,
) -> Result<u32> {
    let mut count = 0;
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(0),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.is_dir() {
            count += add_dir_recursive(archive, &path, home)?;
        } else if meta.is_file() {
            // Path relative to home, e.g. ".config/nvim/init.lua"
            let archive_path = match path.strip_prefix(home) {
                Ok(rel) => rel.to_path_buf(),
                Err(_) => continue,
            };

            let mut file = match std::fs::File::open(&path) {
                Ok(f) => f,
                Err(_) => continue,
            };

            let mut header = tar::Header::new_gnu();
            header.set_size(meta.len());
            header.set_mode(0o644);
            header.set_cksum();

            archive
                .append_data(&mut header, &archive_path, &mut file)
                .map_err(nexus_core::NexusError::Io)?;

            count += 1;
        }
    }

    Ok(count)
}

/// Import config files from a tar.gz archive.
///
/// Extracts all files to the specified root (typically the home directory).
pub fn import_configs(archive_path: &Path, home: &Path) -> Result<u32> {
    let file = std::fs::File::open(archive_path)?;
    let gz = GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);

    let mut count = 0u32;

    for entry in archive
        .entries()
        .map_err(nexus_core::NexusError::Io)?
    {
        let mut entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let rel = match entry.path() {
            Ok(p) => p.into_owned(),
            Err(_) => continue,
        };

        // Security: reject any path containing ".." components
        if rel.components().any(|c| c == std::path::Component::ParentDir) {
            tracing::warn!(path = %rel.display(), "skipping path with traversal component");
            continue;
        }

        let path = home.join(&rel);

        // Security: belt-and-suspenders check after joining
        if !path.starts_with(home) {
            tracing::warn!(path = %path.display(), "skipping path outside home");
            continue;
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = Vec::new();
        entry
            .read_to_end(&mut content)
            .map_err(nexus_core::NexusError::Io)?;

        std::fs::write(&path, &content)?;
        count += 1;
    }

    Ok(count)
}
