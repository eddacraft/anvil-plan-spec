# CI Cross-Implementation Parity

| ID  | Owner  | Priority | Status      |
| --- | ------ | -------- | ----------- |
| CIP | @aneki | medium   | In Progress |

**Last reviewed:** 2026-07-10

## Purpose

The `aps` CLI ships three implementations that must stay in lockstep (index
D-038/D-039): the canonical Rust binary, the feature-frozen bash CLI, and the
PowerShell fallback. CI _does_ run pwsh (the `powershell` job in
`.github/workflows/ci.yml` executes `test/ps-parity.ps1` on a runner with
PowerShell preinstalled), but that harness only exercised a slice of the
fixture corpus — the nested-plans scenarios. Everything else the PowerShell
port does was guarded only by **string-matching** in `test/run.sh` (grepping
for function names and lint codes). String guards prove a function _exists_,
not that it _behaves_. COND-007 demonstrated the cost: `Get-ApsStatus` had
silently mis-parsed spaced `| --- |` separators (W005/W017/W018 never fired in
PowerShell) and a W017 rounding drift went unnoticed — both invisible until a
`pwsh` was fetched by hand and run against the conductor fixtures. This module
widens the behavioural harness to the corpus that would have caught them, and
(CIP-002) makes the three CLIs diff byte-for-byte.

## In Scope

- Extending `test/ps-parity.ps1` (already run by the `powershell` CI job) to
  cover the conductor rules and status gating behaviourally.
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

### CIP-001: Extend the behavioural PowerShell harness to the conductor corpus

- **Intent:** Catch PowerShell behavioural regressions in CI instead of only
  string-guarding the ported surface.
- **Expected Outcome:** `test/ps-parity.ps1` (run by the existing `powershell`
  CI job) exercises the conductor fixtures behaviourally — W002/W006 detection,
  clean-plan silence, and an active conductor emitting W017 before W002 —
  extending the coverage beyond the nested-plans scenarios.
- **Validation:** `pwsh test/ps-parity.ps1` green; reintroducing the COND-007
  `Get-ApsStatus` separator bug fails the status-gating scenario (verified — a
  string guard would still pass).
- **Confidence:** high
- **Dependencies:** none
- **Status:** Complete
- **Notes:** Done 2026-07-10. pwsh was already in CI; the gap was harness
  _coverage_, not pwsh availability — the original premise that CI ran no pwsh
  was wrong. Added scenarios 6–8 to `test/ps-parity.ps1`; verified against a
  fetched pwsh 7.4.6 (all scenarios pass; the Get-ApsStatus revert fails the
  guard). CIP-002 generalises this to a byte-for-byte cross-CLI diff.

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
- The `powershell` CI job already provides pwsh (ubuntu-latest ships it), so no
  new toolchain is needed — CIP-001 was purely a coverage gap in
  `test/ps-parity.ps1`, and CIP-002 hardens it into a cross-CLI byte diff.
