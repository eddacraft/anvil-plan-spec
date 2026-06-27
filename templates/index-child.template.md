<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- This document is non-executable. -->
<!--
  Federated monorepo CHILD plan (nested-plans tier).
  Lives at packages/<pkg>/plans/index.aps.md (D-001) and is a COMPLETE,
  standalone APS plan: it lints, orchestrates, and ships in isolation, and
  remains valid if the package is extracted to its own repo (D-003).
  Work-item and module IDs are BARE within this tree; reference another tree
  with <child-name>:<ID> (D-002). May be linked from a parent root's
  `## Child Plans` section. See plans/designs/2026-06-27-nested-plans.design.md.
-->

# [Package Title]

## Overview

[One paragraph: what this package's plan covers. It stands on its own — do not
assume the parent or sibling plans are present.]

## Problem & Success Criteria

**Problem:** [What problem does this package solve?]

**Success Criteria:**

- [ ] [Measurable outcome 1]
- [ ] [How we know this package's work is done]

## Constraints

- [Package-level constraint]

## Modules

<!--
  IDs are bare within this tree (e.g. AUTH-001), not globally prefixed.
  To depend on another tree's item, use <child-name>:<ID> in Dependencies
  (e.g. core:AUTH-001) — see the cross-tree convention.
-->

| Module                                    | ID  | Owner     | Status | Priority | Tags | Dependencies |
| ----------------------------------------- | --- | --------- | ------ | -------- | ---- | ------------ |
| [module-id](./modules/module-name.aps.md) | MOD | @username | Draft  | medium   | —    | —            |

## Risks & Mitigations

| Risk               | Impact | Likelihood | Mitigation          |
| ------------------ | ------ | ---------- | ------------------- |
| [Risk description] | high   | medium     | [How we address it] |

## Decisions

- **D-001:** [Package-scoped decision] — [rationale]

## Open Questions

- [ ] [Unresolved question]
