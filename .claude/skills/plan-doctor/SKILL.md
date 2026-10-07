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

### Normalise status before judging it

`plans/aps-rules.md` ("Status Vocabulary") is the authority on status
values. Canonical values are `Draft`, `Ready`, `In Progress`, `Complete`,
`Blocked`, and two aliases are accepted — tools normalise these internally
and plan files are not rewritten:

- `Proposed` → `Draft` (not yet actionable)
- `Done` → `Complete` (terminal / compacted items)

Terminal compaction may also use `Merged`, `Released`, or `Shipped`; lint
treats them like `Complete`/`Done` for required-field exemptions, and so
does plan-doctor. These are accepted lifecycle states, not aliases (this
catalogue records them in ADR-0013; `Merged` is the interim post-land
state), so they are never rewritten either.

Apply that table **before** evaluating W03, W04, W05, or W07. A plan that
writes `Done` or `Proposed` throughout is conformant, not defective — never
report those values as status drift.

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
| W02  | Module file numbering contradicts dependency order — a numbered module sorts ahead of a module it depends on, or against the index Modules table order (only meaningful where prefixes exist; their absence is I04) |
| W03  | Status value still unrecognised after alias normalisation: neither canonical, nor a documented alias, nor an accepted lifecycle state — a one-off or freeform value (`WIP`, `Almost done`) an agent cannot place in the flow |
| W04  | Item marked `Ready` whose dependencies are not terminal (`Complete` — including its alias `Done` — or the lifecycle states `Merged`, `Released`, `Shipped`)                                                        |
| W05  | Terminal item with no `Validation:` field or validation evidence                                                                                                                                                  |
| W06  | Action plan in `plans/execution/` whose work item ID no longer exists (orphaned actions)                                                                                                                          |
| W07  | Index module status contradicts its module file (e.g. index says `Done`, items still open)                                                                                                                        |
| W08  | ADR numbering collisions or mixed numbering schemes in `plans/decisions/`                                                                                                                                         |
| W09  | Unrecognised file at the `plans/` root — not a canonical APS artifact                                                                                                                                             |

### Info — worth a look, not necessarily wrong

| Code | Check                                                                                                                                                       |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| I01  | Item `In Progress` with no recent journal entry or matching branch (may be stale)                                                                           |
| I02  | Design doc in `plans/designs/` referenced by no module or work item                                                                                         |
| I03  | Open question in the index resolved nowhere (no ADR, no item)                                                                                                |
| I04  | Module filename carries no `NN-` numeric prefix (aps-rules "Naming Conventions") — a style note, not a defect; report once per directory with a count        |
| I05  | Status value outside the documented vocabulary but used consistently as a local convention (e.g. `Archived` on archived index module rows) — report once per value |

W03 and I05 split the same observation by consistency. A value used once,
or used inconsistently across sibling items, is unmappable drift (W03). A
value applied uniformly to a whole class of rows — the way `Archived` marks
archived module rows in `index.aps.md` — is a local convention this spec
does not document, so it is Info, reported once for the value.

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
- [W03] AUTH-002 status "Almost done" — not canonical, not a documented alias
  Fix: mechanical rename to the nearest canonical value, auto-repairable

## Info
- [I05] status "Archived" on 4 index module rows — consistent local convention
  outside the aps-rules vocabulary; no action needed
- [I04] 91 of 91 module files have no NN- prefix — house style differs from
  aps-rules "Naming Conventions"; renumbering is a content change
...
```

`HEALTHY` = no errors or warnings. `DEGRADED` = warnings only. `BROKEN` =
any error. Info never affects that status and never blocks a run; omit the
Info section entirely when there is nothing to note.

Report each Info cause once, with a count (`4 index rows`, `91 of 91 module
files`), not one line per occurrence. Apply the same collapsing to a warning
that fires uniformly across a directory: a finding that hits 100% of files
is house style, not signal — state the pattern once, say how many files it
covers, and consider whether it belongs in Info instead.

## Repairs

Follow the `aps-planning` rule: never auto-edit plan files unannounced.
Report first; apply repairs only when the user asks (or when running
inside an autonomous loop whose authority covers plan bookkeeping).

**Mechanical — safe to apply on request:**

- normalise an unrecognised status to its nearest canonical value (W03), e.g.
  `Completed` → `Complete` — but the documented aliases `Proposed` and `Done`,
  and the lifecycle states `Merged`, `Released`, and `Shipped`, are accepted
  values that tools normalise internally, so never rewrite them in plan files;
- add an index row for an orphaned module, mirroring the module's own
  status (W01);
- create missing canonical directories (`plans/execution/`,
  `plans/decisions/`, `plans/designs/`);
- reconcile an index module status from its module file (W07).

**Content — always propose, never auto-apply:**

- anything touching intent, scope, outcomes, or dependencies;
- renaming or renumbering module files (W02, I04);
- deleting orphaned files (move to a proposal; the user decides).

After applying repairs, re-run the affected checks and report the final
status — repairs claimed without a re-check are not verified.

## Cross-references

- `aps-planning` — semantic truth validation of individual items
- `dev-loop` (drain mode) — run plan-doctor during Orient when the journal shows
  a gap since the last cycle
- `plans/aps-rules.md` — the layout and conventions these checks enforce
