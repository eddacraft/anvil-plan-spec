# Authentication Module

| ID   | Owner | Priority | Status |
| ---- | ----- | -------- | ------ |
| AUTH | @test | high     | Ready  |

## Purpose

Handle user authentication including login, logout, and session management.

## In Scope

- Login/logout functionality
- Session token management
- Password reset flow

## Out of Scope

- Social login (separate module)
- Two-factor authentication (v2)

## Interfaces

**Depends on:**

- CORE — database access

**Exposes:**

- `authMiddleware` — protects routes
- `AuthContext` — React context for auth state

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one work item defined

## Work Items

### AUTH-001: Implement login endpoint

- **Intent:** Allow users to authenticate with email/password
- **Expected Outcome:** POST /api/auth/login returns JWT on success
- **Validation:** `npm test -- auth.test.ts`
- **Confidence:** high

### AUTH-002: Add logout functionality

- **Intent:** Allow users to end their session
- **Expected Outcome:** POST /api/auth/logout invalidates token
- **Validation:** `npm test -- auth.test.ts`
- **Dependencies:** AUTH-001
