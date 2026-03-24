# Nexus

> Home Command Center — Discovery + Config Management + AI

Nexus is a single Rust binary that gives you a complete picture of your machine:

- **Deep-index** your home directory with instant full-text search
- **Manage configs** for 20+ tools — backup, diff, restore, profile
- **Track changes** in real time with a filesystem daemon
- **Ask AI** questions about your filesystem via Claude

Three surfaces — **CLI**, **TUI dashboard**, **Web UI** — all sharing the same core.

## Why Nexus?

Your home directory is a sprawling, undocumented system. Config files scattered across `~/.config`, dotfiles everywhere, projects in various states. Nexus brings order:

- **One scan** indexes everything into SQLite with FTS5 — search is instant
- **One backup** snapshots all your configs — restore any tool to any point in time
- **One daemon** watches for changes — never lose track of what changed and when
- **One question** to AI — "what rust projects do I have?" gets a real answer

## Supported Config Tools

atuin, btop, fish, flutter, gcloud, gh, git, gitui, katana, mise, nushell, nvim, raycast, sniffnet, starship, stripe, uv, wezterm, zellij, zsh
