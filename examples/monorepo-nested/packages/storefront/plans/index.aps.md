<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- This document is non-executable. -->
<!-- Worked example: CHILD plan (storefront). Standalone; bare IDs within this tree. -->

# Storefront Package

## Overview

Customer-facing web app: browsing, cart, and checkout. Depends on `catalog`
for product lookups via a cross-tree reference. Stands alone as a complete plan.

## Problem & Success Criteria

**Problem:** Customers need a web app that renders products sourced from the
shared `catalog` package.

**Success Criteria:**

- [ ] Cart resolves products via `catalog`
- [ ] Package lints and tests in isolation

## Constraints

- Product data is not reimplemented here; it is consumed from `catalog`

## Modules

| Module                        | ID   | Owner       | Status | Priority | Tags       | Dependencies |
| ----------------------------- | ---- | ----------- | ------ | -------- | ---------- | ------------ |
| [cart](./modules/cart.aps.md) | CART | @storefront | Ready  | high     | storefront | catalog:PROD |

## Risks & Mitigations

| Risk                          | Impact | Likelihood | Mitigation                          |
| ----------------------------- | ------ | ---------- | ----------------------------------- |
| Catalog lookup contract changes | high   | low        | Pin the cross-tree dep, version it |

## Decisions

- **D-001:** Reuse `catalog` lookups rather than cache product data — single
  source of truth

## Open Questions

- [ ] Guest vs authenticated cart persistence
