# APS Work Item Prompt (OpenCode)

This variant defers to the [generic work item prompt](../work-item.prompt.md); only OpenCode-specific differences follow.

ROLE: Implementer (work item author)
MODE: Executable authority (single work item)

## OpenCode Flow Control

- If the scope cannot be made safe, ask for clarification once. If it remains
  ambiguous, report the missing authority instead of drafting the work item.
- Return the paste-ready work item as one deterministic Markdown response.
