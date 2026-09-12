# Continuous Improvement Backlog

| ID  | Type      | Owner  | Priority | Status      |
| --- | --------- | ------ | -------- | ----------- |
| CIB | Conductor | @aneki | medium   | In Progress |

**Last reviewed:** 2026-09-12

## Purpose

Provide a standing intake for small, concrete APS improvements discovered
across completed feature modules, user journeys, reviews, and routine
maintenance. Items stay here when they are too narrow to justify reopening or
creating a dedicated feature module.

## Standing Module Policy

This module remains active while APS is active. Completing every currently
listed item does not complete the module; new findings may be appended as
`CIB-NNN` items. Promote a cluster into a dedicated module when it grows into a
coherent feature or needs its own design decisions.

## In Scope

- Small correctness, consistency, usability, documentation, and maintenance
  improvements with observable outcomes
- Follow-up work spanning completed modules or distribution surfaces
- User-journey regressions that need triage before implementation
- Compatibility cleanup that should not be lost after a larger feature ships
- Native Windows PowerShell parity for every user-facing APS workflow; Git Bash
  or WSL may support agent automation but is not a user prerequisite

## Out of Scope

- Vague ideas without an observable acceptance condition
- Features large enough to need a dedicated module or design document
- Work already owned by an active specialist module
- Replacing `plans/issues.md` as the record of discovered bugs and questions

## Intake Rules

Each item must include an intent, expected outcome, validation, source context,
and confidence. Keep items independently executable. When an item becomes
domain-specific or expands beyond a small maintenance slice, move it to the
owning module and leave a `Superseded by:` reference here.

## Coordinated Modules

| Module                                    | Role in this concern                          | Status      |
| ----------------------------------------- | --------------------------------------------- | ----------- |
| [install](./install.aps.md)               | Public installer and skill distribution       | Complete    |
| [agents](./agents.aps.md)                 | APS planning and status-response surfaces      | In Progress |
| [tui](./tui.aps.md)                       | Interactive initialization journey             | Complete    |
| [monorepo](./monorepo.aps.md)             | Monorepo and nested-plan scaffold expectations | Complete    |
| [ci-parity](./ci-parity.aps.md)           | Native Windows behavioural validation          | Complete    |

## Cross-Module Work Items

None currently. CIB items own only the small coordinating repair; broader work
is promoted back to the relevant module.

## Work Items

### CIB-001: Consolidate plan-status behaviour into APS planning

- **Status:** Draft
- **Intent:** Preserve a simple, consistent plan-status query without shipping
  a separate duplicated `plan-status` skill or active command.
- **Expected Outcome:** Asking “What is the plan status?” or “What’s next?”
  activates the APS planning surface and returns the standard report covering
  module counts, active and blocked items, recent completions, validation, and
  the suggested next item. Current init/setup paths do not install deprecated
  `.claude/commands/plan-status.md`; legacy migration may back up or remove old
  copies without treating them as the source of behaviour.
- **Validation:** Fresh tool-integration scaffolds contain the supported
  `aps-planning` skill/agent surface and no active `.claude/commands/`; prompt
  fixtures for the two natural-language queries produce the documented report;
  `cargo test --manifest-path cli/Cargo.toml` and `./test/run.sh` pass.
- **Identified From:** Review of the standalone `plan-status` copy in
  `anvil-001` against APS decisions D-015 and D-023.
- **Files:** `scaffold/aps-planning/SKILL.md`,
  `scaffold/agents/core/planner-core.md`, `cli/src/scaffold.rs`, scaffold tests
- **Confidence:** high
- **Dependencies:** none

### CIB-002: Hand the public installers to the native TUI

- **Status:** Complete: 2026-07-17
- **Intent:** Make each default interactive installation feel like one APS
  setup journey instead of a shell picker followed by a separate `aps init`.
- **Expected Outcome:** On a supported interactive terminal, the no-argument
  Unix `curl | bash` and Windows PowerShell entrypoints install or locate the
  native APS binary and hand control to its TUI in the same run. Explicit
  installer modes and non-interactive automation remain deterministic, and
  unsupported platforms retain a clear fallback.
