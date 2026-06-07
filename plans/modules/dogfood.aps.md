# Dogfood Module

| ID      | Owner  | Priority | Status      |
| ------- | ------ | -------- | ----------- |
| DOGFOOD | @aneki | high     | In Progress |

**Last reviewed:** 2026-06-08

## Purpose

Make this repository a credible APS example by keeping its own roadmap and
module specs accurate, linked, current, and validated.

## Background

The public roadmap referenced modules that did not exist in `plans/modules/`.
That undermines APS as a planning format: users should be able to inspect this
repo and see the same discipline we recommend elsewhere.

## In Scope

- Every linked module in `plans/index.aps.md` has a corresponding module spec
- Active modules include actionable work items with validation
- Completed modules retain concise historical specs rather than disappearing
- Plan updates are included with changes that affect APS templates, prompts,
  examples, installer behavior, or validation behavior
- Markdown linting remains the baseline verification for plan edits

## Out of Scope

- Rewriting historical completed modules in exhaustive detail
- Replacing the roadmap with generated output
- Implementing ORCH automation before ORCH work is selected

## Interfaces

**Depends on:**

- VAL — plan files must be lintable

**Exposes:**

- `plans/index.aps.md` as the authoritative roadmap
- `plans/modules/*.aps.md` as the complete module registry

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] Decisions resolved
- [x] Work items defined with validation

## Work Items

### DOGFOOD-001: Reconcile roadmap module links — Complete 2026-06-07

- **Intent:** Remove broken plan references from the public roadmap
- **Expected Outcome:** Every markdown link in the Modules tables points to an
  existing module spec, and every active module has a current status.
- **Validation:** `./bin/aps lint plans && npx markdownlint-cli "plans/**/*.md"`
- **Files:** plans/index.aps.md, plans/modules/\*.aps.md
- **Confidence:** high
- **Results:** Full code-vs-plan reconciliation 2026-06-07: all index links
  resolve; dogfood module added to the index (was the only unlisted module);
  REL-001 marked Complete (landed via COMPOUND-003); ORCH-006 given explicit
  Draft status; ORCH interface/file-name drift fixed; stale cross-module
  status references refreshed. Validation passes (22 files, 0 errors).

### DOGFOOD-002: Plan hygiene + completion audit — Complete 2026-06-08

- **Intent:** Make stale plans, broken links, and overstated completions
  detectable before they drift again.
- **Background:** [anvil-001](https://github.com/eddacraft/anvil-001) ran a
  manual APS completion audit on 2026-04-06 across 11 modules and 191 work
  items. It found 8 overstated completions (marked Complete but failing
  their own validation), 19 understated items (marked Draft with full
  implementations already shipped), and stale metadata where module
  headers no longer matched reality. The same audit pattern, formalized,
  would make these drifts cheap to catch.
- **Expected Outcome:** Two-layer hygiene:
  1. **Static checks** (lint-level, fast) — extend `lib/lint.sh` and
     `lib/Lint.psm1` to flag:
     - Module link in `index.aps.md` pointing to a non-existent file
     - Module Status `Ready` / `In Progress` with no `Last reviewed:`
       field, or `Last reviewed:` older than 60 days (configurable)
     - Work item missing Validation while Status is `In Progress` or
       `Complete`
     - Dependencies referencing non-existent IDs across the entire plan
       tree (current W003 only checks intra-file)
  2. **Audit command** (deeper, optional) — `aps audit [module]` runs the
     anvil-001 audit pattern: for each `Complete` work item, attempt to
     resolve its Validation command and report PASS / FAIL / PARTIAL. For
     each `Draft` item whose Files exist with substantive content, flag as
     "understated". For each `Ready` item with no recent `Last reviewed:`,
     flag as "stale". Output a structured report (human + JSON).

- **Validation:** Audit fixtures cover each finding category (overstated,
  understated, stale, broken-link). `aps lint plans` in this repo continues
  to pass. Running `aps audit` against a deliberately broken fixture
  exits non-zero with the right finding codes.
- **Learning:** "Backtick spans in Validation prose are often paths, not
  commands — audit must resolve the executable before running, else false
  A001s drown real findings"
- **Files:** lib/lint.sh, lib/Lint.psm1, lib/audit.sh, lib/rules/,
  bin/aps, test/fixtures/audit/, docs/usage.md, docs/workflow.md
- **Confidence:** medium
- **Dependencies:** DOGFOOD-001, ORCH-001 (reuses the existing work-item
  parser)
- **Status:** Complete: 2026-06-08
- **Action plan:** [execution/DOGFOOD-002.actions.md](../execution/DOGFOOD-002.actions.md)
- **Results:** Static layer: W017 (missing/stale Last reviewed), W018
  (unauditable completion in active module), W019 (broken index link), and
  cross-file W003 resolution — bash + PowerShell engines. Audit layer:
  `aps audit [module]` in new `lib/audit.sh` (not lib/orchestrate.sh as
  planned — kept separate, reuses its parser) with A001–A004 findings,
  PASS/FAIL/PARTIAL verification, `--json`/`--no-run`/`--stale-days`.
  Deviations per D-001/D-002 below. Dogfooded against this repo: initial
  run surfaced 40 findings; after fixing real drift (TUI-001 validation
  command, missing Last reviewed fields) and two false-positive classes,
  the repo audits clean (60 items, 0 findings) and `plans/` lints with
  zero warnings (was 13). 7 new suite tests (22–28); fixtures under
  `test/fixtures/audit/` and `test/fixtures/crossdep/`.

### DOGFOOD-003: Add contribution guidance for APS plan updates — Ready

- **Intent:** Make plan updates part of normal repo contribution hygiene
- **Expected Outcome:** `AGENTS.md` and contributor docs say when APS plan files
  must be updated, how to mark work item status, and what validation to run.
- **Validation:** `npx markdownlint-cli "AGENTS.md" "CONTRIBUTING.md" "docs/**/*.md"`
- **Files:** AGENTS.md, CONTRIBUTING.md, docs/workflow.md
- **Confidence:** high
- **Dependencies:** DOGFOOD-001

## Decisions

- **D-001:** W018 severity and scope — _decided: warning only, and only
  inside still-active modules. The spec asked to flag missing Validation on
  Complete items, but E005 had just been amended to exempt terminal items so
  closeout compaction doesn't reopen errors. Fully Complete modules are
  archives (exempt); a Complete item in an active module warns because the
  audit cannot verify it._
- **D-002:** Broken index links are W019 (warning), not an error — _decided:
  the scaffold seed index intentionally links a placeholder module file the
  user creates later; an error-level check would make every fresh `aps init`
  project fail lint. `aps audit` reports the same condition as A004 with a
  non-zero exit, which is the hard gate where it matters._
- **D-003:** Audit only executes resolvable Validation commands — _decided:
  the first backtick span in Validation prose is often a path or example,
  not a command. If the first word doesn't resolve to an executable, report
  PARTIAL (unverifiable) rather than FAIL (overstated). Same reasoning for
  Ready items in Draft/Blocked modules: not actionable, so not stale (A003
  only fires in Ready / In Progress modules)._
