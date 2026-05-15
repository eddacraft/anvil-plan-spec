<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- This document is non-executable. -->
<!-- For monorepos with multiple packages/apps. See docs/monorepo.md for guidance. -->

# [Plan Title]

## Overview

[One paragraph describing what this plan covers and why it matters]

## Problem & Success Criteria

**Problem:** [What problem are we solving? Why does this work matter?]

**Success Criteria:**

- [ ] [Measurable outcome 1]
- [ ] [Measurable outcome 2]
- [ ] [How we know we're done]

## What's Next

Prioritized queue of ready work across all packages:

| #   | Work Item | Module                                       | Packages    | Owner     | Status |
| --- | --------- | -------------------------------------------- | ----------- | --------- | ------ |
| 1   | AUTH-002  | [auth](./modules/01-auth.aps.md)             | core, api   | @username | Ready  |
| 2   | CLI-001   | [cli](./modules/02-cli.aps.md)               | cli, shared | @username | Ready  |
| 3   | UI-003    | [components](./modules/03-components.aps.md) | web, ui     | —         | Ready  |

<!-- Update this table as work items complete or priorities shift -->

## Modules by Package

<!-- Group modules by the packages they affect. Modules can appear under multiple packages. -->

### apps/[app-name]

- [module-id](./modules/NN-name.aps.md) — [summary of ready items]

### apps/[another-app]

- [module-id](./modules/NN-name.aps.md) — [summary]

### packages/[package-name]

- [module-id](./modules/NN-name.aps.md) — [summary]
- [another-module](./modules/NN-another.aps.md) — no ready items

### packages/[another-package]

- [module-id](./modules/NN-name.aps.md) — [summary]

## Cross-Cutting Concerns

<!-- Modules that span multiple packages deserve visibility -->

- [auth](./modules/01-auth.aps.md) — spans core + api + web

## All Modules

| Module                                       | Scope | Owner     | Status | Priority | Packages    | Dependencies |
| -------------------------------------------- | ----- | --------- | ------ | -------- | ----------- | ------------ |
| [auth](./modules/01-auth.aps.md)             | AUTH  | @username | Ready  | high     | core, api   | —            |
| [cli](./modules/02-cli.aps.md)               | CLI   | @username | Draft  | medium   | cli, shared | auth         |
| [components](./modules/03-components.aps.md) | UI    | @username | Ready  | medium   | web, ui     | —            |

## Constraints

- [Technical constraint, e.g., "Must run on Node 18+"]
- [Product constraint, e.g., "Must not break existing API"]

## Milestones _(optional)_

### M1: [Milestone Name]

- **Target:** [date or scope]
- **Includes:** [modules/features]

## Risks & Mitigations _(optional)_

| Risk               | Impact | Likelihood | Mitigation          |
| ------------------ | ------ | ---------- | ------------------- |
| [Risk description] | high   | medium     | [How we address it] |

## Decisions _(optional)_

- **D-001:** [Short decision] — [rationale] ([ADR-001](./decisions/001-decision.md))

## Open Questions _(optional)_

- [ ] [Unresolved question 1]
- [ ] [Unresolved question 2]
