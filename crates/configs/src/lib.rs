//! Config file manager for ~/.config tools.
//!
//! Discovers installed tools, browses config files,
//! creates backups/snapshots, diffs changes, and restores from snapshots.

mod backup;
mod registry;
mod scanner;

pub use backup::{create_snapshot, list_snapshots, restore_snapshot};
pub use registry::discover_tools;
pub use scanner::scan_tool_files;
