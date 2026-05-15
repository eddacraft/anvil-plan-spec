# APS Planner Skill

> Plan, manage, and execute work using Anvil Plan Spec (APS).

## What This Skill Does

This skill teaches you to use **APS (Anvil Plan Spec)** — a markdown-based
specification format for planning and authorizing work in AI-assisted
development. APS files live in `plans/` and survive context resets, session
clears, and handoffs.

## When to Activate

Activate this skill when:

- Starting a new feature or project that needs planning
- Checking the status of existing APS plans
- Executing a specific work item from an APS spec
- Creating or updating indexes, modules, work items, or action plans

## APS Hierarchy

| Layer           | Purpose                                             | Executable?               |
| --------------- | --------------------------------------------------- | ------------------------- |
| **Index**       | High-level project plan with modules and milestones | No                        |
| **Module**      | Bounded scope with interfaces and work items        | Yes (if Ready)            |
| **Work Item**   | Single coherent change with validation              | Yes (execution authority) |
| **Action Plan** | Ordered actions with checkpoints                    | Yes (granular execution)  |

## Core Rules

1. **Plan before building.** Never start complex work without an APS file.
2. **Read before deciding.** Re-read the relevant spec before major decisions.
3. **Update as you go.** After completing work or discovering scope, update
   the spec immediately.
4. **Never skip validation.** Every work item has a Validation field. Run it.
5. **Specs describe intent, not implementation.** Write what and why, not how.

## Key Responsibilities

- **Install/Update APS** — run scaffold scripts or create `plans/` manually
- **Create Indexes** — overview, modules, milestones, decisions
- **Create Modules** — bounded work areas with 2-8 work items each
- **Draft Work Items** — intent, expected outcome, validation (required fields)
- **Create Action Plans** — decompose work items into actions with checkpoints
- **Track Status** — scan artefacts, report state, suggest next steps
- **Execute Work Items** — verify Ready, create plan if complex, validate
- **Sync at Session End** — update statuses, add discovered work

## Work Item Requirements

Each work item must have:

- **Intent** — one sentence describing the outcome
- **Expected Outcome** — observable/testable result
- **Validation** — command or method to verify completion

## File Structure

```text
plans/
├── aps-rules.md               # AI agent guidance
├── index.aps.md               # Main plan (non-executable)
├── modules/                   # Bounded work areas
├── execution/                 # Action plans
└── decisions/                 # ADRs
```

## Quality Standards

- Success criteria must be measurable and falsifiable
- Avoid solutioneering — propose options, don't commit to implementation
- Mark assumptions explicitly
- Keep specs in sync — update as you work, not after
- Checkpoints are observable state, not instructions
