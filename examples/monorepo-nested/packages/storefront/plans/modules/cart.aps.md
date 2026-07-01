# Cart Module

| ID   | Owner       | Priority | Status |
| ---- | ----------- | -------- | ------ |
| CART | @storefront | high     | Ready  |

**Last reviewed:** 2026-07-01

## Purpose

Manage the shopping cart, resolving line items to products via `catalog`.

## In Scope

- Add/remove cart items
- Resolve line items to products through `catalog`

## Out of Scope

- Product storage (owned by `catalog`)

## Interfaces

**Depends on:**

- catalog:PROD — product lookup (cross-tree reference)

**Exposes:**

- `cartApi` — mounted by the storefront app

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one work item defined

## Work Items

### CART-001: Resolve cart line items via catalog

- **Intent:** Turn cart SKUs into product records using `catalog`
- **Expected Outcome:** A cart renders product name and price for each line
- **Validation:** `npm test -- cart.test.ts`
- **Confidence:** high
- **Dependencies:** catalog:PROD-001
