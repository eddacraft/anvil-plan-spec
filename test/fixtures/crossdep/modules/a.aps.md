# Module A

| ID  | Owner | Status   |
| --- | ----- | -------- |
| A   | @test | Complete |

## Purpose

Fixture: provides work item and decision IDs referenced from module B.

## Work Items

### A-001: Foundation item

- **Intent:** Provide a cross-module dependency target
- **Expected Outcome:** B-001 can depend on this
- **Validation:** `true`
- **Status:** Complete

## Decisions

- **D-001:** Cross-file decision reference target — _decided: yes_
