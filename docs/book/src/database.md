# Database

Nexus uses a single SQLite database at `~/.local/share/nexus/nexus.db`.

## Configuration

- **WAL mode** for concurrent reads
- **64 MB cache** (`cache_size = -65536`)
- **256 MB mmap** (`mmap_size = 268435456`)
- **Parameterized queries only** — no string interpolation

## Tables

| Table | Purpose |
|-------|---------|
| `files` | Indexed home directory entries (path, name, size, category, hash, timestamps) |
| `files_fts` | FTS5 virtual table for full-text search (indexes path and name) |
| `scans` | Scan history (start time, end time, file count, status) |
| `config_tools` | Discovered tools in `~/.config` (name, path, language) |
| `config_files` | Individual config files with blake3 content hashes |
| `config_snapshots` | Backup metadata (timestamp, tool, size) |
| `snapshot_files` | Gzip-compressed file contents (BLOBs) |
| `file_changes` | Filesystem change log from the watcher daemon |
| `ai_queries` | AI query history (query, response, model, tokens) |

## FTS5 Search

The `files_fts` table uses SQLite FTS5 for instant full-text search. It indexes file paths and names, supporting:

- Phrase queries: `"nvim config"`
- Prefix queries: `rust*`
- Boolean operators: `nvim AND lua`
- Column filtering: `path:projects`

## Migrations

All schema migrations are versioned in the `core::db` module and run automatically on first database open.
