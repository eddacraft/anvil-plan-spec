# Continuous Improvement Backlog

| ID  | Type      | Owner  | Priority | Status      |
| --- | --------- | ------ | -------- | ----------- |
| CIB | Conductor | @aneki | medium   | In Progress |

**Last reviewed:** 2026-07-16

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

### CIB-002: Hand the public curl journey to the native TUI

- **Status:** Draft
- **Intent:** Make the default interactive curl installation feel like one APS
  setup journey instead of a shell picker followed by a separate `aps init`.
- **Expected Outcome:** On a supported interactive terminal, the no-argument
  public `curl | bash` entrypoint installs or locates the native APS binary and
  hands control to its TUI in the same run. Explicit installer modes and
  non-interactive automation remain deterministic, and unsupported platforms
  retain a clear shell fallback.
- **Validation:** A PTY-backed test of the public installer reaches the native
  TUI without requiring the user to run a second command; explicit `--cli`,
  `--init`, and non-interactive paths retain their documented behaviour on
  Unix and PowerShell entrypoints.
- **Identified From:** User-observed first-run journey on 2026-07-16: the curl
  command presents the shell CLI and the TUI appears only after `aps init`.
- **Files:** `scaffold/install`, `scaffold/install.ps1`, installer tests,
  `docs/installation.md`
- **Confidence:** medium
- **Dependencies:** none

### CIB-003: Keep init project-shape and root-template choices coherent

- **Status:** Draft
- **Intent:** Ensure the init choices shown to users produce the root plan shape
  they selected.
- **Expected Outcome:** Selecting Monorepo produces a monorepo root
  `plans/index.aps.md` rather than the standard single-project index, while a
  nested/federated selection produces the federation root and child plans.
  Template choices cannot silently contradict the selected project shape, and
  the review screen states which root index will be written.
- **Validation:** An end-to-end test drives the released/installed binary
  through each project-shape choice and asserts the resulting root index
  content and `.aps/config.yml`; the monorepo journey fails if it writes the
  single-project index. Existing non-interactive template selection remains
  covered.
- **Identified From:** User-observed init journey on 2026-07-16: choosing
  Monorepo installs the monorepo template asset but the generated root plan uses
  the old index. The source already has a plan-level unit assertion for the
  monorepo index, so validation must cover the public binary journey and expose
  any release, state, or selection mismatch.
- **Files:** `cli/src/wizard.rs`, `cli/src/scaffold.rs`, `cli/src/config.rs`,
  init journey tests
- **Confidence:** medium
- **Dependencies:** none

## Status Roll-up

- **Concern:** Standing APS maintenance intake
- **Progress:** 0/3 work items Complete
- **Readout:** Three user-facing consistency fixes are captured as Draft and
  await readiness review.

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

## Notes

- Seeded from the standing-CIB pattern already proven in `anvil-001` and
  proposed for APS in `plans/brainstorms/2026-06-15-aps-upstream-brief.md`.
