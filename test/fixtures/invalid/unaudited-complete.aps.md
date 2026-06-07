# Unaudited Module

| ID  | Owner | Status      |
| --- | ----- | ----------- |
| UNA | @test | In Progress |

**Last reviewed:** 2099-01-01

## Purpose

Fixture: Complete work item with no Validation inside an active module.

## Work Items

### UNA-001: Shipped without proof — Complete 2026-06-01

- **Intent:** Trigger W018 via missing Validation on a terminal item
- **Expected Outcome:** Linter flags the completion as unauditable
- **Status:** Complete

### UNA-002: Properly closed item

- **Intent:** Control item that must not trigger W018
- **Expected Outcome:** No warning for validated completion
- **Validation:** `true`
- **Status:** Complete
