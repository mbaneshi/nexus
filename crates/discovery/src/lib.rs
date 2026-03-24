//! Home directory scanner, indexer, and FTS5 searcher.
//!
//! Scans the home directory recursively, indexes files into SQLite,
//! and provides instant full-text search via FTS5.

mod indexer;
mod scanner;
mod searcher;

pub use indexer::index;
pub use scanner::scan;
pub use searcher::{home_stats, search};

#[cfg(test)]
mod tests;
