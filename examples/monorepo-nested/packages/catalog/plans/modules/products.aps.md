# Products Module

| ID   | Owner    | Priority | Status |
| ---- | -------- | -------- | ------ |
| PROD | @catalog | high     | Ready  |

**Last reviewed:** 2026-07-01

## Purpose

Store product records and answer lookups by SKU for dependent packages.

## In Scope

- Product record storage
- Lookup by SKU

## Out of Scope

- Pricing and promotions

## Interfaces

**Depends on:**

- None (base package)

**Exposes:**

- `lookupBySku` — used by dependent packages

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one work item defined

## Work Items

### PROD-001: Implement product lookup by SKU

- **Intent:** Return a product record for a valid SKU
- **Expected Outcome:** `lookupBySku(sku)` returns the matching product
- **Validation:** `npm test -- products.test.ts`
- **Confidence:** high
