# Phase 0 — Foundation

> Prove the architecture with a working scan → index → search pipeline.

## Milestones

### M0.1 — Core Types & Database
- [x] `nexus-core` crate with models, error types, config
- [x] SQLite WAL + FTS5 schema with migrations
- [x] Config loader (TOML) with sensible defaults
- [x] `LlmPort` trait for future AI integration
- [x] Output formatting utilities

### M0.2 — Discovery Engine
- [x] Scanner using `ignore` crate parallel walker
- [x] Batched indexer with FTS5 trigger-based updates
- [x] FTS5 searcher with category/size filtering
- [x] File categorization (config, code, document, image, cache, etc.)

### M0.3 — Config Manager
- [x] Tool registry with 20 known tools
- [x] Tool discovery (scan ~/.config for installed tools)
- [x] Config file scanner (recursive, with blake3 hashing)
- [x] Backup: snapshot to SQLite as gzip-compressed BLOBs
- [x] Restore: decompress and write files from snapshots
- [x] Snapshot listing

### M0.4 — CLI Surface
- [x] Clap binary with subcommands + global flags (--json, --verbose)
- [x] `nexus scan` — index home directory
- [x] `nexus search` — FTS5 search with category filter
- [x] `nexus stats` — home directory statistics
- [x] `nexus config list` — discover and list tools
- [x] `nexus config show <tool>` — list config files
- [x] `nexus config backup [tool]` — create snapshot
- [x] `nexus config snapshots` — list snapshots
- [x] `nexus config restore <id>` — restore from snapshot
- [x] `nexus config init` — create default config
- [x] `nexus config path` — show config file path

### M0.5 — CI/CD & Project Management
- [x] GitHub Actions: fmt, clippy, test, build, coverage
- [x] GitHub Actions: multi-platform release
- [x] lefthook: pre-commit (fmt + clippy), pre-push (test)
- [x] Architecture documentation
- [x] ADRs
- [x] Roadmap
- [x] CLAUDE.md project conventions

### M0.6 — Stub Surfaces
- [x] `nexus-tui` — basic 3-screen dashboard (overview, configs, search)
- [x] `nexus-server` — health, stats, config tools endpoints
- [x] `nexus-ai` — Claude API adapter + context builder
- [x] `nexus-watcher` — filesystem watcher with debouncing

## Next: Phase 1
After foundation is verified (compiles, tests pass, CI green):
1. Wire watcher daemon with IPC
2. Implement AI queries end-to-end
3. Add config diff between snapshots
4. Add config profiles for machine provisioning