- **Validation:** A PTY-backed Unix test drives the no-argument public installer
  into the real native TUI without a second command. The native Windows
  PowerShell job installs the release-shaped GNU archive through the public
  installer and proves the same onboarding handoff with redirected-input
  defaults; Rust wizard tests cover the interactive state machine. Explicit
  CLI, init, and non-interactive paths retain their documented behaviour. The
  Windows user journey must not require Git Bash or WSL.
- **Learning:** "Default installer entrypoints should share one native onboarding handoff; advanced shell modes remain explicit."
- **Identified From:** User-observed first-run journey on 2026-07-16: the curl
  command presents the shell CLI and the TUI appears only after `aps init`.
- **Files:** `scaffold/install`, `scaffold/install.ps1`, installer tests,
  `docs/installation.md`
- **Confidence:** medium
- **Results:** The default interactive Unix and PowerShell entrypoints now
  install the native binary and launch `aps init` in the same run. `--onboard`
  exposes that route explicitly for automation, `--menu` preserves the advanced
  picker, and a Unix PTY regression proves the installed binary renders the
  native TUI. The Windows job is configured to install the shipped GNU archive
  and exercise the PowerShell handoff under both PowerShell 7 and Windows
  PowerShell 5.1. Both native Windows variants passed in CI.

### CIB-003: Keep init project-shape and root-template choices coherent

- **Status:** Complete: 2026-07-17
- **Intent:** Ensure the init choices shown to users produce the root plan shape
  they selected.
- **Expected Outcome:** Selecting Monorepo produces a monorepo root
  `plans/index.aps.md` rather than the standard single-project index, while a
  nested/federated selection produces the federation root and child plans.
  Template choices cannot silently contradict the selected project shape, and
  the review screen states which root index will be written.
- **Validation:** Native Unix and Windows PowerShell journeys use the installed
  binary to scaffold single-project, monorepo, and nested roots, then assert the
  resulting root index content and `.aps/config.yml`; the monorepo journey
  fails if it writes the single-project index. Wizard state-machine tests drive
  interactive shape selection, template toggles, and back-navigation. Config
  replay and non-interactive template selection remain covered, and the
  Windows journey must not require Git Bash or WSL.
- **Learning:** "Project shape must own the generated root index at the scaffold boundary; template choices cannot override it silently."
- **Identified From:** User-observed init journey on 2026-07-16: choosing
  Monorepo installs the monorepo template asset but the generated root plan uses
  the old index. The source already has a plan-level unit assertion for the
  monorepo index, so validation must cover the public binary journey and expose
  any release, state, or selection mismatch.
- **Files:** `cli/src/wizard.rs`, `cli/src/scaffold.rs`, `cli/src/config.rs`,
  init journey tests
- **Confidence:** medium
- **Results:** Project shape is authoritative at the scaffold boundary,
  returning to change shape updates the selected root template, contradictory
  or multiply selected root-template flags fail clearly, and an explicit shape
  replaces a stale root inherited from config. Review names the root index that
  will be written. The monorepo root template also uses the canonical
  `## Modules` heading so the generated plan passes structural lint. Native
  Windows CI confirmed single-project, monorepo, and nested root generation.

### CIB-004: Enforce native Windows PowerShell user journeys

- **Status:** Complete: 2026-07-17
- **Intent:** Turn Windows PowerShell support from a portability claim into a
  behavioural compatibility gate for user-facing APS workflows.
- **Expected Outcome:** Every documented user workflow has a native PowerShell
  route using `aps.exe`, including installation, initialization, setup, update,
  validation, status, and recovery. Windows users do not need Git Bash or WSL;
  those shells may remain documented as optional agent or contributor tools.
- **Validation:** A `windows-latest` CI job starts in PowerShell, stages the
  Windows binary, and exercises a representative user journey through version
  reporting, non-interactive init, setup, lint, next/status, update, and
  doctor/recovery. Existing Ubuntu PowerShell parity and Windows
  cross-compilation remain supporting checks, not substitutes for the native
  runtime journey.
- **Identified From:** User compatibility requirement on 2026-07-17 and audit
  of CI coverage showing Windows cross-compilation plus Ubuntu-hosted PowerShell
  parity, but no native Windows end-to-end job.
- **Files:** `.github/workflows/ci.yml`, `scaffold/install.ps1`,
  `scaffold/update.ps1`, Windows smoke-test harness, user installation and usage
  documentation
