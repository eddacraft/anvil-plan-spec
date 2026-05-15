# Authentication Architecture

| Field   | Value                                                                |
| ------- | -------------------------------------------------------------------- |
| Status  | Approved                                                             |
| Created | 2025-01-05                                                           |
| Modules | [AUTH](../modules/auth.aps.md), [SESSION](../modules/session.aps.md) |

## Problem

The application has no user authentication. We need to add registration, login,
and session management without introducing new infrastructure (no Redis). The
solution must be secure (bcrypt, httpOnly cookies) and work with the existing
PostgreSQL database.

## Constraints

- Existing PostgreSQL database (no new stores)
- No additional infrastructure in v1 (no Redis, no external auth service)
- Password hashing: bcrypt with cost factor 12+
- Must support both API clients and browser-based sessions

## Design

### Token Strategy: JWT + Refresh Tokens

```
Client → POST /login → Server validates credentials
                       → Issues short-lived access token (JWT, 1hr)
                       → Issues long-lived refresh token (DB-backed, 7d)
                       → Returns both in httpOnly cookies

Client → GET /protected → Server validates access token (stateless)
                          → If expired, client uses refresh token
                          → Server issues new access token
```

**Access tokens** are stateless JWTs — no database lookup on every request.
This keeps the auth middleware fast and avoids Redis.

**Refresh tokens** are stored in PostgreSQL. This allows revocation (logout,
password change) without a separate store.

### Module Boundaries

| Module  | Responsibility                                          | Exposes                                                |
| ------- | ------------------------------------------------------- | ------------------------------------------------------ |
| AUTH    | Registration, credential verification, password hashing | `register()`, `verify()`                               |
| SESSION | Token creation, validation, refresh, revocation         | `createSession()`, `validateToken()`, `refreshToken()` |

AUTH handles identity (who you are). SESSION handles access (proving you're
authenticated). This separation means we can swap token strategies later without
touching the auth logic.

### Database Schema

Two new tables:

- `users` — email, password_hash, created_at
- `refresh_tokens` — token_hash, user_id, expires_at, revoked_at

## Alternatives Considered

| Alternative                            | Pros                    | Cons                                    | Verdict                                     |
| -------------------------------------- | ----------------------- | --------------------------------------- | ------------------------------------------- |
| Session cookies (server-side)          | Simple, easy revocation | Requires Redis or DB lookup per request | Rejected — adds infrastructure              |
| OAuth-only (delegate to Google/GitHub) | No password management  | Not all users have OAuth accounts       | Deferred to v2                              |
| Argon2 instead of bcrypt               | More modern, tunable    | Less library support in our stack       | Rejected — bcrypt is proven, well-supported |

## Implementation Notes

The AUTH module should be implemented first (AUTH-001, AUTH-002) since SESSION
depends on the user verification functions. Session module work items should
not start until AUTH is Ready.

## Decisions

- **D-001:** JWT for access tokens — stateless validation, no Redis needed
- **D-002:** DB-backed refresh tokens — allows revocation without Redis
- **D-003:** bcrypt over argon2 — better library support in our stack
- **D-004:** httpOnly cookies for token transport — prevents XSS token theft
