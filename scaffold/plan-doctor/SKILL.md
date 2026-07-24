---
name: plan-doctor
description: >-
  Structural health check for an APS planning directory: broken references,
  orphaned modules, status drift, duplicate IDs, stale index state. Use when
  plans feel out of sync, before an autonomous run, or after merging plan
  changes from several branches.
---

# Plan Doctor

Validate the structural integrity of `plans/` and report actionable
issues. `aps-planning`'s truth gate asks "is this work item still true?";
plan-doctor asks "is the plan a well-formed, internally consistent
artifact?". Run it when plans drift: after long gaps, after merges that
touched plan files, before unattended runs, or when something feels off.

If `plans/index.aps.md` does not exist, this skill does not apply — say so
and stop.

## What to check

Walk the planning directory (`plans/` per `plans/aps-rules.md`) and verify
each of the following. Severity codes keep reports scannable and repeat
runs comparable.

### Errors — the plan is unreliable until fixed

| Code | Check                                                                                        |
| ---- | -------------------------------------------------------------------------------------------- |
| E01  | `index.aps.md` Modules table references a module file that does not exist                    |
| E02  | A module file is unreadable, or has no recognisable work items where its status implies some |
| E03  | Duplicate work item IDs (within a module or across modules)                                  |
| E04  | A dependency references a work item or module ID that exists nowhere in the plan             |

### Warnings — inconsistencies that will mislead an agent

| Code | Check                                                                                                                                                                                                             |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| W01  | Module file on disk that the index Modules table does not list (orphaned module)                                                                                                                                  |
| W02  | Module filename breaks the `NN-name.aps.md` convention, or numbering conflicts with dependency order                                                                                                              |
| W03  | Status outside the native APS transition set (`Draft`, `Ready`, `In Progress`, `Complete`, `Blocked`) and this catalogue's accepted ADR-0013 lifecycle states `Merged` (interim post-land), `Released`, `Shipped` |
| W04  | Item marked `Ready` whose dependencies are not terminal (`Complete`, or lifecycle states `Merged`, `Released`, `Shipped`)                                                                                         |
| W05  | Terminal item with no `Validation:` field or validation evidence                                                                                                                                                  |
| W06  | Action plan in `plans/execution/` whose work item ID no longer exists (orphaned actions)                                                                                                                          |
| W07  | Index module status contradicts its module file (e.g. index says `Done`, items still open)                                                                                                                        |
| W08  | ADR numbering collisions or mixed numbering schemes in `plans/decisions/`                                                                                                                                         |
| W09  | Unrecognised file at the `plans/` root — not a canonical APS artifact                                                                                                                                             |

### Info — worth a look, not necessarily wrong

| Code | Check                                                                             |
| ---- | --------------------------------------------------------------------------------- |
| I01  | Item `In Progress` with no recent journal entry or matching branch (may be stale) |
| I02  | Design doc in `plans/designs/` referenced by no module or work item               |
| I03  | Open question in the index resolved nowhere (no ADR, no item)                     |

These catalogues are a floor, not a ceiling — report anything else that
would mislead an agent reading the plan cold.

## Report

```
# Plan Doctor — <project>

Status: HEALTHY | DEGRADED | BROKEN
Errors: N | Warnings: N | Info: N

## Errors
- [E01] modules table lists 12-foo.aps.md — file missing
  Fix: restore the file or remove the row (content fix — propose, don't auto-apply)

## Warnings
- [W03] AUTH-002 status "Done" — canonical is "Complete"
  Fix: mechanical rename, auto-repairable
...
```

`HEALTHY` = no errors or warnings. `DEGRADED` = warnings only. `BROKEN` =
any error.

## Repairs

Follow the `aps-planning` rule: never auto-edit plan files unannounced.
Report first; apply repairs only when the user asks (or when running
inside an autonomous loop whose authority covers plan bookkeeping).

**Mechanical — safe to apply on request:**

- normalise unambiguous status aliases to the native APS vocabulary (W03), e.g.
  `Done` → `Complete` — but `Merged`, `Released`, and `Shipped` are accepted
  lifecycle states, not aliases, and are never normalised;
- add an index row for an orphaned module, mirroring the module's own
  status (W01);
- create missing canonical directories (`plans/execution/`,
  `plans/decisions/`, `plans/designs/`);
- reconcile an index module status from its module file (W07).

**Content — always propose, never auto-apply:**

- anything touching intent, scope, outcomes, or dependencies;
- renaming or renumbering module files;
- deleting orphaned files (move to a proposal; the user decides).

After applying repairs, re-run the affected checks and report the final
status — repairs claimed without a re-check are not verified.

## Cross-references

- `aps-planning` — semantic truth validation of individual items
- `dev-loop` (drain mode) — run plan-doctor during Orient when the journal shows
  a gap since the last cycle
- `plans/aps-rules.md` — the layout and conventions these checks enforce
