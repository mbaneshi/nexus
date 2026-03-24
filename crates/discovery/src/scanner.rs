//! Recursive filesystem scanner using the `ignore` crate's parallel walker.

use nexus_core::Result;
use nexus_core::models::{FileCategory, FileEntry, Progress};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Scan a directory tree, calling `on_entry` for each file/directory found.
pub fn scan(
    root: &Path,
    excludes: &[String],
    mut on_progress: impl FnMut(Progress),
    mut on_entry: impl FnMut(FileEntry),
) -> Result<()> {
    let root = root.to_path_buf();
    let files_scanned = Arc::new(AtomicU64::new(0));
    let total_size = Arc::new(AtomicU64::new(0));

    let mut builder = ignore::WalkBuilder::new(&root);
    builder
        .hidden(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .follow_links(false)
        .max_depth(None);

    // Apply exclude filters
    let mut overrides = ignore::overrides::OverrideBuilder::new(&root);
    for exclude in excludes {
        let pattern = format!("!{exclude}/");
        let _ = overrides.add(&pattern);
    }
    if let Ok(ov) = overrides.build() {
        builder.overrides(ov);
    }

    let root_depth = root.components().count();

    for entry in builder.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path().to_path_buf();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let extension = path.extension().map(|e| e.to_string_lossy().to_string());

        let size = metadata.len();
        let is_dir = metadata.is_dir();
        let depth = (path.components().count().saturating_sub(root_depth)) as u32;
        let parent_path = path.parent().map(|p| p.to_string_lossy().to_string());
        let category = FileCategory::from_path(&path);

        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let created_at = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);

        let count = files_scanned.fetch_add(1, Ordering::Relaxed);
        total_size.fetch_add(size, Ordering::Relaxed);

        // Report progress every 1000 files
        if count % 1000 == 0 {
            on_progress(Progress {
                files_scanned: count,
                total_size: total_size.load(Ordering::Relaxed),
                current_path: path.to_string_lossy().to_string(),
            });
        }

        on_entry(FileEntry {
            id: None,
            path,
            name,
            extension,
            size,
            modified_at,
            created_at,
            is_dir,
            depth,
            parent_path,
            category,
        });
    }

    Ok(())
}
