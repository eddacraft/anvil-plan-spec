# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

## [0.4.0] - 2026-06-21

**Release narrative:** [plans/releases/v0.4.0.md](./plans/releases/v0.4.0.md)
for theme, success criteria, and risks.

### Added

- **Conductor / crosscutting module type** (COND module — Complete) — a new
  `Type: Conductor` module type for concerns that coordinate work across
  vertical modules (releases, audits, budgets, migrations).
  `templates/conductor.template.md`, parser `is_conductor()`, lint **W002**
  (unresolved cross-module references in a conductor) and **W006** (index
  Conductor-section entries must be `Type: Conductor`), an optional index
  subsection, and `docs/conductor-modules.md`. `release-planning` is the
  canonical instance.
- **Binary-first distribution** (INSTALL-011…018) — `aps` installs as a global
  binary via crates.io (`cargo install aps-cli`), GitHub release binaries,
  Scoop, and `cargo binstall`. `aps init` scaffolds project content only;
  `.aps/config.yml` pins `cli_version` and carries runtime path defaults;
  runtime discovery respects project `plans_dir` / `docs_dir` without `--plans`
  or direnv. `aps doctor` diagnoses toolchain drift; `aps setup` is an
  interactive picker for optional integrations; the public `curl | bash`
  installer gains a TTY mode picker with explicit non-interactive flags
  (`--cli`, `--init`, `--agent`, `--upgrade`); `aps upgrade` cleans up
  legacy v1 and bulky v2 footprints. A migration path moves projects from the
  vendored CLI to the global binary.
- **Release planning addon** (release-planning module — Complete) —
  `plans/releases/` layout, `release.template.md`, R001–R004 lint rules,
  scaffolding on `aps init`, and `docs/release-planning.md`.
- **`aps audit` command** (DOGFOOD-002) — formalizes the anvil-001 completion
  audit: executes Complete items' Validation commands (PASS/FAIL/PARTIAL),
  flags understated Drafts whose Files already exist (A002), stale Ready
  items (A003), and broken index links (A004). `--json`, `--no-run`, and
  `--stale-days` supported; non-zero exit on findings for CI gating.
- **Plan hygiene lint checks** (DOGFOOD-002) — W017 (active module missing
  or stale `**Last reviewed:**`, threshold `APS_STALE_DAYS`), W018 (Complete
  item without Validation in a still-active module), W019 (index `## Modules`
  link to a non-existent file). Bash and PowerShell rule engines.
- **MCP server** (ORCH-006 — orchestrate module now Complete) — optional
  `mcp/` package exposing the `aps` CLI command surface to MCP-capable agents
  as a single codemode tool. Direct commands and natural-language requests
  ("what's the next ready work item in auth?") route to allowlisted CLI
  invocations; malformed input returns help instead of failing the transport.
  TypeScript on the MCP SDK (D-004), runs directly on Node >= 22.18 — no
  build step. See `mcp/README.md` and the "MCP Server" section in
  `docs/usage.md`.

- **Compound-engineering archive patterns** (COMPOUND module — Complete) —
  `templates/solution.template.md`, `templates/completed-index.template.md`,
  and `templates/release.template.md` ship the doc-only halves of the Learn
  phase. `plans/completed.aps.md` seeded from this repo's shipped work, and
  `plans/releases/v0.3.0.md` authored as the proof-of-concept release
  narrative.
- **Workflow guide additions** — `docs/workflow.md` "Completion and Archival"
  section rewritten to reference the completed roll-up and `plans/completed/`
  long-form notes; new "Release Narrative" section explains when and how to
  write a release doc.
- **Rust CLI and TUI wizard** (TUI module — Complete) — Ratatui-based `aps
  init` wizard covering profile, project shape, AI tooling, template/path
  customization, scaffold execution, and non-interactive config replay.
  Native Rust `aps lint` and `aps next` reach byte-for-byte parity with bash
  (TUI-009); bash implementations are feature-frozen per orchestrate D-006.
  `aps setup` mode picker (TUI-007) and agent bootstrap flow (TUI-008).
  Wizard hardening: bracketed paste support (TUI-010) and key-release filter
  plus Index/MonorepoIndex exclusivity (TUI-011).
- **Conductor agent** (ORCH-005) — a coordinator agent role for cross-module
  work across Claude Code, Codex, Copilot, OpenCode, and Gemini harnesses,
  complementing the MCP server.
- **Claude Code Tasks integration** (TASKS-001) — documented APS-to-Tasks
  mapping with prompts for wave planning, agent assignment, task dispatch,
  and status sync under `docs/ai/prompting/claudecode/`.
- **Status vocabulary reconciliation** (SPEC-001) — canonical
  `Draft / Ready / In Progress / Complete / Blocked` with `Proposed → Draft`
  and `Done → Complete` aliases normalised by the tooling.
- **APS plan-update contribution guidance** (DOGFOOD-003) — AGENTS.md
  documents when and how to update plans alongside template, prompt, and
  validation changes.

### Changed

- **Version bumped to 0.4.0** — one semver across all distribution channels
  (D-036); `.aps/config.yml` pins `cli_version: 0.4.0`.
- **W003 resolves across the plan tree** (DOGFOOD-002) — dependency
  references to work items and decisions in _other_ module files (and the
  index) no longer warn; only IDs missing from the entire plan are flagged.
  Message changed from "not found in this file" to "not found in plan".
