# Integrations Module

| ID           | Owner  | Priority | Status |
| ------------ | ------ | -------- | ------ |
| INTEGRATIONS | @aneki | low      | Draft  |

## Purpose

Explore low-lock-in integrations that expose APS plans to external workflows
without making external systems the source of truth.

## In Scope

- JSON export from APS markdown
- GitHub issue or project sync experiments
- GitHub Action wrapper around validation
- Read-only integration patterns that preserve markdown as canonical

## Out of Scope

- Jira, Linear, or Notion plugins
- Hosted sync service
- Bidirectional sync that can overwrite APS markdown without review

## Interfaces

**Depends on:**

- VAL — export and action checks need parser behavior
- ORCH — status/dependency commands may provide structured data later

**Exposes:**

- Future `aps export` command
- Future GitHub Action documentation

## Work Items

### INTEGRATIONS-001: Define JSON export shape — Draft

- **Intent:** Let external tools consume APS without owning APS state
- **Expected Outcome:** Documented JSON shape for index, modules, work items,
  dependencies, statuses, files, and validation commands.
- **Validation:** Prototype export against `plans/` and `examples/` produces
  stable JSON suitable for CI or GitHub sync experiments
- **Files:** docs/, bin/aps, lib/lint.sh, lib/Lint.psm1
- **Confidence:** low
