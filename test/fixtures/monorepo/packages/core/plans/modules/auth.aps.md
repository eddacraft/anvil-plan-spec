# Authentication Module

| ID   | Owner | Priority | Status |
| ---- | ----- | -------- | ------ |
| AUTH | @core | high     | Ready  |

**Last reviewed:** 2026-06-27

## Purpose

Issue and verify session tokens for the platform.

## In Scope

- Login / logout
- Token issuance and verification

## Out of Scope

- Social login

## Interfaces

**Depends on:**

- None (base package)

**Exposes:**

- `verifyToken` — used by dependent packages

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one work item defined

## Work Items

### AUTH-001: Implement token issuance

- **Intent:** Issue a signed session token on successful login
- **Expected Outcome:** `issueToken(user)` returns a verifiable JWT
- **Validation:** `npm test -- auth.test.ts`
- **Confidence:** high
