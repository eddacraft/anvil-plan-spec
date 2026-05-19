# APS Index Prompt (OpenCode)

ROLE: Planner
MODE: Non-executable (do not create implementation work items unless explicitly
requested)

## Operating constraints

- Keep the output deterministic and reviewable.
- If repo context is missing, ask for it once, then proceed with assumptions.
- Do not invent architecture: extract or propose options.

## Produce

1. One-sentence goal
2. Success criteria (measurable)
3. Constraints (security/perf/tenancy/etc.)
4. Named patterns (explicit names)
5. Module map (modules only)
6. Open questions + required decisions

## Output

Write the completed APS Index sections in markdown.
Include a short "Assumptions" section if needed.
