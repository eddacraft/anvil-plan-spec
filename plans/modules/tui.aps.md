# TUI Onboarding Module

| ID  | Owner  | Status                  |
| --- | ------ | ----------------------- |
| TUI | @aneki | Complete                |

**Last reviewed:** 2026-06-09

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
- **Confidence:** medium
- **Dependencies:** TUI-001
- **Files:** cli/src/wizard.rs, cli/src/main.rs
- **Results:** Templates (multi-select with profile/shape-derived defaults +
  custom path entry), Paths (three text inputs with live directory preview;
  empty fields restore defaults), and Components (checkbox, all on by
  default) sections added after tool config. Text-entry steps switch the
  event loop to a raw character mapper so typing q/h/j/k/l edits instead of
  navigating; Ctrl+C still quits. Unit coverage for defaults, toggling,
  custom-path editing, and back-navigation.

### TUI-004: Implement scaffold and summary steps — Complete 2026-06-08

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
- **Results:** Review → Scaffold → Summary steps wired into the wizard.
  `plan_steps()` maps selections to a deterministic step list (pure,
  per-combination tests); `ScaffoldRun` executes one step per frame so
  progress renders without flicker; failures show inline and never halt the
  run; existing files are never overwritten. Content embedded at compile
  time from scaffold/ and templates/, so the binary scaffolds offline.
  Summary lists installed steps, errors, and per-tool post-install notes.
  The lint-rules component gates a structural verify step that was replaced
  by native lint in TUI-009.

### TUI-005: Non-interactive fallback and config-driven init — Complete 2026-06-08

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
- **Results:** Full wizard surface exposed as flags (--profile, --shape,
  --tools, --templates, --custom-template, --plans-dir, --docs-dir,
  --tooling-root, --components, plus --hooks/--model/--no-agents
  overrides). `aps init --from .aps/config.yml` replays the config the
  scaffold writes; flags override replayed values. Non-TTY auto-falls back
  to flag mode with wizard defaults. Verified: flag-driven and replayed
  scaffolds are byte-identical; collisions and invalid flags exit 1.

### TUI-006: Cross-compilation and release — Complete 2026-06-08

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
- **Results:** release.yml builds all 5 targets (linux x64/arm64 on native
  runners, darwin x64/arm64 on macOS, windows x64 via mingw-w64 — native
  runners replaced `cross`/`cargo-zigbuild` since GitHub now provides arm64
  runners), smoke-tests native builds, and publishes tar.gz/zip assets with
  SHA256SUMS on v* tags. Release profile (lto, strip) yields ~1.3 MB
  binaries. `scaffold/install --binary` downloads the matching release
  asset and falls back to the bash CLI on any failure. First release tag
  after merge will exercise the workflow end-to-end.

### TUI-007: Add setup mode picker — Complete 2026-06-08

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
- **Results:** `aps setup` opens a picker with all six flows plus a full-
  footprint option; `aps setup <thing>` shortcuts (cli, init, agent, hooks,
  upgrade, all, or a tool name) bypass it. Bulky/destructive flows gate
  behind Confirm in the picker and a TTY prompt or --yes in shortcuts.
  Upgrade refreshes only generated files that already exist. The bash-side
  picker (`bin/aps setup`) remains INSTALL-012's deliverable; the Rust
  binary implements the full command contract.

### TUI-008: Add agent bootstrap flow — Complete 2026-06-08

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
- **Results:** `aps setup agent` and `scaffold/install --agent` both create
  the minimal footprint (plans/ with index, rules, project-context) plus
  `plans/agent-next-steps.md` carrying the read-rules → ask-intent →
  populate-context → draft-index → wait-for-approval workflow. Verified the
  two paths produce identical file trees.

