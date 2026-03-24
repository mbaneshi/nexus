# Contributing

## Setup

```bash
git clone https://github.com/mbaneshi/nexus.git
cd nexus
cargo build
cargo test
```

**Requirements:**
- Rust 1.85+ (edition 2024)
- [lefthook](https://github.com/evilmartians/lefthook) for git hooks (optional but recommended)

## Development workflow

```bash
# Run tests
cargo test

# Lint (must pass with 0 warnings)
cargo clippy -- -D warnings

# Format
cargo fmt --all

# Check format without changing
cargo fmt --all --check
```

## Code conventions

- All public functions must have doc comments
- Error handling: `thiserror` in library crates, `color-eyre` in the binary
- Use `tracing` for logging, never `println!` in library code
- Database access only through `core::db` — no raw SQL in binaries
- All SQL queries use parameterized statements
- TUI: follow ratatui's Widget trait pattern
- Frontend: use `pnpm`, never `npm`

## Architecture rules

- `core` has zero dependency on any other workspace crate
- Feature crates depend on `core` only, never on each other
- Surfaces (CLI, TUI, server) compose feature crates
- Only 2 port traits: `LlmPort` and `ConfigStorePort`

## Pull requests

- CI must pass (fmt, clippy, test, build)
- Keep PRs focused — one concern per PR
- Write tests for new functionality
