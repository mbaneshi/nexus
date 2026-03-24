# Nexus

> Home Command Center — Discovery + Config Management + AI

[![CI](https://github.com/mbaneshi/nexus/actions/workflows/ci.yml/badge.svg)](https://github.com/mbaneshi/nexus/actions/workflows/ci.yml)
[![Release](https://github.com/mbaneshi/nexus/actions/workflows/release.yml/badge.svg)](https://github.com/mbaneshi/nexus/releases)
[![Docs](https://github.com/mbaneshi/nexus/actions/workflows/docs.yml/badge.svg)](https://mbaneshi.github.io/nexus/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

One binary that gives you a complete picture of your machine: deep-index your home directory with instant full-text search, manage configs for 20+ tools, track filesystem changes in real time, and ask AI questions about it all.

**Three surfaces — CLI, TUI, Web UI — all sharing the same core.**

## Features

- **Home Discovery** — parallel scan of `~` into SQLite with FTS5 instant search, file categorization, and stats
- **Config Management** — browse, backup, diff, restore, and profile 20+ tools in `~/.config` (atuin, nvim, git, fish, zsh, …)
- **Filesystem Watcher** — daemon that tracks changes in real time with auto-snapshots
- **AI Integration** — natural language queries over your filesystem via Claude API
- **TUI Dashboard** — 4-screen terminal UI with syntax highlighting, interactive search, and live stats
- **REST API** — 10 endpoints for programmatic access
- **Web UI** — SvelteKit frontend (coming soon)

## Quick Start

### Install from release

Download the latest binary from [Releases](https://github.com/mbaneshi/nexus/releases):

```bash
# macOS (Apple Silicon)
curl -sL https://github.com/mbaneshi/nexus/releases/latest/download/nexus-$(curl -s https://api.github.com/repos/mbaneshi/nexus/releases/latest | grep tag_name | cut -d '"' -f4)-aarch64-apple-darwin.tar.gz | tar xz
sudo mv nexus /usr/local/bin/

# macOS (Intel)
# Replace aarch64-apple-darwin with x86_64-apple-darwin

# Linux (x86_64)
# Replace aarch64-apple-darwin with x86_64-unknown-linux-gnu
```

### Build from source

```bash
git clone https://github.com/mbaneshi/nexus.git
cd nexus
cargo build --release
# Binary at target/release/nexus
```

**Requirements:** Rust 1.85+ (edition 2024)

## Usage

```bash
# Index your home directory
nexus scan

# Search instantly
nexus search "nvim config"
nexus search --category config "keybinding"

# View stats
nexus stats

# Config management
nexus config list              # show discovered tools
nexus config show nvim         # view config files
nexus config backup            # snapshot all configs
nexus config snapshots         # list snapshots
nexus config diff nvim         # diff current vs last snapshot
nexus config restore <id>      # restore from snapshot

# Profiles (machine provisioning)
nexus config profile save work
nexus config profile apply work

# Filesystem watcher
nexus daemon start             # start watching ~/.config
nexus daemon status
nexus changes                  # show recent changes

# AI queries (requires ANTHROPIC_API_KEY)
nexus ask "what rust projects do I have?"

# Surfaces
nexus tui                      # terminal dashboard
nexus serve                    # web server at http://localhost:3000
```

## Architecture

```
                    ┌─────────┐  ┌─────────┐  ┌──────────┐
                    │   CLI   │  │   TUI   │  │  Server  │
                    └────┬────┘  └────┬────┘  └────┬─────┘
                         │           │             │
          ┌──────────────┼───────────┼─────────────┼──────────────┐
          │              │           │             │              │
     ┌────▼─────┐  ┌─────▼────┐  ┌──▼───┐  ┌─────▼────┐  ┌─────▼────┐
     │Discovery │  │ Configs  │  │Watch │  │   AI    │  │  Core   │
     │          │  │          │  │      │  │         │  │         │
     │ scanner  │  │ registry │  │notify│  │ Claude  │  │ models  │
     │ indexer  │  │ backup   │  │daemon│  │ context │  │ SQLite  │
     │ search   │  │ diff     │  │IPC   │  │ queries │  │ config  │
     │ category │  │ profiles │  │      │  │         │  │ errors  │
     └──────────┘  └──────────┘  └──────┘  └─────────┘  └─────────┘
```

All feature crates depend only on `core` — never on each other. See the [full architecture docs](https://mbaneshi.github.io/nexus/).

## Documentation

Full documentation is available at **[mbaneshi.github.io/nexus](https://mbaneshi.github.io/nexus/)**.

- [Architecture](https://mbaneshi.github.io/nexus/architecture.html)
- [Decision Records](https://mbaneshi.github.io/nexus/decisions.html)
- [API Reference](https://mbaneshi.github.io/nexus/api.html)
- [Contributing](https://mbaneshi.github.io/nexus/contributing.html)

## Development

```bash
cargo test                       # run all 34 tests
cargo clippy -- -D warnings      # lint (must pass clean)
cargo fmt --all --check          # format check
```

Git hooks via [lefthook](https://github.com/evilmartians/lefthook):
- **pre-commit:** fmt + clippy
- **pre-push:** full test suite

## Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2024, MSRV 1.85) |
| Database | SQLite (WAL + FTS5) via rusqlite |
| CLI | clap 4 |
| TUI | ratatui + crossterm |
| Server | axum + tower-http |
| AI | Claude API via reqwest |
| Frontend | SvelteKit (pnpm) |
| CI/CD | GitHub Actions |

## License

MIT
