# Orchestrate Module

| ID   | Owner  | Status   |
| ---- | ------ | -------- |
| ORCH | @aneki | Complete |

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
scanning markdown. Research into BMAD Method and Overseer informed the
approach — see [Research](#research) and the
[design doc](../designs/2026-06-08-orchestrate.design.md).

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

- `aps next [module]` — resolve the next-ready work item
- `aps start <ID>` — mark In Progress, suggest VCS branch, assemble context
  package
- `aps complete <ID>` — validate state transition, mark Complete, prompt for
  learnings
- `aps complete <ID> --learning "insight"` — attach learning to work item
  inline (per D-002; no standalone `aps learn` command shipped)
- `aps graph [module]` — show dependency graph with status coloring
- Optional: MCP server exposing the CLI command surface to MCP-capable agents
- Optional: Conductor agent (`.claude/agents/aps-conductor.md` etc.)
- Optional: Context package generator

## Designs

- [Orchestration CLI Design](../designs/2026-06-08-orchestrate.design.md) —
  as-built design: CLI operations, context packaging, state machine, MCP
  server, Conductor agent

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] Decisions resolved (D-001 through D-007; D-006 ratified 2026-06-08
      jointly with tui D-031)
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
- **Expected Outcome:** MCP server (technology per D-004) wrapping the CLI
  command surface behind a single tool. Agents send direct commands or
  requests like "next ready item in auth" and get structured results.
- **Validation:** MCP tool discovery succeeds; agent can call `aps next`
  through MCP; server handles malformed input gracefully
- **Learning:** "Node >=22.18 runs the TS server directly (no build step);
  TypeScript 6 no longer auto-includes @types — declare types:[node]
  explicitly"
- **Confidence:** low
- **Dependencies:** ORCH-001, ORCH-002
- **Status:** Complete: 2026-06-08
- **Action plan:** [execution/ORCH-006.actions.md](../execution/ORCH-006.actions.md)
- **Results:** Implemented as `mcp/` package — `src/index.ts` (server, MCP
  SDK per D-004) + `src/route.ts` (allowlisted command routing with
  natural-language fallback; shell metacharacters rejected, execution via
  `execFile` without a shell). Validated end-to-end with an MCP client
  against the orchestrate fixture: tool discovery, NL-routed `next`, direct
  `graph`, malformed-input error, and CLI-failure resilience. 14 tests
  (9 routing + 5 e2e) wired into `test/run.sh` as test 21 (skips when Node
  unavailable). Docs: `mcp/README.md` + MCP Server section in
  `docs/usage.md`.

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
- **D-006:** Bash-only CLI vs Rust migration — _decided 2026-06-08: amend
  D-004's "CLI stays pure bash". Bash remains the zero-dependency reference
  implementation; ORCH commands are ported to the Rust binary on the shared
  parser (parser + `lint` + `next` first, via tui TUI-009; `start`,
  `complete`, `graph` follow once parity is proven), with the fixtures under
  `test/fixtures/orchestrate/` reused as the parity suite so the two
  implementations cannot drift. The bash version is feature-frozen after
  parity is reached. Revisit if a fallback CLI (plain-prompt mode) lands in
  `eddacraft/eddacraft-tui` — that could change the fallback story and the
  bash freeze. See tui D-031 for the TUI-side counterpart._
- **D-007:** ORCH-006 (MCP server) disposition — _decided 2026-06-08:
  implement now. The server wraps the `aps` command surface (it shells out to
  whichever binary provides it), so it is agnostic to D-006's bash-vs-Rust
  outcome and need not wait for it. Owner call: ship ORCH-006 and complete
  the module with all six items._

## Relationship to Other Modules

| Module       | Relationship                                                                                           |
| ------------ | ------------------------------------------------------------------------------------------------------ |
| **TASKS**    | Alternative/complement — TASKS is Claude Code-specific; ORCH is tool-agnostic. Could coexist or merge. |
| **AGENT**    | ORCH's Conductor agent extends the Planner agent concept                                               |
| **VAL**      | ORCH reuses VAL's markdown parser                                                                      |
| **COMPOUND** | ORCH's learning capture feeds into COMPOUND's solution docs                                            |
| **INSTALL**  | ORCH CLI commands and MCP server need installation support                                             |

## Research

- [Orchestration Patterns: BMAD, Overseer, and APS](../research/orchestration-patterns.md)
