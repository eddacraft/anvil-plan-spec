# Monorepo Module

| ID   | Owner  | Priority | Status |
| ---- | ------ | -------- | ------ |
| MONO | @aneki | medium   | Draft  |

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

- [ ] Purpose and scope are clear
- [ ] D-001 through D-004 resolved
- [ ] Work items confirmed against resolved decisions

## Work Items

### MONO-001: Define the nested plan convention

- **Intent:** Establish how child `index.aps.md` files are declared,
  discovered, and linked from a parent plan
- **Expected Outcome:** Convention documented; parent and child index
  templates exist; an example nested tree lints clean as fixtures
- **Validation:** `./bin/aps lint test/fixtures/monorepo/plans` passes on a
  two-level fixture tree
- **Confidence:** medium
- **Dependencies:** D-001, D-002, D-003
- **Status:** Draft

### MONO-002: Lint traversal of nested plans

- **Intent:** Make `aps lint` validate a federated tree as one plan
- **Expected Outcome:** Lint follows child index links from the root,
  validates each child as a full plan, resolves cross-tree dependencies
  (W003), and flags work-item ID collisions between trees
- **Validation:** Fixture with cross-tree deps and a deliberate ID collision
  produces the expected results
- **Confidence:** medium
- **Dependencies:** MONO-001
- **Status:** Draft

### MONO-003: Orchestration across nested plans

- **Intent:** Let agents navigate a federated plan as one queue
- **Expected Outcome:** `aps next`/`start`/`complete`/`graph`/`audit` operate
  across all child plans from the root, and accept a scope argument to work
  within one child
- **Validation:** `aps next` from the fixture root returns the correct item
  across trees; scoped invocation returns only the child's items
- **Confidence:** medium
- **Dependencies:** MONO-001, MONO-002
- **Status:** Draft

### MONO-004: Root roll-up view

- **Intent:** Answer "where is everything?" from the root index
- **Expected Outcome:** Parent index shows each child plan with aggregated
  status (modules complete/total, next ready item); convention documented
  for keeping it current at session end
- **Validation:** Roll-up section in the fixture root lints clean and matches
  the child plan states
- **Confidence:** low
- **Dependencies:** MONO-001
- **Status:** Draft

### MONO-005: Scaffold support for nested layouts

- **Intent:** Make the nested layout reachable from `aps init`
- **Expected Outcome:** Init offers a nested/monorepo option that creates a
  root plan plus at least one child plan wired to it
- **Validation:** Test-suite init run produces a tree that `aps lint` accepts
- **Confidence:** medium
- **Dependencies:** MONO-001
- **Status:** Draft

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

- **D-001:** Child plan location — _open._ Candidates: co-located with the
  package (`packages/<pkg>/plans/index.aps.md`, most portable) vs centralised
  (`plans/<pkg>/index.aps.md`, simplest discovery). Co-location is the
  stronger candidate because extraction-portability is the motivating case.
- **D-002:** ID namespacing across trees — _open._ Module IDs are only unique
  per tree today; this repo already has a D-026 collision between its index
  and tui module. Options: required per-tree prefix, path-qualified IDs
  (`core:AUTH-001`) for cross-tree references, or collision detection only.
- **D-003:** Child autonomy — _open._ Children as fully standalone plans
  (own Problem/Success Criteria, work without the parent) vs subordinate
  views of one root plan. Standalone is the stronger candidate — it keeps
  every plan portable and lets `--plans` point anywhere.
- **D-004:** Relationship to tagged modules — _open (leaning coexist)._
  Package tags remain the default for small monorepos; nested indexes are
  the opt-in tier when packages need independent plans. Docs must carry the
  decision guidance.

## Relationship to Other Modules

| Module       | Relationship                                                       |
| ------------ | ------------------------------------------------------------------ |
| **VAL**      | Lint learns federated traversal (extends DOGFOOD-002's tree index) |
| **ORCH**     | Navigation commands learn nested roots                             |
| **SCAFFOLD** | Init gains a nested layout option                                  |
| **TPL**      | Parent/child index templates                                       |
| **SPEC**     | ID namespacing (D-002) may need vocabulary/schema support          |
