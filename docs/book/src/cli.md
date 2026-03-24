# CLI Reference

The `nexus` binary provides 17 commands organized into groups.

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output in JSON format |
| `--verbose` | Enable verbose logging |

## Discovery

### `nexus scan`

Index your home directory. Uses the `ignore` crate for parallel walking (respects `.gitignore`).

### `nexus search <query>`

Full-text search across indexed files.

| Flag | Description |
|------|-------------|
| `--category <cat>` | Filter by category (config, code, doc, image, cache) |
| `--limit <n>` | Max results (default 20) |

### `nexus stats`

Show home directory statistics: file counts by category, total size, last scan time.

## Config Management

### `nexus config list`

List all discovered config tools in `~/.config`.

### `nexus config show <tool>`

Display config files for a specific tool.

### `nexus config backup`

Create a gzip-compressed snapshot of all config files.

### `nexus config snapshots`

List all saved snapshots with timestamps and sizes.

### `nexus config restore <snapshot-id>`

Restore config files from a snapshot.

### `nexus config diff <tool>`

Show diff between current config and the last snapshot (added/modified/removed files).

### `nexus config init`

Initialize the nexus config file at `~/.config/nexus/config.toml`.

### `nexus config path`

Print the config file path.

## Profiles

### `nexus config profile save <name>`

Save current configs as a named profile for machine provisioning.

### `nexus config profile list`

List all saved profiles.

### `nexus config profile apply <name>`

Apply a saved profile (restore configs from it).

### `nexus config profile delete <name>`

Delete a saved profile.

## Daemon

### `nexus daemon start`

Start the filesystem watcher daemon. Monitors `~/.config` for changes and records them. Auto-snapshots when a registered tool's config changes.

### `nexus daemon stop`

Stop the running daemon.

### `nexus daemon status`

Show whether the daemon is running.

### `nexus changes`

Display recent filesystem changes recorded by the daemon.

| Flag | Description |
|------|-------------|
| `--limit <n>` | Max changes to show |

## AI

### `nexus ask "<query>"`

Ask a natural language question about your filesystem. Requires `ANTHROPIC_API_KEY`.

## Surfaces

### `nexus tui`

Launch the terminal dashboard (4 screens: overview, configs, search, changes).

### `nexus serve`

Start the web server at `http://localhost:3000`.
