# Orchestrate Module

| ID   | Owner  | Status      |
| ---- | ------ | ----------- |
| ORCH | @aneki | In Progress |

> **Note:** This is an exploratory "or" spec — an alternative to the TASKS
> module for providing programmatic plan navigation. TASKS focuses on Claude
> Code Tasks integration specifically. ORCH provides tool-agnostic CLI +
> optional MCP orchestration on top of existing APS markdown. The two could
> coexist (TASKS as one integration, ORCH as the general layer) or one may
> absorb the other. Decision deferred until both are better understood.

## Purpose

Provide programmatic orchestration on top of APS markdown specs — dependency
resolution, state-machine enforcement, context packaging, and learning
propagation — without introducing a database or breaking APS's
portable-markdown philosophy.

## Background

APS specs already contain everything needed for orchestration: work items with
dependencies, status fields, modules with scope boundaries. What's missing is
a programmatic interface for agents to navigate the plan without manually
scanning markdown.

Research into BMAD Method and Overseer (see
[docs/research/orchestration-patterns.md](../../docs/research/orchestration-patterns.md))
identified a synthesis: use BMAD's pattern (LLM as orchestrator, files as
truth) with Overseer's pattern (programmatic `nextReady()`, state enforcement)
by building CLI tools that read from and write back to markdown.

### Key Design Principles

1. **Markdown is the database** — no SQLite, no separate state store. The CLI
   parses `.aps.md` files directly and writes changes back to them.
2. **Progressive enhancement** — the CLI is optional. Agents and humans can
   always work directly with markdown. The CLI just makes it faster and less
   error-prone.
3. **Tool-agnostic** — the CLI works in any shell. An optional MCP server wraps
   it for agents that support MCP. A Conductor agent wraps it in prompts for
   agents that don't.
4. **No drift** — there is no second source of truth that can get out of sync.

## In Scope

- CLI commands for plan navigation (`next`, `start`, `complete`, `status`,
  `learn`)
- Dependency resolution from work item `Dependencies` fields
- State-machine enforcement (prevent invalid status transitions)
- Context packaging (assemble focused briefing when starting a work item)
- Learning capture and propagation (attach insights to work items/modules)
- Optional MCP server wrapping CLI operations
- Optional Conductor agent definition for prompt-based orchestration
- Optional VCS integration (branch-per-work-item)

## Out of Scope

- Replacing direct markdown editing (CLI is additive, not required)
- Real-time sync daemons
- Hosted services or cloud components
- Vendor-specific integrations (those belong in TASKS or INTEGRATIONS)
- Automated agent-to-agent communication (agents are dispatched by humans or
  by the Conductor agent, not by each other)

## Interfaces

**Depends on:**

- VAL (validation) — reuse markdown parser for reading APS documents
- AGENT (agents) — Conductor agent is a variant of the Planner agent

**Exposes:**

- `aps next [module]` — resolve next-ready work item (DFS through deps)
- `aps start <ID>` — mark In Progress, optionally create VCS branch, assemble
  context package
- `aps complete <ID>` — validate state transition, mark Complete, prompt for
  learnings
- `aps complete <ID> --learning "insight"` — attach learning to work item
  inline (per D-002; no standalone `aps learn` command shipped)
- `aps graph [module]` — show dependency graph with status coloring
- Optional: MCP server with codemode (single `execute` tool, natural language
  routing à la Overseer)
- Optional: Conductor agent (`.claude/agents/aps-conductor.md` etc.)
- Optional: Context package generator

## Concept Design

### CLI Operations

```
$ aps next
→ AUTH-003: Implement token refresh
  Module: auth | Dependencies: AUTH-001 ✓, AUTH-002 ✓ | Status: Ready

$ aps start AUTH-003
→ Marked AUTH-003 as In Progress
→ Context package: .aps/context/AUTH-003.md (assembled from module scope,
  parent decisions, dependency learnings)
→ Branch: work/AUTH-003 (created, checked out)

$ aps complete AUTH-003
→ Validated: AUTH-003 was In Progress → Complete ✓
→ Learning? (optional): "Token refresh needs retry logic for network failures"
→ Marked AUTH-003 as Complete
→ Learning attached and propagated to auth module

$ aps graph auth
→ AUTH-001 [Complete] ──→ AUTH-003 [Complete]
  AUTH-002 [Complete] ──┘
  AUTH-004 [Ready] ←── AUTH-003
  AUTH-005 [Draft] (blocked by AUTH-004)
```

