# Examples Module

| ID       | Owner  | Priority | Status |
| -------- | ------ | -------- | ------ |
| EXAMPLES | @aneki | medium   | Draft  |

## Purpose

Provide worked APS examples that demonstrate realistic planning structures,
execution plans, decisions, and design documents.

## In Scope

- Example projects for small features and multi-module work
- Valid action plans under `examples/*/execution/`
- Example design documents and decisions
- Fixtures that double as validation coverage where practical

## Out of Scope

- Large fake applications
- Generated examples with no explanatory value

## Interfaces

**Exposes:**

- `examples/user-auth/`
- `examples/opencode-companion/`
- `test/fixtures/valid/`

## Work Items

### EXAMPLES-001: Expand worked examples — Draft

- **Intent:** Show APS in more than one project shape and tool workflow
- **Expected Outcome:** Examples cover single-feature, multi-module, agent-led,
  and execution-plan workflows with valid markdown.
- **Validation:** `./bin/aps lint examples test/fixtures/valid`
- **Files:** examples/, test/fixtures/valid/
- **Confidence:** medium
