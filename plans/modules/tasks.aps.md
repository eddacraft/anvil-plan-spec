# Tasks Module

| ID    | Owner  | Priority | Status |
| ----- | ------ | -------- | ------ |
| TASKS | @aneki | medium   | Draft  |

## Purpose

Explore Claude Code Tasks integration as an execution layer on top of APS work
items and action plans.

## In Scope

- Mapping APS work items to Claude Code task dispatch
- Wave planning prompts
- Agent assignment guidance
- Status sync between task results and APS markdown

## Out of Scope

- Tool-agnostic orchestration, which belongs to ORCH
- Replacing APS action plans
- Persisting task state outside markdown

## Interfaces

**Depends on:**

- AGENT — Planner agent owns task dispatch guidance
- ORCH — may provide dependency and context primitives later

**Exposes:**

- `docs/ai/prompting/claudecode/tasks-from-module.prompt.md`
- `docs/ai/prompting/claudecode/agent-assignment.prompt.md`
- `docs/ai/prompting/claudecode/wave-planning.prompt.md`
- `docs/ai/prompting/claudecode/sync-status.prompt.md`

## Work Items

### TASKS-001: Document APS-to-Tasks mapping — Draft

- **Intent:** Clarify when Claude Code Tasks should be used for APS execution
- **Expected Outcome:** Prompt and documentation explain how to turn a module's
  ready work items into parallel or sequential Task dispatches.
- **Validation:** Dry-run against an example module produces coherent task
  assignments without changing plan status prematurely
- **Files:** docs/ai/prompting/claudecode/, docs/ai-agent-guide.md
- **Confidence:** medium