### TUI-009: Port `aps lint` and shared parser to Rust — Complete 2026-06-08

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
- **Results:** Shared parser in cli/src/parser.rs (file typing, metadata
  tables, sections, work items, field continuations, status normalization,
  dependency tokens) serving cli/src/lint.rs and cli/src/next.rs. Parity
  verified byte-for-byte against `./bin/aps` on this repo's plans/, both
  fixture sets (text + --json), and orchestrate fixtures (default, module
  filter, no-match, single file, missing target) including exit codes.
  Bash quirks preserved deliberately (section-relative W010/W011 line
  numbers, single-line JSON). Bash lint/next are now feature-frozen per
  orchestrate D-006.

### TUI-010: Add bracketed paste support to wizard text entry

- **Intent:** Carry forward council finding C-006 (session council-e077b725,
  deferred during the TUI-003 review): without bracketed paste, a multi-line
  clipboard replays each newline as an Enter keypress, which can drive the
  wizard through Components and Review into scaffold execution without user
  review.
- **Expected Outcome:** `EnableBracketedPaste` is set alongside
  `EnterAlternateScreen` (and cleared by `TerminalGuard` on exit). The event
  loop handles `Event::Paste` explicitly during text entry: pasted text is
  sanitized (control/bidi/zero-width characters stripped per `is_text_char`,
  first line only) and inserted into the focused field. Paste events outside
  text-entry steps are ignored. This also gives pasted input the same single
  sanitization choke point as typed input.
- **Validation:** Unit tests for the paste-sanitization helper (multi-line
  input truncates to first line, control/bidi characters stripped); manual
  pty test confirms pasting multi-line text into a path field inserts only
  the first line and does not advance the wizard
- **Learning:** "crossterm gates Event::Paste behind the bracketed-paste feature flag — without it the variant doesn't exist and paste arrives as replayed keystrokes"
- **Confidence:** high
- **Dependencies:** TUI-003
- **Files:** cli/src/wizard.rs, cli/Cargo.toml
- **Status:** Complete: 2026-06-08
- **Results:** `EnableBracketedPaste` set on wizard entry and cleared by
  `TerminalGuard` (crossterm `bracketed-paste` feature enabled).
  `Event::Paste` routes to `WizardState::paste()`, which is a no-op outside
  text entry and otherwise inserts `sanitize_paste(text)` — first line only,
  control/bidi/zero-width characters stripped via the same `is_text_char`
  filter as typed input. 4 new unit tests (78 total). Behavioral smoke test:
  a three-line paste into the Paths step in an empty directory creates zero
  files and the wizard stays on Paths; previously the newlines replayed as
  Enter and drove the wizard into scaffold execution.

### TUI-011: Wizard input hardening — key-release filter and index exclusivity — Complete 2026-06-09

- **Intent:** Carry forward two findings from council session council-b2bd78ac
  (the review of the superseded PR #60) that TUI-010 did not cover, so the
  full council pass on the wizard is accounted for.
- **Expected Outcome:** (1) the event loop ignores non-`Press` key events at
  both poll sites, so terminals that also report key releases (Windows) do not
  double every keystroke and navigation step; (2) selecting `Index` or
  `MonorepoIndex` deselects the other, since both scaffold `index.aps.md` and
  selecting both would have the scaffold write one over the other.
- **Validation:** `cargo test` unit test asserts toggling one index template
  clears the other and at most one is ever selected; key-release filtering is
  exercised through the event loop (manual check on a release-reporting
  terminal).
- **Confidence:** high
- **Dependencies:** TUI-003, TUI-010
- **Files:** cli/src/wizard.rs
- **Status:** Complete: 2026-06-09
- **Results:** `KeyEventKind::Press` guard added to the scaffold-step and
  main poll sites. `toggle_template` makes `Index`/`MonorepoIndex` mutually
  exclusive. 1 new unit test (80 total); fmt + clippy `-D warnings` clean
  (the gate landed in #65).

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

### Wave 7: Hardening follow-ups (post-release)

- TUI-010: Bracketed paste support (council C-006)

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
