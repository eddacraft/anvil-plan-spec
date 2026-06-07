# Orchestration CLI Design — ORCH Module

| Field   | Value                                                |
| ------- | ---------------------------------------------------- |
| Date    | 2026-06-08                                           |
| Status  | As-built (normalised from module spec)               |
| Modules | [orchestrate](../modules/orchestrate.aps.md)         |
| Scope   | ORCH-001 through ORCH-006                            |

> This design was originally embedded in `orchestrate.aps.md` as a "Concept
> Design" section. It was extracted to comply with APS layering rules (module
> specs hold interfaces and boundaries; design docs hold the how). Content
> reflects the as-built system.

## Problem

APS specs contain everything needed for orchestration — work items with
dependencies, status fields, module boundaries — but agents had no
programmatic interface to navigate a plan without manually scanning markdown.

Research into BMAD Method and Overseer (see
[orchestration patterns](../research/orchestration-patterns.md)) identified a
synthesis: BMAD's pattern (LLM as orchestrator, files as truth) combined with
Overseer's pattern (programmatic `nextReady()`, state enforcement) via CLI
tools that read from and write back to markdown.

## Design

### Key Principles

1. **Markdown is the database** — no SQLite, no separate state store. The CLI
   parses `.aps.md` files directly and writes changes back to them.
2. **Progressive enhancement** — the CLI is optional. Agents and humans can
   always work directly with markdown. The CLI just makes it faster and less
   error-prone.
3. **Tool-agnostic** — the CLI works in any shell. An optional MCP server wraps
   it for agents that support MCP. A Conductor agent wraps it in prompts for
   agents that don't.
4. **No drift** — there is no second source of truth that can get out of sync.

### CLI Operations

```text
$ aps next
→ AUTH-003: Implement token refresh
  Module: auth | Dependencies: AUTH-001 ✓, AUTH-002 ✓ | Status: Ready

$ aps start AUTH-003
→ Marked AUTH-003 as In Progress
→ Context package: .aps/context/AUTH-003.md (assembled from module scope,
  parent decisions, dependency learnings)
→ Suggested branch: work/AUTH-003 (advisory only, per D-003)

$ aps complete AUTH-003
→ Validated: AUTH-003 was In Progress → Complete ✓
→ Learning? (optional): "Token refresh needs retry logic for network failures"
→ Marked AUTH-003 as Complete

$ aps graph auth
→ AUTH-001 [Complete] ──→ AUTH-003 [Complete]
  AUTH-002 [Complete] ──┘
  AUTH-004 [Ready] ←── AUTH-003
  AUTH-005 [Draft] (blocked by AUTH-004)
```

Implemented as `lib/orchestrate.sh` (parsing + resolution helpers) wired
through `bin/aps`, reusing the VAL module's markdown parser. Essentially
`grep + parse + sed` on structured markdown — no runtime dependencies.

### Context Packaging

When `aps start` is called, it assembles an ephemeral context package at
`.aps/context/<ID>.md` (gitignored, per D-005):

```markdown
# Context: AUTH-003 — Implement token refresh

## Work Item

[Pulled from auth.aps.md — intent, outcome, validation, files]

## Module Scope

[Pulled from auth.aps.md — purpose, in-scope, interfaces]

## Decisions

[Pulled from auth.aps.md — relevant decisions]

## Dependency Learnings

- AUTH-001: "JWT library requires explicit algorithm whitelist"
- AUTH-002: "Session store must handle concurrent access"

## Related Files

[From work item Files field, expanded to actual paths]
```

This is analogous to BMAD's story files — a self-contained context that lets
an agent start fresh with everything it needs.

### State Machine

```text
Work Item States:
  Draft ──→ Ready ──→ In Progress ──→ Complete
                 ↑         │
                 └─────────┘ (reset if blocked)

Module States:
  Draft ──→ Ready ──→ In Progress ──→ Complete
```

The CLI enforces valid transitions. `aps complete` rejects items not In
Progress. `aps start` rejects items whose dependencies aren't Complete.

### MCP Server (ORCH-006)

A thin TypeScript server (MCP SDK, per D-004) that wraps the CLI as a single
codemode tool, à la Overseer:

```json
{
  "name": "aps",
  "description": "APS plan orchestration",
  "inputSchema": {
    "type": "object",
    "properties": {
      "request": { "type": "string" }
    }
  }
}
```

The agent sends either a direct command (`next auth`) or a natural-language
request (`"What's the next ready work item in the auth module?"`). The server
routes intent to a CLI invocation against an allowlisted command set
(`next`, `start`, `complete`, `graph`, `lint`), executes `bin/aps`, and
returns the structured result. Malformed or unroutable requests return a
help-text error rather than failing the transport.

### Conductor Agent (ORCH-005)

A rich agent definition (like BMAD's BMad Master) that:

- Knows the full APS lifecycle and rules
- Has a decision tree: assess plan → pick next item → dispatch to agent →
  validate checkpoint → capture learnings
- Uses CLI commands internally when available, falls back to direct markdown
  reading when not
- Works on any platform as a prompt file (multi-harness via
  `scaffold/agents/`)

### Phasing

| Phase | Items              | Theme                     |
| ----- | ------------------ | ------------------------- |
| 1     | ORCH-001, ORCH-002 | CLI foundation            |
| 2     | ORCH-003, ORCH-004 | Context and visualization |
| 3     | ORCH-005, ORCH-006 | Agent integration         |

Phase 1 was achievable quickly; phases 2–3 are progressive enhancements that
add value but aren't required for basic use.

## Decisions

Recorded in [orchestrate.aps.md](../modules/orchestrate.aps.md) — D-001
(coexist with TASKS), D-002 (inline learning storage), D-003 (advisory VCS
integration), D-004 (TypeScript MCP server), D-005 (ephemeral context
packages).

## Notes

- The core bet: **markdown can be both human-readable AND machine-queryable**
  if you build a thin CLI layer that parses it. No database needed.
- Captures patterns from BMAD Method (prompt-based orchestration, step-file
  architecture, context packaging) and Overseer (programmatic `nextReady()`,
  state enforcement, learning propagation).
