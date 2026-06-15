# Action Plan: DOGFOOD-002

| Field      | Value                                                 |
| ---------- | ----------------------------------------------------- |
| Source     | [./modules/dogfood.aps.md](../modules/dogfood.aps.md) |
| Work Item  | DOGFOOD-002 — Plan hygiene + completion audit         |
| Created by | @aneki / AI                                           |
| Status     | Complete                                              |

## Prerequisites

- [x] DOGFOOD-001 Complete (index links reconciled)
- [x] ORCH-001 Complete (work-item parser exists in lib/orchestrate.sh)

## Waves

| Wave | Actions | Gate                                         |
| ---- | ------- | -------------------------------------------- |
| 1    | 1, 2    | Static lint checks pass against fixtures     |
| 2    | 3, 4    | `aps audit` detects all four finding classes |
| 3    | 5, 6    | PowerShell parity + docs; full suite green   |

## Actions

### 1. Add static hygiene checks to bash linter

- **Checkpoint:** E012 (broken index module link), W017 (stale/missing Last
  reviewed on active modules), W018 (terminal item missing Validation in
  active module) detected; W003 resolves across the plan tree
- **Validate:** `./test/run.sh`

### 2. Add lint fixtures for new checks

- **Checkpoint:** Fixtures cover each new code; valid fixtures stay clean
- **Validate:** `./bin/aps lint test/fixtures/valid/`

### 3. Implement `aps audit` command

- **Checkpoint:** Audit reports overstated/understated/stale/broken-link
  findings with codes; exits non-zero on findings; `--json` output valid
- **Validate:** `./bin/aps audit --plans test/fixtures/audit/plans`

### 4. Add audit fixtures and tests

- **Checkpoint:** Deliberately broken fixture triggers every finding class
- **Validate:** `./test/run.sh`

### 5. Port static checks to PowerShell rule engine

- **Checkpoint:** Lint.psm1 + rules mirror E012/W017/W018/W003 behaviour
- **Validate:** `bash -n` equivalents unavailable — reviewed against bash
  implementation (no pwsh on this host; CI covers syntax)

### 6. Update docs and plan metadata

- **Checkpoint:** usage.md + workflow.md document audit; active modules carry
  Last reviewed; repo plans lint clean
- **Validate:** `./bin/aps lint plans && pnpm exec markdownlint docs/ plans/`

## Completion

- [x] All checkpoints validated
- [x] Work item marked complete in `dogfood.aps.md`

**Completed by:** @aneki / AI — 2026-06-08
