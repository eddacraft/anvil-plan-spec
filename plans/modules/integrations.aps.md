# Integrations Module

| ID           | Owner  | Priority | Status |
| ------------ | ------ | -------- | ------ |
| INTEGRATIONS | @aneki | medium   | In Progress |

**Last reviewed:** 2026-07-16

## Purpose

Explore low-lock-in integrations that expose APS plans to external workflows
without making external systems the source of truth.

## In Scope

- JSON export from APS markdown
- GitHub issue or project sync experiments
- GitHub Action wrapper around validation and rollup
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

- `aps export --json` command (Rust + bash, parity-bound)
- `action.yml` — composite GitHub Action (lint + optional rollup PR comment)
- `docs/integrations.md` — export shape + Action usage

## Work Items

### INTEGRATIONS-001: Define JSON export shape — In Progress

- **Intent:** Let external tools consume APS without owning APS state
- **Expected Outcome:** Documented JSON shape for index, modules, work items,
  dependencies, statuses, files, and validation commands. Stable key order,
  versioned with a top-level `schema` field so consumers can detect breaks.
- **Validation:** Prototype export against `plans/` and `examples/` produces
  stable JSON suitable for CI or GitHub sync experiments
- **Files:** docs/integrations.md
- **Confidence:** medium

### INTEGRATIONS-002: Implement `aps export --json` — In Progress

- **Intent:** Give non-CLI stakeholders and CI a machine-readable view of a
  plan tree (the substrate for dashboards, PR comments, and sync experiments).
- **Expected Outcome:** `aps export --json` walks the same plan tree the linter
  sees and emits the INTEGRATIONS-001 shape on stdout. Implemented in the Rust
  binary and bash CLI with byte-identical output (D-039; no PowerShell surface,
  matching `next`/`rollup`). Deterministic ordering — file order for modules,
  document order for work items.
- **Validation:** Shared fixtures under `test/fixtures/` exercised by
  `test/cli-parity.sh`; `jq .` round-trips the output; running twice
  byte-matches.
- **Dependencies:** INTEGRATIONS-001
- **Files:** cli/src/export.rs, cli/src/main.rs, bin/aps, lib/export.sh
- **Confidence:** medium

### INTEGRATIONS-003: GitHub Action for lint + rollup — In Progress

- **Intent:** One-line CI adoption for teams: lint plans on every PR and
  optionally post the rollup as a sticky PR comment (completes the second half
  of D-007: "standalone CLI first, then GitHub Action wrapper").
- **Expected Outcome:** A composite `action.yml` at the repo root installs a
  pinned `aps` release binary, runs `aps lint` against the configured plans
  dir, and (opt-in) posts `aps rollup` output as a PR comment. Inputs:
  `version`, `plans-dir`, `rollup-comment`. Documented in
  `docs/integrations.md` alongside the existing `docs/ci-lint-example.yml`.
- **Validation:** Action consumed by this repo's own CI on a PR (dogfood);
  lint failure fails the check; rollup comment renders.
- **Dependencies:** None (rollup comment reuses existing `aps rollup`)
- **Files:** action.yml, docs/integrations.md, .github/workflows/
- **Confidence:** medium

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified (VAL, ORCH)
- [x] Work items defined with validation

## Notes

Promoted Draft → Ready on 2026-07-16 as part of the team-rollout push
(v0.7.0): a new team needs central lint enforcement (Action) and a
machine-readable status surface (export) before plans stay truthful at team
scale. GitHub issue/project sync stays exploratory and has no work item yet.
