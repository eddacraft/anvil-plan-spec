# APS in Monorepos

This guide explains how to use APS effectively in monorepos with multiple packages and applications.

## The Problem

Standard APS assumes a single `plans/` directory for one project. In a monorepo with structure like:

```text
monorepo/
├── apps/
│   ├── api/
│   ├── web/
│   └── cli/
├── packages/
│   ├── core/
│   ├── shared/
│   └── ui/
└── plans/           # Single plans/ covering everything
```

You'll hit these pain points:

1. **Scope ambiguity** — "Does AUTH-001 affect `api`, `web`, both, or the `core` package?"
2. **Navigation difficulty** — "I know what I'm looking for but can't find which module it's in"
3. **Cross-cutting concerns** — Work spans multiple packages and doesn't fit cleanly in one module
4. **Dependency confusion** — Hard to track inter-module dependencies across packages
5. **Prioritization gaps** — No clear "what's next" view across the whole monorepo

## The Solution: Tagged Modules + Views

Keep a single `plans/` at the monorepo root. Add:

- **Explicit package tags** on modules and work items
- **"What's Next" view** in the index for prioritized work queue
- **"By Package" view** in the index for navigation
- **Session rituals** to keep docs in sync

## Index Structure for Monorepos

Use the [index-monorepo.template.md](../templates/index-monorepo.template.md) template. Key additions:

### What's Next Section

A prioritized queue across all packages:

```markdown
## What's Next

| #   | Work Item | Module     | Packages    | Owner | Status |
| --- | --------- | ---------- | ----------- | ----- | ------ |
| 1   | AUTH-002  | auth       | core, api   | @josh | Ready  |
| 2   | CLI-001   | cli        | cli, shared | @josh | Ready  |
| 3   | UI-003    | components | web, ui     | —     | Ready  |
```

This answers "what should I work on next?" without digging through modules.

### Modules by Package Section

Navigation view grouped by package:

```markdown
## Modules by Package

### apps/api

- [auth](./modules/01-auth.aps.md) — AUTH-002 ready

### apps/web

- [components](./modules/03-components.aps.md) — UI-003 ready

### packages/core

- [auth](./modules/01-auth.aps.md) — AUTH-002 ready
- [data](./modules/04-data.aps.md) — no ready items

### packages/shared

- [cli](./modules/02-cli.aps.md) — CLI-001 ready
```

Note: Modules can appear under multiple packages. This grouping is a _view_, not ownership—ownership lives in the module metadata.

### Cross-Cutting Section

For modules that span multiple packages:

```markdown
## Cross-Cutting Concerns

- [auth](./modules/01-auth.aps.md) — spans core + api + web
```

## Module Metadata

Add a `Packages` column to module metadata:

```markdown
# Auth Module

| ID   | Owner | Priority | Status | Packages  |
| ---- | ----- | -------- | ------ | --------- |
| AUTH | @josh | high     | Ready  | core, api |
```

Work items can inherit or narrow the package scope:

```markdown
### AUTH-002: Token refresh

| Status | Packages  | Dependencies |
| ------ | --------- | ------------ |
| Ready  | core, api | AUTH-001     |

- **Intent:** Implement refresh token rotation
- **Expected Outcome:** Refresh endpoint returns new token pair
- **Validation:** `npm test -w packages/core -- auth.test.ts`
```

Key points:

- `Packages` at module level = which packages this module touches
- `Packages` per work item = can be subset of module's packages
- Validation commands use workspace flags (`-w packages/core`, `--filter=@myorg/core`)

## Driving Monorepo Work with the CLI

The orchestration commands work the same in a monorepo as anywhere else:

```bash
aps next                              # next ready item across all modules and packages
aps next core                         # filter to the `core` module
aps start AUTH-002                    # start an item; context package includes module scope
aps complete AUTH-002 --learning "..."
aps graph                             # see the full dependency graph
```

`aps next` honours module-level dependencies declared in `index.aps.md`, so a
work item in `cli` whose module depends on `core` won't be returned as ready
until `core` is done.

For multi-package work that affects more than one module, surface the affected
packages in the work item's metadata table (see Module Metadata above) — the
context package carries this through to whichever agent picks it up.

## Session Rituals

These rituals are common to every APS workflow, not just monorepos — they
live in [docs/workflow.md](workflow.md). The monorepo-specific notes:

- Always declare which package(s) you're touching when you `aps start` an item
  (the context package surfaces this).
- Capture cross-package discoveries as Draft work items in the appropriate
  module, tagged with packages affected.

## Nested Plans (Federated Tier)

Tagged modules keep **one** `plans/` for the whole repo. That works while the
packages share a backlog. When packages have **independent owners, release
cadences, or extraction plans**, the single index becomes a bottleneck: every
module competes for one namespace, the index grows unboundedly, and a package
can't carry its plan with it if it's spun out into its own repo.

Nested plans are the heavier, federated tier. Each package gets its **own**
`index.aps.md` + `modules/`, and a root index _links_ the children instead of
owning their modules:

```text
monorepo/
├── plans/
│   └── index.aps.md                       # Federation root: ## Child Plans + ## Roll-up
└── packages/
    ├── catalog/
    │   └── plans/
    │       ├── index.aps.md               # Standalone child plan
    │       └── modules/products.aps.md
    └── storefront/
        └── plans/
            ├── index.aps.md               # Standalone child plan
            └── modules/cart.aps.md        # dep: catalog:PROD-001 (cross-tree)
```

