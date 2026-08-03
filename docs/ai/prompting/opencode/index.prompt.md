# APS Index Prompt (OpenCode)

This variant defers to the [generic index prompt](../index.prompt.md); only OpenCode-specific differences follow.

ROLE: Planner
MODE: Non-executable (do not create implementation work items unless explicitly
requested)

## OpenCode Flow Control

- If repository context is missing, ask for it once, then proceed with explicit
  assumptions.
- Return the completed index and any assumptions as one deterministic Markdown
  response.
