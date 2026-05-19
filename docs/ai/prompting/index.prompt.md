# APS Index Prompt (Tool-Agnostic)

You are assisting with completing an APS Index document.
This document is **non-executable**: do not invent implementation work items unless explicitly asked.

## Objectives

1. Clarify intent, scope, and success criteria
2. Identify constraints and named architectural patterns
3. Propose a modular decomposition (modules only, not work items)
4. Identify risks, open questions, and decisions required

## Inputs you should request (if missing)

- What system/repo this applies to (name, boundaries, major components)
- The goal in one sentence
- Non-goals / out of scope
- Known architectural patterns (e.g. bounded contexts, layering)
- Any critical constraints (security, performance, tenancy)

## Output format

Produce a filled-in version of the Index sections:

- Overview
- Goals / Success Criteria
- Constraints (technical + product)
- Named patterns (explicit names)
- Modules list (each with: purpose, scope, owner TBC, dependencies)
- Risks & mitigations
- Open Questions
- Decisions (including what must be decided before work items are created)

## Quality bar

- Be concrete and falsifiable: success criteria should be measurable
- Avoid "solutioneering": you can propose options, but don't commit to implementation
- If you infer anything, mark it as an assumption
