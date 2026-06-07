# TUI Onboarding Module

| ID  | Owner  | Status      |
| --- | ------ | ----------- |
| TUI | @aneki | In Progress |

**Last reviewed:** 2026-06-08

## Purpose

Provide a customization frontend for APS project setup. The shell-prompt wizard
(INSTALL module) proved the concept but is limited to a few coarse presets.
The TUI wizard unlocks granular configuration — users choose exactly what to
deploy rather than getting a one-size-fits-all scaffold.

Built with Rust/Ratatui via the shared `eddacraft/eddacraft-tui` crate,
matching the Anvil product family's visual identity.

## Background

The INSTALL module (v0.3) shipped a shell-prompt wizard with 3 prompts:
profile, scope, and AI tooling. This covers the basics but forces broad
presets — you get "small feature" or "multi-module initiative" with little
control over what actually gets installed.

Users need finer-grained choices:

- **What to deploy:** pick individual agents, specific hooks, opt in/out of
  lint, rules, context docs
- **Where things go:** custom paths for plans directory, docs location
- **Project shape:** monorepo vs single-project, with monorepo-specific
  options (workspace detection, per-package plans)
- **Templates:** choose from available templates or bring your own, not just
  the 4 scope presets
- **Tool configuration:** per-tool hook verbosity, model preferences, which
  skills to install

Shell prompts can't scale to this many options without becoming painful. A
TUI with sections, back-navigation, and sensible defaults makes granular
customization accessible.

The base TUI framework lives at `eddacraft/eddacraft-tui` (Ratatui widgets,
shared theme, keyboard conventions). APS consumes this as a crate dependency.

### Why Now

- Shell-prompt wizard is complete and stable — the interim step served its
  purpose
- Users are asking for more control over what gets scaffolded
- `eddacraft/eddacraft-tui` provides shared widgets (select, multi-select,
  spinner, results dashboard) ready for consumption
- Anvil already uses the same TUI library — proven patterns exist
- Single static binary distribution simplifies install (`curl` downloads binary
  instead of bash scripts)

## In Scope

- Rust CLI binary (`aps-cli`) with `init` subcommand implementing the TUI
  wizard
- Wizard sections: Profile, Project Shape, Templates, AI Tooling, Paths,
  Scaffold, Summary
- Setup mode picker for `aps setup` and the public installer hand-off:
  install CLI, initialize repo, agent bootstrap, upgrade, or add integrations
- Granular customization within each section:
  - **Project shape:** monorepo vs single, workspace detection, per-package
    plans
  - **Templates:** choose from available templates, custom template paths
  - **AI tooling:** per-tool agent selection, hook verbosity, model preferences
  - **Paths:** custom plans directory, docs location, tooling root
  - **Components:** opt in/out of individual features (lint, rules,
    project-context, agents, hooks)
- Consume `eddacraft/eddacraft-tui` for widgets and theme
- Non-interactive fallback via flags and config file
- Cross-compiled static binaries for 5 targets (linux-x64, linux-arm64,
  darwin-arm64, darwin-x64, windows-x64)
- GitHub release asset publishing
- Port `aps lint` to native Rust (shared parser with ORCH module)

## Out of Scope

- Removing the shell-prompt wizard (kept as lightweight alternative)
- MCP server or orchestration commands (separate ORCH module)
- Developing `eddacraft/eddacraft-tui` itself (consumed as dependency)
- Runtime configuration changes (wizard is for initial setup; `config.yml`
  handles updates)

## Interfaces

**Depends on:**

- INSTALL (In Progress, follow-up) — defines what gets scaffolded; TUI
  replaces the shell-prompt frontend while reusing scaffold logic
- `eddacraft/eddacraft-tui` (external) — shared Ratatui widgets and theme

**Exposes:**

- `aps init` — TUI wizard (replaces shell-prompt wizard as default interactive
  path)
- `aps setup` — TUI setup picker for optional integrations and upgrade paths
- `aps init --non-interactive` — flag-based fallback for CI
- `aps lint` — optional native port of bash linter
- GitHub release binaries per platform

## Decisions

- **D-026:** Where does `aps-cli` source live? — _decided: in this repo under
  `cli/`. Keeps everything together. Acknowledged trade-off: APS grows from
  pure templates/docs into a repo with deployable code — increased scope and
  CI complexity._
- **D-027:** Shared TUI components — _decided: consume `eddacraft/eddacraft-tui`
  as git dependency for now. Publish as crate later if needed._
- **D-028:** Should `aps lint` be ported to Rust? — _decided: yes. The Rust
  markdown parser serves TUI, ORCH (`aps next`), and lint from a single
  codebase. Also provides a reference implementation portable to
  `eddacraft/anvil-001` (currently TS). Trade-off: reimplements working bash
  code, but the shared parser justifies the cost._
