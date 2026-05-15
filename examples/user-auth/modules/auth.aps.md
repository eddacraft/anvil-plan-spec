<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- Executable only if work items exist and status is Ready. -->

# Authentication Module

| ID   | Owner | Priority | Status |
| ---- | ----- | -------- | ------ |
| AUTH | @josh | high     | Ready  |

## Purpose

Handle user registration and credential verification. This module owns password hashing, user creation, and login validation.

## In Scope

- User registration (email + password)
- Password hashing and verification
- Login credential validation
- User lookup by email

## Out of Scope

- Session management (see SESSION module)
- OAuth/social login (future work)
- Password reset flow (future work)

## Interfaces

**Depends on:**

- Database — user table with email, password_hash columns

**Exposes:**

- `registerUser(email, password)` → User
- `verifyCredentials(email, password)` → User | null

## Boundary Rules

- AUTH must not depend on SESSION
- AUTH must not issue tokens (that's SESSION's job)

## Acceptance Criteria

- [ ] Passwords are hashed with bcrypt (cost ≥12)
- [ ] Duplicate emails are rejected
- [ ] Invalid credentials return null, not an error
- [ ] No plaintext passwords in logs

## Risks & Mitigations

| Risk                    | Mitigation                       |
| ----------------------- | -------------------------------- |
| Timing attacks on login | Use constant-time comparison     |
| Weak passwords          | Enforce minimum length (8 chars) |

## Work Items

### AUTH-001: Create user registration function

- **Intent:** Allow new users to register with email and password
- **Expected Outcome:** `registerUser()` creates user record with hashed password
- **Scope:** New `auth.ts` module, user table migration
- **Non-scope:** Login, sessions, API routes
- **Files:** `src/auth/auth.ts`, `migrations/001_users.sql`
- **Dependencies:** None
- **Validation:** `npm test -- auth.test.ts`
- **Confidence:** high
- **Risks:** None significant

### AUTH-002: Create credential verification function

- **Intent:** Verify email/password combinations for login
- **Expected Outcome:** `verifyCredentials()` returns user if valid, null if not
- **Scope:** Add function to `auth.ts`
- **Non-scope:** Session creation, token issuance
- **Files:** `src/auth/auth.ts`
- **Dependencies:** AUTH-001
- **Validation:** `npm test -- auth.test.ts`
- **Confidence:** high
- **Risks:** Must use constant-time comparison

## Execution

Action Plan: [../execution/AUTH-001.actions.md](../execution/AUTH-001.actions.md)

## Decisions

- **D-001:** Use bcrypt over argon2 — better library support in Node.js

## Notes

- Consider adding rate limiting at API layer (not this module's concern)
