# APS Planning Skill

> Persistent, structured planning for AI agents using Anvil Plan Spec.

## What This Skill Does

This skill teaches you to use **APS (Anvil Plan Spec)** — a markdown-based
planning format — as persistent memory for complex tasks. APS files live in
`plans/` and survive context resets, session clears, and handoffs.

**Core insight:** Your context window is RAM (volatile, limited). The filesystem
is disk (persistent, unlimited). Anything important gets written to APS files.

## Hard Rules

1. **Plan before building.** Never start a complex task without an APS file.
   If `plans/` doesn't exist, create it. If no spec covers this work, create
   one before writing code.

2. **Read before deciding.** Before any major implementation decision, re-read
   the relevant APS spec. After many tool calls, your original goals drift out
   of your attention window. Re-reading brings them back.

3. **Update as you go.** After completing a work item or discovering something
   important, update the APS file immediately. Stale specs lose trust.

4. **Never skip validation.** Every work item has a Validation field. Run it
   before marking anything complete.

5. **Specs describe intent, not implementation.** Write *what* and *why*, never
   *how*. Implementation emerges from code patterns and agent judgment.

## When to Trigger APS Planning

Use APS when the user asks you to:

- Build a new feature with multiple parts
- Plan, design, or architect something
- Work on something that spans multiple files or domains
- Execute work that needs coordination or sequencing
- Pick up where a previous session left off

**Don't use APS for:** Quick one-off fixes, single-file edits, questions, or
trivial changes.

## The APS Workflow

```
Assess → Plan → Execute → Validate → Update
```

### 1. Assess

Before planning, understand what exists:

```
1. Check: Does plans/ directory exist?
2. If plans/ does not exist → run: aps init
   (This bootstraps the APS structure. If aps is not on PATH, check bin/aps in the anvil-plan-spec repo or run: curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/install.sh | sh)
   After init completes, continue to step 3.
3. Check: Does .aps/config.yml exist?
   If it exists, compare its cli_version with the installed CLI (aps --version).
   If they differ → run: aps update
   (This updates templates, the skill file, and tool files to the current version.)
   After update completes, continue to step 4.
4. Check: Does plans/index.aps.md exist?
5. Check: Are there module files in plans/modules/?
6. Check: Are there design docs in designs/?
7. Read plans/aps-rules.md if present (agent guidance)
8. Identify: Is this new work or continuing existing work?
```

### 2. Plan (pick the right template)

| Situation | Action |
|-----------|--------|
| Complex architecture needing review | Create a Design doc in `designs/` |
| Quick feature (1-3 items) | Create a Simple spec |
| Bounded work area with interfaces | Create a Module spec |
| Multi-module initiative | Create an Index + Modules |
| Complex work item needing breakdown | Create an Action Plan |

**Simple spec** — for self-contained features:

```markdown
# [Feature Name]

| ID | Owner | Status |
|----|-------|--------|
| FEAT | @user | Draft |

## Purpose
[What problem this solves]

## Work Items

### FEAT-001: [Title]
- **Intent:** [What this achieves]
- **Expected Outcome:** [Testable result]
- **Validation:** `[command]`
```

**Module spec** — for bounded areas with interfaces:

```markdown
# [Module Title]

| ID | Owner | Priority | Status |
|----|-------|----------|--------|
| AUTH | @user | medium | Draft |

## Purpose
[Why this module exists]

## In Scope
- [What this module handles]

## Work Items

### AUTH-001: [Title]
- **Intent:** [One sentence]
- **Expected Outcome:** [Observable result]
- **Validation:** `[command]`
- **Confidence:** medium
```

**Index** — for multi-module initiatives:

```markdown
# [Plan Title]

## Overview
[What this plan covers]

## Problem & Success Criteria
**Problem:** [What we're solving]
**Success Criteria:**
- [ ] [Measurable outcome]

## Modules
| Module | Purpose | Status |
|--------|---------|--------|
| [auth](./modules/auth.aps.md) | Authentication | Draft |
```

### 3. Execute

For each work item:

