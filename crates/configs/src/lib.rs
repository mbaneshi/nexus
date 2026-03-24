//! Config file manager for ~/.config tools.
//!
//! Discovers installed tools, browses config files,
//! creates backups/snapshots, diffs changes, restores from snapshots,
//! and manages named profiles for machine provisioning.

mod backup;
mod diff;
mod profile;
mod registry;
mod scanner;

pub use backup::{create_snapshot, list_snapshots, restore_snapshot};
pub use diff::{DiffEntry, DiffStatus, diff_snapshot, diff_snapshots, file_content_diff};
pub use profile::{Profile, apply_profile, delete_profile, list_profiles, save_profile};
pub use registry::discover_tools;
pub use scanner::scan_tool_files;

#[cfg(test)]
mod tests;