### Context Packaging

When `aps start` is called, it assembles a context package:

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

```
Work Item States:
  Draft ──→ Ready ──→ In Progress ──→ Complete
                 ↑         │
                 └─────────┘ (reset if blocked)

Module States:
  Draft ──→ Ready ──→ In Progress ──→ Complete
```

The CLI enforces valid transitions. `aps complete` rejects items not In
Progress. `aps start` rejects items whose dependencies aren't Complete.

### MCP Server (Optional)

Wraps CLI operations as a single MCP codemode tool:

```json
{
  "name": "aps",
  "description": "APS plan orchestration",
  "inputSchema": {
    "type": "object",
    "properties": {
      "command": { "type": "string" }
    }
  }
}
```

Agent sends: `"What's the next ready work item in the auth module?"`
Server parses intent, runs `aps next auth`, returns structured result.

### Conductor Agent (Optional)

A rich agent definition (like BMAD's BMad Master) that:

- Knows the full APS lifecycle and rules
- Has a decision tree: assess plan → pick next item → dispatch to agent →
  validate checkpoint → capture learnings
- Uses CLI commands internally when available, falls back to direct markdown
  reading when not
- Works on any platform as a prompt file

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] Decisions resolved (D-001 through D-005)
- [x] Work items defined (ORCH-001 through ORCH-006)

## Work Items

### ORCH-001: Implement `aps next` command — Complete 2026-04-26

- **Intent:** Enable programmatic dependency resolution from APS markdown
- **Expected Outcome:** `aps next [module]` parses work items from `.aps.md`
  files, resolves dependency chains, returns next Ready item whose deps are
  all Complete. Reuses VAL module's markdown parser.
- **Validation:** `aps next` returns correct item in a project with mixed
  statuses and cross-module dependencies
- **Confidence:** high
- **Dependencies:** VAL (parser)
- **Status:** Complete
- **Action plan:** [execution/ORCH-001.actions.md](../execution/ORCH-001.actions.md)
- **Results:** Implemented as `lib/orchestrate.sh` (extract + collect +
  resolve_next helpers) plus `cmd_next` in `bin/aps`. Status detection handles both
  header-suffix (`— Complete <date>`) and explicit `- **Status:**` markers.
  Cross-module dependency resolution verified end-to-end via fixture and
  against this repo's plans/. JSON output for tooling. 5 new tests added
  (run.sh tests 16–20); full suite green.

### ORCH-002: Implement `aps start` and `aps complete`

- **Intent:** Enforce state transitions and capture learnings
- **Expected Outcome:** `aps start <ID>` marks In Progress in markdown,
  suggests branch name (`work/<ID>`), assembles context package. `aps complete
<ID>` validates transition (must be In Progress), marks Complete, prompts
  for optional learning (`- **Learning:** "..."`). Both reject invalid
  transitions with clear error messages.
- **Validation:** Status fields update in-place; invalid transitions rejected;
  learning appended when provided
- **Confidence:** high
- **Dependencies:** ORCH-001
- **Status:** Complete

### ORCH-003: Implement context packaging

- **Intent:** Assemble focused context when starting a work item
- **Expected Outcome:** `aps start` generates `.aps/context/<ID>.md`
  (gitignored, ephemeral) with: work item details, module scope and
  interfaces, relevant decisions, dependency learnings, related file paths
- **Validation:** Context file contains all expected sections; regeneration
  produces fresh output
- **Confidence:** medium
- **Dependencies:** ORCH-002
- **Status:** Complete

### ORCH-004: Implement `aps graph`

- **Intent:** Visualize dependency graph with status
- **Expected Outcome:** ASCII graph showing work items, dependency arrows,
  and status indicators (color or symbols). Optionally scoped to a module.
- **Validation:** Graph renders correctly for example project with 5+ items
  and cross-module deps
- **Confidence:** medium
- **Dependencies:** ORCH-001
- **Status:** Complete

### ORCH-005: Create Conductor agent

- **Intent:** Provide prompt-based orchestration for any tool
- **Expected Outcome:** Agent definition (multi-harness via scaffold/agents/)
  that can drive APS workflows: assess plan state, pick next item, dispatch
  to implementer agent, validate checkpoint, capture learnings. Uses CLI
  commands when available, falls back to direct markdown reading.
