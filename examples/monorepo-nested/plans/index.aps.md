<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- This document is non-executable. -->
<!--
  Worked example: a federated nested-plans monorepo (MONO-006). Two packages
  with independent owners each keep a standalone plan; this root links them and
  rolls up their status. See docs/monorepo.md (Nested Plans) and the
  nested-plans design doc.
-->

# Shop Platform

## Overview

A two-package storefront monorepo. `catalog` owns product data and search;
`storefront` owns the customer-facing web app and depends on `catalog` for
product lookups. Each package ships on its own cadence, so each keeps a
standalone plan. This root links the child plans and rolls up their status; it
owns no modules of its own.

## Problem & Success Criteria

**Problem:** `catalog` and `storefront` have separate owners and release
cadences, so a single shared backlog forces unrelated work into one queue.

**Success Criteria:**

- [ ] Each package plan lints and ships independently
- [ ] Cross-package work is traceable via cross-tree references

## Constraints

- Packages stay independently releasable
- Storefront consumes catalog through a versioned contract, not internals

## Child Plans

- [catalog](../packages/catalog/plans/index.aps.md) — product data and search
- [storefront](../packages/storefront/plans/index.aps.md) — customer web app

## Roll-up

<!-- Regenerate with `aps rollup --plans examples/monorepo-nested/plans`. -->

| Child      | Modules (complete/total) | Next ready item | Status |
| ---------- | ------------------------ | --------------- | ------ |
| catalog    | 0/1                      | PROD-001        | Ready  |
| storefront | 0/1                      | —               | Ready  |

## Modules

A federation root owns no modules of its own; work lives in the child plans.

| Module | ID  | Owner | Status | Priority | Tags | Dependencies |
| ------ | --- | ----- | ------ | -------- | ---- | ------------ |

## Risks & Mitigations

| Risk                          | Impact | Likelihood | Mitigation                              |
| ----------------------------- | ------ | ---------- | --------------------------------------- |
| Cross-tree refs drift untyped | medium | medium     | Lint traversal resolves them (MONO-002) |

## Decisions

- **D-001:** Adopt nested plans over tagged modules — packages have independent
  owners and release cadences
