//! `nexus search` — FTS5 search over indexed files.

use color_eyre::eyre;
use nexus_core::models::{FileCategory, SearchQuery};
use rusqlite::Connection;

pub fn run(
    conn: &Connection,
    query: &str,
    category: Option<&str>,
    limit: usize,
    json: bool,
) -> eyre::Result<()> {
    let search_query = SearchQuery {
        text: query.to_string(),
        category: category.map(FileCategory::from_str_lossy),
        limit,
        ..Default::default()
    };

    let results = nexus_discovery::search(conn, &search_query)?;

    if json {
        nexus_core::output::print_json(&results)?;
    } else if results.is_empty() {
        println!("No results for '{query}'");
    } else {
        println!("PATH                                                             SIZE CATEGORY");
        println!("{}", "-".repeat(80));
        for r in &results {
            let path = r.path.to_string_lossy();
            let display = if path.len() > 58 {
                format!("...{}", &path[path.len() - 55..])
            } else {
                path.to_string()
            };
            println!(
                "{:<60} {:>10} {}",
                display,
                nexus_core::output::format_size(r.size),
                r.category.as_str()
            );
        }
        println!("\n{} results", results.len());
    }

    Ok(())
}
