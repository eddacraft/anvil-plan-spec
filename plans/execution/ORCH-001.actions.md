# Action Plan: ORCH-001

| Field | Value |
|-------|-------|
| Source | [./modules/orchestrate.aps.md](../modules/orchestrate.aps.md) |
| Work Item | ORCH-001 — Implement `aps next` command |
| Created by | @aneki / AI |
| Status | In Progress |

## Prerequisites

- [x] VAL parser exists (`lib/rules/common.sh`, `lib/rules/workitem.sh`)
- [x] Module status convention documented in aps-rules.md

## Waves

| Wave | Actions | Gate |
|------|---------|------|
| 1 | 1, 2 | Parser helpers in lib/orch.sh, unit-tested |
| 2 | 3, 4 | `aps next` wired through bin/aps and lints clean |
| 3 | 5, 6 | End-to-end test passes against this repo's plans/ |

## Actions

### Action 1 — Add `lib/orch.sh` with parsing helpers

**Purpose**
Centralise the new parsing logic (work items + statuses + dependencies)
that ORCH commands will share. Keep `bin/aps` thin.

**Produces**

- `lib/orch.sh` exposing:
  - `orch_collect_work_items <plans_dir>` — emits `MODULE_FILE\tID\tSTATUS\tDEPS` per item
  - `orch_resolve_next <plans_dir> [module_filter]` — picks next Ready item with all deps Complete

**Checkpoint**
Sourced from a test script, the helpers print correct rows for plans/.

**Validate**
`bash -c 'source lib/output.sh; source lib/orch.sh; orch_collect_work_items plans/' | head`

**Wave** 1

### Action 2 — Add unit tests for `lib/orch.sh`

**Purpose**
Confirm dependency resolution handles cross-module deps, mixed statuses,
and empty inputs.

**Produces**

- `test/orch_test.sh` (bats-free, plain bash) with:
  - fixture plans tree under `test/fixtures/orch/`
  - cases: simple deps satisfied, deps not satisfied, cross-module deps,
    no Ready items, module filter

**Checkpoint**
`bash test/orch_test.sh` exits 0 with all cases green.

**Validate**
`bash test/orch_test.sh`

**Wave** 1

### Action 3 — Wire `aps next` subcommand into `bin/aps`

**Purpose**
Expose orchestration via the existing CLI entry point.

**Produces**

- New `cmd_next` in `bin/aps` (or a `lib/cmd_next.sh` extracted helper)
- `aps next` and `aps next <module>` work
- `--json` flag emits structured output (id, module, status, dependencies)
- Help text updated

**Checkpoint**
`./bin/aps next` returns one work item from this repo's plans/.

**Validate**
`./bin/aps next && ./bin/aps next orch && ./bin/aps next --json`

**Wave** 2

### Action 4 — Update lint target to ignore `lib/orch.sh` source if needed

**Purpose**
Make sure existing linting commands still pass.

**Produces**

No changes expected, but verify `aps lint` still reports the same baseline
(0 errors, 4 W003 cross-module warnings — those will be addressed in
COND-003).

**Checkpoint**
`./bin/aps lint` matches pre-change output.

**Validate**
`./bin/aps lint`

**Wave** 2

### Action 5 — End-to-end verification against this repo's plans/

**Purpose**
Catch regressions and confirm real-world behaviour.

**Produces**

- `aps next` on this repo returns ORCH-002 (after ORCH-001 is marked
  Complete) or TUI-002 (after TUI-001), proving multi-module ordering works
- Output documented in module spec Notes

**Checkpoint**
Output matches manual expectation; recorded as a Validation entry in the
work item.

**Wave** 3

### Action 6 — Mark ORCH-001 Complete

**Purpose**
Close the loop in APS itself.

**Produces**

- `plans/modules/orchestrate.aps.md` updated:
  - ORCH-001 status → Complete with date
  - Results note (capabilities + edge cases handled)
  - Module status flips back to Ready or remains In Progress depending on
    whether ORCH-002 is starting immediately

**Checkpoint**
Module spec reflects completion; `aps lint` clean.

**Wave** 3

## Completion

- [ ] All checkpoints validated
- [ ] Work item marked complete in `orchestrate.aps.md`

**Completed by:** TBD