- **D-029:** Setup picker scope — _decided: `aps setup` with no arguments opens
  a TUI picker. `aps setup <thing>` remains a shortcut for non-interactive or
  scripted use. The public `curl | bash` installer should hand off to the same
  choice model rather than silently installing the full project footprint._
- **D-030:** Picker implementation — _decided: use `eddacraft/eddacraft-tui`
  Select, MultiSelect, Confirm, Spinner, and ResultsDashboard widgets for the
  Rust path. Shell fallback mirrors the same choices with numbered prompts._
- **D-031:** Rust binary vs `bin/aps` coexistence — _decided 2026-06-08: the
  Rust binary becomes primary once TUI-006 ships; `bin/aps` remains the
  zero-dependency fallback with identical command surface. The shared Rust
  parser, native `lint`, and `next` parity are tracked by TUI-009; remaining
  ORCH commands follow once parity is proven (orchestrate D-006). Revisit if
  a fallback CLI (plain-prompt mode) lands in `eddacraft/eddacraft-tui` —
  that could replace the bash fallback and shell-prompt wizard entirely._

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] D-026 resolved (in this repo under `cli/`)
- [x] D-027 resolved (git dependency)
- [x] D-028 resolved (yes, port lint to Rust)
- [x] D-031 resolved (Rust primary post-TUI-006, bash fallback; revisit on
      eddacraft-tui fallback CLI)
- [x] Work items defined with validation

## Work Items

### TUI-001: Project setup and eddacraft-tui integration — Complete 2026-04-26

- **Intent:** Establish the Rust project structure and confirm `eddacraft-tui`
  crate integration works
- **Expected Outcome:** `cli/` directory (or separate repo per D-026) with
  `Cargo.toml` depending on `eddacraft-tui`, clap for CLI parsing, and a
  minimal `aps --version` command that compiles and runs
- **Validation:** `cargo build --release --manifest-path cli/Cargo.toml`
  produces a static binary; binary
  prints version; `eddacraft-tui` widgets render in a test harness
- **Confidence:** high
- **Dependencies:** D-026, D-027
- **Action plan:** [execution/TUI-001.actions.md](../execution/TUI-001.actions.md)
- **Results:** `cli/` Cargo crate scaffolded (edition 2024, clap 4 derive,
  eddacraft-tui as git dep from `github.com/EddaCraft/eddacraft-tui`).
  `aps --version` prints `aps 0.4.0-dev`; `cargo tree` confirms the
  eddacraft-tui crate links cleanly. Subcommand stubs for `init`, `lint`,
  `next` exit with code 2 + a "not yet implemented" hint to surface the
  intended CLI shape from day one. Live widget render harness deferred to
  TUI-002, which is the natural place to introduce actual TUI sections.

### TUI-002: Implement core wizard sections (Profile, Project Shape, AI Tooling) — Complete 2026-05-16

- **Intent:** Build the primary selection screens that determine what gets
  scaffolded
- **Expected Outcome:** Wizard sections using eddacraft-tui widgets:
  1. **Profile** (single-select): solo dev / team / AI agent operator
  2. **Project shape** (single-select + conditional): monorepo vs single
     project. Monorepo selection exposes workspace tool detection (pnpm, turbo,
     nx, lerna) and per-package plan options.
  3. **AI tooling** (multi-select + per-tool config): select tools, then
     configure each — which agents to install, hook verbosity
     (full/minimal/none), model preferences where applicable.
     Back-navigation via Esc. Keyboard conventions match Anvil. Selections stored
     in wizard state.
- **Validation:** User can navigate forward and back through all sections;
  monorepo options only appear when monorepo selected; per-tool config only
  shows for selected tools; q/Ctrl+C exits cleanly
- **Confidence:** high
- **Dependencies:** TUI-001
- **Action plan:** [execution/TUI-002.actions.md](../execution/TUI-002.actions.md)
- **Results:** `aps init` now launches a Ratatui wizard with Profile, Project
  Shape, AI Tooling, per-tool config, and Summary sections. Wizard state has
  unit coverage for navigation, conditional monorepo options, selected-tool
  config, q/Ctrl+C exit handling, and summary-before-completion behavior.

### TUI-003: Implement template and path customization sections — Complete 2026-06-08

- **Intent:** Let users control what templates get installed and where files go
- **Expected Outcome:** Two additional wizard sections:
  1. **Templates** (multi-select): choose from available plan templates
     (quickstart, module, index, monorepo-index), option to specify custom
     template path. Profile and project shape inform defaults but user can
     override.
  2. **Paths** (text inputs with defaults): plans directory (default:
     `plans/`), docs location, tooling root (default: `.aps/`). Preview of
     resulting directory structure updates live as paths change.
  3. **Components** (checkbox): opt in/out of individual features — lint rules,
     aps-rules.md, project-context.md, designs/ directory, decisions/ directory
