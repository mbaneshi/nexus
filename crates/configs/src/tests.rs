use std::fs;
use tempfile::TempDir;

fn create_mock_config_home() -> TempDir {
    let dir = TempDir::new().unwrap();
    let home = dir.path();

    let nvim = home.join(".config/nvim");
    fs::create_dir_all(&nvim).unwrap();
    fs::write(
        nvim.join("init.lua"),
        "-- nvim config\nvim.opt.number = true",
    )
    .unwrap();

    let git = home.join(".config/git");
    fs::create_dir_all(&git).unwrap();
    fs::write(git.join("config"), "[user]\n  name = Test").unwrap();
    fs::write(git.join("ignore"), "*.swp\n.DS_Store").unwrap();

    let starship = home.join(".config/starship.toml");
    fs::write(starship, "format = '$all'").unwrap();

    dir
}

#[test]
fn test_discover_tools_finds_nvim_and_git() {
    let dir = create_mock_config_home();
    let tools = crate::discover_tools(dir.path()).unwrap();

    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"nvim"), "should find nvim");
    assert!(names.contains(&"git"), "should find git");
}

#[test]
fn test_discover_tools_counts_files() {
    let dir = create_mock_config_home();
    let tools = crate::discover_tools(dir.path()).unwrap();

    let git = tools.iter().find(|t| t.name == "git").unwrap();
    assert_eq!(git.file_count, 2, "git should have 2 config files");
}

#[test]
fn test_scan_tool_files() {
    let dir = create_mock_config_home();
    let nvim_dir = dir.path().join(".config/nvim");

    let files = crate::scan_tool_files(1, "nvim", &nvim_dir).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].path.to_string_lossy().contains("init.lua"));
    assert!(!files[0].content_hash.is_empty());
}

#[test]
fn test_backup_and_restore() {
    let dir = create_mock_config_home();
    let nvim_dir = dir.path().join(".config/nvim");

    let conn = nexus_core::db::open_in_memory().unwrap();

    let snap_id = crate::create_snapshot(&conn, Some(1), Some("test"), &nvim_dir).unwrap();
    assert!(snap_id > 0);

    let snapshots = crate::list_snapshots(&conn, None).unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].label.as_deref(), Some("test"));

    // Modify the file
    fs::write(nvim_dir.join("init.lua"), "-- modified config").unwrap();

    // Restore
    let count = crate::restore_snapshot(&conn, snap_id).unwrap();
    assert_eq!(count, 1);

    // Verify original content restored
    let content = fs::read_to_string(nvim_dir.join("init.lua")).unwrap();
    assert_eq!(content, "-- nvim config\nvim.opt.number = true");
}

#[test]
fn test_diff_snapshot_no_changes() {
    let dir = create_mock_config_home();
    let nvim_dir = dir.path().join(".config/nvim");

    let conn = nexus_core::db::open_in_memory().unwrap();
    let snap_id = crate::create_snapshot(&conn, Some(1), None, &nvim_dir).unwrap();

    let diffs = crate::diff_snapshot(&conn, snap_id, &nvim_dir).unwrap();
    assert!(diffs.is_empty());
}

#[test]
fn test_diff_snapshot_modified() {
    let dir = create_mock_config_home();
    let nvim_dir = dir.path().join(".config/nvim");

    let conn = nexus_core::db::open_in_memory().unwrap();
    let snap_id = crate::create_snapshot(&conn, Some(1), None, &nvim_dir).unwrap();

    fs::write(nvim_dir.join("init.lua"), "-- changed").unwrap();

    let diffs = crate::diff_snapshot(&conn, snap_id, &nvim_dir).unwrap();
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].status, crate::DiffStatus::Modified);
}

#[test]
fn test_diff_snapshot_added_and_removed() {
    let dir = create_mock_config_home();
    let nvim_dir = dir.path().join(".config/nvim");

    let conn = nexus_core::db::open_in_memory().unwrap();
    let snap_id = crate::create_snapshot(&conn, Some(1), None, &nvim_dir).unwrap();

    // Add a file
    fs::write(nvim_dir.join("plugins.lua"), "return {}").unwrap();
    // Remove the original
    fs::remove_file(nvim_dir.join("init.lua")).unwrap();

    let diffs = crate::diff_snapshot(&conn, snap_id, &nvim_dir).unwrap();
    assert_eq!(diffs.len(), 2);
    let statuses: Vec<_> = diffs.iter().map(|d| &d.status).collect();
    assert!(statuses.contains(&&crate::DiffStatus::Added));
    assert!(statuses.contains(&&crate::DiffStatus::Removed));
}

#[test]
fn test_profile_save_list_delete() {
    let dir = create_mock_config_home();
    let conn = nexus_core::db::open_in_memory().unwrap();

    conn.execute(
            "INSERT INTO config_tools (id, name, config_dir, description, language) VALUES (1, 'nvim', ?1, 'Neovim', 'lua')",
            [dir.path().join(".config/nvim").to_string_lossy().as_ref()],
        ).unwrap();

    let nvim_dir = dir.path().join(".config/nvim");
    let tools = vec![(1i64, "nvim".to_string(), nvim_dir)];

    let profile_id = crate::save_profile(&conn, "macbook", Some("test"), &tools).unwrap();
    assert!(profile_id > 0);

    let profiles = crate::list_profiles(&conn).unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].name, "macbook");

    let deleted = crate::delete_profile(&conn, "macbook").unwrap();
    assert!(deleted);

    let profiles = crate::list_profiles(&conn).unwrap();
    assert!(profiles.is_empty());
}
