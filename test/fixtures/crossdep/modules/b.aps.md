# Module B

| ID  | Owner | Status |
| --- | ----- | ------ |
| B   | @test | Draft  |

## Purpose

Fixture: dependencies resolve across files (no W003 expected) except one
deliberately unknown ID (W003 expected).

## Work Items

### B-001: Dependent item

- **Intent:** Depend on IDs defined in module A
- **Expected Outcome:** Cross-file dependency resolution succeeds
- **Validation:** `true`
- **Dependencies:** A-001, D-001

### B-002: Item with unknown dependency

- **Intent:** Keep truly missing IDs detectable
- **Expected Outcome:** W003 still fires for GHOST-999
- **Validation:** `true`
- **Dependencies:** GHOST-999
