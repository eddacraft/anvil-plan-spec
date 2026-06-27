# Action Plan: MONO-001

| Field      | Value                                                   |
| ---------- | ------------------------------------------------------- |
| Source     | [./modules/monorepo.aps.md](../modules/monorepo.aps.md) |
| Work Item  | MONO-001 — Define the nested plan convention            |
| Created by | @aneki / AI                                             |
| Status     | Complete                                                |

## Decisions in force

This work item depends on D-001..D-003 (all resolved 2026-06-26):

- **D-001 (co-located):** child plans live at `packages/<pkg>/plans/index.aps.md`;
  discovery is by directory layout — any `**/plans/index.aps.md` below the root
  is a child plan.
- **D-002 (bare IDs, qualified cross-tree refs):** IDs stay unprefixed within a
  tree; cross-tree references use the path-derived child name (`core:AUTH-001`).
- **D-003 (standalone children):** each child is a complete APS plan (own
  Problem / Success Criteria / Modules) that lints in isolation.

MONO-001 only defines the convention, the templates, and a lintable fixture
tree. Lint *traversal* (MONO-002), orchestration (MONO-003), and the roll-up
(MONO-004) build on it and are out of scope here.

## Prerequisites

- [x] D-001..D-003 resolved (see above)
- [x] VAL parser exists and lints a single plan root (`./bin/aps lint <dir>`)
- [x] Existing templates to model against (`templates/index*.template.md`)

## Scope boundary

In scope: the markdown convention (declaration + linking), the parent and
child templates, and a two-level fixture tree where the root and **each child
independently** lint clean. Out of scope: making one `aps lint` invocation walk
the whole tree (that is MONO-002 — until then each plan root is linted
separately).

## Waves

| Wave | Actions | Gate                                                        | Done |
| ---- | ------- | ----------------------------------------------------------- | ---- |
| 1    | 1, 2    | Convention written down; child-link marker + ref syntax fixed | ✅ 2026-06-27 |
| 2    | 3, 4    | Parent + child templates exist and lint clean standalone    | ✅ 2026-06-27 |
| 3    | 5, 6    | Two-level fixture tree lints clean per-root; MONO-001 closed | ✅ 2026-06-27 |

**Wave 1 output:** authoritative spec at
[../designs/2026-06-27-nested-plans.design.md](../designs/2026-06-27-nested-plans.design.md)
— convention (§1) + machine-readable markers (§2). Non-collision check passed
(zero matches in real plan content) and surfaced a parser refinement for
MONO-002 (skip code spans/fences; scan only dependency/interface field values).

**Wave 2 output:** `templates/index-nested.template.md` (federated root —
`## Child Plans` + `## Roll-up` stub + empty federation-root Modules table) and
`templates/index-child.template.md` (standalone child plan, bare IDs noted).
Both are markdownlint-clean and, instantiated as `index.aps.md`, lint **✓ valid**
under `./bin/aps lint` (templates themselves are `.template.md`, which lint
skips with W000 — structural validity proven via instantiation).