- **E005 exempts terminal work items** — `aps lint` no longer requires the
  `Intent` / `Expected Outcome` / `Validation` fields on work items whose
  `Status` is a completed state (`Done`, `Complete`, `Merged`, `Released`,
  `Shipped`). Completed items are commonly compacted to `Status` + a short
  summary at closeout, with full detail preserved in version history; requiring
  the fields reopened E005 on every shipped item. Active states
  (`Proposed` / `Ready` / `In Progress` / `Blocked` / `Draft` / `Deferred`) are
  still checked. Applies to both the bash (`lib/rules/workitem.sh`) and
  PowerShell (`lib/rules/WorkItem.psm1`) rule engines.

### Fixed

- **Installer** — hardened per council review; mode picker respects TTY vs
  non-interactive flags and avoids surprise bulky installs.
- **TUI wizard** — multi-line paste no longer replays newlines as Enter and
  drives scaffold execution without review; Windows terminals that report
  key releases no longer double every keystroke; selecting `Index` or
  `MonorepoIndex` deselects the other so scaffold output is not overwritten.
- **Scaffold** — init output includes all referenced paths.
- **Prepare script** — `husky || true` so installs that omit dev dependencies
  do not fail when the husky binary is unavailable.

## [0.3.0] - 2026-05-20

**Release narrative:** [plans/releases/v0.3.0.md](./plans/releases/v0.3.0.md)
for theme, success criteria, and risks.

### Added

- **Orchestration CLI** — `aps next` resolves the next ready work item across
  modules, honouring work-item and module-level dependencies (ORCH-001).
- **State-machine commands** — `aps start <ID>` and `aps complete <ID>` enforce
  the Ready → In Progress → Complete lifecycle. `start` validates that every
  dependency is Complete; `complete` requires the item to be In Progress and
  stamps Status with the UTC date (ORCH-002).
- **Context packaging** — `aps start` writes `.aps/context/<ID>.md` with the
  work item, module scope, decisions, dependency learnings, and related files
  (ORCH-003). Gitignored, regenerated on each `start`.
- **Dependency graph** — `aps graph [module]` renders work items with their
  upstream dependencies and per-item status (ORCH-004).
- **Learning capture** — `aps complete --learning "..."` inserts a
  `- **Learning:**` line after `- **Validation:**` (per ORCH D-002). Learnings
  travel with the work item and surface as "Dependency Learnings" in downstream
  context packages.
- **v2 layout migration** — `aps migrate` converts existing projects to the
  `.aps/` consolidated tooling root, with shell-prompt wizard.
- **TUI init wizard** — first Ratatui-based onboarding flow for `aps init`
  (TUI-001).
- **Multi-agent ports** — APS agents ported to Codex, GitHub Copilot, OpenCode,
  and Gemini in addition to Claude Code; added APS planner, librarian, and
  conductor agents.
- **Global install** — `--global` flag for system-wide CLI installation.
- **Designs folder** — `designs/` added as a standard APS artifact for design
  documents alongside specs and plans.
- **Wave-based execution** — action plans support wave-based parallel execution
  guidance for concurrent agents.

### Changed

- Scaffold renamed `steps.template.md` → `actions.template.md` end-to-end to
  match the "Actions" terminology used in Work Items and prompts.
- Skill install decoupled from `aps init` — install once globally, opt in per
  project.
- `/plan` skill auto-bootstraps `aps init` and performs a version check.
- Canonical primary branch promoted from `dev` to `main`; CI updated.
- TUI framework decision: Rust + Ratatui (replacing earlier OpenTUI/Bun
  exploration).

### Fixed

- Installer: only matching runtime files installed; legacy init runtime files
  included; piped installs prompt correctly; APS orchestration library
  installed; PowerShell variant includes orchestrate library.
- PowerShell scripts write BOM-free session baseline files.
- Module status row parsing skips the markdown separator row.
- Scaffold backs up `aps-rules.md` and hook scripts during migration.

## [0.2.0] - 2026-02-20

First release of Anvil Plan Spec (APS).

### Added

- **Templates** — Index, Module, Simple, Actions, and Quickstart templates in `templates/`
- **Scaffold** — One-command setup via `curl | bash` with `--update` support
- **Validation CLI** — `aps lint` command to validate APS documents with CI integration
- **Hooks system** — SessionStart, PreToolUse, PostToolUse, Stop hooks with install
  script and hook configuration
- **PowerShell port** — Full PS CLI (`aps.ps1`), scaffold module, all hook scripts
  ported to PS, one-liner PS install/update
- **CLI improvements** — `init` and `update` subcommands, improved validation rules
  (field checks, ID regex), issues tracker rule
- **Docs restructure** — Extracted installation guide, CLI usage guide, AI agent guide
  from README; README refocused as landing page
- **Prompts** — Tool-agnostic and OpenCode/Claude variants in `docs/ai/prompting/`
- **Examples** — User Authentication and OpenCode Companion worked examples
- **Planning specs** — v0.3 install and agents module specs (meta: APS plans its own
  development)
- **Documentation** — Getting started guide, workflow guide, ADR template, project structure
- **Roadmap** — Planned features and direction
- **Claude Code Tasks** — Integration guidance in aps-rules.md

### Changed

- Renamed "Leaf" template to "Module" for clarity
- Renamed "Steps" template to "Actions" for clarity
- Changed `SCOPE` placeholder to `ID` in templates to avoid confusion with In/Out Scope sections

### Documentation

- README with hierarchy diagram, quick start, and principles
- AGENTS.md with collaboration rules for AI contributors
- CONTRIBUTING.md with scope guardrails and PR process
- Getting started guide with decision tree
- Workflow guide with day-in-the-life scenarios
- Monorepo support guide
