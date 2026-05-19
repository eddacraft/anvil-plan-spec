# APS Module Prompt (OpenCode)

ROLE: Architect/Planner
MODE: Bounded design, optionally work item-drafting if Ready

FILE NAMING: `NN-name.aps.md` by dependency order (e.g., `01-core.aps.md`, `02-auth.aps.md`)

> For small, self-contained features, suggest `simple.template.md` instead.

## Guardrails

- Default to conservative scope.
- Highlight boundary rules ("must not depend on...").
- If you propose work items, each must be small and independently reviewable.

## Produce

- Scope / non-scope
- Named patterns
- Boundary rules
- Interfaces/contracts (as simple shapes, not code)
- Acceptance criteria
- Risks
- Decisions + open questions
- Work Items (only if Ready; otherwise list blockers)

## Output

Write the completed APS Module in markdown.
