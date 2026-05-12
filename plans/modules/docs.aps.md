# Documentation Module

| ID   | Owner  | Priority | Status   |
| ---- | ------ | -------- | -------- |
| DOCS | @aneki | high     | Complete |

## Purpose

Document the APS workflow clearly enough that users can adopt it without
reading every template or prompt.

## In Scope

- Getting started guide
- Installation guide
- Workflow guide
- Monorepo guide
- AI agent guide
- Terminology and contribution guidance

## Out of Scope

- Hosted docs site
- Tool-specific plugin documentation outside APS-owned integrations

## Interfaces

**Exposes:**

- `README.md`
- `docs/getting-started.md`
- `docs/installation.md`
- `docs/workflow.md`
- `docs/monorepo.md`
- `docs/ai-agent-guide.md`
- `docs/TERMINOLOGY.md`

## Work Items

### DOCS-001: Publish onboarding documentation — Complete

- **Intent:** Make common APS workflows discoverable
- **Expected Outcome:** Documentation covers installation, plan authoring,
  execution, and agent collaboration with examples.
- **Validation:** `npx markdownlint-cli "**/*.md"`
- **Files:** README.md, docs/
- **Confidence:** high
