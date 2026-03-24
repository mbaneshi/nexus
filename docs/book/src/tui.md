# TUI Dashboard

Launch with `nexus tui`. The terminal UI has 4 screens navigable with Tab/Shift+Tab.

## Screens

### Overview

Home directory stats at a glance: file counts by category, total size, last scan time, category breakdown chart.

### Configs

Browse discovered config tools. Select a tool to view its files with syntax highlighting (via syntect). Navigate with arrow keys.

### Search

Interactive full-text search. Type your query and see FTS5 results update. Navigate results with arrow keys.

### Changes

Recent filesystem changes recorded by the daemon. Shows file path, change type, and timestamp.

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch screens |
| `↑` / `↓` | Navigate lists |
| `Enter` | Select item |
| `Ctrl+R` | Refresh data |
| `?` | Toggle help overlay |
| `q` / `Esc` | Quit |
