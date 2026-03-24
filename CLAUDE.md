# Nexus — Project Conventions

> Home Command Center: Discovery + Config Management + AI
> Rust + SvelteKit | One binary, complete picture of your machine

## Overview
`nexus` is a Rust workspace with 8 crates and a SvelteKit frontend. It combines:
- **Home discovery** — deep index of ~ with FTS5 search, file categorization, change tracking
- **Config management** — browse, backup, diff, restore all 25 tools in ~/.config
- **AI integration** — natural language queries over your filesystem via Claude API
- **Three surfaces** — CLI, TUI dashboard, Web UI — all sharing the same core

The binary is named `nexus`.

## Build Commands
```bash
cargo build                              # build all Rust crates
cargo run -p nexus-cli -- scan           # index home directory
cargo run -p nexus-cli -- search QUERY   # FTS5 search
cargo run -p nexus-cli -- config list    # show all config tools
cargo run -p nexus-cli -- config backup  # snapshot configs
cargo run -p nexus-cli -- tui            # launch TUI dashboard
cargo run -p nexus-cli -- serve          # start web server
cargo run -p nexus-cli -- ask "query"    # AI query
cargo test                               # run all tests
cargo clippy -- -D warnings              # lint (must pass with 0 warnings)
```

## Code Conventions
- Rust edition 2024, MSRV 1.85
- All public functions must have doc comments
- Error handling: `thiserror` for library errors in feature crates, `color-eyre` in binary
- Async runtime: tokio (multi-thread) for server, single-thread for CLI commands
- Database access only through `core::db` module — no raw SQL in binaries
- All SQL queries use parameterized statements, never string interpolation
- TUI: follow ratatui's component pattern (Widget trait implementations)
- Frontend: components in `frontend/src/lib/`, pages in `frontend/src/routes/`
- Use `tracing` for all logging, never `println!` in library code
- **NEVER use npm.** Use `pnpm` for all frontend/Node.js work.

## Architecture Rules
- `core` has zero dependency on any feature crate, `tui`, `cli`, or `server`
- Feature crates (`discovery`, `configs`, `watcher`, `ai`) depend on `core` only (never on each other)
- `tui`, `server`, `cli` are integration layers that compose feature crates
- Feature crates are synchronous iterator-based APIs (except `ai` which is async)
- Parallelism comes from rayon and the `ignore` crate's parallel walker
- Server is async (tokio/axum) and wraps sync calls in `spawn_blocking`
- Progress reporting via callback closures: `Box<dyn Fn(Progress) + Send>`
- TUI uses message-passing architecture (Event enum → update → draw)
- Only 2 port traits: `LlmPort` (AI abstraction) and `ConfigStorePort` (nexus config)
- Everything else uses concrete types — no unnecessary abstraction

## Crate Layout
```
crates/
  core/          — models, config, db (SQLite WAL + FTS5), error, output
  discovery/     — home scanner, indexer, searcher, categorizer
  configs/       — ~/.config manager: registry (25 tools), backup, diff, restore, profiles
  watcher/       — filesystem daemon (notify + IPC + auto-snapshots)
  ai/            — Claude API adapter, query templates, context builder
  cli/           — clap binary with subcommands
  tui/           — ratatui dashboard (4 screens: overview, configs, search, AI)
  server/        — axum REST API + embedded SvelteKit
```

## Database
- Single SQLite database at `~/.local/share/nexus/nexus.db`
- WAL mode, 64MB cache, 256MB mmap
- FTS5 for instant full-text search
- Tables: files, files_fts, scans, config_tools, config_files, config_snapshots, snapshot_files, file_changes, ai_queries
- All migrations versioned in `core::db`

## Testing
- Unit tests inline in each module (`#[cfg(test)]`)
- Integration tests in individual crate `tests/` directories
- Frontend tests with vitest + Playwright
- Run `cargo test` before committing

## File Naming
- Rust: snake_case modules, one type per file when the type is large
- Svelte: PascalCase component files, kebab-case route folders

## Integration with Existing Projects
- Patterns from `free-storage-app` (fs): scanner, indexer, FTS5, daemon, TUI, server
- Patterns from `all-in-one-rusvel`: hexagonal ports (LlmPort), forge-engine mission planning, CI/CD, project management
- Config tools tracked: atuin, btop, fish, flutter, gcloud, gh, git, gitui, katana, mise, nushell, nvim, raycast, sniffnet, starship, stripe, uv, wezterm, zellij, zsh
