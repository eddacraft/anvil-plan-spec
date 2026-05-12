# Prompts Module

| ID      | Owner  | Priority | Status |
| ------- | ------ | -------- | ------ |
| PROMPTS | @aneki | medium   | Draft  |

## Purpose

Maintain APS prompting entry points for generic agents and tool-specific
harnesses without drifting away from the shared APS lifecycle.

## In Scope

- Generic index, module, work item, and actions prompts
- OpenCode-specific prompt variants
- Claude Code orchestration prompts
- Guidance for when tool-specific variants should exist versus linking back to
  generic prompts

## Out of Scope

- Full agent packaging, which belongs to AGENT
- Prompt marketplace distribution

## Interfaces

**Exposes:**

- `docs/ai/prompting/*.prompt.md`
- `docs/ai/prompting/opencode/*.prompt.md`
- `docs/ai/prompting/claudecode/*.prompt.md`

## Work Items

### PROMPTS-001: Normalize prompt variants — Draft

- **Intent:** Keep tool-specific prompts consistent with the generic APS rules
- **Expected Outcome:** Prompt variants clearly state their tool-specific
  differences and otherwise defer to shared APS concepts.
- **Validation:** Review prompt variants for duplicated stale rules; markdownlint
  passes for `docs/ai/prompting/`
- **Files:** docs/ai/prompting/
- **Confidence:** medium
