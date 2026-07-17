# Continuous Improvement Backlog

| ID  | Type      | Owner  | Priority | Status      |
| --- | --------- | ------ | -------- | ----------- |
| CIB | Conductor | @aneki | medium   | In Progress |

**Last reviewed:** 2026-07-17

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

- **Status:** In Progress
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
  PowerShell 5.1; native execution evidence is still pending.

### CIB-003: Keep init project-shape and root-template choices coherent

- **Status:** In Progress
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
  `## Modules` heading so the generated plan passes structural lint.

### CIB-004: Enforce native Windows PowerShell user journeys

- **Status:** In Progress
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
- **Progress:** The first native Windows run successfully installed and
  executed the shipped GNU archive, then exposed Windows path separators being
  misclassified by Rust lint. Path classification is now normalised and backed
  by Windows-style index, module, action, release, and template regressions;
  PowerShell 7 subsequently completed the full native journey. Windows
  PowerShell 5.1 then exposed an un-BOMed UTF-8 em dash inside an executable
  installer string; that message is now ASCII-safe. Native 5.1 confirmation
  remains pending.

## Status Roll-up

- **Concern:** Standing APS maintenance intake
- **Progress:** 0/4 work items Complete
- **Readout:** CIB-002, CIB-003, and CIB-004 are implemented locally and remain
  In Progress pending their native Windows CI evidence. CIB-001 remains
  isolated in its own worktree.

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