- **Validation:** Custom paths produce valid scaffold at specified locations;
  template selection matches scaffolded output; component toggles respected
- **Learning:** "eddacraft-tui KeyHandler maps j/k/q to navigation — text entry needs a literal key mapping while editing or typed characters get swallowed"
- **Confidence:** medium
- **Dependencies:** TUI-001
- **Files:** cli/src/wizard.rs, cli/src/main.rs
- **Status:** Complete: 2026-06-08
- **Results:** Three wizard steps added between ToolConfig and Done:
  Templates (multi-select with profile/shape-informed defaults + custom
  template path), Paths (three editable fields with live directory-structure
  preview), Components (five toggles, decisions/ off by default). Text
  editing uses a literal key mapping so q/j/k type instead of navigating;
  empty path commits restore defaults; custom template without a path is
  dropped on advance. 10 new state-machine tests (19 total), clippy clean.
  Scaffold-output validation (custom paths produce a valid scaffold) lands
  with TUI-004, which wires the scaffold execution these selections feed.

### TUI-004: Implement scaffold and summary steps

- **Intent:** Execute scaffold with visual progress and show results
- **Expected Outcome:** Two final wizard steps:
  1. **Scaffold** — spinner/progress widget showing each action (create dirs,
     install templates, install skills, install agents, configure hooks, run
     lint). Scaffold logic reimplemented in Rust (not shelling out to bash).
     Errors shown inline.
  2. **Summary** — results dashboard showing installed components, per-tool
     post-install instructions (e.g., `codex skills install`), custom paths
     used, next steps, doc links. Matches EddaCraft visual style.
- **Validation:** Scaffold produces correct file structure for all selection
  combinations; summary accurately reflects what was installed; progress
  renders without flicker
- **Confidence:** medium
- **Dependencies:** TUI-002, TUI-003
- **Files:** cli/src/wizard.rs, cli/src/scaffold.rs, cli/src/main.rs

### TUI-005: Non-interactive fallback and config-driven init

- **Intent:** Support CI, piped environments, and repeatable setups
- **Expected Outcome:** Two non-interactive paths:
  1. **Flags:** `aps init --non-interactive --profile solo --shape monorepo
--tools claude-code,copilot --plans-dir docs/plans` — all wizard options
     available as CLI flags.
  2. **Config file:** `aps init --from .aps/config.yml` — replay a previous
     configuration (enables team-wide standardization: commit config, teammates
     run `aps init --from`).
     Auto-detects non-TTY and falls back to flag mode with smart defaults.
- **Validation:** Both paths produce valid scaffold; config-driven init matches
  TUI-driven init for same selections; exit code 0/non-zero
- **Confidence:** high
- **Dependencies:** TUI-004
- **Files:** cli/src/main.rs, cli/src/wizard.rs, cli/src/config.rs

### TUI-006: Cross-compilation and release

- **Intent:** Distribute as pre-built binaries via GitHub releases
- **Expected Outcome:** CI workflow (GitHub Actions) that cross-compiles for 5
  targets using `cross` or `cargo-zigbuild`, creates GitHub release with
  binaries as assets. `curl | bash` installer updated to optionally download
  binary instead of bash scripts.
- **Validation:** Binaries run on each target platform; GitHub release has all
  5 assets; `curl` installer can fetch and install the binary
- **Confidence:** medium
- **Dependencies:** TUI-005
- **Files:** .github/workflows/release.yml, scaffold/install, cli/Cargo.toml

### TUI-007: Add setup mode picker

- **Intent:** Make setup choices obvious before APS writes bulky files.
- **Expected Outcome:** `aps setup` opens an `eddacraft-tui` picker with these
  top-level choices: install APS CLI on this machine, initialize minimal APS in
  this repo, initialize this repo for an AI agent, add tool integrations,
  configure hooks, and upgrade an existing APS project. Tool integration and
  component choices use MultiSelect; destructive upgrade actions use Confirm.
- **Validation:** `aps setup` can complete each top-level flow. `aps setup all`
  requires confirmation. `aps setup claude-code` and other shortcuts bypass the
  picker and install only the requested component. Shell fallback presents the
  same choices when the Rust TUI is unavailable.
- **Confidence:** high
- **Dependencies:** TUI-002, TUI-004, INSTALL-010, INSTALL-012
- **Files:** cli/src/main.rs, cli/src/setup.rs, bin/aps, scaffold/install
- **Related:** INSTALL-010 and INSTALL-012 define the shell and CLI command
  contracts that the TUI implements; listed as dependencies because this
  item's validation criteria are defined by those contracts.