1. Confirm the work item has **Ready** status
2. Re-read the work item's Intent and Expected Outcome
3. If complex, create an Action Plan in `plans/execution/`
4. Implement one work item at a time
5. Run the Validation command

### 4. Validate

- Run every work item's Validation command
- Check: Does the outcome match Expected Outcome?
- If it diverged, update the spec to reflect reality

### 5. Update

After completing work:

- Mark work items complete: add `- **Status:** Complete` or checkmark
- Capture any new work discovered as Draft items
- Update the module/index status if all items are done
- Brief note on what completed and what's next

## Behavioral Reinforcement

### The 5-Operation Rule

After every 5 tool operations (Read, Write, Edit, Bash), pause and ask:

- Am I still working toward the current work item's Intent?
- Have I discovered something that should be captured in the spec?
- Should I re-read the plan to refresh my goals?

If the answer to any is "yes" or "maybe", update or re-read the APS file.

### Goal Drift Prevention

When you notice yourself:

- Making changes not described in any work item → Stop. Check the spec.
- Unsure what to do next → Re-read the module's Work Items section.
- Discovering scope creep → Add a new Draft work item, don't expand current one.
- Hitting a blocker → Update the work item with `Blocked: [reason]`.

### Session Continuity

**Starting a session:**

1. Read `plans/index.aps.md` (or the relevant module)
2. Find work items with Ready or In Progress status
3. Declare: "Working on [ID]: [title]"
4. Resume from where the spec says work left off

**Ending a session:**

1. Update all work item statuses
2. Add any discovered work as Draft items
3. Commit APS changes to git so the next session picks up cleanly

## File Layout

```
designs/                          # Technical designs (optional, project root)
└── YYYY-MM-DD-slug.design.md     # Architecture/approach documents

plans/
├── aps-rules.md              # Agent guidance (read first)
├── index.aps.md              # Root plan (if multi-module)
├── modules/                  # Module specs
│   ├── 01-core.aps.md
│   └── 02-auth.aps.md
├── execution/                # Action plans for complex items
│   └── AUTH-001.actions.md
└── decisions/                # Architecture decisions (optional)
```

## Design Document Normalisation

When you encounter a design doc in `designs/` that's missing standard sections:

1. Don't reject it — any markdown in `designs/` is a valid design
2. Add the minimum fields: `## Problem`, `## Design`, and Status metadata table
3. Don't rewrite the author's content — append missing sections or infer from
   existing content
4. This can happen in the background, after the main planning/execution work

## Work Item Format

Every work item must have these three fields:

- **Intent:** What outcome this achieves (one sentence)
- **Expected Outcome:** Observable/testable result
- **Validation:** Command to verify completion

Optional fields: Confidence, Dependencies, Files, Non-scope, Status.

## Action Plan Format (for complex work items)

```markdown
# Action Plan: AUTH-001

## Actions

### Action 1 — [Verb] [target]
**Purpose:** [Why]
**Produces:** [Artefacts]
**Checkpoint:** [Observable state — max 12 words]
**Validate:** `[command]`
```

Checkpoints are lean: max 12 words, no implementation detail.

## Naming Conventions

- Module files: `NN-name.aps.md` (e.g., `01-core.aps.md`)
- Work item IDs: `PREFIX-NNN` (e.g., `AUTH-001`, `CORE-002`)
- Action plans: `WORK-ITEM-ID.actions.md` or `MODULE.actions.md`
- Simple specs: `feature-name.aps.md`

## Validation CLI

If the `aps` CLI is available, validate your specs:

```bash
./bin/aps lint              # Lint all plans/
./bin/aps lint plans/modules/auth.aps.md  # Lint one file
```

## Anti-Patterns

| Don't | Do |
|-------|-----|
| Start coding without a spec | Create at least a Simple spec first |
| Write implementation details in specs | Write intent and outcomes only |
| Forget to update status after completing work | Update immediately |
| Expand scope mid-work-item | Add a new Draft item instead |
| Skip validation commands | Always run them before marking complete |
| Stuff everything in context | Write findings to APS files |
