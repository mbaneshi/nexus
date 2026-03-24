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
fn test_export_configs_creates_archive() {
    let dir = create_mock_config_home();
    let home = dir.path();
    let output_dir = TempDir::new().unwrap();
    let archive_path = output_dir.path().join("configs.tar.gz");

    let tool_dirs = vec![
        ("nvim".to_string(), home.join(".config/nvim")),
        ("git".to_string(), home.join(".config/git")),
    ];

    let count = crate::export_configs(home, &tool_dirs, &archive_path).unwrap();
    assert_eq!(count, 3, "should export 3 files (init.lua + config + ignore)");
    assert!(archive_path.exists(), "archive file should exist");

    let meta = fs::metadata(&archive_path).unwrap();
    assert!(meta.len() > 0, "archive should not be empty");
}

#[test]
fn test_import_configs_restores_files() {
    let dir = create_mock_config_home();
    let home = dir.path();
    let output_dir = TempDir::new().unwrap();
    let archive_path = output_dir.path().join("configs.tar.gz");

    let tool_dirs = vec![
        ("nvim".to_string(), home.join(".config/nvim")),
        ("git".to_string(), home.join(".config/git")),
    ];

    crate::export_configs(home, &tool_dirs, &archive_path).unwrap();

    // Import into a fresh directory
    let import_home = TempDir::new().unwrap();
    let count = crate::import_configs(&archive_path, import_home.path()).unwrap();
    assert_eq!(count, 3, "should import 3 files");

    // Verify files exist and have correct content
    let init_lua = import_home.path().join(".config/nvim/init.lua");
    assert!(init_lua.exists(), "init.lua should be restored");
    assert_eq!(
        fs::read_to_string(&init_lua).unwrap(),
        "-- nvim config\nvim.opt.number = true"
    );

    let git_config = import_home.path().join(".config/git/config");
    assert!(git_config.exists(), "git config should be restored");
    assert_eq!(
        fs::read_to_string(&git_config).unwrap(),
        "[user]\n  name = Test"
    );

    let git_ignore = import_home.path().join(".config/git/ignore");
    assert!(git_ignore.exists(), "git ignore should be restored");
    assert_eq!(
        fs::read_to_string(&git_ignore).unwrap(),
        "*.swp\n.DS_Store"
    );
}

#[test]
fn test_import_configs_rejects_path_traversal() {
    // Build a malicious archive with a path traversal entry by writing raw tar bytes.
    // The `tar` crate's `append_data` rejects `..`, so we construct the header manually.
    let archive_dir = TempDir::new().unwrap();
    let archive_path = archive_dir.path().join("evil.tar.gz");

    {
        let file = fs::File::create(&archive_path).unwrap();
        let gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut out = std::io::BufWriter::new(gz);

        let content = b"malicious content";
        let path_bytes = b"../../../etc/passwd";

        // Build a minimal 512-byte POSIX tar header
        let mut header = [0u8; 512];
        header[..path_bytes.len()].copy_from_slice(path_bytes);
        // File mode "0000644\0"
        header[100..108].copy_from_slice(b"0000644\0");
        // Owner/group uid/gid "0000000\0"
        header[108..116].copy_from_slice(b"0000000\0");
        header[116..124].copy_from_slice(b"0000000\0");
        // File size in octal, 11 chars + NUL
        let size_octal = format!("{:011o}\0", content.len());
        header[124..136].copy_from_slice(size_octal.as_bytes());
        // Mtime "00000000000\0"
        header[136..148].copy_from_slice(b"00000000000\0");
        // Typeflag '0' = regular file
        header[156] = b'0';

        // Compute checksum: sum of all bytes with checksum field treated as spaces
        header[148..156].copy_from_slice(b"        ");
        let cksum: u32 = header.iter().map(|&b| b as u32).sum();
        let cksum_str = format!("{:06o}\0 ", cksum);
        header[148..156].copy_from_slice(&cksum_str.as_bytes()[..8]);

        use std::io::Write;
        out.write_all(&header).unwrap();
        out.write_all(content).unwrap();
        // Pad to 512-byte boundary
        let padding = 512 - (content.len() % 512);
        if padding < 512 {
            out.write_all(&vec![0u8; padding]).unwrap();
        }
        // Two zero blocks to end archive
        out.write_all(&[0u8; 1024]).unwrap();

        let gz = out.into_inner().unwrap();
        gz.finish().unwrap();
    }

    let import_home = TempDir::new().unwrap();
    let count = crate::import_configs(&archive_path, import_home.path()).unwrap();
    assert_eq!(count, 0, "path traversal entries should be rejected");
}

#[test]
fn test_export_import_round_trip() {
    let dir = create_mock_config_home();
    let home = dir.path();
    let archive_dir = TempDir::new().unwrap();
    let archive_path = archive_dir.path().join("round-trip.tar.gz");

    let tool_dirs = vec![
        ("nvim".to_string(), home.join(".config/nvim")),
        ("git".to_string(), home.join(".config/git")),
    ];

    let export_count = crate::export_configs(home, &tool_dirs, &archive_path).unwrap();

    let import_home = TempDir::new().unwrap();
    let import_count = crate::import_configs(&archive_path, import_home.path()).unwrap();
    assert_eq!(
        export_count, import_count,
        "import count should match export count"
    );

    // Verify every original file matches its imported copy
    let original_files = [
        (".config/nvim/init.lua", "-- nvim config\nvim.opt.number = true"),
        (".config/git/config", "[user]\n  name = Test"),
        (".config/git/ignore", "*.swp\n.DS_Store"),
    ];

    for (rel_path, expected_content) in &original_files {
        let imported = import_home.path().join(rel_path);
        assert!(imported.exists(), "{rel_path} should exist after import");
        let content = fs::read_to_string(&imported).unwrap();
        assert_eq!(
            &content, expected_content,
            "content mismatch for {rel_path}"
        );
    }
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
