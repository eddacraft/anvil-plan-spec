# Status Alias Module

| ID    | Owner | Status |
| ----- | ----- | ------ |
| ZSTAT | @test | Ready  |

## Purpose

Validate Done/Proposed status aliases in orchestration.

## Work Items

### ZSTAT-001: Compacted done dependency

- **Status:** Done
- **Summary:** anvil-001-style terminal alias; satisfies dependency checks.

### ZSTAT-002: Ready item behind Done dependency

- **Intent:** Ensure Done normalizes to Complete for dependency resolution
- **Expected Outcome:** `aps next zstatus` returns this item once ZSTAT-001 is Done
- **Validation:** `bash test/orchestrate.sh`
- **Dependencies:** ZSTAT-001
- **Status:** Ready
