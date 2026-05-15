# Context Package Regeneration Prevents Stale Agent Handoffs

Generated APS context packages can drift if they are treated as durable project
state instead of temporary handoff material.

## Symptom

- **Error message:** None.
- **Behavior:** An agent starts from a context package that no longer reflects
  the current module spec, work item dependencies, or captured learnings.
- **Context:** This appears when context packages are committed, copied between
  branches, or reused after module docs change.

## Investigation

1. Storing generated packages in git made them easy to inspect, but every module
   edit created a second source of truth.
2. Asking agents to manually refresh copied context worked inconsistently
   because the stale file still looked authoritative.

## Root Cause

Context packages are derived from versioned APS sources: module specs, work
items, decisions, and dependency learnings. Persisting the generated package
turns an output artifact into a competing input artifact.

## Solution

Treat context packages as ephemeral and regenerate them from source whenever a
work item starts.

### Code Changes

No code example required. The durable convention is:

- Generate packages under `.aps/context/`.
- Keep `.aps/context/` out of version control.
- Recreate the package with `aps start <WORK-ITEM-ID>` after plan edits.

### Configuration Changes

```gitignore
.aps/context/
```

### Commands

```bash
aps start AUTH-003
```

## Prevention

- [ ] Treat `.aps/context/` as generated output, not source material.
- [ ] Link agents back to `plans/modules/*.aps.md` for canonical state.
- [ ] Capture reusable handoff lessons with `aps complete --learning "..."`.

## Related

- **Work item:** ORCH-003
- **Work item:** COMPOUND-001
- **PR:** N/A
- **Similar issues:** None yet
- **Documentation:** [APS workflow guide](../../workflow.md)

## Metadata

| Field       | Value             |
| ----------- | ----------------- |
| Date        | 2026-05-15        |
| Component   | APS orchestration |
| Severity    | minor             |
| Time to fix | 30 minutes        |
