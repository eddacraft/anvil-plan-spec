# Feature With Compacted Done Items

| ID   | Owner | Status |
| ---- | ----- | ------ |
| DONE | @test | Ready  |

## Purpose

Terminal (completed) work items are commonly compacted to Status + a short
summary once shipped; their full Intent/Expected Outcome/Validation detail lives
in version history. Such items must not trigger E005.

## Work Items

### DONE-001: Compacted completed item (no required fields)

- **Status:** Done
- **Summary:** Shipped; detail in history. Exempt from E005.

### DONE-002: Completed via merge

- **Status:** Merged 2026-05-26 via PR #123
- **Summary:** Landed on main; compacted.

### DONE-003: Released item

- **Status:** Released/Shipped via v0.7.0-beta
- **Summary:** Cut in the v0.7.0-beta release.

### DONE-004: Active item still requires fields

- **Status:** Ready
- **Intent:** Active work items keep the required fields.
- **Expected Outcome:** E005 still fires for active items missing fields.
- **Validation:** `aps lint` passes for this file.
