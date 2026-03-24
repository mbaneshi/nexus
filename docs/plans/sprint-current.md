# Sprint: 2026-03-24 — Phase 1 Complete

## Verified State (2026-03-24)

- **8 crates** in workspace, **37 Rust source files**, ~7,000 lines
- **17 tests** all passing, **0 clippy warnings**
- **16 CLI commands** working
- **3 API endpoints** (health, stats, config/tools)
- **3 TUI screens** (overview, configs, search)
- **GitHub Actions CI** running (fmt, clippy, test, build, coverage)
- **2 commits** on main, pushed to github.com/mbaneshi/nexus

## What's Done (Phase 0 + Phase 1)

### Core
- [x] Types: FileEntry, ConfigTool, ConfigFile, ConfigSnapshot, SearchQuery, etc.
- [x] SQLite WAL + FTS5 schema (9 tables + indexes + triggers)
- [x] TOML config loader with defaults (~/.config/nexus/config.toml)
- [x] LlmPort trait for AI abstraction
- [x] open_in_memory() for testing

### Discovery
- [x] Scanner (ignore crate parallel walker)
- [x] Batched indexer (5000-entry transactions)
- [x] FTS5 searcher with category/size filtering
- [x] File categorization (config, code, doc, image, cache, etc.)
- [x] Home stats aggregation

### Configs
- [x] 20-tool registry (atuin→zsh)
- [x] Tool discovery (scan ~/.config)
- [x] File scanner with blake3 hashing
- [x] Backup: gzip-compressed SQLite snapshots
- [x] Restore from snapshots
- [x] Diff: current vs snapshot (added/modified/removed)
- [x] Diff: snapshot vs snapshot
- [x] Profiles: save/list/apply/delete for machine provisioning

### Watcher
- [x] notify-based filesystem watcher with debouncing
- [x] Daemon management (start/stop/status via PID file)

### AI
- [x] Claude API adapter implementing LlmPort
- [x] Context builder (aggregates DB stats for LLM)
- [x] Query templates (filesystem + config system prompts)

### CLI (16 commands)
- [x] scan, search, stats
- [x] config list/show/backup/snapshots/restore/diff/init/path
- [x] config profile save/list/apply/delete
- [x] daemon start/stop/status
- [x] ask (AI query)
- [x] tui, serve

### Surfaces
- [x] TUI: 3-screen dashboard (overview, configs, search)
- [x] Server: 3 endpoints (health, stats, config/tools)

### CI/CD
- [x] GitHub Actions: fmt, clippy, test, build, coverage
- [x] GitHub Actions: multi-platform release (linux, macOS arm64/x86)
- [x] lefthook: pre-commit + pre-push hooks

## What's Next (Phase 2 — Surfaces + Polish)

### Priority 1: Expand TUI
- [ ] Interactive search screen (type query, see results live)
- [ ] Config detail view (show file contents with syntax highlighting)
- [ ] Recent changes panel (from file_changes table)
- [ ] Keybindings help bar

### Priority 2: Expand Web API + Frontend
- [ ] Search endpoint: GET /api/search?q=&category=&limit=
- [ ] Config endpoints: backup, snapshots, diff, restore
- [ ] Daemon status endpoint
- [ ] SvelteKit frontend scaffold (pnpm, Tailwind)
- [ ] Dashboard page: home stats + config overview
- [ ] Search page: live FTS5 search
- [ ] Config page: browse tools, view files, trigger backup

### Priority 3: Wire Daemon to DB
- [ ] Daemon writes file_changes to database
- [ ] Auto-snapshot ~/.config on changes (configurable)
- [ ] `nexus changes` command to show recent filesystem changes

### Priority 4: More Tests
- [ ] CLI integration tests (assert_cmd)
- [ ] Server route tests
- [ ] Watcher tests (with tempdir)

### Stretch
- [ ] `nexus config export` — export configs as a portable tar.gz
- [ ] `nexus config import` — import from tar.gz
- [ ] Syntax highlighting in `nexus config show` (syntect)
- [ ] Embed SvelteKit build in binary (rust-embed)