- **Validation:** Task-tool validation confirms Conductor instructions navigate
  two dependent fixture work items; test suite verifies Conductor variants
  install across supported harnesses
- **Confidence:** medium
- **Dependencies:** ORCH-001, AGENT-001
- **Validation Evidence:** 2026-05-12 Task validation passed for AUTH-003 ->
  AUTH-004 navigation using `--plans`, context package use, validation, and
  learning capture
- **Status:** Complete

### ORCH-006: Create MCP server

- **Intent:** Expose orchestration to MCP-capable agents
- **Expected Outcome:** TypeScript MCP server (using MCP SDK) wrapping CLI
  operations. Single codemode tool with natural language routing. Agents
  send requests like "next ready item in auth" and get structured results.
- **Validation:** MCP tool discovery succeeds; agent can call `aps next`
  through MCP; server handles malformed input gracefully
- **Confidence:** low
- **Dependencies:** ORCH-001, ORCH-002
- **Status:** Draft

## Decisions

- **D-001:** Should ORCH absorb TASKS or coexist? — _decided: coexist. ORCH
  is the tool-agnostic CLI layer (dependency resolution, state machine,
  context packaging). TASKS is a Claude Code-specific integration that can
  leverage ORCH's foundation. Revisit if TASKS never matures._
- **D-002:** Learning storage format — _decided: inline in work item metadata
  (`- **Learning:** "..."` after Validation field). Simpler, keeps everything
  in one place, no sync issues. Learnings are per-work-item, not per-module._
- **D-003:** VCS integration scope — _decided: advisory only. `aps start`
  suggests a branch name (`work/AUTH-003`) but does not create it. APS manages
  planning, not git workflow. Users have their own branching strategies._
- **D-004:** MCP server language — _decided: TypeScript using MCP SDK (Phase 3,
  optional). The MCP protocol requires JSON-RPC over stdio which shell can't
  handle cleanly. CLI stays pure bash. MCP server is an optional wrapper for
  agents that support MCP._
- **D-005:** Context package location — _decided: ephemeral at `.aps/context/`
  (gitignored). Context packages are assembled fresh on `aps start` from
  versioned source data (work items, modules, decisions). No need to version
  the assembled output — it would just be clutter._

## Execution Strategy

### Phase 1: CLI Foundation

- ORCH-001: `aps next` (dependency resolution)
- ORCH-002: `aps start` / `aps complete` (state machine)

### Phase 2: Context and Visualization

- ORCH-003: Context packaging
- ORCH-004: `aps graph`

### Phase 3: Agent Integration

- ORCH-005: Conductor agent
- ORCH-006: MCP server

## Relationship to Other Modules

| Module       | Relationship                                                                                           |
| ------------ | ------------------------------------------------------------------------------------------------------ |
| **TASKS**    | Alternative/complement — TASKS is Claude Code-specific; ORCH is tool-agnostic. Could coexist or merge. |
| **AGENT**    | ORCH's Conductor agent extends the Planner agent concept                                               |
| **VAL**      | ORCH reuses VAL's markdown parser                                                                      |
| **COMPOUND** | ORCH's learning capture feeds into COMPOUND's solution docs                                            |
| **INSTALL**  | ORCH CLI commands and MCP server need installation support                                             |

## Notes

- This module is explicitly exploratory. It captures patterns from BMAD Method
  (prompt-based orchestration, step-file architecture, context packaging) and
  Overseer (programmatic `nextReady()`, state enforcement, learning
  propagation) and synthesizes them for APS.
- The core bet: **markdown can be both human-readable AND machine-queryable**
  if you build a thin CLI layer that parses it. No database needed.
- The CLI should be implementable as an extension of the existing `./bin/aps`
  script, reusing the VAL module's parser.
- Phase 1 (CLI foundation) is achievable quickly — it's essentially `grep +
parse + sed` on structured markdown.
- Phases 2-3 are progressive enhancements that add value but aren't required
  for basic use.

## Research

- [Orchestration Patterns: BMAD, Overseer, and APS](../../docs/research/orchestration-patterns.md)
- [BMAD Plugin Feasibility](../../docs/research/bmad-plugin-feasibility.md)
