# Conductor (Crosscutting) Modules

Most APS modules are **vertical slices**: a module owns a single domain —
auth, ingestion, billing — and its work items end-to-end. But some concerns
are **horizontal**: they span, coordinate, or sequence work _across_ several
domains. Release cuts, security audits, performance budgets, migration waves,
observability rollouts. Forcing one of these into a vertical module either
duplicates work items across domains or grows a sprawling "misc" module.

A **conductor module** (also called _crosscutting_) models that horizontal
concern as its own first-class artifact. The conductor is the score; the
vertical modules are the players.

> The name echoes the ORCH-005 _Conductor agent_ — an agent role that
> coordinates work across modules. A conductor module is the same idea
> promoted from "an agent role" to "a module type."

## Conductor vs vertical: which do I reach for?

| Aspect           | Vertical (the default)              | Conductor / Crosscutting          |
| ---------------- | ----------------------------------- | --------------------------------- |
| Boundary         | A **domain** (auth, billing)        | A **concern** (release, audit)    |
| Owns work items? | Yes — all of them                   | Optional — may reference others   |
| Dependencies     | On other modules                    | On work items _across_ modules    |
| Status means     | Feature complete                    | Concern addressed in this slice   |
| Lifecycle        | Draft → Ready → Complete → Archived | Often recurring or rolling        |

Use a **vertical** module when the work has a natural home in one domain —
even if it touches shared infrastructure. A module is _not_ a conductor just
because it is _about_ a crosscutting topic: the `conductor` module that
introduced this very type is a normal vertical feature module — it owns
COND-001…006 end-to-end and coordinates nothing.

Use a **conductor** module when the work's job _is_ the coordination: it
references and sequences work items that other modules own, and "done" means
the concern is addressed for this slice, not that a feature is built.

Quick test: **if you removed every cross-module reference, would the module
still have a reason to exist?** If yes, it is vertical. If what's left is an
empty shell, it is a conductor.

## Anatomy of a conductor module

Start from [`templates/conductor.template.md`](../templates/conductor.template.md).
Two things distinguish it from a vertical module:

1. **The `Type` column.** The metadata table carries `Type: Conductor`
   (placed after `ID`). Vertical modules omit the column entirely.

   ```markdown
   | ID  | Type      | Owner  | Priority | Status   |
   | --- | --------- | ------ | -------- | -------- |
   | REL | Conductor | @aneki | medium   | Complete |
   ```

2. **Coordination sections.** Two optional sections name the work this
   conductor pulls together:

   - `## Coordinated Modules` — the modules it spans, and each one's role in
     the concern.
   - `## Cross-Module Work Items` — work items owned by _other_ modules,
     referenced by their existing ID. The owning module stays the source of
     truth; the conductor only tracks the roll-up.

   A `## Status Roll-up` then gives the headline: how much of the coordinated
   concern is done, as a count across modules.

A conductor **may still own its own work items** — the coordinating work
("extract the template", "add the lint rule") lives in a normal `## Work
Items` section alongside the references to others.

## Worked example: release planning

[`plans/modules/release-planning.aps.md`](../plans/modules/release-planning.aps.md)
is the canonical conductor in this repo. A release is not a domain — it is a
slice of completed work from many modules bundled into a versioned artifact.
That is a concern, not a vertical, so it carries `Type: Conductor`.

It shows the hybrid shape in practice:

- **It owns coordinating work items** — REL-001…005 (extract the template,
  scaffold the directory, add lint rules, write the docs, the optional CLI).
- **It references work owned elsewhere** — its work items depend on
  `ORCH-001` (parser reuse) and build on `COMPOUND-003`, IDs that live in
  other module files. Those references resolve against the plan tree, so the
  linter does not flag them.
- **Its lifecycle is rolling** — one release plan per version under
  `plans/releases/`. The module describes the recurring _practice_; each
  `plans/releases/vX.Y.Z.md` is one turn of it.

See the [Release Planning guide](release-planning.md) for the full release
workflow this conductor governs.

## Recurring vs one-shot lifecycle

Vertical modules terminate: they reach `Complete` and are archived. Many
conductors don't — a performance budget or a security posture is never
"done", it is maintained. For those, use the **`Recurring`** status in the
metadata table instead of `Complete`:

| Shape       | Example                          | Terminal status        |
| ----------- | -------------------------------- | ---------------------- |
| One-shot    | A single migration wave          | `Complete`             |
| Recurring   | Perf budget, security posture    | `Recurring` (no end)   |
| Rolling     | Release planning (one per cut)   | Module recurs; each cut completes |

A conductor that recurs should say so in its `## Notes` so readers don't wait
for a completion that will never come.

## How the linter helps

`aps lint` is conductor-aware (see the [lint code reference](usage.md#aps-lint)):

- **W002** — inside a conductor, a work-item ID referenced in `## Coordinated
  Modules` or `## Cross-Module Work Items` that resolves _nowhere_ in the plan
  tree is flagged as a likely typo. Legitimate cross-module references are not
  warned; only unresolved ones.
- **W006** — a module listed under a `### Conductor / Crosscutting` subsection
  of the index whose file is not marked `Type: Conductor`. This keeps the
  index's conductor grouping honest.

## Placing conductors in the index

Group conductor modules under a dedicated `### Conductor / Crosscutting`
subsection of `## Modules` in `index.aps.md`, so the type is visible at a
glance. List only genuine `Type: Conductor` modules there — the feature
module that happens to be _about_ conductors belongs with the other feature
modules. `aps init` seeds this subsection (optional) in the index template.

## Creating one

```sh
cp templates/conductor.template.md plans/modules/<concern>.aps.md
```

Fill in the metadata table with `Type: Conductor`, name the modules you
coordinate, reference the cross-module work items by ID, and add a status
roll-up. Run `aps lint` to confirm every reference resolves.

> When in doubt, prefer a vertical module. The conductor type earns its keep
> only when coordination across modules _is_ the work. If running a concern
> through this lens feels forced, it probably wants to be a regular module
> that happens to reference others.
