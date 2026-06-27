<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- This document is non-executable. -->
<!-- Fixture: nested-plans CHILD (api). Standalone plan; bare IDs within this tree. -->

# API Package

## Overview

HTTP surface and request handlers. Depends on `core` for authentication via a
cross-tree reference. Stands alone as a complete plan.

## Problem & Success Criteria

**Problem:** The platform needs an HTTP layer that authenticates against the
shared `core` auth primitives.

**Success Criteria:**

- [ ] Protected routes verify tokens via `core`
- [ ] Package lints and tests in isolation

## Constraints

- Auth logic is not reimplemented here; it is consumed from `core`

## Modules

| Module                                | ID   | Owner | Status | Priority | Tags | Dependencies |
| ------------------------------------- | ---- | ----- | ------ | -------- | ---- | ------------ |
| [handlers](./modules/handlers.aps.md) | HND  | @api  | Ready  | high     | api  | core:AUTH    |

## Risks & Mitigations

| Risk                       | Impact | Likelihood | Mitigation                       |
| -------------------------- | ------ | ---------- | -------------------------------- |
| Core auth contract changes | high   | low        | Pin the cross-tree dep, version it |

## Decisions

- **D-001:** Reuse `core` auth rather than reimplement — single source of truth

## Open Questions

- [ ] Rate-limiting strategy
