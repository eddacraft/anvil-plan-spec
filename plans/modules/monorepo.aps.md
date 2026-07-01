# Monorepo Module

| ID   | Owner  | Priority | Status |
| ---- | ------ | -------- | ------ |
| MONO | @aneki | medium   | In Progress |

**Last reviewed:** 2026-06-27

## Purpose

Support monorepos whose packages need their own plans: multiple nested
`index.aps.md` files that remain individually valid APS plans while rolling
up into one federated view at the repo root.

## Background

Current monorepo support ([docs/monorepo.md](../../docs/monorepo.md),
[index-monorepo.template.md](../../templates/index-monorepo.template.md))
keeps a single `plans/` root and adds package tags plus "What's Next" /
"By Package" views. That works while one team shares one backlog, but breaks
down when packages have independent owners, lifecycles, or scale: every
module competes for one namespace, the index grows unboundedly, and a package
cannot carry its plan with it if extracted to its own repo.

Nested indexes are the heavier complement: each package (or app) gets its own
`index.aps.md` + `modules/`, and the root index links the children instead of
owning their modules. Tagged modules stay the recommended default for small
monorepos — this module covers the federated tier above it.

## In Scope

- A convention for declaring child plans from a parent `index.aps.md` (and
  discovering them without the parent, e.g. by directory layout)
- Child plans that are complete, standalone APS plans — lintable and
  orchestratable in isolation, portable if a package is extracted
- `aps lint` traversal across nested plan trees, including cross-tree
  dependency resolution and ID-collision detection between trees
- `aps next` / `start` / `complete` / `graph` / `audit` operating across
  nested roots, with scoping to a single child plan
- Root-level roll-up: aggregated status of child plans visible from the
  parent index
- Scaffold/init support for creating a nested layout
- Documentation and a worked example

## Out of Scope

- Replacing the tagged-modules approach (it remains the default; see D-004)
- Cross-repo federation (children live in the same repository)
- Generated/derived indexes — the root index stays hand-authored markdown
- Tool-specific workspace integrations (pnpm/nx/turbo awareness)

## Interfaces

**Depends on:**

- VAL (validation) — lint must learn to traverse child plan roots
- ORCH (orchestrate) — navigation commands must traverse child plan roots
- SCAFFOLD/INSTALL — init needs a nested layout option
- TPL (templates) — parent + child index templates

**Exposes:**

- Nested plan convention (documented in docs/monorepo.md)
- `--plans` semantics extended to federated trees
- Roll-up view in the parent index

## Ready Checklist

- [x] Purpose and scope are clear
- [x] D-001 through D-004 resolved
- [x] Work items confirmed against resolved decisions

## Work Items

### MONO-001: Define the nested plan convention — Complete 2026-06-27

- **Intent:** Establish how child `index.aps.md` files are declared,
  discovered, and linked from a parent plan
- **Expected Outcome:** Convention documented; parent and child index
  templates exist; an example nested tree lints clean as fixtures
- **Validation:** `./bin/aps lint test/fixtures/monorepo/plans` passes on a
  two-level fixture tree
- **Confidence:** medium
- **Dependencies:** D-001, D-002, D-003 (all resolved)
- **Status:** Complete
- **Results:** Convention + machine-readable grammar in
  [../designs/2026-06-27-nested-plans.design.md](../designs/2026-06-27-nested-plans.design.md)
  (declaration via `## Child Plans`, path-derived child names, directory-layout
  discovery, `<name>:<ID>` cross-tree refs). Templates
  `templates/index-nested.template.md` (federated root) and
  `templates/index-child.template.md` (standalone child) lint valid when
  instantiated. Fixture `test/fixtures/monorepo/` (root + `core` + `api` with a
  `core:AUTH-001` cross-tree dep): parent, `core`, and whole-tree lint clean;
  `api` linted alone shows the one expected cross-tree W003. Two findings handed
  to MONO-002 (recursive find already resolves whole-tree deps; W003 must become
  `<name>:`-prefix-aware) — see the design doc's Findings section.

### MONO-002: Lint traversal of nested plans — Complete 2026-06-30

- **Intent:** Make `aps lint` validate a federated tree as one plan
- **Expected Outcome:** Lint follows child index links from the root,
  validates each child as a full plan, resolves cross-tree dependencies
  (W003), and flags work-item ID collisions between trees
- **Validation:** Fixture with cross-tree deps and a deliberate ID collision
  produces the expected results
- **Confidence:** medium
- **Dependencies:** MONO-001 (complete)
- **Status:** Complete
- **Action plan:** [../execution/MONO-002.actions.md](../execution/MONO-002.actions.md)
- **Results:** `cmd_lint` follows a parent index's `## Child Plans` links and
  pulls each child tree into the lint set transitively (`normalize_path`,
  `resolve_child_plan_links`, `expand_child_plans` in `lib/lint.sh`), so a
  federated root lints the whole tree as one plan. W003
  (`lib/rules/workitem.sh`) is now `<name>:<ID>`-aware: cross-tree refs resolve
  against an in-scope child registry (`build_child_registry`/`APS_CHILD_IDS`),
  stay silent when the named child is absent (standalone children lint clean),
  and warn on a genuine miss. New **W020** (`check_cross_tree_collisions`)
  warns when one work-item ID is defined in more than one child tree (per
  D-002, a warning — each tree stays independently valid). Fixture
  `test/fixtures/monorepo/` (root + `core` + `api`, `core:AUTH-001` cross-tree
  dep) lints clean as a federation; `api` linted alone is clean; bad-ref and
  collision scenarios warn — `test/run.sh` tests 42–46. **PowerShell parity**
  delivered: `lib/Lint.psm1` + `lib/rules/WorkItem.psm1` mirror traversal,
  prefix-aware W003, and W020; verified behaviourally against all four
  scenarios under PowerShell 7.4.6 (output identical to bash), and guarded in
  CI by a string-parity check (test 46). W020 documented in `docs/usage.md`
  and the [design doc](../designs/2026-06-27-nested-plans.design.md).

