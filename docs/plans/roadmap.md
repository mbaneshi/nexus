# Nexus — Roadmap

> 3 phases. 8 crates. One binary.

---

## Phase 0 — Foundation ← YOU ARE HERE

> Walking skeleton: scan home, store in SQLite, search via CLI.

**Deliverables:**
- `nexus-core`: types, DB schema (WAL + FTS5), config (TOML), error types
- `nexus-discovery`: scanner (ignore crate), indexer (batched inserts), searcher (FTS5)
- `nexus-configs`: tool registry (20 tools), discovery, backup/restore
- `nexus-cli`: scan, search, stats, config (list/show/backup/snapshots/restore/init/path)
- CI/CD: GitHub Actions (fmt, clippy, test, build, coverage, release)
- Git hooks: lefthook (pre-commit, pre-push)

**Proves:** End-to-end: `nexus scan` → `nexus search` → `nexus config list` → `nexus config backup`

---

## Phase 1 — Watcher + AI

> Live filesystem monitoring and natural language queries.

**Deliverables:**
- `nexus-watcher`: notify daemon, debounced changes, auto-snapshots on config changes
- `nexus-ai`: Claude API adapter, context builder, query templates
- CLI: `nexus ask`, `nexus daemon start|stop|status`
- Config diff: compare snapshots, show changes
- Config profiles: named profiles for machine provisioning

**Proves:** `nexus daemon start` watches ~/.config, auto-snapshots on change. `nexus ask "what changed today?"` returns useful answers.

---

## Phase 2 — Surfaces

> TUI dashboard and web UI.

**Deliverables:**
- `nexus-tui`: 4-screen dashboard (Overview, Configs, Search, AI Chat)
- `nexus-server`: axum REST API (stats, search, config tools, snapshots)
- SvelteKit frontend: dashboard with file browser, config viewer, search, AI chat
- Frontend embedded in binary via rust-embed

**Proves:** Full app experience: CLI + TUI + Web, all powered by the same core crates.

---

## Future Ideas (Not Scheduled)

- Git-backed dotfile versioning (automatic commits to a dotfiles repo)
- Machine setup profiles (provision new machines from snapshots)
- Plugin system for custom tool definitions
- Cross-machine sync via git remote
- MCP server integration
- Integration with rusvel's forge-engine for mission planning
- Homebrew formula for easy installation

---

## Crate Count

| Phase | Crates | Total |
|-------|--------|-------|
| 0 | core, discovery, configs, cli + CI/CD | 4 |
| 1 | watcher, ai | 6 |
| 2 | tui, server | 8 |

All 8 crates defined from the start. Phases expand what's implemented, not what exists.
