# Configuration

Nexus stores its own config at `~/.config/nexus/config.toml`.

## Initialize

```bash
nexus config init
```

This creates a default config file. View the path with:

```bash
nexus config path
```

## Settings

```toml
# ~/.config/nexus/config.toml

[scan]
# Directories to exclude from scanning
exclude = ["node_modules", ".git", "target", "__pycache__"]

[database]
# Database location (default: ~/.local/share/nexus/nexus.db)
path = "~/.local/share/nexus/nexus.db"

[server]
# Server port
port = 3000

[ai]
# Claude model to use
model = "claude-sonnet-4-20250514"
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | Required for AI features (`nexus ask`) |
| `NEXUS_LOG` | Set log level (e.g., `debug`, `info`, `warn`) |

## Database

All data is stored in a single SQLite database at `~/.local/share/nexus/nexus.db`.

- WAL mode for concurrent reads
- 64 MB cache, 256 MB mmap
- FTS5 for instant full-text search
- Parameterized queries only (no string interpolation)
