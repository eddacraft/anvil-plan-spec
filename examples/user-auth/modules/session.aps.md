<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- Executable only if work items exist and status is Ready. -->

# Session Module

| ID      | Owner | Priority | Status |
| ------- | ----- | -------- | ------ |
| SESSION | @josh | high     | Draft  |

## Purpose

Manage user sessions using JWT tokens. Issue access tokens on login, validate tokens on protected routes, and handle token refresh.

## In Scope

- JWT access token issuance
- Token validation middleware
- Refresh token storage and rotation

## Out of Scope

- User registration/login logic (see AUTH module)
- API route definitions

## Interfaces

**Depends on:**

- AUTH — `verifyCredentials()` for login
- Database — refresh_tokens table

**Exposes:**

- `login(email, password)` → { accessToken, refreshToken }
- `validateToken(token)` → User | null
- `refreshSession(refreshToken)` → { accessToken, refreshToken }

## Boundary Rules

- SESSION depends on AUTH, not vice versa
- SESSION must not access password hashes directly

## Acceptance Criteria

- [ ] Access tokens expire in 1 hour
- [ ] Refresh tokens expire in 7 days
- [ ] Refresh tokens are rotated on use
- [ ] Invalid tokens return 401

## Risks & Mitigations

| Risk                | Mitigation                                  |
| ------------------- | ------------------------------------------- |
| Token theft         | Short access token expiry, httpOnly cookies |
| Refresh token reuse | Rotate on each use, detect reuse            |

## Work Items

> **Status: Draft** — Blocked on AUTH module completion

No work items authorised. Blockers:

- [ ] AUTH-001 and AUTH-002 must be complete
- [ ] Decision needed on refresh token storage schema

## Decisions

(none yet)

## Notes

- Will need to coordinate with API module on middleware integration
