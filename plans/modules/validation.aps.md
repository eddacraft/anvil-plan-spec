# Validation Module

| ID  | Owner  | Priority | Status   |
| --- | ------ | -------- | -------- |
| VAL | @aneki | high     | Complete |

## Purpose

Provide a lightweight CLI linter that validates APS markdown structure in local
development and CI.

## In Scope

- `aps lint` command
- POSIX shell and PowerShell wrappers
- Validation fixtures for valid and invalid APS documents
- Markdown structure checks for required sections and IDs

## Out of Scope

- Full semantic dependency resolution
- JSON Schema export
- Hosted validation service

## Interfaces

**Exposes:**

- `bin/aps`
- `bin/aps.ps1`
- `lib/lint.sh`
- `lib/Lint.psm1`
- `test/fixtures/`

## Work Items

### VAL-001: Implement APS lint command — Complete

- **Intent:** Catch malformed APS documents before review or merge
- **Expected Outcome:** CLI validates required APS metadata, structure, and work
  item IDs against fixture coverage.
- **Validation:** `./bin/aps lint test/fixtures/valid`
- **Files:** bin/, lib/, test/fixtures/
- **Confidence:** high