- **Confidence:** medium
- **Results:** The first native Windows run successfully installed and
  executed the shipped GNU archive, then exposed Windows path separators being
  misclassified by Rust lint. Path classification is now normalised and backed
  by Windows-style index, module, action, release, and template regressions;
  PowerShell 7 subsequently completed the full native journey. Windows
  PowerShell 5.1 then exposed an un-BOMed UTF-8 em dash inside an executable
  installer string; that message is now ASCII-safe. Its next run completed
  installation, hooks, and lint before the harness promoted the expected
  non-zero `aps next` stderr into a terminating error. Expected native failures
  are now captured without relaxing strict handling for the rest of the
  journey. The final CI run passed the full native journey under both
  PowerShell 7 and Windows PowerShell 5.1.

### CIB-005: Realign plan-doctor rules with the documented status vocabulary

- **Status:** Complete: 2026-09-12
- **Intent:** Make the bundled `plan-doctor` skill usable on a healthy consumer
  plan by judging it against `plans/aps-rules.md` rather than a stricter
  private convention.
- **Expected Outcome:** W03 accepts the documented aliases (`Proposed` → `Draft`,
  `Done` → `Complete`) after normalising through the alias table, W04 counts
  `Done` as terminal, the undocumented `Archived` case reports under its own
  quieter code, and the "no numeric filename prefix" half of W02 is advisory
  rather than a warning. A structurally healthy plan produces no warnings.
- **Validation:** `npx markdownlint-cli "scaffold/plan-doctor/SKILL.md"` and
  `cargo test --manifest-path cli/Cargo.toml` pass; the rule table and every
  worked example in the skill agree with `plans/aps-rules.md` § Status
  Vocabulary; a consumer plan using `Done`/`Proposed` statuses reports clean.