The convention (see the
[nested-plans design](../plans/designs/2026-06-27-nested-plans.design.md)):

- **Co-located children (D-001).** Child plans live with their package at
  `packages/<pkg>/plans/index.aps.md`, so a package carries its plan when
  extracted. `aps` discovers any `**/plans/index.aps.md` below the root.
- **Declared from the parent.** The root's `## Child Plans` section links each
  child index. The link text is the child's path-derived name and is the prefix
  for cross-tree references.
- **Bare IDs per tree, path-qualified across trees (D-002).** IDs stay
  unprefixed within a tree (`PROD-001`); reference another tree with
  `<name>:<ID>` (`catalog:PROD-001`). `aps lint` warns (W020) when one ID is
  defined in more than one tree, since that makes cross-tree refs ambiguous.
- **Standalone children (D-003).** Each child is a complete APS plan that lints,
  orchestrates, and ships in isolation. The parent rolls up status; it does not
  own child modules.

### Working a federation with the CLI

Every orchestration command traverses the whole tree from the root, the same
way `aps lint` does, and `--child <name>` scopes to one child:

```bash
aps lint   examples/monorepo-nested/plans              # validate the whole federation
aps next   --plans examples/monorepo-nested/plans      # next ready item across all children
aps next   --child catalog --plans .../plans           # …scoped to one child
aps graph  --plans examples/monorepo-nested/plans      # cross-tree edges render as catalog:PROD-001
aps start  catalog:PROD-001 --plans .../plans          # mutate the owning child file
aps rollup --plans examples/monorepo-nested/plans      # regenerate the root ## Roll-up table
```

Mutations (`start`/`complete`) write the work item in its owning child file; an
ambiguous bare ID is refused until you disambiguate with `<name>:<ID>` or
`--child`. See [usage.md](usage.md#nested-plans-federated-orchestration) for the
full command reference.

### Scaffolding a nested layout

```bash
aps init --scope nested          # bash CLI
aps init --templates index-nested   # Rust CLI
```

Both create a federation root plus starter `core` and `api` child plans that
lint clean as one tree. Keep the root's `## Roll-up` current by pasting the
output of `aps rollup` at session end (the root stays hand-authored).

### Worked example

A complete, lint-clean federation lives at
[examples/monorepo-nested/](../examples/monorepo-nested/): a `catalog` +
`storefront` shop where `storefront`'s cart depends on `catalog:PROD-001` across
trees. Run `aps lint examples/monorepo-nested/plans` to see the whole tree
validate as one plan.

## Tags vs Nested: Which Tier?

Both tiers coexist (D-004); tagged modules remain the default. Choose by how
independent the packages are:

| Signal                                    | Tagged modules | Nested plans |
| ----------------------------------------- | -------------- | ------------ |
| Packages share one backlog and cadence    | ✅ default     | overkill     |
| Independent owners / release cadences      | strained       | ✅           |
| A package may be extracted to its own repo | painful        | ✅ portable  |
| Small repo, informal coordination          | ✅             | overkill     |
| Dozens of packages, unbounded index growth | strained       | ✅           |

Start with tagged modules. Graduate to nested plans only when a package needs to
own its lifecycle.

### Migrating tags → nested

Per-package, incrementally — the tiers coexist during the move:

1. **Create the child plan.** `aps init --scope nested` scaffolds the shape, or
   add `packages/<pkg>/plans/index.aps.md` by hand from
   [index-child.template.md](../templates/index-child.template.md).
2. **Move the package's modules** out of the root `plans/modules/` into the
   child's `modules/`, dropping their `Packages:` tags (the tree location now
   carries that meaning). Keep IDs bare within the child.
3. **Rewrite cross-package dependencies** that now cross trees as `<name>:<ID>`
   (e.g. `PROD-001` consumed from `storefront` becomes `catalog:PROD-001`).
4. **Link the child** from the root's `## Child Plans`, remove the migrated rows
   from the root `## Modules`, and add the child to `## Roll-up`
   (`aps rollup` prints the row).
5. **Verify:** `aps lint plans` should validate the federation clean; resolve
   any W020 ID collisions surfaced across trees.

Leave packages that still share a backlog as tagged modules — you don't have to
migrate everything.

## When to Use Monorepo Structure

Use **tagged-module** monorepo conventions when:

- You have 3+ packages/apps in one repository
- Work regularly spans multiple packages
- Multiple agents or developers work in parallel
- You need clear "what's next" prioritization

Graduate to **nested plans** (above) when packages have independent owners,
release cadences, or extraction plans.

Stick with standard APS when:

- Single package repository
- Work rarely crosses package boundaries
- Small team, informal coordination works

## File Locations

```text
monorepo/
├── plans/
│   ├── aps-rules.md              # Agent guidance (includes session rituals)
│   ├── index.aps.md              # Uses index-monorepo format
│   ├── modules/
│   │   ├── 01-auth.aps.md        # Packages: core, api
│   │   ├── 02-cli.aps.md         # Packages: cli, shared
│   │   └── 03-components.aps.md  # Packages: web, ui
│   ├── execution/
│   └── decisions/
├── apps/
│   ├── api/
│   ├── web/
│   └── cli/
└── packages/
    ├── core/
    ├── shared/
    └── ui/
```

## Next Steps

- Use [index-monorepo.template.md](../templates/index-monorepo.template.md) for your index
- Add `Packages` field to your modules using [module.template.md](../templates/module.template.md)
- Ensure your `aps-rules.md` includes session rituals
