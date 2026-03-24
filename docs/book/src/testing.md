# Testing

Nexus has 34 tests across the workspace.

## Running tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p nexus-core
cargo test -p nexus-discovery
cargo test -p nexus-configs
cargo test -p nexus-server
cargo test -p nexus-watcher

# With output
cargo test -- --nocapture
```

## Test breakdown

| Crate | Tests | Coverage |
|-------|-------|----------|
| core | 8 | DB operations, config loading, output formatting |
| discovery | 4 | scan, index, search, stats |
| configs | 8 | discovery, scan, backup, diff, profiles |
| server | 10 | all 10 API endpoints |
| watcher | 4 | watch, modify detection, nonexistent paths, daemon status |

## Test strategy

- **Unit tests** are inline in each module (`#[cfg(test)]`)
- **Integration tests** live in each crate's `tests/` directory
- Core provides `open_in_memory()` for test databases — no disk I/O in tests
- Server tests use axum's `TestClient` for endpoint testing
- Frontend tests use vitest + Playwright (when the SvelteKit frontend is ready)

## Coverage

CI runs `cargo-llvm-cov` and uploads to Codecov. View coverage on the [Codecov dashboard](https://codecov.io/gh/mbaneshi/nexus).
