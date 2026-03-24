# Sprint: 2026-03-24 — Initial Scaffold

## Verified State

- **8 crates** in workspace
- **20 known config tools** in registry
- **9 SQLite tables** with FTS5
- **10 CLI commands** (scan, search, stats, config list/show/backup/snapshots/restore/init/path)
- **3 API endpoints** (health, stats, config/tools)
- **3 TUI screens** (overview, configs, search)
- **CI/CD** — 5-job GitHub Actions pipeline + multi-platform release

## Workspace Structure

```
crates/
  core/       — 6 modules: lib, config, db, models, error, output, ports
  discovery/  — 3 modules: scanner, indexer, searcher
  configs/    — 3 modules: registry, scanner, backup
  watcher/    — 1 module: filesystem watcher
  ai/         — 3 modules: claude, context, queries
  cli/        — main + 5 command modules
  tui/        — 3-screen dashboard
  server/     — 3 API routes
```

## Priority Items

1. **Verify clean compile** — `cargo build` with 0 errors
2. **Fix any clippy warnings** — `cargo clippy -- -D warnings`
3. **Run tests** — `cargo test` (currently: core config + db tests)
4. **Test scan** — `nexus scan --root /tmp/test` on a small directory
5. **Test config list** — `nexus config list` to discover tools
6. **Add more tests** — indexer, searcher, config registry
7. **Wire chrono** — add to watcher Cargo.toml (used in FileChange)
