# Quick Start

## 1. Index your home directory

```bash
nexus scan
```

This recursively walks `~`, respecting `.gitignore` patterns, and indexes everything into SQLite with FTS5.

## 2. Search

```bash
nexus search "nvim config"
nexus search --category config "keybinding"
```

## 3. Manage configs

```bash
nexus config list          # discover tools in ~/.config
nexus config backup        # snapshot all configs
nexus config diff nvim     # see what changed since last backup
```

## 4. Start the watcher

```bash
nexus daemon start         # watch ~/.config for changes
nexus changes              # see recent filesystem changes
```

## 5. Launch the dashboard

```bash
nexus tui                  # terminal UI
# or
nexus serve                # web server at localhost:3000
```

## 6. Ask AI (optional)

Set `ANTHROPIC_API_KEY` in your environment, then:

```bash
nexus ask "what are my largest config files?"
```
