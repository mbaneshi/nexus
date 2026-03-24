# Nexus — Architecture

> Home Command Center: Discovery + Config Management + AI

## Overview

Nexus is a Rust workspace with 8 crates that provides:
1. **Home Discovery** — deep scan of ~ into SQLite with FTS5 instant search
2. **Config Management** — browse, backup, diff, restore all ~/.config tools
3. **AI Integration** — natural language queries over your filesystem via Claude
4. **Three Surfaces** — CLI, TUI dashboard, Web UI sharing the same core

## Architecture Style

**Hybrid** — combining two proven patterns:
- **Feature crate isolation** (from `free-storage-app`): each concern is its own crate, depends only on core
- **Port traits** (from `all-in-one-rusvel`): but only where abstraction is needed (LLM access), not everywhere

## Dependency Graph

```
nexus-core  (zero deps on other workspace crates)
  │
  ├── nexus-discovery   (depends on core only)
  ├── nexus-configs     (depends on core only)
  ├── nexus-watcher     (depends on core only)
  ├── nexus-ai          (depends on core only)
  │
  ├── nexus-cli         (composes all feature crates)
  ├── nexus-tui         (composes core + discovery + configs)
  └── nexus-server      (composes core + discovery + configs + ai)
```

**Rule: Feature crates never depend on each other.**

## Crate Responsibilities

### nexus-core
- Domain types: `FileEntry`, `ConfigTool`, `ConfigFile`, `ConfigSnapshot`, `SearchQuery`
- Database: SQLite WAL + FTS5, migrations, pragmas
- Config: TOML loader for `~/.config/nexus/config.toml`
- Error types: `NexusError` with variants for each concern
- Port traits: `LlmPort` (only external boundary needing abstraction)
- Output: formatting utilities (JSON, table, human-readable sizes)

### nexus-discovery
- Scanner: recursive walk using `ignore` crate (respects .gitignore patterns)
- Indexer: batched SQLite inserts with FTS5 trigger-based updates
- Searcher: FTS5 queries with category/size filtering
- Categorizer: classifies files (config, code, document, image, cache, etc.)

### nexus-configs
- Registry: 20 known tool definitions (path patterns, languages)
- Scanner: discovers installed tools in ~/.config
- Backup: snapshot configs as gzip-compressed BLOBs in SQLite
- Restore: decompress and write files from snapshots
- (Future) Diff: compare snapshots, show what changed
- (Future) Profiles: named setup profiles for machine provisioning

### nexus-watcher
- Filesystem daemon using `notify` crate
- Debounced change tracking
- (Future) Auto-snapshot on config changes
- (Future) IPC via Unix socket for daemon control

### nexus-ai
- Claude API adapter implementing `LlmPort`
- Context builder: creates filesystem summary from DB for LLM
- Query templates: system prompts for filesystem and config queries
- Query history stored in SQLite

### nexus-cli
- Clap-based binary with subcommands
- Global flags: `--json`, `--verbose`
- Commands: scan, search, stats, config (list/show/backup/snapshots/restore/init/path), ask, tui, serve

### nexus-tui
- Ratatui terminal dashboard
- 3 screens: Overview, Configs, Search
- Message-passing architecture (Event → update → draw)

### nexus-server
- Axum REST API with CORS
- Endpoints: /api/health, /api/stats, /api/config/tools
- (Future) Embedded SvelteKit frontend via rust-embed

## Data Layer

Single SQLite database at `~/.local/share/nexus/nexus.db`.

Key tables:
- `files` + `files_fts` — indexed home directory with full-text search
- `scans` — scan history and status
- `config_tools` — discovered tools in ~/.config
- `config_files` — individual config files with content hashes
- `config_snapshots` + `snapshot_files` — backup storage
- `file_changes` — filesystem change log
- `ai_queries` — AI query history

Performance: WAL mode, 64MB cache, 256MB mmap, parameterized queries only.

## Integration Points

### From free-storage-app (fs)
- Scanner pattern: `ignore` crate parallel walker
- Indexer pattern: batched SQLite inserts with FTS5
- Daemon pattern: `notify` + debouncing + IPC
- CLI pattern: clap derive with `--json` global flag
- Server pattern: axum + `spawn_blocking` for sync calls

### From all-in-one-rusvel
- Port trait design: `LlmPort` for AI abstraction
- CI/CD: GitHub Actions (fmt, clippy, test, build, coverage, release)
- Git hooks: lefthook (pre-commit: fmt+clippy, pre-push: tests)
- Project management: phased roadmap, sprint tracking, ADRs

## Stack

- Rust edition 2024, MSRV 1.85
- SQLite (rusqlite) with WAL, FTS5, parameterized queries
- tokio for async (server), rayon for parallelism (scanner)
- axum for HTTP, clap for CLI, ratatui for TUI
- Claude API for AI queries
- SvelteKit for web frontend (future)
- GitHub Actions for CI/CD
