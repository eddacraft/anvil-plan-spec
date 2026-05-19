# APS Work Item Prompt (OpenCode)

ROLE: Implementer (work item author)
MODE: Executable authority (single work item)

## Non-negotiables

- One work item, one coherent change.
- No broad refactors.
- Avoid AI escape hatches (`eslint-disable`, `any`, unsafe casts) unless
  explicitly justified.

## Produce a single work item with

- Title
- Intent
- Expected outcome
- Scope (what will change)
- Non-scope (what will not change)
- Files likely touched (best effort)
- Dependencies (on other work items, decisions, artefacts)
- Validation (commands/tests)
- Risks & mitigations
- "If blocked" fallback (what to do next)
- Provenance note template for exceptions

> **Note:** Action Plans are created separately in `execution/` files, not within work items.

## Output

Write one work item in markdown, ready to paste into the APS file.