### TUI-008: Add agent bootstrap flow

- **Intent:** Support the common remote-planning workflow where a user pastes a
  curl command to an agent before they are at their computer.
- **Expected Outcome:** The setup picker and non-interactive flags include an
  agent bootstrap mode. It creates minimal planning files plus agent-readable
  next steps, and does not install hooks, agents, local CLI runtime, or tool
  integrations unless selected. The summary tells the agent to read
  `plans/aps-rules.md`, ask for project intent, populate
  `plans/project-context.md`, draft `plans/index.aps.md`, and wait for an
  approved work item before implementation.
- **Validation:** `curl | bash -s -- --agent` and the picker path produce the
  same minimal agent-ready repo footprint.
- **Confidence:** high
- **Dependencies:** TUI-007, INSTALL-010
- **Files:** cli/src/setup.rs, scaffold/install
- **Related:** INSTALL-010 defines the public installer flag and shell fallback
  for this flow.

### TUI-009: Port `aps lint` and shared parser to Rust

- **Intent:** Implement D-028/D-031 — one Rust markdown parser serving lint,
  `next`, and the wizard, replacing the `lint`/`next` stubs shipped in TUI-001
- **Expected Outcome:** A parser module in `cli/` that reads `.aps.md` files
  (modules, work items, dependencies, decisions; status via both the
  `— Complete <date>` header suffix and explicit `- **Status:**` markers, per
  ORCH-001's conventions). On top of it: native `aps lint` implementing the
  bash rule set (same E/W codes and exit behavior) and native `aps next` with
  matching output including `--json`. Bash implementations stay untouched and
  are feature-frozen once parity is reached (orchestrate D-006). `start`,
  `complete`, and `graph` are explicitly out of scope here — they follow in a
  later item once parity is proven.
- **Validation:** Rust `aps lint` output matches `./bin/aps lint` on this
  repo's plans/ and on `test/fixtures/valid` + `test/fixtures/invalid`; Rust
  `aps next` matches bash output on `test/fixtures/orchestrate/`; parser unit
  tests pass via `cargo test`
- **Confidence:** medium
- **Dependencies:** TUI-001, D-028, D-031
- **Files:** cli/src/parser.rs, cli/src/lint.rs, cli/src/main.rs

## Execution Strategy

### Wave 1: Foundation

- TUI-001: Project setup + eddacraft-tui integration

### Wave 2: Wizard sections (depends on Wave 1, parallel)

- TUI-002: Core sections (profile, project shape, AI tooling)
- TUI-003: Template and path customization sections

### Wave 3: Scaffold + fallback (depends on Wave 2)

- TUI-004: Scaffold execution + summary dashboard
- TUI-005: Non-interactive fallback + config-driven init

### Wave 4: Distribution (depends on Wave 3)

- TUI-006: Cross-compilation and GitHub releases

### Wave 5: Setup UX cleanup (depends on Wave 3)

- TUI-007: Setup mode picker
- TUI-008: Agent bootstrap flow

### Wave 6: Native parser and lint port (depends on Wave 1; parallel with 2–5)

- TUI-009: Shared Rust parser + `aps lint` + `aps next` parity

## Relationship to Other Modules

| Module      | Relationship                                                         |
| ----------- | -------------------------------------------------------------------- |
| **INSTALL** | TUI replaces INSTALL's shell-prompt frontend; scaffold logic stays   |
| **ORCH**    | Shared markdown parser opportunity if lint is ported to Rust (D-028) |
| **VAL**     | Native lint port would subsume VAL's bash linter                     |

## Notes

- The shell-prompt wizard (`scaffold/install`) remains as the lightweight,
  zero-dependency fallback, but it should expose the same high-level choices as
  the TUI. It should not silently install the full project footprint.
- `eddacraft/eddacraft-tui` provides: Select, MultiSelect, Confirm, Spinner,
  Header, ResultsDashboard widgets plus the shared EddaCraft theme. APS should
  not duplicate these.
- `aps setup` is the primary post-init customization surface. Bare
  `aps setup` opens the picker; `aps setup <thing>` is the scripted shortcut.
- Keyboard conventions are shared across the Anvil product family — arrows/j-k
  for navigation, Enter to confirm, Space to toggle, Esc to go back, q to quit.
- The binary replaces `bin/aps` (bash) as the primary CLI. The bash version
  remains for users who don't want to download a binary.
- Cross-compilation target list: `x86_64-unknown-linux-gnu`,
  `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`,
  `x86_64-apple-darwin`, `x86_64-pc-windows-gnu`.
