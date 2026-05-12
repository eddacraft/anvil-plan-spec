# APS Rules for AI Agents

> This file guides AI agents working with APS specs in this repository.
> Keep it in `plans/` so agents discover it when exploring the planning directory.
> **APS-managed** — safe to update via `aps update`. Project-specific context
> lives in `plans/project-context.md` (user-owned, never overwritten).

## Core Principle

**Specs describe intent. Tasks authorise execution. Steps are checkpoints, not tutorials.**

## Hierarchy

| Layer | Purpose | You Write | You DON'T Write |
|-------|---------|-----------|-----------------|
| Index | Plan overview | Modules, milestones, risks | Implementation details |
| Module | Bounded work area | Interfaces, tasks, boundaries | Code snippets |
| Task | Execution authority | Outcome, validation command | How to implement |
| Step | Checkpoint | Observable state | Implementation steps |

## Steps: The Lean Rule

Steps translate task intent into **observable checkpoints**. They are NOT implementation guides.

### Format

```markdown
### 1. [Action verb] [target]

- **Checkpoint:** [Observable state — max 12 words]
- **Validate:** `[command]` (optional)
```

### What Goes WHERE

| Write in Step | Write NOWHERE (emerges from patterns) |
|---------------|---------------------------------------|
| "Auth middleware exists" | Which library to use |
| "Tests pass" | Test implementation details |
| "Migration applied" | SQL schema definition |
| "Function handles errors" | Try/catch structure |

### Anti-Patterns (NEVER do this)

```markdown
# BAD: Implementation tutorial disguised as step
### 1. Create authentication middleware

- **Checkpoint:** Middleware created in src/middleware/auth.ts that:
  - Extracts JWT from Authorization header
  - Validates token using jsonwebtoken library
  - Decodes payload and extracts user ID
  - Attaches user object to request context
  - Returns 401 if token invalid or expired
- **Validate:** `npm test -- auth.middleware.test.ts`
```

```markdown
# GOOD: Observable checkpoint only
### 1. Create authentication middleware

- **Checkpoint:** Auth middleware validates requests, attaches user to context
- **Validate:** `npm test -- auth.middleware.test.ts`
```

### Why Lean Steps?

1. **Implementation emerges** from existing patterns + agent judgment
2. **Specs don't rot** — checkpoints stay valid even when code changes
3. **Agents stay autonomous** — they figure out HOW, you verify WHAT
4. **Review stays fast** — humans scan checkpoints, not implementation plans

## Task Rules

Tasks are **execution authority** — permission to make changes.

### Required Fields

- **Intent:** One sentence — what outcome this achieves
- **Expected Outcome:** Testable/observable result
- **Validation:** Command to verify completion

### Optional Fields

- **Scope/Non-scope:** What will and won't change
- **Dependencies:** Other task IDs that must complete first
- **Confidence:** low/medium/high
- **Files:** Best-effort list (not exhaustive)

### Task Anti-Patterns

| Don't | Do |
|-------|-----|
| "Implement JWT auth using jsonwebtoken" | "Add token-based authentication" |
| "Create UserService class with methods..." | "User operations are encapsulated" |
| "Add try/catch blocks to all handlers" | "API errors return consistent format" |

## Action Plans: Waves and Parallel Execution

Action plans can optionally group actions into **waves** for parallel execution.
Actions in the same wave are independent — they can run concurrently. Each wave
completes before the next begins.

### Wave Table Format

```markdown
## Waves

| Wave | Actions | Gate |
|------|---------|------|
| 1 | 1, 2 | Both checkpoints pass |
| 2 | 3 | Checkpoint passes |
```

### Action-Level Fields

Actions support optional execution metadata:

- **Wave** N — which wave this action belongs to
- **Depends on** 1, 2 — action numbers that must complete first
- **Agent** type — agent type for dispatch (e.g., general-purpose, tdd-coach)

### When to Use Waves

| Use Waves | Stay Sequential |
|-----------|-----------------|
| 3+ actions with independent work | Each action depends on the previous |
| Multi-agent dispatch needed | Single-agent linear execution |
| Work item has natural parallel boundaries | Actions share mutable state |

### When NOT to Use Waves

- Simple work items (< 4 actions)
- All actions modify the same files
- Actions are inherently sequential (schema -> migration -> seed)

## Naming Conventions

### Module Files

Name module files with a numeric prefix based on dependency order:

```text
modules/
├── 01-core.aps.md      # Foundation, no dependencies
├── 02-auth.aps.md      # Depends on core
├── 03-payments.aps.md  # Depends on auth
└── 04-ui.aps.md        # Depends on all above
```

- Use zero-padded numbers (`01-`, `02-`, not `1-`, `2-`)
- Order matches dependency flow (foundational -> dependent)
- Order should reflect the Modules table in `index.aps.md`

### Task IDs

Tasks use the module's ID prefix: `AUTH-001`, `AUTH-002`, `CORE-001`, etc.