### MONO-003: Orchestration across nested plans

- **Intent:** Let agents navigate a federated plan as one queue
- **Expected Outcome:** `aps next`/`start`/`complete`/`graph`/`audit` operate
  across all child plans from the root, and accept a scope argument to work
  within one child
- **Validation:** `aps next` from the fixture root returns the correct item
  across trees; scoped invocation returns only the child's items
- **Confidence:** medium
- **Dependencies:** MONO-001 (complete), MONO-002 (complete)
- **Status:** Ready
- **Action plan:** [../execution/MONO-003.actions.md](../execution/MONO-003.actions.md)

### MONO-004: Root roll-up view

- **Intent:** Answer "where is everything?" from the root index
- **Expected Outcome:** Parent index shows each child plan with aggregated
  status (modules complete/total, next ready item); convention documented
  for keeping it current at session end
- **Validation:** Roll-up section in the fixture root lints clean and matches
  the child plan states
- **Confidence:** low
- **Dependencies:** MONO-001 (complete)
- **Status:** Ready
- **Notes:** The `index-nested` template and fixture root already carry a
  `## Roll-up` stub table — this item populates it and documents the refresh
  ritual.

### MONO-005: Scaffold support for nested layouts

- **Intent:** Make the nested layout reachable from `aps init`
- **Expected Outcome:** Init offers a nested/monorepo option that creates a
  root plan plus at least one child plan wired to it
- **Validation:** Test-suite init run produces a tree that `aps lint` accepts
- **Confidence:** medium
- **Dependencies:** MONO-001 (complete)
- **Status:** Ready
- **Notes:** Scaffold from `templates/index-nested.template.md` +
  `templates/index-child.template.md`; the `test/fixtures/monorepo/` layout is
  the target shape.

### MONO-006: Documentation and worked example

- **Intent:** Teach when to use tags vs nested indexes, and how to migrate
- **Expected Outcome:** docs/monorepo.md gains a "Nested Plans" tier with
  decision guidance (tags vs nested) and a migration path; a worked example
  lands under examples/
- **Validation:** `pnpm exec markdownlint docs/monorepo.md examples/` passes;
  example tree lints clean
- **Confidence:** high
- **Dependencies:** MONO-001 through MONO-004
- **Status:** Draft

## Decisions

- **D-001:** Child plan location — _decided 2026-06-26: co-located._ Child
  plans live with their package at `packages/<pkg>/plans/index.aps.md` (a
  child carries its plan when extracted to its own repo, which is the
  motivating case). Discovery without the parent is by directory layout:
  `aps` treats any `**/plans/index.aps.md` below the root as a child plan.
  Centralised `plans/<pkg>/` is not blessed — co-location is the single
  convention so discovery stays unambiguous.
- **D-002:** ID namespacing across trees — _decided 2026-06-26: bare IDs
  per tree, path-qualified for cross-tree references, collision detection
  always on._ Module/work-item IDs stay unprefixed within a tree (a child
  extracted to its own repo carries no redundant prefix — consistent with
  D-001/D-003). Cross-tree references qualify with the child's path-derived
  name (`core:AUTH-001`). `aps lint` flags duplicate IDs **within** a tree
  (existing behaviour) and reports cross-tree collisions as a warning rather
  than an error, since each tree is independently valid. A required global
  prefix was rejected — it taxes the common standalone case to serve the
  rarer cross-tree reference.
- **D-003:** Child autonomy — _decided 2026-06-26: standalone._ Each child
  is a complete APS plan with its own Problem / Success Criteria / Modules
  and lints, orchestrates, and ships in isolation. The parent links children
  and rolls up their status; it does not own their modules. This keeps every
  plan portable and lets `--plans` point at any node in the tree.
- **D-004:** Relationship to tagged modules — _decided 2026-06-26: coexist,
  tags remain the default._ Package tags (docs/monorepo.md) stay the
  recommended approach for monorepos that share one backlog. Nested indexes
  are the opt-in federated tier for packages with independent owners,
  lifecycles, or extraction plans. Docs carry the tags-vs-nested decision
  guidance and a migration path (MONO-006).

## Relationship to Other Modules

| Module       | Relationship                                                       |
| ------------ | ------------------------------------------------------------------ |
| **VAL**      | Lint learns federated traversal (extends DOGFOOD-002's tree index) |
| **ORCH**     | Navigation commands learn nested roots                             |
| **SCAFFOLD** | Init gains a nested layout option                                  |
| **TPL**      | Parent/child index templates                                       |
| **SPEC**     | ID namespacing (D-002) may need vocabulary/schema support          |
