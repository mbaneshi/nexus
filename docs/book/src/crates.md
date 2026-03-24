# Crate Layout

```
crates/
  core/          — models, config, db (SQLite WAL + FTS5), error, output
  discovery/     — home scanner, indexer, searcher, categorizer
  configs/       — ~/.config manager: registry, backup, diff, restore, profiles
  watcher/       — filesystem daemon (notify + IPC + auto-snapshots)
  ai/            — Claude API adapter, query templates, context builder
  cli/           — clap binary with subcommands
  tui/           — ratatui dashboard (4 screens)
  server/        — axum REST API + embedded SvelteKit
```

## nexus-core

The foundation. Zero dependencies on other workspace crates.

- **Domain types:** `FileEntry`, `ConfigTool`, `ConfigFile`, `ConfigSnapshot`, `SearchQuery`
- **Database:** SQLite WAL + FTS5, migrations, pragmas (9 tables)
- **Config:** TOML loader for `~/.config/nexus/config.toml`
- **Error types:** `NexusError` with variants for each concern
- **Port traits:** `LlmPort` (AI abstraction)
- **Output:** formatting utilities (JSON, table, human-readable sizes)

## nexus-discovery

Home directory scanner and search engine.

- **Scanner:** recursive walk using `ignore` crate (parallel, respects `.gitignore`)
- **Indexer:** batched SQLite inserts (5000 entries per transaction) with FTS5 triggers
- **Searcher:** FTS5 queries with category/size filtering
- **Categorizer:** classifies files (config, code, document, image, cache, etc.)

## nexus-configs

Config file manager for `~/.config`.

- **Registry:** 20 known tool definitions (path patterns, languages)
- **Scanner:** discovers installed tools, hashes files with blake3
- **Backup:** gzip-compressed snapshots stored as BLOBs in SQLite
- **Restore:** decompress and write files from snapshots
- **Diff:** current vs snapshot, snapshot vs snapshot comparison
- **Profiles:** named config sets for machine provisioning

## nexus-watcher

Filesystem change tracking daemon.

- **Watcher:** `notify` crate with debouncing
- **Daemon:** PID file management, start/stop/status
- **Auto-snapshot:** snapshots `~/.config` tools when their files change
- **Change log:** records all changes to `file_changes` table

## nexus-ai

Claude API integration.

- **Adapter:** implements `LlmPort` trait for Claude API
- **Context builder:** aggregates DB stats into LLM context
- **Templates:** system prompts for filesystem and config queries
- **History:** stores queries and responses in SQLite

## nexus-cli

The binary. Composes all feature crates via clap subcommands (17 commands).

## nexus-tui

Terminal dashboard with 4 screens (overview, configs, search, changes). Uses ratatui with message-passing architecture.

## nexus-server

Axum REST API with 10 endpoints. Uses `spawn_blocking` to bridge sync feature crates into async handlers.
