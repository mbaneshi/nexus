# Architecture Decision Records

## ADR-001: Hybrid Architecture (Concrete + Ports)

**Decision:** Use concrete types everywhere except for LLM access, which gets a port trait.

**Context:** Full hexagonal architecture with 15 port traits is overhead for a solo builder maintaining 8 crates. Only LLM access genuinely benefits from abstraction (swappable providers).

**Consequence:** Only `LlmPort` is abstract. Database, filesystem, and config access use concrete types. This can be expanded later if needed.

---

## ADR-002: Single SQLite Database

**Decision:** Store everything in one SQLite database at `~/.local/share/nexus/nexus.db`.

**Context:** Separate databases would complicate joins and backups. SQLite WAL mode handles concurrent reads well.

**Consequence:** Config snapshots stored as gzip-compressed BLOBs. Portable — copy one file to provision a new machine.

---

## ADR-003: Feature Crate Isolation

**Decision:** Feature crates depend only on `nexus-core`, never on each other.

**Context:** Prevents circular dependencies and allows independent compilation.

**Consequence:** If `configs` needs search, it goes through `core` types, not by importing `discovery`.

---

## ADR-004: FTS5 with Path Indexing

**Decision:** Include file paths in the FTS5 index, not just names.

**Context:** Users search by directory context ("rust projects", "nvim config"). Path-aware search is significantly more useful.

**Consequence:** Larger FTS index, but home directories are manageable in size.

---

## ADR-005: Config Snapshots as Compressed BLOBs

**Decision:** Store config file contents as gzip-compressed BLOBs in SQLite.

**Context:** Alternatives considered: git repo for dotfiles, tar archives. SQLite BLOBs are simplest — no external dependencies, self-contained, easy to restore.

**Consequence:** Git-backed versioning is a future enhancement, not a requirement.

---

## ADR-006: Three Surfaces, One Core

**Decision:** CLI, TUI, and Web server all compose the same feature crates.

**Context:** Each surface has different UX needs but identical data access patterns. Sharing core logic prevents divergence.

**Consequence:** Feature crates must be surface-agnostic. No stdout printing in library code.

---

## ADR-007: Known Tool Registry

**Decision:** Hard-code definitions for 20 known config tools rather than auto-detecting.

**Context:** Auto-detection would require heuristics and produce false positives. A curated registry is more reliable.

**Consequence:** Adding a new tool requires a code change. Acceptable — the list changes rarely.

---

## ADR-008: Claude-First AI

**Decision:** Ship with Claude API only, use `LlmPort` trait for future providers.

**Context:** Adding Ollama/OpenAI support can come later through the trait abstraction without changing calling code.

**Consequence:** Requires `ANTHROPIC_API_KEY` for AI features. All other features work without it.

---

## ADR-009: CI/CD from Day One

**Decision:** Ship with GitHub Actions CI (fmt, clippy, test, build, coverage) and multi-platform release automation.

**Consequence:** Every PR runs the full pipeline. Releases triggered by git tags (`v*`).
