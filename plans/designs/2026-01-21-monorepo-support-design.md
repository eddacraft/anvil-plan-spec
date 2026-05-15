# Monorepo Support Design

**Date:** 2026-01-21
**Status:** Draft
**Scope:** Add monorepo conventions to APS spec

---

## Problem

APS currently assumes a single `plans/` directory for one project. In monorepos with multiple packages, this creates:

1. **Scope ambiguity** - Unclear which package a work item affects
2. **Navigation difficulty** - Hard to find the right module
3. **Cross-cutting concerns** - Work spanning packages doesn't fit cleanly
4. **Dependency confusion** - Hard to track inter-module dependencies
5. **Prioritization gaps** - No clear "what's next" across packages

## Solution: Tagged Modules + Ritualized Workflow

Keep single `plans/` at monorepo root. Add:

- Explicit package tags on modules and work items
- "What's Next" and "By Package" views in the index
- Session start/end rituals for agent discipline

---

## Index Structure

The index supports both "what's next" and "where does this live" queries:

```markdown
# Project Index

## What's Next

Prioritized queue of ready work across all packages:

| #   | Work Item | Module     | Packages            | Owner | Status |
| --- | --------- | ---------- | ------------------- | ----- | ------ |
| 1   | AUTH-002  | auth       | platform, anvil-api | @josh | Ready  |
| 2   | CLI-001   | cli        | anvil-cli, shared   | @josh | Ready  |
| 3   | UI-003    | components | anvil-ui            | —     | Ready  |

## Modules by Package

### apps/anvil-api

- [auth](./modules/01-auth.aps.md) — AUTH-002 ready

### apps/anvil-cli

- [cli](./modules/02-cli.aps.md) — CLI-001 ready

### apps/anvil-ui

- [components](./modules/03-components.aps.md) — UI-003 ready

### packages/platform

- [auth](./modules/01-auth.aps.md) — AUTH-002 ready
- [core](./modules/04-core.aps.md) — no ready items

### packages/shared

- [cli](./modules/02-cli.aps.md) — CLI-001 ready
- [utils](./modules/05-utils.aps.md) — no ready items

## Cross-Cutting Concerns

- [auth](./modules/01-auth.aps.md) — spans platform + anvil-api
```

Modules can appear under multiple packages. The package grouping is a _view_, not ownership—ownership lives in the module's metadata.

---

## Module Metadata

Each module explicitly declares which packages it affects:

```markdown
# Auth Module

| ID   | Owner | Priority | Status | Packages            |
| ---- | ----- | -------- | ------ | ------------------- |
| AUTH | @josh | high     | Ready  | platform, anvil-api |

## Purpose

Unified authentication flow across API and platform package.

## Work Items

### AUTH-001: Token generation ✓ Complete: 2026-01-15

...

### AUTH-002: Token refresh

| Status | Packages            | Dependencies |
| ------ | ------------------- | ------------ |
| Ready  | platform, anvil-api | AUTH-001     |

Intent: Implement refresh token rotation
Checkpoint: Refresh endpoint returns new token pair
Validate: `npm test -w packages/platform -- auth.test.ts`

### AUTH-003: Handle refresh edge cases (Draft)

| Status | Packages | Discovered      |
| ------ | -------- | --------------- |
| Draft  | platform | During AUTH-002 |
```

Key additions for monorepo:

- `Packages` column at module level (which packages this module touches)
- `Packages` per work item (can be subset of module's packages)
- Validation commands use workspace flags (`-w packages/platform`)

---

## Session Rituals

### Session Start Ritual

When an agent begins work on the monorepo:

```markdown
### 1. Orient to Current State

Read in order:

1. `plans/index.aps.md` - "What's Next" section
2. The module(s) tagged for your target package
3. Any work item you're about to execute

### 2. Confirm Execution Authority

Before touching code, verify:

- [ ] Work item exists and status = Ready
- [ ] You understand which package(s) are affected
- [ ] Checkpoint is clear and observable

### 3. Declare Intent

State which work item you're executing:

> "Executing AUTH-002 (platform, anvil-api): Implement token refresh"

If no Ready work item exists for what you're asked to do:

- Create Draft work item first
- Ask human to mark Ready before proceeding
- OR if trivial fix, note in session end summary
```

Key principle: Agents don't freelance. They either execute authorized work or surface that authorization is missing.

### Session End Ritual

```markdown
### 1. Update Work Item Status

For each work item touched:

- `In Progress` → if work started but not complete
- `Complete: YYYY-MM-DD` → if checkpoint passes
- `Blocked: [reason]` → if stuck on dependency/question

### 2. Capture Discovered Work

Any new work uncovered during execution:

- Add as Draft work item to appropriate module
- Tag with affected packages
- Note dependency on current work if relevant

Example:

### AUTH-003: Handle token refresh edge case (Draft)

| Discovered during | Packages | Status |
| ----------------- | -------- | ------ |
| AUTH-002          | platform | Draft  |

Intent: Handle expired refresh tokens gracefully

### 3. Update "What's Next"

In `plans/index.aps.md`:

- Remove completed items
- Add any new Ready items
- Re-sequence if priorities shifted

### 4. Session Summary

Leave a brief note (in commit message or plan file):

> "Session: Completed AUTH-002. Discovered AUTH-003 (draft).
> Next recommended: CLI-001 or review AUTH-003 for Ready status."
```

Key principle: The next agent (or human) should be able to pick up exactly where you left off without archaeology.

---

## Spec Placement

Where this guidance lives in APS:

| Location                               | Content                        |
| -------------------------------------- | ------------------------------ |
| `docs/monorepo.md`                     | Dedicated guide (new)          |
| `docs/getting-started.md`              | Add "Monorepo Setup" section   |
| `templates/index-monorepo.template.md` | Index with package views (new) |
| `templates/module.template.md`         | Add Packages field             |
| `plans/aps-rules.md`                   | Add session rituals            |

### aps-rules.md additions

```markdown
## Monorepo Conventions

### Package Tagging

Every module declares `Packages: pkg1, pkg2` in metadata.
Work items inherit or narrow the package scope.

### Session Rituals

See Session Start Ritual and Session End Ritual sections.
```

---

## Summary

| Component               | Purpose                           |
| ----------------------- | --------------------------------- |
| Package tags on modules | "Where does this live?"           |
| "What's Next" table     | "What should I work on?"          |
| Package views in index  | Navigation by package             |
| Session start ritual    | Orient before coding              |
| Session end ritual      | Update state, capture discoveries |

---

## Implementation

To implement this design:

1. Create `docs/monorepo.md` with full guidance
2. Create `templates/index-monorepo.template.md`
3. Update `templates/module.template.md` with Packages field
4. Update `plans/aps-rules.md` with session rituals
5. Update `docs/getting-started.md` with monorepo section
