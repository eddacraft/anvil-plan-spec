# CI Cross-Implementation Parity

| ID  | Owner  | Priority | Status |
| --- | ------ | -------- | ------ |
| CIP | @aneki | medium   | Ready  |

**Last reviewed:** 2026-07-10

## Purpose

The `aps` CLI ships three implementations that must stay in lockstep (index
D-038/D-039): the canonical Rust binary, the feature-frozen bash CLI, and the
PowerShell fallback. CI enforces this only by **string-matching** the ported
surface (grepping `test/run.sh` for function names and lint codes) because the
CI image carries no `pwsh` and the Rust behavioural suite lives in `cargo test`.
String guards prove a function _exists_, not that it _behaves_ — a
behaviourally-broken PowerShell linter passes them. COND-007 demonstrated the
cost: `Get-ApsStatus` had silently mis-parsed spaced `| --- |` separators for
an unknown length of time (W005/W017/W018 never fired in PowerShell), and a
W017 rounding drift went unnoticed — both invisible until a `pwsh` was fetched
by hand and run against the fixtures. This module closes that gap by running
the PowerShell and cross-CLI checks behaviourally in CI.

## In Scope

- Installing PowerShell in the CI test job and executing `bin/aps.ps1`
  against the fixture corpus.
- A cross-CLI harness that asserts bash, Rust, and PowerShell emit
  byte-identical lint findings over `test/fixtures/`.

## Out of Scope

- New lint rules or CLI features (they belong to their own modules).
- Reworking the string-parity guards in `test/run.sh` — they stay as a fast,
  pwsh-free smoke check; behavioural execution supplements them.

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified (none blocking CIP-001)
- [x] At least one work item defined

## Work Items

### CIP-001: Run the PowerShell linter behaviourally in CI

- **Intent:** Catch PowerShell behavioural regressions in CI instead of only
  string-guarding the ported surface.
- **Expected Outcome:** The PR test workflow installs `pwsh` and runs
  `bin/aps.ps1 lint` over `test/fixtures/` (and the valid/invalid corpora),
  asserting expected findings and exit codes — the same coverage the bash CLI
  already gets in `test/run.sh`.
- **Validation:** CI job green on `main`; reintroducing the COND-007
  `Get-ApsStatus` separator bug (or dropping a ported rule) turns the job red.
- **Confidence:** high
- **Dependencies:** none
- **Status:** Ready

### CIP-002: Cross-CLI byte-diff parity harness

- **Intent:** Replace per-rule string guards with a structural check that all
  three CLIs produce identical lint output.
- **Expected Outcome:** A harness runs bash, the Rust binary, and `pwsh` over
  each fixture directory, normalises the finding lines, and fails on any
  divergence in code/message/line/order; wired into CI alongside CIP-001.
- **Validation:** Harness passes on the current fixtures; a synthetic
  divergence (a rule ported to only one CLI, or a reordered check) fails it.
  This is the automated form of the manual three-way `diff` sweep run in
  COND-007.
- **Confidence:** medium
- **Dependencies:** CIP-001
- **Status:** Ready

## Notes

- Surfaced by COND-007 (see [conductor](./conductor.aps.md)) — the conductor
  W002/W006 backport whose fetched-`pwsh` verification exposed two latent
  PowerShell parity bugs that every existing CI check had missed.
- A cheaper partial alternative to a full `pwsh`-in-CI job is a container that
  bundles PowerShell only for the parity job; CIP-001 should pick whichever
  keeps the main test matrix fast.
