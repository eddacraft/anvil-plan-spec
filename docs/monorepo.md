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

## Session Rituals

These rituals ensure agents keep documentation in sync with their work.

### Session Start Ritual

When an agent begins work:

#### 1. Orient to Current State

Read in order:

1. `plans/index.aps.md` — "What's Next" section
2. The module(s) tagged for your target package
3. Any work item you're about to execute

#### 2. Confirm Execution Authority

Before touching code, verify:

- [ ] Work item exists and status = Ready
- [ ] You understand which package(s) are affected
- [ ] Checkpoint is clear and observable

#### 3. Declare Intent

State which work item you're executing:

> "Executing AUTH-002 (core, api): Implement token refresh"

If no Ready work item exists for what you're asked to do:

- Create Draft work item first
- Ask human to mark Ready before proceeding
- OR if trivial fix, note in session end summary

**Key principle:** Agents don't freelance. They either execute authorized work or surface that authorization is missing.

### Session End Ritual

When an agent completes work:

#### 1. Update Work Item Status

For each work item touched:

- `In Progress` → if work started but not complete
- `Complete: YYYY-MM-DD` → if checkpoint passes
- `Blocked: [reason]` → if stuck on dependency/question

#### 2. Capture Discovered Work

Any new work uncovered during execution:

- Add as Draft work item to appropriate module
- Tag with affected packages
- Note dependency on current work if relevant

Example:

```markdown
### AUTH-003: Handle token refresh edge case

| Status | Packages | Discovered during |
| ------ | -------- | ----------------- |
| Draft  | core     | AUTH-002          |

- **Intent:** Handle expired refresh tokens gracefully
```

#### 3. Update "What's Next"

In `plans/index.aps.md`:

- Remove completed items
- Add any new Ready items
- Re-sequence if priorities shifted

#### 4. Session Summary

Leave a brief note (in commit message or plan file):

> "Session: Completed AUTH-002. Discovered AUTH-003 (draft). Next recommended: CLI-001 or review AUTH-003 for Ready status."

**Key principle:** The next agent (or human) should be able to pick up exactly where you left off without archaeology.

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
