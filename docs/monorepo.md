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

## When to Use Monorepo Structure

Use monorepo conventions when:

- You have 3+ packages/apps in one repository
- Work regularly spans multiple packages
- Multiple agents or developers work in parallel
- You need clear "what's next" prioritization

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
