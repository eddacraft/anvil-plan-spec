# APS Work Item Prompt (Tool-Agnostic)

You are assisting with creating a single APS Work Item.
A work item is **execution authority**. Keep it small.

## Objectives

- Convert intent into a minimal, reviewable change
- Minimise blast radius
- Make exceptions explicit and accountable

## Hard rules

- One work item = one coherent change
- Avoid `eslint-disable`, `any`, unsafe casts, broad suppressions
- No broad refactors unless explicitly the work item's purpose
- If you cannot scope safely, split into smaller work items

## Required work item fields

- Work item title
- Intent (one sentence)
- Expected outcome (observable/testable)
- Scope (what will change)
- Non-scope (what will not change)
- Files likely touched (best effort)
- Validation steps (tests/checks)
- Risks + mitigations
- Dependencies (on other work items, decisions, artefacts)
- Provenance note template (for future suppressions/exceptions)

## Preferred routing hints

Optional, but include them whenever you can judge them — harnesses use them to
route the item:

- Model — the AI model that should execute the item (e.g. `claude-opus-5`)
- Reasoning — `low | medium | high | max`, the effort the executing model
  should spend

Omit either to inherit the module table's `Model` / `Reasoning` column.

## Output style

- Be crisp and specific
- Prefer bullets over paragraphs
- Flag assumptions explicitly
