# Validation Module

| ID  | Owner  | Priority | Status      |
| --- | ------ | -------- | ----------- |
| VAL | @aneki | high     | In Progress |

**Last reviewed:** 2026-07-16

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

### VAL-002: Fence-aware shared parser helpers — Ready

- **Intent:** Close ISS-001 — work-item headers inside fenced code blocks are
  parsed as real items by the shared helpers (`get_work_items` and friends),
  producing false E005s and phantom `aps next` / `aps graph` entries. A new
  team's first plan with a code example hits this on day one.
- **Expected Outcome:** All shared parser helpers skip fenced regions
  (``` and ~~~, matching the fence-awareness `build_id_index` gained in
  DOGFOOD-002) in all three CLIs (D-039). Lint, next, graph, rollup, and
  export see identical item sets with fences present or absent.
- **Validation:** Shared fixture with a fenced fake work item added to
  `test/fixtures/` and wired into `test/cli-parity.sh`; cargo test + bash +
  pwsh suites green; ISS-001 closed in `plans/issues.md`.
- **Dependencies:** None
- **Files:** lib/lint.sh, lib/orchestrate.sh, lib/Lint.psm1, cli/src/parser.rs,
  test/fixtures/
- **Confidence:** high
