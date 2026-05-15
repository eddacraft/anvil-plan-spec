# APS Action Plan Prompt (Tool-Agnostic)

You are creating or executing APS Action Plans - the granular execution layer.
Action Plans translate Work Item intent into ordered, observable actions.

## File naming

- **Per-work-item** (`WORK-ITEM-ID.actions.md`): Complex projects, independent work items
- **Per-module** (`MODULE.actions.md`): Simple projects, tightly coupled work items

## Relationship to other layers

- **Work Item** = What to achieve (outcome)
- **Action Plan** = What actions to take (checkpoints)
- **Implementation** = How to code it (emerges from patterns + judgment)

## Rules

- Actions describe WHAT to do, not HOW to implement
- "How" only appears when referencing existing patterns
- Each action must include: Purpose, Produces, and Checkpoint
- If >8 actions, recommend splitting the Work Item
- No time prescriptions (actions aren't estimates)

## Action format

Each action must include:

- Action heading with verb + target (e.g., "Action 1 — Create middleware function")
- **Purpose**: Why this action exists
- **Produces**: Concrete artefacts or state
- **Checkpoint**: Observable state (max ~12 words)

Optional fields:

- **Validate**: Command to verify completion
- **Wave**: Wave number for parallel execution (see below)
- **Agent**: Agent type for dispatch (e.g., general-purpose, tdd-coach, architect)
- **Depends on**: Action numbers that must complete first (e.g., 1, 2)
- **Status**: Only if Blocked or Deferred, with reason

## Waves (parallel execution)

Use waves when actions within a work item can run concurrently. Actions in the
same wave have no dependencies on each other. Each wave completes before the
next begins.

Add an optional **Waves** table before the actions:

```markdown
## Waves

| Wave | Actions | Gate                  |
| ---- | ------- | --------------------- |
| 1    | 1, 2    | Both checkpoints pass |
| 2    | 3       | Checkpoint passes     |
```

Rules:

- **When to use waves**: 3+ actions where at least two are independent
- **When to skip**: Sequential plans where each action depends on the previous
- **Gate**: What must be true before the next wave starts (usually "all checkpoints pass")
- **Depends on**: Use on individual actions for fine-grained ordering across waves
- **Agent**: Assign agent types when different actions need different expertise

Waves are execution metadata, not implementation detail. They describe
scheduling, not how to code.

## Creating action plans (Propose mode)

- Extract actions from Work Item intent
- Order by dependency (what must exist first)
- Identify independent actions — group into waves where possible
- Keep actions independently verifiable
- Flag assumptions explicitly
- Avoid implementation detail in checkpoints
- Assign agent types only when expertise differs between actions

## Executing action plans (Execute mode)

- Validate prerequisites before starting
- If waves exist: dispatch wave actions in parallel, wait for gate
- If no waves: complete one action fully before proceeding
- Mark blocked actions with reason
- Do not skip checkpoints

## Output

Write action plans in markdown matching `templates/actions.template.md`.
