# Feature With Incomplete Work Items

| ID  | Owner | Status |
| --- | ----- | ------ |
| INC | @test | Draft  |

## Purpose

Test detection of missing required fields in work items.

## Work Items

### INC-001: Missing intent

- **Expected Outcome:** Should trigger E005 for missing Intent
- **Validation:** `echo test`

### INC-002: Missing outcome

- **Intent:** Test missing Expected Outcome
- **Validation:** `echo test`

### INC-003: Missing validation

- **Intent:** Test missing Validation field
- **Expected Outcome:** Should trigger E005
