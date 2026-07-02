<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- This document is non-executable. -->
<!--
  Federated monorepo ROOT plan (nested-plans tier).
  Use when packages have independent owners/lifecycles and each needs its own
  standalone plan. For a single shared backlog, prefer the tagged-modules
  approach (index-monorepo.template.md). See docs/monorepo.md and
  plans/designs/2026-06-27-nested-plans.design.md.
-->

# [Federation Title]

## Overview

[One paragraph: what this monorepo covers and how its packages relate. The root
links child plans and rolls up their status; it does not own their modules.]

## Problem & Success Criteria

**Problem:** [What problem does the whole monorepo solve?]

**Success Criteria:**

- [ ] [Cross-cutting outcome 1]
- [ ] [How we know the federation as a whole is done]

## Constraints

- [Repo-wide constraint, e.g., "Packages stay independently releasable"]

## Child Plans

<!--
  Each child is a complete, standalone APS plan under its package
  (D-001: packages/<pkg>/plans/index.aps.md). The link text is the child's
  path-derived name and is the prefix for cross-tree references (<name>:<ID>).
  One list item per child:
-->

- [core](../packages/core/plans/index.aps.md) — [shared domain + persistence]
- [api](../packages/api/plans/index.aps.md) — [HTTP surface and handlers]

## Roll-up

<!--
  Aggregated status of each child plan. Regenerate the rows with `aps rollup`
  and paste them here at session end (the root stays hand-authored). One row
  per child, matching the ## Child Plans list above.
-->

| Child | Modules (complete/total) | Next ready item | Status |
| ----- | ------------------------ | --------------- | ------ |
| core  | 0/0                      | —               | —      |
| api   | 0/0                      | —               | —      |

## Modules

<!--
  A federation root owns no modules of its own — work lives in the child plans
  above (D-003). Keep this section (an index requires it) with the note and an
  empty table, OR list genuinely root-level crosscutting modules (e.g. a
  release cut spanning packages) here.
-->

A federation root owns no modules of its own; work lives in the child plans.

| Module | ID  | Owner | Status | Priority | Tags | Dependencies |
| ------ | --- | ----- | ------ | -------- | ---- | ------------ |

## Risks & Mitigations

| Risk               | Impact | Likelihood | Mitigation          |
| ------------------ | ------ | ---------- | ------------------- |
| [Risk description] | high   | medium     | [How we address it] |

## Decisions

- **D-001:** [Root-level decision] — [rationale]
