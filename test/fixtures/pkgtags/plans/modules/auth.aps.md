# Auth

| ID   | Owner  | Priority | Status   | Packages |
| ---- | ------ | -------- | -------- | -------- |
| AUTH | @aneki | medium   | Complete | core     |

## Purpose

Authentication slice tagged with workspace packages. The module-level tag
(`core`) resolves via `packages/core`; item-level tags mix valid and typo'd
entries to exercise W022.

## Work Items

### AUTH-001: Login

- **Status:** Complete
- **Packages:** api, storefront

### AUTH-002: Session storage

- **Status:** Complete
- **Packages:** packages/core, packages/wrong