- **Identified From:** [Issue #132](https://github.com/eddacraft/anvil-plan-spec/issues/132),
  filed 2026-07-26 from the `anvil-001` vendoring review — roughly 385 warnings
  fired on a plan with no real defects (162 `Done` + 132 `Proposed` statuses
  false-flagged by W03, every one of 91 modules flagged by W02).
- **Files:** `scaffold/plan-doctor/SKILL.md`
- **Confidence:** high
- **Dependencies:** none
- **Notes:** The skill bytes are managed and embedded in the binary
  (`include_str!` in `cli/src/scaffold.rs`), so hand-editing them in a consumer
  repo desyncs `.aps-managed.json` and is overwritten on the next vend — the
  fix has to ship from here and reaches users only on a release.
- **Results:** `scaffold/plan-doctor/SKILL.md` now opens `## What to check`
  with a "Normalise status before judging it" section quoting
  `plans/aps-rules.md` § Status Vocabulary, and instructs that the alias table
  be applied before W03, W04, W05, and W07 evaluate. W03 narrows to values
  still unrecognised after normalisation (`WIP`, `Almost done`); W04 counts
  `Done` as terminal alongside `Merged`/`Released`/`Shipped`; W02 keeps only
  the prefix-versus-dependency-order conflict. Two Info codes were added: I04
  for a missing `NN-` filename prefix (reported once per directory with a
  count) and I05 for a consistently-applied undocumented status such as
  `Archived`. A collapsing rule was added to the report section — a finding
  that hits 100% of files is house style, not signal — which addresses the
  root cause rather than just the three symptoms. The skill now agrees with
  its sibling `aps-planning` skill, which already recognised the aliases.
  Frontmatter untouched; `cli/src/scaffold.rs` needed no change (its
  assertions check file count and frontmatter shape, not bytes).
  markdownlint and `cargo test` (201 tests) pass.

### CIB-006: Port release-plan lint rules to bash and PowerShell

- **Status:** Complete: 2026-09-12
- **Intent:** Retire the last known three-way lockstep gap so a malformed
  release plan cannot pass the fallback CLIs.
- **Expected Outcome:** The bash and PowerShell linters discover
  `plans/releases/v*.md` (excluding `README.md` and the dotfile template) and
  implement R001–R004 with the same codes, severities, and messages as the Rust
  linter. All three CLIs report identical findings and identical file counts on
  this repo's `plans/`, and release fixtures in the shared parity suite keep
  them that way.
- **Validation:** `./bin/aps lint plans` and the Rust binary agree on file count
  and findings; `./test/cli-parity.sh` covers a valid release plan plus a
  malformed one exercising each of R001–R004; `./test/run.sh` and
  `./test/ps-parity.ps1` pass.
- **Identified From:** Release review 2026-09-12 — bash reported 42 files
  checked against the Rust binary's 49, the difference being exactly the seven
  `plans/releases/v*.md` records. REL-003 implemented the rules Rust-only, which
  D-039 later superseded.
- **Files:** `lib/rules/release.sh`, `lib/rules/Release.psm1`, `lib/lint.sh`,
  `bin/aps`, `bin/aps.ps1`, `test/cli-parity.sh`, `test/fixtures/**`
- **Confidence:** high
- **Dependencies:** none
- **Notes:** D-039 makes bash and PowerShell maintained peers, not frozen
  fallbacks — REL-003's "aps lint is now the Rust CLI" rationale predates it.
- **Results:** Added `lib/rules/release.sh` and `lib/rules/Release.psm1` as
  hand-ports of `lint_release`, wired into `lib/lint.sh`, `lib/Lint.psm1`,
  `bin/aps`, and `bin/aps.ps1`. Discovery was the larger half of the defect:
  `get_file_type` needed a `release` branch at the Rust classifier's
  precedence (between `actions` and `module`) and `find_aps_files` needed a
  `releases/` clause, pinned to `LC_ALL=C sort` to match Rust's byte-order
  sort. File counts went 42 → 50 in bash and PowerShell, matching Rust, and
  `diff` of the full output (plus `--json`) is byte-identical across all
  three on `plans`, `plans/releases`, a single record, a relative target, and
  the no-arg default. Two further causes of CI blindness were fixed: the
  parity harness carried no release fixtures, and its `findings()` regex
  matched only `(E|W)[0-9]{3}`, so R-codes were invisible even when present.
  Fixtures `release/plans` (clean, plus the `README.md` and dotfile-template
  exclusions) and `release-invalid/plans` (one record per failure mode, ten
  findings) now pin the behaviour. The PowerShell port needed `-cmatch`/`-cne`
  for the Target/Status rows, the `.md` extension, and `README.md` — the same
  case-insensitivity class of bug recorded at `lib/rules/Common.psm1:115`;
  without it `| target |` would have satisfied R002. Verified against Rust on
  a scratch fixture of `V0.3.0.md`, `v.md`, `readme.md`, `README.MD`,
  `v1.2.0-beta.md`, `v9.aps.md`, and `sub/v2.0.0.md`.
- **Follow-ups:** Sourcing a new rule module from `bin/aps` without adding it
  to the installer manifests broke a vendored bash CLI on startup; fixed
  across all fourteen manifest sites (`lib/scaffold.sh` ×4, `lib/Scaffold.psm1`,
  `scaffold/{init.sh,install,install.ps1,update,update.ps1,upgrade}`,
  `cli/src/migrate.rs`, `cli/src/doctor.rs`) and pinned by a new `test/run.sh`
  guard that derives the rule list from `bin/aps` itself, so the next rule
  module cannot repeat it. Rule weaknesses and residual cross-CLI divergences
  were reported rather than unilaterally fixed in one CLI — see CIB-008.

### CIB-007: Give the fallback CLIs a version surface

- **Status:** Draft
- **Intent:** Let the bash and PowerShell CLIs report which version they are,
  so the toolchain a project is actually running is always observable.
- **Expected Outcome:** `aps --version` (and the bare `version` command if the
  Rust binary accepts one) reports the CLI's own version from all three
  implementations, with identical formatting. The value is the same
  `APS_CLI_VERSION` the fallbacks already stamp into `.aps/config.yml`.
- **Validation:** `aps --version` emits the same string from the Rust binary,
  `bin/aps`, and `bin/aps.ps1`; a parity fixture or test leg pins the three-way
  identity; `./test/run.sh` and `./test/ps-parity.ps1` pass.
- **Identified From:** Release review 2026-09-12 — the Rust binary answers
  `aps --version` with `aps 0.9.0`, while `bin/aps --version` and
  `bin/aps --help` have no version surface at all (`--version` errors with
  "Unknown command"). The fallbacks already know their version: both default
  `APS_CLI_VERSION` and write it as `cli_version`.
- **Files:** `bin/aps`, `bin/aps.ps1`, `lib/scaffold.sh`, `lib/Scaffold.psm1`,
  `test/cli-parity.sh`
- **Confidence:** high
- **Dependencies:** none
- **Notes:** Small but load-bearing for D-044's "single on-disk version
  surface" and for CLI-003's honest-version-surface intent: a user debugging a
  `cli_version` pin mismatch on a fallback CLI currently has no way to ask the
  CLI what it is.

### CIB-008: Harden the release-plan rules and close residual lint divergences

- **Status:** Draft
- **Intent:** Tighten release-plan validation beyond the structural minimum and
  retire the three-CLI lint divergences surfaced while porting R001–R004.
- **Expected Outcome:** R001 validates a plausible version rather than
  `v` + one digit; a `*.aps.md` file under `releases/` is not silently denied
  module linting; R002 requires the Target and Status rows to belong to the
  same header table. Separately, the three known cross-CLI divergences are
  either fixed in all three implementations or documented as accepted
  differences with a parity-suite note: symlinked directory traversal on
  `aps lint .`, PowerShell's culture-aware file sort versus byte-order sorting
  in Rust and bash, and the W020/W021 grouping order.
- **Validation:** Fixtures covering `v0garbage.md`, `v9.aps.md`, and a record
  whose Target and Status rows sit in different tables; `./test/cli-parity.sh`
  green with mixed-case sibling filenames and a symlinked directory in scope;
  `./test/run.sh` and `./test/ps-parity.ps1` pass.
- **Identified From:** CIB-006 (2026-09-12). The porter mirrored the Rust
  reference exactly, as D-039 requires, and reported the weaknesses rather than
  silently "improving" one implementation — which would itself have created a
  divergence.
- **Files:** `cli/src/lint.rs`, `cli/src/parser.rs`, `lib/rules/release.sh`,
  `lib/rules/Release.psm1`, `lib/lint.sh`, `lib/Lint.psm1`,
  `test/cli-parity.sh`, `test/fixtures/**`
- **Confidence:** medium
- **Dependencies:** CIB-006
- **Notes:** Any change here lands in all three CLIs in the same work item —
  fixing the Rust rule alone would re-open the gap CIB-006 just closed.
  The PowerShell sort fix (`[StringComparer]::Ordinal`) changes ordering for
  every file type, not just release plans, so it needs its own parity pass.

## Status Roll-up

- **Concern:** Standing APS maintenance intake
- **Progress:** 5/8 work items Complete
- **Readout:** CIB-002, CIB-003, and CIB-004 are complete with native Windows
  CI evidence. CIB-001 remains Draft and isolated in its own worktree. CIB-005
  (plan-doctor false positives, issue #132) and CIB-006 (release-lint three-CLI
  parity) were intaken from the 2026-09-12 release review and are Ready.
  CIB-005 and CIB-006 completed on 2026-09-12. CIB-007 (version surface on the
  fallback CLIs) and CIB-008 (release-rule hardening plus residual lint
  divergences) were intaken from the same review and are Draft.

## Decisions

- **D-001:** Lifecycle — _decided 2026-07-16: standing conductor module._ CIB
  remains active while APS is active and does not close when temporarily empty.
- **D-002:** Plan-status ownership — _decided 2026-07-16: APS planning owns the
  behaviour._ A standalone skill is unnecessary; compatibility aliases may
  forward to the canonical planning surface during migration.
- **D-003:** Installer interaction — _decided 2026-07-16: one interactive
  journey._ The no-argument curl entrypoint should hand off to the native TUI
  on supported terminals rather than require a second `aps init` command.
- **D-004:** Init selection authority — _decided 2026-07-16: the generated root
  index must match the reviewed project-shape/template selection._ Internal
  template installation must not diverge from the user-visible choice.
- **D-005:** Windows user contract — _decided 2026-07-17: native PowerShell is
  first-class for user-facing APS workflows._ Git Bash and WSL may be used by
  agents or contributors, but users must be able to install and operate APS
  without them. Windows compatibility requires native behavioural evidence;
  cross-compilation and PowerShell-on-Ubuntu checks are supporting evidence.

## Notes

- Seeded from the standing-CIB pattern already proven in `anvil-001` and
  proposed for APS in `plans/brainstorms/2026-06-15-aps-upstream-brief.md`.