**Wave 3 output:** fixture tree `test/fixtures/monorepo/` (root + `core` + `api`,
`api`'s HND-001 depends on `core:AUTH-001`). Lint matrix:

| Target | Result |
| ------ | ------ |
| `plans` (root alone) | ✓ clean |
| `packages/core/plans` (child alone) | ✓ clean |
| `packages/api/plans` (child alone) | 1× W003 — expected cross-tree dep, MONO-002 |
| whole tree (recursive) | ✓ clean (combined index resolves cross-tree) |
| repo default `./bin/aps lint` | unaffected (test/fixtures excluded) |

MONO-001 marked Complete; MONO-002/004/005 promoted to Ready (sole blocker
cleared). Second finding recorded for MONO-002: W003 must become
`<name>:`-prefix-aware (see design doc Findings).

## Actions

### Action 1 — Specify the nested-plan convention

**Purpose**
Pin down, in prose, exactly how a parent declares children, how children are
discovered without the parent, and how cross-tree references are written —
grounded in D-001..D-003 so MONO-002/003 have an unambiguous target.

**Produces**

- A "Nested Plans" convention section drafted for `docs/monorepo.md` (final
  prose lands in MONO-006; this is the authoritative spec the templates and
  fixtures implement). It must state:
  - **Declaration:** parent `index.aps.md` lists each child under a
    `## Child Plans` section with a relative link to the child
    `index.aps.md` and the child's path-derived name.
  - **Child name derivation:** the name is the package path segment(s) leading
    to `plans/` (e.g. `packages/core/plans/` → `core`,
    `apps/web/plans/` → `web`). This name is the cross-tree reference prefix.
  - **Discovery without parent:** any `**/plans/index.aps.md` at or below the
    invocation root is a plan root; the topmost is the parent.
  - **Cross-tree reference syntax:** `<child-name>:<ID>` (e.g. `core:AUTH-001`)
    in `Dependencies:` / interface fields. Bare IDs always resolve within the
    current tree.

**Checkpoint**
The four points above are written and internally consistent with D-001..D-003.

**Wave** 1

### Action 2 — Fix the machine-readable markers

**Purpose**
Decide the exact tokens MONO-002's parser will key on, so templates and
fixtures are authored against the real grammar (not guessed at later).

**Produces**

- Marker definitions recorded alongside the convention:
  - Child declaration is parsed from links under a heading matching
    `^##\s+Child Plans\s*$` (case-sensitive), each list item of the form
    `- [<child-name>](<relative-path-to-child-index>) — <summary>`.
  - Cross-tree reference regex: `\b([a-z0-9][a-z0-9-]*):([A-Z]+-[0-9]+)\b`.
  - Confirm these do not collide with existing single-plan grammar (conductor
    `Type:` marker, `W003` cross-module deps, completed-index roll-ups).

**Checkpoint**
Grammar documented; a manual grep of the regex over an example child link and a
cross-tree dep matches as intended and does not match existing intra-tree IDs.

**Validate**
`grep -nE '\b([a-z0-9][a-z0-9-]*):([A-Z]+-[0-9]+)\b' plans/**/*.aps.md` returns
nothing (no false positives against the current single-tree repo).

**Wave** 1

### Action 3 — Author the parent (root) index template

**Purpose**
Give users a starting point for a federated root that links children and
carries nothing package-specific itself.

**Produces**

- `templates/index-nested.template.md` — a root `index.aps.md` with:
  - standard Problem / Success Criteria / Constraints
  - a `## Child Plans` section using the Action 2 link form
  - a placeholder `## Roll-up` section (populated by MONO-004; present as a
    stub here so the structure is stable)
  - a comment pointing to `docs/monorepo.md` "Nested Plans"

**Checkpoint**
`./bin/aps lint templates/index-nested.template.md` (or the repo's template lint
target) treats it as a valid plan root.

**Validate**
`pnpm exec markdownlint templates/index-nested.template.md`

**Wave** 2

### Action 4 — Author the child index template

**Purpose**
Give each package a drop-in standalone plan that satisfies D-003.

**Produces**

- `templates/index-child.template.md` — a complete standalone plan (own
  Problem / Success Criteria / Modules table) that lints in isolation, with a
  comment noting it may be referenced from a parent's `## Child Plans` and that
  its IDs are bare within this tree.

**Checkpoint**
Linting the child template **alone** passes; it has no dependency on a parent.

**Validate**
`pnpm exec markdownlint templates/index-child.template.md`

**Wave** 2

### Action 5 — Build the two-level fixture tree

**Purpose**
Provide the canonical example the rest of MONO-00x tests against, and prove the
convention round-trips through real files.

**Produces**

- `test/fixtures/monorepo/plans/index.aps.md` — parent linking two children
- `test/fixtures/monorepo/packages/core/plans/index.aps.md` + `modules/` — child `core`
- `test/fixtures/monorepo/packages/api/plans/index.aps.md` + `modules/` — child `api`
- The `api` child includes one cross-tree dependency on `core:<ID>` to exercise
  the reference syntax (resolution is MONO-002's job; here it must merely be
  well-formed and lint clean as text).

**Checkpoint**
Each plan root lints clean **independently** (per the MONO-001 scope boundary):

**Validate**

```sh
./bin/aps lint test/fixtures/monorepo/plans
./bin/aps lint test/fixtures/monorepo/packages/core/plans
./bin/aps lint test/fixtures/monorepo/packages/api/plans
```

All three pass. (A single whole-tree `aps lint` invocation is MONO-002.)

**Wave** 3

### Action 6 — Mark MONO-001 Complete

**Purpose**
Close the loop in APS itself and unblock MONO-002/004/005 (which depend only on
MONO-001).

**Produces**

- `plans/modules/monorepo.aps.md` updated:
  - MONO-001 status → Complete with date and a Results note (convention,
    templates, fixture paths)
  - MONO-002, MONO-004, MONO-005 status → Ready (their sole blocker was
    MONO-001)
  - `Last reviewed:` bumped
- `./bin/aps lint` clean across the repo

**Checkpoint**
Module spec reflects completion; lint clean.

**Validate**
`./bin/aps lint`

**Wave** 3

## Completion

- [x] All checkpoints validated
- [x] Fixture tree lints clean per-root (parent + `core` clean; `api`-alone has
      the one expected cross-tree W003; whole-tree clean)
- [x] Work item marked complete in `monorepo.aps.md`
- [x] Downstream items (MONO-002/004/005) promoted to Ready

**Completed by:** @aneki / AI — 2026-06-27
