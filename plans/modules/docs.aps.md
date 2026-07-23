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

### DOCS-002: Reposition APS around intent-first planning — Complete 2026-07-23

- **Intent:** Make APS's separation of planning intent and execution authority
  immediately understandable to a new reader.
- **Expected Outcome:** The README leads with intent-focused planning, explains
  why specifications and work items have different responsibilities, and
  shows how outcome-based authority lets implementation adapt without agent
  scope drift.
- **Validation:** `npx markdownlint-cli "README.md" "plans/modules/docs.aps.md"`
- **Learning:** "Portability is a benefit; separating intent from execution
  authority is the product idea."
- **Files:** README.md, plans/modules/docs.aps.md
- **Confidence:** high
- **Results:** Reframed the README around the specification, work item, and
  implementation boundary; replaced the implementation-led quick tour with an
  intent and authority example; moved installation below the core concept;
  retained the existing adoption, platform, template, and release references.
