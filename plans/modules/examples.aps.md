# Examples Module

| ID       | Owner  | Priority | Status |
| -------- | ------ | -------- | ------ |
| EXAMPLES | @aneki | medium   | In Progress |

**Last reviewed:** 2026-07-16

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
- `examples/team-payments/`
- `test/fixtures/valid/`
- `docs/team-rollout.md`

## Work Items

### EXAMPLES-001: Expand worked examples — Draft

- **Status:** Draft
- **Intent:** Show APS in more than one project shape and tool workflow
- **Expected Outcome:** Examples cover single-feature, multi-module, agent-led,
  and execution-plan workflows with valid markdown.
- **Validation:** `./bin/aps lint examples test/fixtures/valid`
- **Files:** examples/, test/fixtures/valid/
- **Confidence:** medium

### EXAMPLES-002: Team rollout guide + multi-owner example — Complete 2026-07-16

- **Intent:** Give a team adopting APS the conventions the format alone
  doesn't answer: who owns the index, how plan changes are reviewed, how
  concurrent status edits avoid merge conflicts, which gate-policy preset to
  start on, and pinning `cli_version` so the whole team lints identically.
- **Expected Outcome:** `docs/team-rollout.md` covers the above with concrete
  commands, plus a lint-clean multi-owner example under
  `examples/team-payments/` — one index, 2–3 modules with different owners,
  a conductor module, and in-flight statuses that show a plan mid-execution
  (not a finished artifact).
- **Validation:** `./bin/aps lint examples/team-payments`; markdownlint
  passes; guide links resolve; a reader can run the rollout steps verbatim.
- **Dependencies:** None (references INTEGRATIONS-003 Action once it exists)
- **Files:** docs/team-rollout.md, examples/team-payments/
- **Confidence:** high
- **Results:** Team rollout guide + lint-clean multi-owner example. Landed 2026-07-16.
