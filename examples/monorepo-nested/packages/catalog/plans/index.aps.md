<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- This document is non-executable. -->
<!-- Worked example: CHILD plan (catalog). Standalone; bare IDs within this tree. -->

# Catalog Package

## Overview

Product catalog: the source of truth for product records and search. Consumed
by `storefront` through a versioned lookup contract. Stands alone as a complete
plan.

## Problem & Success Criteria

**Problem:** Product data and search must live in one owned, releasable package
with a stable contract for dependents.

**Success Criteria:**

- [ ] Product lookup is usable by dependent packages
- [ ] Package lints and tests in isolation

## Constraints

- No dependency on sibling packages

## Modules

| Module                              | ID   | Owner    | Status | Priority | Tags    | Dependencies |
| ----------------------------------- | ---- | -------- | ------ | -------- | ------- | ------------ |
| [products](./modules/products.aps.md) | PROD | @catalog | Ready  | high     | catalog | —            |

## Risks & Mitigations

| Risk               | Impact | Likelihood | Mitigation                |
| ------------------ | ------ | ---------- | ------------------------- |
| Lookup contract churn | medium | low        | Version the lookup schema |

## Decisions

- **D-001:** Expose product lookup by SKU — portable across packages

## Open Questions

- [ ] Search relevance tuning strategy
