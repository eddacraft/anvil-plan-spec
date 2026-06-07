# Demo Module

| ID   | Owner | Status      |
| ---- | ----- | ----------- |
| DEMO | @test | In Progress |

**Last reviewed:** 2026-01-01

## Purpose

Audit fixture covering every finding class.

## Work Items

### DEMO-001: Honest completion

- **Intent:** Complete item whose validation passes
- **Expected Outcome:** Audit reports PASS, no finding
- **Validation:** `true`
- **Status:** Complete

### DEMO-002: Overstated completion

- **Intent:** Complete item whose validation fails
- **Expected Outcome:** Audit reports FAIL with finding A001
- **Validation:** `false`
- **Status:** Complete

### DEMO-003: Unverifiable completion

- **Intent:** Complete item with prose-only validation
- **Expected Outcome:** Audit reports PARTIAL, no finding
- **Validation:** Manual verification by a human
- **Status:** Complete

### DEMO-004: Understated draft

- **Intent:** Draft item whose file already exists with content
- **Expected Outcome:** Audit reports finding A002
- **Validation:** `true`
- **Files:** test/fixtures/audit/existing-artifact.txt
- **Status:** Draft

### DEMO-005: Stale ready item

- **Intent:** Ready item in a module reviewed long ago
- **Expected Outcome:** Audit reports finding A003
- **Validation:** `true`
- **Status:** Ready

### DEMO-006: Completion with unresolvable command

- **Intent:** Complete item whose backtick span is a path, not a command
- **Expected Outcome:** Audit reports PARTIAL (command not found), no finding
- **Validation:** Inspect `src/made-up-path/` for the generated artifacts
- **Status:** Complete
