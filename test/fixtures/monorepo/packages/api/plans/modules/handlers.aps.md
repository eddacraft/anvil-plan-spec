# Request Handlers Module

| ID  | Owner | Priority | Status |
| --- | ----- | -------- | ------ |
| HND | @api  | high     | Ready  |

**Last reviewed:** 2026-06-27

## Purpose

HTTP request handlers for the API surface, with auth enforced via `core`.

## In Scope

- Route handlers
- Auth middleware wiring

## Out of Scope

- Token issuance (owned by `core`)

## Interfaces

**Depends on:**

- core:AUTH — token verification (cross-tree reference)

**Exposes:**

- `apiRouter` — mounted by the server entrypoint

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one work item defined

## Work Items

### HND-001: Protect routes with core auth

- **Intent:** Verify session tokens on protected routes using `core`
- **Expected Outcome:** Requests without a valid token receive 401
- **Validation:** `npm test -- handlers.test.ts`
- **Confidence:** high
- **Dependencies:** core:AUTH-001
