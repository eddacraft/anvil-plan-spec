# Feature With Bad Task IDs

| ID   | Owner | Status |
| ---- | ----- | ------ |
| TEST | @test | Draft  |

## Purpose

Test detection of malformed task IDs.

## Work Items

### Task 1: Wrong format

- **Intent:** Should trigger W001 warning
- **Expected Outcome:** Warning about ID format
- **Validation:** `echo test`

### TEST-1: Missing padding

- **Intent:** Should trigger W001 warning (needs TEST-001)
- **Expected Outcome:** Warning about ID format
- **Validation:** `echo test`
