# APS Module Prompt (Tool-Agnostic)

You are assisting with completing an APS Module document.
This document is **bounded and near-executable**, but should remain conservative.

## File naming

Name files with numeric prefix by dependency order: `01-core.aps.md`, `02-auth.aps.md`

> **Note:** For small, self-contained features without complex interfaces,
> suggest using `templates/simple.template.md` instead.

## Objectives

1. Define scope and boundaries precisely
2. Define interfaces/contracts (inputs/outputs) where relevant
3. Capture constraints, named patterns, and "must not" rules
4. Draft work items only if the module is "Ready"; otherwise list blockers

## Rules

- Prefer small, reviewable changes
- If a module is too large, recommend splitting

## Output format

Fill:

- Module Overview
- In Scope / Out of Scope
- Interfaces/Contracts (inputs/outputs, dependencies, what this module exposes)
- Named patterns that apply
- Boundary rules (e.g. "Payments must not depend on Identity")
- Acceptance criteria
- Risks & mitigations
- Open questions & decisions
- Work Items:
  - If Ready: 2-8 work items maximum, each small and independently reviewable
  - If Not Ready: "No work items authorised" + list blockers

## Work item drafting guidelines (if applicable)

Each work item must include:

- Intent (one sentence)
- Expected outcome (testable)
- Files likely touched (best effort)
- "Do not touch" boundaries
- Validation (tests, checks)
- Risks & mitigations
