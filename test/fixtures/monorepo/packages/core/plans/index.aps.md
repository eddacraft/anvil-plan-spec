<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- This document is non-executable. -->
<!-- Fixture: nested-plans CHILD (core). Standalone plan; bare IDs within this tree. -->

# Core Package

## Overview

Shared domain model, persistence, and authentication primitives consumed by the
other packages. Stands alone as a complete plan.

## Problem & Success Criteria

**Problem:** Domain and auth logic must live in one owned, releasable package.

**Success Criteria:**

- [ ] Auth primitives are usable by dependent packages
- [ ] Package lints and tests in isolation

## Constraints

- No dependency on sibling packages

## Modules

| Module                       | ID   | Owner | Status | Priority | Tags | Dependencies |
| ---------------------------- | ---- | ----- | ------ | -------- | ---- | ------------ |
| [auth](./modules/auth.aps.md) | AUTH | @core | Ready  | high     | core | —            |

## Risks & Mitigations

| Risk                  | Impact | Likelihood | Mitigation              |
| --------------------- | ------ | ---------- | ----------------------- |
| Token format churn    | medium | low        | Version the token claim |

## Decisions

- **D-001:** JWT for session tokens — portable across packages

## Open Questions

- [ ] Refresh-token rotation policy
