# Action Plan: MONO-002

| Field      | Value                                                   |
| ---------- | ------------------------------------------------------- |
| Source     | [./modules/monorepo.aps.md](../modules/monorepo.aps.md) |
| Work Item  | MONO-002 — Lint traversal of nested plans               |
| Created by | @aneki / AI                                             |
| Status     | In Progress                                             |

## Goal

Make `aps lint`, pointed at a federated **parent** root, validate the whole
tree as one plan: follow `## Child Plans` links, lint each child as a full
plan, resolve `<name>:<ID>` cross-tree dependencies, and flag work-item ID
collisions between trees.

## Grounding (from MONO-001 findings)

- `lib/lint.sh` `find_aps_files` recurses and `build_id_index` indexes every
  found file — so once child files are in the lint set, cross-tree deps already
  resolve. The missing piece is **discovery**: children live at
  `packages/<pkg>/plans/`, outside the parent `plans/`, so a default
  `aps lint plans/` never sees them.
- `check_w003_dependencies` (`lib/rules/workitem.sh`) extracts `[A-Z]+-[0-9]{3}`
  and **ignores any `<name>:` prefix** — so it cannot validate that a cross-tree
  ref names the right child, nor stay quiet when a child is linted in isolation.
- The bash linter is canonical (tested by `test/run.sh`); `lib/*.psm1` is a
  PowerShell parity port (W017/W018/W019 are mirrored there).

## Waves

| Wave | Actions | Gate                                                         | Done |
| ---- | ------- | ------------------------------------------------------------ | ---- |
| 1    | 1, 2    | `## Child Plans` traversal: parent root lints children too   | ✅ 2026-06-27 |
| 2    | 3, 4    | `<name>:`-prefix-aware W003 (federated resolves, isolated clean, bad ref warns) | ✅ 2026-06-27 |
| 3    | 5, 6    | Cross-tree ID-collision detection (W020) + collision fixture | ✅ 2026-06-27 |
| 4    | 7, 8    | PowerShell parity + docs/closeout                            | ✅ 2026-06-30 |

**Bash implementation (Waves 1–3) is complete and fully tested** via
`test/run.sh` tests 42–45 (traversal, bad-ref, isolated-clean, W020 collision).
New warning code **W020** documented in `docs/usage.md`; as-built behaviour in
the [design doc](../designs/2026-06-27-nested-plans.design.md). **PowerShell
parity (Action 7) is complete**: `lib/Lint.psm1` and `lib/rules/WorkItem.psm1`
mirror traversal, prefix-aware W003, and W020. Verified behaviourally against
all four scenarios under PowerShell 7.4.6 (a locally-fetched self-contained
build) — output identical to bash — and guarded in CI by a string-parity
check (test 46), matching the repo's existing `install.ps1` parity convention.

## Actions

### Action 1 — Child-plan traversal in `cmd_lint`

**Purpose**
When a linted directory contains a parent `index.aps.md` with a
`## Child Plans` section, pull each linked child plan tree into the files to
lint and into the ID index — transitively, with dedup.

**Produces**

- `lib/lint.sh`: `normalize_path` (lexical `.`/`..` collapse, keeps relative
  paths relative), `resolve_child_plan_links <index>` (emit child index paths
  from the `## Child Plans` section, resolved against the parent dir), and an
  expansion step in `cmd_lint` that grows `files`/`index_files` transitively
  with a visited set.

**Checkpoint**
`aps lint test/fixtures/monorepo/plans` now lints 5 files (parent + both
children + their modules), not 1, and resolves `core:AUTH-001` (no W003).

**Validate**
`./bin/aps lint test/fixtures/monorepo/plans`

**Wave** 1

### Action 2 — Test: federated traversal

**Purpose**
Lock in that pointing lint at the parent root validates the whole tree.

**Produces**

- `test/run.sh` case: lint `fixtures/monorepo/plans`, assert it reports all
  five files and emits no W003 for the cross-tree dep.

**Validate**
`bash test/run.sh`

**Wave** 1

### Action 3 — Prefix-aware W003

**Purpose**
Teach `check_w003_dependencies` the `<name>:<ID>` form: in federated scope,
validate the prefix names a discovered child and the ID exists there; in
isolation (named tree absent), treat it as an intentional external ref and stay
silent so standalone children lint clean.

**Produces**

- `lib/rules/workitem.sh`: parse optional `([a-z0-9][a-z0-9-]*):` prefix; a
  child-name→IDs registry (or reuse `APS_TREE_IDS` for presence + a child map);
  distinct message for a genuine cross-tree miss vs the bare-ID "not found".

**Checkpoint**
Federated lint resolves `core:AUTH-001`; `api` linted alone is clean; a
deliberately bad ref (`core:AUTH-999` or `ghost:X-001`) warns.

**Validate**
`./bin/aps lint test/fixtures/monorepo/packages/api/plans` (clean) and a
temp bad-ref fixture (warns).

**Wave** 2

### Action 4 — Tests: prefix-aware W003

**Produces**

- `test/run.sh` cases: isolated child clean; bad cross-tree ref warns.

**Validate**
`bash test/run.sh`

**Wave** 2

### Action 5 — Cross-tree ID-collision detection

**Purpose**
Per D-002, IDs are bare per tree and may collide across trees; surface
collisions as a warning (not an error — each tree is independently valid).

**Produces**

- `lib/lint.sh`: during traversal, map each work-item ID to the tree(s) that
  define it; emit a new warning (next free `W0xx`) when an ID is defined in
  more than one tree. Register the code in the rules reference/docs.

**Checkpoint**
A two-tree fixture sharing an ID reports the collision warning exactly once.

**Wave** 3

### Action 6 — Collision fixture + test

**Produces**

- `test/fixtures/monorepo-collision/` (or extend a copy) with the same ID in
  two trees; `test/run.sh` case asserting the collision warning fires and that
  the clean `test/fixtures/monorepo/` does **not** trip it.

**Validate**
`bash test/run.sh`

**Wave** 3

### Action 7 — PowerShell parity

**Purpose**
Keep `lib/*.psm1` in step with the bash linter (traversal, prefix-aware W003,
collision check).

**Produces**

- Mirrored changes in `lib/Lint.psm1` and `lib/rules/WorkItem.psm1`.

**Wave** 4

### Action 8 — Docs + closeout

**Produces**

- Design doc + `docs/` rules reference updated with the new warning code and
  traversal behaviour
- `monorepo.aps.md`: MONO-002 → Complete with Results; promote MONO-003 to
  Ready (deps MONO-001/002 met)
- `./bin/aps lint` clean; `bash test/run.sh` green

**Wave** 4

## Completion

- [x] All checkpoints validated
- [x] `bash test/run.sh` green (tests 42–46, including PS parity guard)
- [x] PowerShell parity ported and verified under PowerShell 7.4.6
- [x] MONO-002 marked complete; MONO-003 promoted to Ready

**Completed by:** @aneki / AI — 2026-06-30
