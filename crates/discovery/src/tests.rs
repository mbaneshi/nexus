use nexus_core::models::{FileCategory, SearchQuery};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_tree() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::write(root.join("readme.md"), "# Hello").unwrap();
    fs::write(root.join("main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("config.toml"), "[server]\nport = 3000").unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn hello() {}").unwrap();
    fs::write(root.join("photo.jpg"), vec![0xFF, 0xD8, 0xFF]).unwrap();

    dir
}

#[test]
fn test_file_category_from_path() {
    assert_eq!(
        FileCategory::from_path(&PathBuf::from("/home/user/.config/nvim/init.lua")),
        FileCategory::Config
    );
    assert_eq!(
        FileCategory::from_path(&PathBuf::from("/home/user/project/main.rs")),
        FileCategory::Code
    );
    assert_eq!(
        FileCategory::from_path(&PathBuf::from("/home/user/docs/readme.md")),
        FileCategory::Document
    );
    assert_eq!(
        FileCategory::from_path(&PathBuf::from("/home/user/photo.jpg")),
        FileCategory::Image
    );
    assert_eq!(
        FileCategory::from_path(&PathBuf::from("/home/user/node_modules/foo.js")),
        FileCategory::Cache
    );
}

#[test]
fn test_scan_collects_files() {
    let dir = create_test_tree();
    let mut entries = Vec::new();

    crate::scan(dir.path(), &[], |_progress| {}, |entry| entries.push(entry)).unwrap();

    assert!(
        entries.len() >= 5,
        "expected at least 5 entries, got {}",
        entries.len()
    );

    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"readme.md"));
    assert!(names.contains(&"main.rs"));
    assert!(names.contains(&"config.toml"));
}

#[test]
fn test_index_and_search() {
    let dir = create_test_tree();
    let conn = nexus_core::db::open_in_memory().unwrap();

    let mut entries = Vec::new();
    crate::scan(dir.path(), &[], |_| {}, |entry| entries.push(entry)).unwrap();

    let scan_id = crate::index(&conn, &dir.path().to_string_lossy(), &entries).unwrap();
    assert!(scan_id > 0);

    // Search for "main"
    let results = crate::search(
        &conn,
        &SearchQuery {
            text: "main".to_string(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        !results.is_empty(),
        "search for 'main' should return results"
    );
    assert!(results.iter().any(|r| r.name == "main.rs"));
}

#[test]
fn test_home_stats() {
    let dir = create_test_tree();
    let conn = nexus_core::db::open_in_memory().unwrap();

    let mut entries = Vec::new();
    crate::scan(dir.path(), &[], |_| {}, |entry| entries.push(entry)).unwrap();
    crate::index(&conn, &dir.path().to_string_lossy(), &entries).unwrap();

    let stats = crate::home_stats(&conn).unwrap();
    assert!(stats.total_files > 0);
    assert!(stats.total_size > 0);
    assert!(!stats.by_category.is_empty());
}
