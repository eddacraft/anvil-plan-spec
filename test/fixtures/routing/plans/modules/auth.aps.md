# Auth

| ID   | Owner  | Priority | Status      | Model         | Reasoning |
| ---- | ------ | -------- | ----------- | ------------- | --------- |
| AUTH | @aneki | medium   | In Progress | claude-opus-5 | high      |

**Last reviewed:** 2026-09-07

## Purpose

Module-level routing hints; AUTH-001 inherits both (no item fields), AUTH-002
overrides both at item level (mixed case is accepted).

## Work Items

### AUTH-001: Login

- **Intent:** i
- **Expected Outcome:** o
- **Validation:** v
- **Status:** Ready

### AUTH-002: Session storage

- **Intent:** i
- **Expected Outcome:** o
- **Validation:** v
- **Status:** Ready
- **Dependencies:** AUTH-001
- **Model:** claude-sonnet-5
- **Reasoning:** Low
