# Proposed Module

| ID    | Owner | Status   |
| ----- | ----- | -------- |
| PROP  | @test | Proposed |

## Purpose

Validate Proposed module status alias (maps to Draft; not actionable).

## Work Items

### PROP-001: Ready item in proposed module

- **Intent:** Ready work items in Proposed modules must not surface in `aps next`
- **Expected Outcome:** `aps next proposed` reports no ready item
- **Validation:** `bash test/orchestrate.sh`
- **Status:** Ready
