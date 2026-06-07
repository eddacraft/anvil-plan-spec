<!-- APS: See docs/workflow.md → "Completion and Archival" for guidance -->
<!-- This document is an archive of completed task-level detail. -->

# Completed Work Archive

| Field   | Value      |
| ------- | ---------- |
| Scope   | ALL        |
| Status  | Active     |
| Updated | 2026-06-08 |

## Purpose

Historical record of all completed task-level work, grouped by release and
module area. Individual module specs in `plans/modules/` remain the
authoritative description of intent and decisions. This file preserves the
**task tables** for reference and traceability after a module has been
compacted.

Long-form implementation notes — when they exist — live in
`plans/completed/<release>-<module>.md` and are linked from the relevant
section below.

---

## Unreleased

_Will roll into the next cut. See [`plans/index.aps.md`](./index.aps.md) for
the active roadmap._

### Compound-Engineering Library

| Task         | Module   | Description                                | Status                |
| ------------ | -------- | ------------------------------------------ | --------------------- |
| COMPOUND-001 | compound | Solution library workflow                  | Complete: 2026-05-15  |
| COMPOUND-002 | compound | Completed-work archive pattern             | Complete: 2026-05-22  |
| COMPOUND-003 | compound | Release narrative convention               | Complete: 2026-05-22  |

### Orchestration

| Task     | Module      | Description                                  | Status               |
| -------- | ----------- | -------------------------------------------- | -------------------- |
| ORCH-001 | orchestrate | `aps next` — dependency resolution           | Complete: 2026-04-26 |
| ORCH-002 | orchestrate | `aps start` / `aps complete` — state machine | Complete             |
| ORCH-003 | orchestrate | Context packaging (`.aps/context/`)          | Complete             |
| ORCH-004 | orchestrate | `aps graph` — dependency visualization       | Complete             |
| ORCH-005 | orchestrate | Conductor agent (multi-harness)              | Complete: 2026-05-12 |
| ORCH-006 | orchestrate | MCP server (`mcp/`)                          | Complete: 2026-06-08 |

### TUI Onboarding

| Task    | Module | Description                                       | Status               |
| ------- | ------ | ------------------------------------------------- | -------------------- |
| TUI-001 | tui    | Project setup + eddacraft-tui integration         | Complete: 2026-04-26 |
| TUI-002 | tui    | Core wizard sections (profile, shape, AI tooling) | Complete: 2026-05-16 |
| TUI-003 | tui    | Template, path, and component sections            | Complete: 2026-06-08 |
| TUI-004 | tui    | Native scaffold execution + summary               | Complete: 2026-06-08 |
| TUI-005 | tui    | Non-interactive flags + config replay             | Complete: 2026-06-08 |
| TUI-006 | tui    | Cross-compilation release pipeline                | Complete: 2026-06-08 |
| TUI-007 | tui    | `aps setup` mode picker + shortcuts               | Complete: 2026-06-08 |
| TUI-008 | tui    | Agent bootstrap flow (`--agent`)                  | Complete: 2026-06-08 |
| TUI-009 | tui    | Rust parser + native `lint`/`next` parity         | Complete: 2026-06-08 |

---

## v0.3.0 — Orchestration & Multi-Agent Reach

_See [`plans/releases/v0.3.0.md`](./releases/v0.3.0.md) for theme, success
criteria, and risks._

### Multi-Agent Distribution

| Task      | Module | Description                                  | Status                |
| --------- | ------ | -------------------------------------------- | --------------------- |
| AGENT-001 | agents | APS Planner agent (Claude Code)              | Complete: 2026-02-21  |
| AGENT-002 | agents | APS Librarian agent (Claude Code)            | Complete: 2026-02-21  |
| AGENT-003 | agents | Port agents to Codex format                  | Complete: 2026-02-21  |
| AGENT-004 | agents | Port agents to Copilot, OpenCode, Gemini     | Complete: 2026-02-21  |
| AGENT-005 | agents | Agent documentation                          | Complete: 2026-03-24  |
| AGENT-006 | agents | Cross-harness testing                        | Complete: 2026-03-28  |

---

## v0.2.0 — First Release

_Foundational templates, validation, scaffolding, and documentation. The
release that turned the idea into a usable convention._

### Templates & Scaffolding

| Task         | Module    | Description                                | Status   |
| ------------ | --------- | ------------------------------------------ | -------- |
| TPL-001      | templates | Mark optional fields and simplify templates | Complete |
| SCAFFOLD-001 | scaffold  | Create starter plan scaffold               | Complete |

### Validation

| Task    | Module     | Description                  | Status   |
| ------- | ---------- | ---------------------------- | -------- |
| VAL-001 | validation | Implement APS lint command   | Complete |

### Documentation

| Task     | Module | Description                       | Status   |
| -------- | ------ | --------------------------------- | -------- |
| DOCS-001 | docs   | Publish onboarding documentation  | Complete |

---

## Conventions

- **Status column stays Complete.** This file is an archive; anything not
  Complete belongs in [`plans/index.aps.md`](./index.aps.md).
- **Don't rewrite history.** When a module ships, copy its task table as-is.
  If a task was later renamed or scoped down, note it inline rather than
  rewriting the original row.
- **Releases match `plans/releases/<version>.md`.** Use the same version
  string so the narrative and the task roll-up cross-reference cleanly.
- **Module specs stay in `plans/modules/`** by default. Move them into
  `plans/archive/modules/` only when the module is unlikely to be revisited
  and the active `modules/` directory needs to stay readable.
