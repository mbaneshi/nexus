# Architecture Overview

Nexus is a Rust workspace with 8 crates using a **hybrid architecture** — concrete types everywhere, port traits only where abstraction is needed.

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

## Design Principles

- **Feature crate isolation** — each concern is its own crate, depends only on core
- **Minimal abstraction** — only 2 port traits: `LlmPort` (AI) and `ConfigStorePort` (nexus config)
- **Sync-first** — feature crates use synchronous iterator-based APIs (except `ai` which is async)
- **Parallelism via rayon** — scanner uses `ignore` crate's parallel walker
- **Message-passing TUI** — Event enum → update → draw cycle
- **spawn_blocking bridge** — async server wraps sync feature crate calls

## Data Flow

```
User Input
    │
    ▼
┌──────────┐     ┌──────────────┐     ┌──────────┐
│ Surface  │────▶│ Feature Crate│────▶│   Core   │
│ (CLI/TUI/│     │ (discovery/  │     │ (SQLite  │
│  Server) │     │  configs/ai) │     │  models) │
└──────────┘     └──────────────┘     └──────────┘
```

Surfaces compose feature crates. Feature crates use core for data access. Core owns the database and domain types.