## Creating APS Documents

### When Asked to Plan

1. Read existing `plans/index.aps.md` if present
2. Identify which template fits (index, module, simple)
3. Fill sections with **intent**, not implementation
4. Mark assumptions explicitly
5. Leave tasks empty until module is Ready

### When Asked to Execute

1. Find the task in the relevant `.aps.md` file
2. Check task has **Ready** status
3. Create steps file in `plans/execution/` if complex
4. Execute one step at a time, validate checkpoint
5. Mark task complete when validation passes

## File Locations

```text
plans/
├── aps-rules.md           # This file (APS-managed agent guidance)
├── project-context.md     # Project-specific context (user-owned)
├── index.aps.md           # Root plan
├── issues.md              # Development-time discoveries
├── modules/               # Module specs (numbered by dependency order)
│   ├── 01-core.aps.md
│   └── 02-auth.aps.md
├── execution/                 # Action plans
│   ├── [WORK-ITEM-ID].actions.md  # Per-work-item (complex projects)
│   └── [MODULE].actions.md        # Per-module (simple projects)
├── decisions/             # ADRs (optional)
│   └── [NNN]-[title].md
└── designs/               # Technical designs (optional)
    └── YYYY-MM-DD-slug.design.md
```

## Design Documents

Design docs live in `plans/designs/`. They capture architectural thinking
**before** committing to modules and work items.

### When to Create

- Multi-module work with non-obvious architecture
- Multiple viable approaches that need comparison
- Work that needs review before defining work items
- Cross-cutting concerns that span several modules

### When to Skip

- Straightforward single-module features
- Bug fixes or small enhancements
- Work where the approach is already well established

### Naming

`plans/designs/YYYY-MM-DD-slug.design.md` — date-prefixed, descriptive slug.

### Linking

Reference designs from the Index or Module metadata:

```markdown
## Designs
- [Auth Architecture](designs/2025-01-05-auth-architecture.design.md)
```

### Accept-Then-Normalise

If a design doc already exists in free-form (created by another agent or human),
**accept it**. Don't reject it for missing sections. Instead:

1. Add the minimum fields: `## Problem`, `## Design`, and the Status metadata table
2. Don't rewrite the author's content — append missing sections or infer from
   existing content
3. This normalisation can happen in the background, after the main work

## Monorepo Conventions

For repositories with multiple packages/apps. See `docs/monorepo.md` for full guidance.

### Package Tagging

Every module declares `Packages: pkg1, pkg2` in metadata. Work items inherit or narrow the package scope.

### Session Start Ritual

Before touching code:

1. **Orient** — Read `plans/index.aps.md` "What's Next" section, then relevant module(s)
2. **Confirm authority** — Work item exists, status = Ready, packages are clear
3. **Declare intent** — State: "Executing AUTH-002 (core, api): [description]"

If no Ready work item exists:

- Create Draft work item first
- Ask human to mark Ready before proceeding
- OR if trivial fix, note in session end summary

### Session End Ritual

After completing work:

1. **Update status** — Mark work items: `In Progress`, `Complete: YYYY-MM-DD`, or `Blocked: [reason]`
2. **Capture discovered work** — Add as Draft items with package tags
3. **Log discoveries** — Add issues (ISS-NNN) or questions (Q-NNN) to `plans/issues.md`
4. **Update "What's Next"** — Remove completed, add new Ready items, re-sequence if needed
5. **Session summary** — Brief note: what completed, what discovered, what's next

**Key principle:** The next agent should pick up exactly where you left off without archaeology.

## Issues & Questions Tracker

Use `plans/issues.md` to log development-time discoveries:

- **Issues (ISS-NNN)** — Bugs, limitations, edge cases noticed during development
- **Questions (Q-NNN)** — Unknowns that need answers, deferred decisions

### When to Log

| Log as Issue | Log as Question |
|--------------|-----------------|
| "API rate-limits at 100 req/min" | "Should retry logic live in client or transport?" |
| "Login fails intermittently on Safari" | "What's the session expiry policy?" |
| "Edge case: empty array not handled" | "Do we need to support IE11?" |

### Referencing

From work items, notes, or commits:

- `See ISS-001` or `Related: ISS-001, Q-002`
- In commits: `Addresses ISS-001`

### Not a Bug Tracker

This is for **planning-level visibility**, not routine bugs. Use your project's bug tracker for:

- User-reported bugs
- Production incidents
- Detailed reproduction steps

## Quick Reference

| If agent is... | Check for... |
|----------------|--------------|
| Writing a design | Problem + Design sections present? No implementation prescriptions? |
| Writing steps | Max 12 words per checkpoint? No implementation detail? |
| Writing tasks | Outcome-focused? Has validation command? |
| Planning module | Boundaries clear? No premature tasks? |
| Executing | Task status is Ready? Prerequisites met? |
| In monorepo | Packages tagged? "What's Next" updated? |
| Found issue/question | Logged in issues.md with proper ID? |
