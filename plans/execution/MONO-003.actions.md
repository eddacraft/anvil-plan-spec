# Action Plan: MONO-003

| Field      | Value                                                   |
| ---------- | ------------------------------------------------------- |
| Source     | [../modules/monorepo.aps.md](../modules/monorepo.aps.md) |
| Work Item  | MONO-003 — Orchestration across nested plans            |
| Created by | @aneki / AI                                             |
| Status     | Complete                                                |

## Goal

Make APS orchestration commands operate across a federated nested-plan tree.
From a parent root, `aps next`, `aps start`, `aps complete`, `aps graph`, and
`aps audit` should see child plans discovered via `## Child Plans`; scoped
invocations should operate within one child plan.

## Grounding

- MONO-001 defines the nested-plan convention, child-link grammar, and
  `<name>:<ID>` cross-tree reference syntax.
- MONO-002 makes `aps lint` expand child plans transitively, resolve
  cross-tree dependencies, and warn on cross-tree ID collisions.
- ORCH commands already support a single plan root; MONO-003 extends discovery
  and scoping without changing the markdown format.

## Scope Boundary

In scope: command discovery, dependency resolution, scoped execution, tests,
and docs for nested plans.

Out of scope: generated root roll-up content (MONO-004), init/scaffold support
(MONO-005), and the full worked example/docs pass (MONO-006).

## Waves

| Wave | Actions | Gate |
| ---- | ------- | ---- |
| 1    | 1, 2    | `aps next` traverses child plans and supports child scope |
| 2    | 3, 4    | Mutating commands target the owning child file safely |
| 3    | 5, 6    | `graph` and `audit` traverse/scoped behavior is covered |
| 4    | 7, 8    | Docs, parity checks, and plan closeout are complete |

## Actions

### Action 1 — Share nested-plan expansion with orchestration

**Purpose**
Reuse MONO-002's child-plan traversal semantics for orchestration so commands
pointed at a federated parent operate on the whole tree.

**Produces**

- Shared shell helpers, or equivalent orchestration-local helpers, that expand a
  parent plan root by following `## Child Plans` links transitively.
- `aps next --plans test/fixtures/monorepo/plans` sees parent and child work
  items in one candidate set.

**Checkpoint**
Federated `aps next` considers work items from linked child plans.

**Validate**
`./bin/aps next --plans test/fixtures/monorepo/plans`

**Wave** 1

### Action 2 — Add federated `next` tests and child scope

**Purpose**
Lock in the expected queue behavior for parent-wide and child-scoped
orchestration.

**Produces**

- Test coverage showing parent-root `next` returns the correct item across
  trees.
- Test coverage for scoped invocation returning only a named child's items.
- Documented scope syntax chosen for this command surface.

**Checkpoint**
The fixture can prove both federated and scoped `next` behavior.

**Validate**
`./test/run.sh`

**Wave** 1

### Action 3 — Make mutating commands locate the owning child file

**Purpose**
Ensure `aps start <ID>` and `aps complete <ID>` update the work item in its
actual child module file, not the parent root or a duplicate ID in another tree.

**Produces**

- Owner-resolution logic for nested trees.
- Clear behavior for ambiguous bare IDs across children, consistent with
  MONO-002's W020 warning: require scope or a path-qualified reference when the
  target is ambiguous.

**Checkpoint**
Starting or completing a child work item modifies only the expected child module
file.

**Validate**
Run `start`/`complete` against a disposable copy of the monorepo fixture and
inspect the changed file set.

**Wave** 2

### Action 4 — Add mutation safety tests

**Purpose**
Prevent accidental parent-tree edits, wrong-child edits, and ambiguous-ID
updates.

**Produces**

- Fixture tests for child item start/complete.
- Fixture test for ambiguous duplicate IDs requiring disambiguation.
- Fixture test showing scoped mutation succeeds.

**Checkpoint**
Mutation tests fail before the implementation and pass after it.

**Validate**
`./test/run.sh`

**Wave** 2

### Action 5 — Extend `graph` and `audit` traversal

**Purpose**
Keep non-mutating orchestration views consistent with `next` and lint.

**Produces**

- `aps graph` renders dependencies across child plans, including
  `<name>:<ID>` references.
- `aps audit` follows child plans from a parent root and supports child scoping.

**Checkpoint**
Graph and audit outputs include child-plan work items when invoked at the
parent root.

**Validate**

```sh
./bin/aps graph --plans test/fixtures/monorepo/plans
./bin/aps audit --no-run --plans test/fixtures/monorepo/plans
```

**Wave** 3

### Action 6 — Add graph/audit tests

**Purpose**
Guard nested traversal across every ORCH command named in MONO-003.

**Produces**

- Tests asserting graph output includes cross-tree edges.
- Tests asserting audit traverses all child plans from the parent and can be
  scoped to one child.

**Checkpoint**
Graph/audit nested behavior is covered by the suite.

**Validate**
`./test/run.sh`

**Wave** 3

### Action 7 — Document command behavior

**Purpose**
Make nested orchestration predictable for users and agents.

**Produces**

- `docs/usage.md` or `docs/monorepo.md` updated with nested-plan command
  behavior, scope syntax, ambiguity handling, and examples.
- Design doc updated if implementation semantics differ from MONO-001/002.

**Checkpoint**
Docs explain parent-wide and child-scoped orchestration without requiring users
to read the implementation.

**Validate**
`npx markdownlint-cli "docs/**/*.md" "plans/**/*.md"`

**Wave** 4

### Action 8 — Reconcile plan state and close out

**Purpose**
Keep APS dogfood state accurate after implementation.

**Produces**

- `plans/modules/monorepo.aps.md` updated with MONO-003 results.
- MONO-004/MONO-005 readiness checked; no status changed unless warranted by
  dependency state.
- Validation commands recorded in the work item results.

**Checkpoint**
Plans reflect the shipped behavior and lint cleanly.

**Validate**

```sh
./bin/aps lint plans
./test/run.sh
npx markdownlint-cli "**/*.md"
```

**Wave** 4

## Completion

- [x] All checkpoints validated
- [x] Parent-wide and child-scoped orchestration commands tested
- [x] Work item marked complete in `monorepo.aps.md`
