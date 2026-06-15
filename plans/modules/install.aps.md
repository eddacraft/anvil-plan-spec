# Install v2 Module

| ID      | Owner  | Priority | Status                  |
| ------- | ------ | -------- | ----------------------- |
| INSTALL | @aneki | high     | In Progress (follow-up) |

**Last reviewed:** 2026-06-15

## Purpose

Overhaul how APS gets distributed to user projects. Replace the current
scattered install footprint (`aps-planning/`, `bin/`, `lib/`,
`.claude/commands/`) with a clean `.aps/` tooling root, an interactive
installer that adapts to project type and AI tooling, and optional agent
packages.

## Background

The current install (v1) was built for Claude Code-only, single-project use.
Since then:

- Claude Code commands are being deprecated in favour of skills
- Users work across multiple AI tools (Claude Code, Codex, Copilot)
- APS has agents (Planner, Librarian) that live in personal configs but
  aren't distributable
- The `aps-planning/` directory clutters project roots with non-obvious naming
- `bin/` + `lib/` scatter 18+ files across two top-level directories
- No project type detection (monorepo vs simple gets identical install)

Follow-up review in May 2026 found the public `curl | bash` entrypoint still
ships the old v1 footprint (`aps-planning/`, `bin/`, `lib/`, and
`.claude/commands/`) even though `aps init` has moved toward the v2 layout.
The distribution model needs one more cleanup pass: separate CLI installation,
minimal repo initialization, agent bootstrap, optional setup, and upgrade.

## In Scope

- `.aps/` directory as single tooling root
- `config.yml` to record install choices (profile, scope, tools, options)
- Shell-prompt install wizard (profile, scope, AI tools) with non-interactive
  fallback
- Skill file generation replacing deprecated commands
- Optional agent packaging (APS Planner, Librarian)
- Multi-tool support: Claude Code, Codex, Copilot, Gemini, OpenCode, generic
- Migration path from v1 layout to v2
- Update script respecting `config.yml`
- `aps-rules.md` split: APS-managed rules + user-owned `project-context.md`
- `plans/designs/` and `plans/issues.md` as scaffold artifacts
- Agent context bootstrap for `project-context.md`
- Public installer mode picker (`install`, `init`, `agent-init`, `setup`,
  `upgrade`)
- Minimal default repo footprint with bulky integrations opt-in
- Upgrade cleanup for v1 and bulky v2 project installs
- Global release binary as the default CLI (Mac, Linux, Windows)
- Multi-channel distribution aligned to release semver (GitHub releases,
  install script, crates.io, Scoop)
- `.aps/config.yml` as the per-project contract (`cli_version`, runtime
  `plans_dir` / `docs_dir` / `tooling_root`)
- Runtime config discovery so global `aps` respects project paths without
  `--plans` or direnv
- Documented migration from vendored bash CLI + direnv to global binary

## Out of Scope

- Changing APS spec format or templates
- MCP server (separate TASKS module concern)
- Tool-specific plugin/extension development
- TUI wizard (future replacement for shell-prompt wizard)
- `mise` / `asdf` manifest generation (document manual mirroring of
  `cli_version` for now; optional follow-up)

Follow-up scope note: the TUI remains the richer interactive frontend, but the
public shell installer must expose the same high-level choices in a lightweight
picker so users do not have to read documentation before choosing the safe path.

## Interfaces

**Depends on:**

- SCAFFOLD (Complete) — evolves the existing scaffold
- VAL (Complete) — CLI moves into `.aps/` but linting logic unchanged

**Exposes:**

- `scaffold/install` — new interactive installer (replaces current)
- `scaffold/agent-init` — safe repo bootstrap for agents without assuming a
  global CLI install
- `scaffold/update` — migration-aware updater
- `aps setup` — interactive setup picker for optional integrations
- `aps upgrade` — dry-run-first cleanup and migration for existing projects
- `.aps/config.yml` — project configuration schema
- `.claude/skills/aps-planning/` — skill files (cross-tool compatible)
- `.agents/skills/aps-planning/` — Codex/Gemini-compatible copy (optional)
- `.claude/agents/aps-planner.md` — optional agent (Claude Code)
- `.claude/agents/aps-librarian.md` — optional agent (Claude Code)

## Decisions

- **D-011:** `.aps/` as tooling root — _decided: yes, consolidate all
  APS-owned files (CLI, scripts, config, ephemeral) under `.aps/`_
- **D-012:** CLI location — _decided: `.aps/bin/aps` with PATH hint.
  **Amended by D-034 (2026-06-15):** primary CLI is the global release binary
  on PATH; `.aps/bin/` is optional for vendored/pinned toolchains only. direnv
  is optional, not the default activation path._
- **D-013:** Skill/instruction format per tool — _decided, see below_
- **D-014:** Agent model defaults — _decided: Planner on Opus, Librarian on
  Sonnet_
- **D-022:** External planning repo reversed — _decided: plans move back to
  main repo_
- **D-023:** Commands fully dropped — _decided: skills only, no
  `.claude/commands/` shipped_
- **D-024:** aps-rules.md split — _decided: `aps-rules.md` (APS-managed) +
  `project-context.md` (user-owned)_
- **D-025:** designs/ and issues.md into plans/ — _decided: single planning
  content root_
- **D-030:** Public install entrypoint — _decided: `curl | bash` must show an
  upfront mode picker when TTY is available. Non-interactive use must support
  explicit flags (`--cli`, `--init`, `--agent`, `--upgrade`) and avoid surprise
  bulky installs._
- **D-031:** Command split — _decided: `install` installs the CLI, `aps init`
  creates minimal planning files, `aps setup` adds optional integrations,
  `agent-init` bootstraps a repo for remote/agent-led planning, and
  `aps upgrade` cleans existing projects._
- **D-032:** Default footprint — _decided: bare defaults only. Hooks, agents,
  skill copies, local vendored runtime, release templates, and extra tool
  integrations are opt-in choices._
- **D-033:** Upgrade safety — _decided: generated legacy files can be removed
  only after backup. Ambiguous or user-owned files are reported for manual
  review._
- **D-034:** Global binary-first distribution — _decided 2026-06-15: the
  default install path ships the release `aps` binary to the machine
  (`~/.aps/bin` or package manager), not a vendored bash `bin/` + `lib/` tree
  into every project. `aps init` scaffolds planning content and
  `.aps/config.yml` only. Bash CLI remains an opt-in fallback (`--bash`, air
  gap, or explicit `--local-cli`). Cross-platform: GitHub release assets
  (TUI-006), install script, crates.io (`cargo install` / `cargo binstall`),
  Scoop manifest on Windows. Amends D-012._
- **D-035:** `.aps/config.yml` project contract — _decided 2026-06-15: the
  file is the committed per-repo manifest, not init replay metadata only.
  Required fields for the contract: `cli_version` (semver pin for the global
  toolchain), `plans_dir`, `docs_dir`, `tooling_root`. The global binary
  discovers the nearest `.aps/config.yml` by walking up from cwd. Explicit
  flags (`--plans`, etc.) override config. `cli_version` mismatch warns locally
  and may fail in `--strict` / CI. `aps init` writes `cli_version` from the
  running binary._
- **D-036:** Install-channel version alignment — _decided 2026-06-15: GitHub
  release tag, install-script `VERSION=`, crates.io publish, and Scoop manifest
  all reference the same semver. Project pinning is `cli_version` in
  `.aps/config.yml`, not channel-specific pins. CI installs the pinned release
  then runs `aps` without extra flags when config is present._

**D-013 Detail: Multi-Tool Skill Compatibility (Researched 2026-02-19)**

Five target tools support a `<name>/SKILL.md` skill format with identical
frontmatter (name + description). The differences are discovery paths and
whether skills require explicit installation.

| Tool            | Skill Paths (project)                                     | Auto-discover? | Install Cmd?                             |
| --------------- | --------------------------------------------------------- | -------------- | ---------------------------------------- |
| **Claude Code** | `.claude/skills/`                                         | Yes            | No                                       |
| **Codex**       | `.agents/skills/`                                         | No             | `codex skills install <path>`            |
| **Copilot**     | `.github/skills/`, `.claude/skills/`                      | Yes            | No                                       |
| **OpenCode**    | `.opencode/skills/`, `.claude/skills/`, `.agents/skills/` | Yes            | No                                       |
| **Gemini**      | `.gemini/skills/`, `.agents/skills/`                      | No             | `gemini skills install <path>` or `link` |

**Convergence points:**

- `.claude/skills/` — auto-discovered by Claude Code, Copilot, OpenCode (3/5)
- `.agents/skills/` — used by Codex, Gemini, OpenCode (3/5) but Codex and
  Gemini require explicit install/link

**Install-required tools (Codex, Gemini):** Just dropping files into
`.agents/skills/` is NOT enough. Users must run an install or link command.
The APS installer should:

1. Place skill files at `.agents/skills/aps-planning/` (shared location)
2. Print post-install instructions for tools that need them:
   - Codex: `codex skills install .agents/skills/aps-planning`
   - Gemini: `gemini skills link . --scope workspace` (links project skills)
3. Or attempt to run the install command automatically if the tool CLI is
   detected on PATH

Instruction files (project-level guidance, not skills):

| Tool            | Instruction File                                  |
| --------------- | ------------------------------------------------- |
| **Claude Code** | `CLAUDE.md`                                       |
| **Codex**       | `AGENTS.md` (hierarchical, concatenated root→cwd) |
| **Copilot**     | `AGENTS.md`, `.github/copilot-instructions.md`    |
| **Gemini**      | `GEMINI.md`                                       |
| **OpenCode**    | N/A (uses skills)                                 |

**Strategy:** Primary skill install to `.claude/skills/aps-planning/`
(auto-discovered by 3 tools). Copy to `.agents/skills/aps-planning/` for
Codex/Gemini users (with post-install instructions for their CLIs). For
instruction files, append an APS section to `AGENTS.md` (shared by Codex +
Copilot) and/or `GEMINI.md`. Agents are Claude Code-specific
(`.claude/agents/`).

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] D-011 confirmed (.aps/ as tooling root)
- [x] D-012 confirmed (.aps/bin/aps with PATH hint)
- [x] D-013 resolved (multi-tool research complete)
- [x] D-014 confirmed (Planner=Opus, Librarian=Sonnet)
- [x] Work items defined with validation

## Work Items

### INSTALL-001: Define `.aps/` directory structure

- **Intent:** Establish the canonical layout for all APS-owned tooling files
- **Expected Outcome:** Documented directory structure showing where CLI,
  scripts, config, and ephemeral files live under `.aps/`. Skills install to
  `.claude/skills/aps-planning/` (cross-tool compatible) with optional copy
  to `.agents/skills/aps-planning/` for Codex. Agents install to
  `.claude/agents/`. Hook scripts live under `.aps/scripts/`.
- **Validation:** Structure documented in this spec; team agrees on layout
- **Confidence:** high
- **Non-scope:** `plans/` directory (unchanged)
- **Status:** Complete: 2026-02-19 — canonical layout documented in Notes
  section of this spec with full project layout diagram

### INSTALL-002: Create `config.yml` schema

- **Intent:** Record install-time choices so update script can refresh without
  re-asking questions
- **Expected Outcome:** YAML schema covering: APS version, project type
  (simple/monorepo), selected AI tools (multi-value), skill install flag,
  hooks preference (full/minimal/none), optional agents list
- **Validation:** Example config.yml in this spec; schema covers all install
  choices
- **Confidence:** high
- **Status:** Complete: 2026-02-19 — schema documented below

#### config.yml Schema

```yaml
# .aps/config.yml — written by installer, read by updater
aps:
  version: "0.3.0" # APS release version installed
  config_schema: 1 # config.yml schema version
  installed: "2026-02-19" # date of initial install
  updated: "2026-02-19" # date of last aps update

project:
  type: simple # simple | monorepo
  monorepo_tool: ~ # pnpm | turbo | lerna | nx (null if simple)
  profile: solo # solo | team | agent

tools: # Multi-select — one or more entries
  - name: claude-code
    skill: .claude/skills/aps-planning
    hooks: full # full | minimal | none
    agents: # Optional list
      - aps-planner
      - aps-librarian
  - name: codex
    skill: .agents/skills/aps-planning
    instruction_file: AGENTS.md
  - name: gemini
    skill: .agents/skills/aps-planning
    instruction_file: GEMINI.md
  - name: copilot
    skill: .claude/skills/aps-planning
    instruction_file: AGENTS.md
  - name: opencode
    skill: .claude/skills/aps-planning
  - name: generic # No tool integration
```

Notes on schema:

- `tools` is a list — multi-select produces multiple entries
- Each tool entry records where its files were placed
- `hooks` and `agents` only apply to claude-code
- `instruction_file` records which file got an APS section appended
- Updater reads this to know what to refresh without re-asking
- `profile` determines template defaults and guidance tone
- Canonical tool identifiers: `claude-code`, `copilot`, `codex`, `opencode`,
  `gemini`, `generic`

### INSTALL-003: Build shell-prompt install wizard — Complete 2026-03-28

- **Intent:** Replace the current non-interactive install with a guided
  shell-prompt wizard that adapts to the project
- **Expected Outcome:** Install script with three prompts:
  1. **Profile** (single-select): solo dev / team adoption / AI agent setup
  2. **Scope** (single-select): small feature / module / multi-module / monorepo
  3. **AI Tooling** (multi-select): Claude Code / Copilot / Codex / OpenCode /
     Gemini / None
     Followed by scaffold, agent context bootstrap message, and `aps lint`
     verification. Non-interactive fallback via flags:
     `--profile solo --scope small --tools claude-code,copilot`. Writes choices
     to `.aps/config.yml`.
- **Validation:** Run install in test dir; script asks questions, creates
  `.aps/config.yml` with answers, installs correct files per selection;
  non-interactive mode works with flags
- **Confidence:** medium
- **Dependencies:** INSTALL-001, INSTALL-002
- **Results:** Implemented in `lib/scaffold.sh` cmd_init with prompt_select,
  prompt_multi, and full non-interactive flag support. Tested with single-tool
  and multi-tool selections. APS_LOCAL env var for local development.

### INSTALL-004: Convert commands to skill — Complete 2026-03-28

- **Intent:** Replace deprecated `.claude/commands/` with skill format
- **Expected Outcome:** `/plan` and `/plan-status` behaviours merged into the
  SKILL.md at `.claude/skills/aps-planning/SKILL.md`. Supporting files
  (reference.md, examples.md) go alongside it. `hooks.md` moves to docs
  (human reference, not agent content). No `.claude/commands/` created.
- **Validation:** Skill triggers on "plan this project" and "what's the plan
  status" intent; no `.claude/commands/` directory created
- **Confidence:** high
- **Non-scope:** Skill content rewrite (just repackaging)
- **Results:** v2 init installs skill to `.claude/skills/aps-planning/`.
  No `.claude/commands/` created. Migration backs up old commands to
  `.aps/backup/commands/`.

### INSTALL-005: Package APS Planner agent — Complete 2026-03-28

- **Intent:** Ship a dispatchable APS planning agent that users can opt into
- **Expected Outcome:** `.claude/agents/aps-planner.md` derived from the
  existing `anvil-plan-spec.md` agent in code-env, adapted for
  distribution. Install places it when user selects the option.
- **Validation:** Agent file installs to `.claude/agents/` when selected;
  agent can be dispatched via Task tool
- **Confidence:** high
- **Dependencies:** INSTALL-003 (needs interactive install to offer it)
- **Results:** Agent installed automatically when claude-code tool selected.
  Agent files from scaffold/agents/claude-code/.

### INSTALL-006: Package Librarian agent — Complete 2026-03-28

- **Intent:** Ship an optional repo hygiene agent alongside the planner
- **Expected Outcome:** `.claude/agents/aps-librarian.md` derived from the
  existing `librarian.md` agent in code-env, scoped to APS-relevant
  concerns (archiving, cross-refs, orphan detection)
- **Validation:** Agent file installs to `.claude/agents/` when selected;
  agent scans plans/ and reports findings
- **Confidence:** high
- **Dependencies:** INSTALL-003
- **Results:** Installed alongside planner when claude-code tool selected.

### INSTALL-007: Add multi-tool instruction generation — Complete 2026-03-28

- **Intent:** Support all major AI coding tools with appropriate skill and
  instruction files
- **Expected Outcome:** Install generates tool-appropriate files per selection:
  - **Claude Code:** skill at `.claude/skills/aps-planning/`, agents at
    `.claude/agents/`, hooks in `settings.local.json`
  - **Codex:** skill at `.agents/skills/aps-planning/`, agents at
    `.codex/agents/*.toml` + config entries in `.codex/config.toml`,
    APS section appended to `AGENTS.md`. Post-install: print
    `codex skills install` command
  - **Copilot:** skill at `.claude/skills/aps-planning/`, agents at
    `.github/agents/`, APS section appended to `AGENTS.md`
  - **Gemini:** skill at `.agents/skills/aps-planning/`, APS section appended
    to `GEMINI.md`. Post-install: print `gemini skills link` command
  - **OpenCode:** skill at `.claude/skills/aps-planning/`, agents at
    `.opencode/agents/`
  - **Generic:** just `plans/` + CLI, no tool integration
    Multi-select means a single install can target multiple tools (e.g. Claude
    Code + Codex installs to both `.claude/skills/` and `.agents/skills/`).
- **Validation:** Install with each tool selection produces the expected files
  in the expected locations; install-required tools get printed instructions
- **Confidence:** medium
- **Dependencies:** INSTALL-003
- **Results:** All 6 tool targets implemented with correct file placement.
  Multi-select tested with claude-code,codex,copilot combo. Post-install
  instructions printed for Codex and Gemini. v2_install_tools dispatches
  per tool.

### INSTALL-008: Build migration from v1 to v2 — Complete 2026-03-28

- **Intent:** Existing APS users can update without manual restructuring
- **Expected Outcome:** `aps migrate` detects v1 layout (presence of
  `aps-planning/`, `bin/aps`, `.claude/commands/plan.md`), moves files to
  `.aps/`, updates hook paths in `settings.local.json`, removes old dirs,
  creates `config.yml` (inferring choices from what was installed), splits
  `aps-rules.md` into APS-managed + `project-context.md`, moves `designs/`
  to `plans/designs/`, backs up removed files to `.aps/backup/`. Supports
  `--dry-run` to preview without modifying.
- **Validation:** Run migrate in a v1 project; old directories removed, `.aps/`
  created, hooks still work, plans/ untouched, backup exists, dry-run mode
  previews without changes
- **Confidence:** medium
- **Dependencies:** INSTALL-001, INSTALL-002, INSTALL-003, INSTALL-009
- **Risks:** Edge cases in hook path rewriting; users with custom
  modifications to scaffolded files; aps-rules.md split heuristic
- **Results:** cmd_migrate implemented with dry-run, confirmation prompt,
  file moves, backup, cleanup, hook path rewriting, and config inference.
  Tested end-to-end on simulated v1 layout.

### INSTALL-009: Split aps-rules.md and add project-context.md — Complete 2026-03-28

- **Intent:** Separate APS-managed format rules from user-owned project context
- **Expected Outcome:** New `aps-rules.md` template containing only APS format
  rules (hierarchy, naming, status flows, work item structure, action plan
  format). New `project-context.md` template with sections for overview, team,
  tech stack, conventions, and active decisions. Agent bootstrap contract:
  planner agent populates `project-context.md` on first run if it contains
  TODO markers.
- **Validation:** Fresh install produces both files; `aps-rules.md` contains no
  project-specific content; `project-context.md` has TODO markers; planner
  agent can detect and populate it
- **Confidence:** high
- **Dependencies:** INSTALL-001
- **Results:** `aps-rules-v2.md` template created with v2 file locations
  (plans/designs/, project-context.md). `project-context.md` template with
  HTML comment TODO markers. Both installed by v2 init and migrate.

### INSTALL-010: Split install, init, setup, agent bootstrap, and upgrade — Complete 2026-06-14

- **Intent:** Make the public APS entrypoints match the actual user journeys
  without forcing a large project-local footprint.
- **Expected Outcome:** `scaffold/install` no longer defaults to project
  scaffolding. In a TTY it shows a mode picker: install APS CLI on this
  machine, initialize APS in this repo, initialize this repo for an AI agent,
  upgrade an existing APS project, or add a tool integration. Non-interactive
  flags provide the same choices: `--cli`, `--init`, `--agent`, `--upgrade`,
  and `--setup <tool>`.
- **Validation:** Running the advertised `curl | bash` command in a TTY shows
  the picker before writing files. Running with `--cli` installs only the CLI.
  Running with `--init` creates only minimal planning files. Running with
  `--agent` creates minimal planning files plus agent-readable next steps.
- **Learning:** "-r /dev/tty is not enough to know a terminal is usable — on
  a detached process (curl|bash in CI) the node exists but opening it fails;
  probe by actually opening it"
- **Confidence:** high
- **Dependencies:** INSTALL-003, INSTALL-008, INSTALL-009
- **Files:** scaffold/install, scaffold/install.ps1, docs/installation.md,
  README.md
- **Status:** Complete: 2026-06-14
- **Action plan:** [execution/INSTALL-010.actions.md](../execution/INSTALL-010.actions.md)
- **Results:** `scaffold/install` no longer defaults to the heavy project
  scaffold. Flags `--cli`/`--init`/`--agent`/`--upgrade`/`--setup <tool>`
  select a mode; with none, a TTY picker runs and a non-terminal session
  prints usage and exits non-zero (no silent scaffold). The old default
  scaffold became `install_init`; `--global` aliases `--cli`. `--upgrade`
  hands to the update entrypoint (deep cleanup is INSTALL-013); `--setup`
  ensures a CLI then runs `aps setup <tool>`. PowerShell `install.ps1`
  brought to the same mode surface (it previously had only `--global`).
  Tests 30–31 cover the mode contract and arg guards; docs + README show
  the picker and flags. Full suite green; `aps lint` + markdownlint clean.

### INSTALL-011: Make `aps init` minimal by default

- **Intent:** Keep APS adoption lightweight for new projects and avoid making
  the repo look like it vendors an SDK.
- **Expected Outcome:** Default `aps init` creates bare planning content only:
  `plans/index.aps.md`, `plans/aps-rules.md`, `plans/project-context.md`, and
  the minimum directories needed for modules/execution. It does not install
  hooks, agents, skill copies, `.claude/commands/`, root `aps-planning/`, or a
  project-local CLI runtime unless explicitly selected.
- **Validation:** Fresh `aps init --non-interactive` has no root
  `aps-planning/`, no root `bin/`, no root `lib/`, no `.claude/commands/`, and
  no `.aps/lib/` unless `--local-cli` or equivalent is selected.
- **Confidence:** high
- **Dependencies:** INSTALL-010
- **Files:** lib/scaffold.sh, scaffold/install, scaffold/install.ps1,
  test/run.sh
- **Status:** Complete: 2026-06-15
- **Results:** `aps init` (bash `cmd_init`) now scaffolds planning content +
  `.aps/config.yml` only. The vendored bash CLI (`.aps/bin` + `.aps/lib`) and
  hook scripts (`.aps/scripts`) are gated behind new `--local-cli`/`--bash`
  and `--hooks` flags; PATH/direnv setup and the layout printout follow suit.
  `.aps/.gitignore` (context ignore) moved into `write_config` so it is always
  written. The curl `scaffold/install` and `scaffold/install.ps1` `--init`
  modes were rewritten to match: minimal v2 templates + `write_min_config`,
  no root `bin/`/`lib/`, no `aps-planning/`, no `.claude/commands/`; same
  opt-in flags. Test 17 rewritten to assert the minimal footprint, Test 17b
  covers `--local-cli`/`--hooks`, Test 33 statically guards the curl
  installers. docs/installation.md updated. Full suite green; markdownlint
  clean.

### INSTALL-012: Add `aps setup` picker and shortcuts

- **Intent:** Give users a clear place to add optional integrations after the
  minimal plan exists.
- **Expected Outcome:** `aps setup` with no arguments opens a picker for common
  setup tasks: install APS CLI on this machine, initialize minimal planning in
  this repo, agent bootstrap, tool integrations, hooks, and upgrade. Shortcut
  forms remain available: `aps setup claude-code`, `aps setup opencode`,
  `aps setup codex`, `aps setup hooks`, `aps setup agent`, and
  `aps setup all`.
- **Validation:** `aps setup` is safe and asks before writing optional files.
  Direct shortcuts install only the requested component. `aps setup all`
  requires confirmation before installing the full footprint.
- **Confidence:** high
- **Dependencies:** INSTALL-010
- **Related:** TUI-007 provides the richer picker frontend for this command.
- **Files:** bin/aps, lib/scaffold.sh, cli/src/, docs/installation.md
- **Status:** Complete: 2026-06-15
- **Results:** The native `aps setup` (cli/src/setup.rs, TUI-007/008) already
  shipped the Ratatui picker, shortcut keys (`cli`/`init`/`agent`/`hooks`/
  `upgrade`/`all` + tool names), and confirmation gating for `all`/`upgrade`.
  This work brings the bash CLI to parity: new `cmd_setup` in lib/scaffold.sh
  plus `bin/aps` dispatch provides a numbered picker when no component is
  named, the same shortcut keys, and `setup all` confirmation (skippable with
  `--yes`). Each shortcut writes only its component (verified by Test 34:
  hooks pulls no CLI/skill; a tool key installs only that skill+agents; an
  unknown target exits non-zero; `all` without confirmation writes nothing).
  Bare `aps setup` errors clearly when non-interactive. docs/installation.md
  gains an "Add Integrations" section. Full suite green; markdownlint clean.

### INSTALL-013: Add safe upgrade cleanup for existing projects

- **Intent:** Help existing APS users remove or migrate generated bloat without
  risking user-authored planning content.
- **Expected Outcome:** `aps upgrade` detects v1 and bulky v2 layouts, shows a
  dry-run by default, creates `.aps/backup/<timestamp>/` before deleting or
  moving generated files, and rewrites known hook paths from
  `aps-planning/scripts/` to the selected current location when hooks are kept.
  A companion `scaffold/upgrade` curl entrypoint supports agent-safe dry-runs.
- **Validation:** Upgrade never deletes `plans/**`, `AGENTS.md`, `CLAUDE.md`,
  `GEMINI.md`, or user-modified settings automatically. Known generated files
  (`aps-planning/hooks.md`, `.claude/commands/plan.md`, root `bin/`, root
  `lib/`, superseded `.aps/lib/`) are backed up before removal. Ambiguous files
  are listed for manual review.
- **Confidence:** high
- **Dependencies:** INSTALL-008, INSTALL-010
- **Files:** lib/scaffold.sh, scaffold/upgrade, docs/installation.md,
  test/run.sh
- **Status:** Complete: 2026-06-15
- **Results:** New `cmd_upgrade` in lib/scaffold.sh (+ `bin/aps` dispatch) and
  a self-contained `scaffold/upgrade` curl entrypoint. Both dry-run by default
  and detect generated bloat: root `bin/aps` + `lib/`, v1 `aps-planning/`,
  `.claude/commands/plan*.md`, and superseded `.aps/bin` + `.aps/lib`. On
  `--apply` every removed path is copied to `.aps/backup/<timestamp>/` first;
  `plans/**`, `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, and settings are never
  deleted, and hook paths in `settings.local.json` are rewritten
  `aps-planning/scripts/` -> `.aps/scripts/` (with a backup) when kept. A
  `lib/` mixing APS and non-APS files is reported ambiguous and left
  untouched. The curl entrypoint refuses to modify non-interactively without
  `--yes`, so its dry run is agent-safe. A `set -e` foot-gun (a scan function
  returning the status of a trailing `&&` short-circuit) was fixed with an
  explicit `return 0`. Tests 35–36 cover dry-run, backup-before-remove,
  protections, hook rewrite, ambiguity, and the curl entrypoint. docs updated;
  full suite green; markdownlint clean.

### INSTALL-014: Extend `.aps/config.yml` as the project contract

- **Intent:** Formalize per-project toolchain and path pinning so global `aps`
  knows which release and which directories a repo expects.
- **Expected Outcome:** `.aps/config.yml` gains `cli_version: "x.y.z"` written
  by `aps init` from the running binary. `plans_dir`, `docs_dir`, and
  `tooling_root` are documented as runtime defaults (not init-only). Schema
  docs cover forward-compatible unknown keys. Bash and Rust init paths write
  the same shape. `aps init --from` replays `cli_version` when absent in older
  configs (warn + inherit current binary version).
- **Validation:** Round-trip parse tests in `cli/src/config.rs`; fixture
  `test/fixtures/config/` with alternate `plans_dir`; `aps lint plans` passes
  on this repo after `cli_version` is added to `.aps/config.yml` (if dogfooded)
- **Confidence:** high
- **Dependencies:** INSTALL-002, TUI-005
- **Files:** cli/src/scaffold.rs, cli/src/config.rs, lib/scaffold.sh,
  docs/installation.md, test/fixtures/
- **Status:** Complete: 2026-06-15
- **Results:** `Selections` gained a `cli_version` field (cli/src/scaffold.rs)
  plus a `CLI_VERSION = env!("CARGO_PKG_VERSION")` constant; `config_yaml`
  emits `cli_version` first, `parse_config` reads it, and `build_selections`
  stamps the running binary's version (warning when an older `--from` config
  predates the field, preserving an explicit pin otherwise). The bash
  `write_config` and both curl installers' `write_min_config` now emit the same
  top-level contract keys: `cli_version`, `plans_dir`, `docs_dir`,
  `tooling_root`. Fixture `test/fixtures/config/alt-plans-dir.yml` pins an
  alternate `plans_dir`; new Rust tests cover round-trip, stamp/replay, and the
  fixture; bash Test 37 asserts the contract keys. This repo now dogfoods
  `.aps/config.yml`; `aps lint plans/` stays clean. Unknown keys remain ignored
  for forward compatibility. cargo test (82) + bash suite green; fmt + clippy +
  markdownlint clean.

### INSTALL-015: Ship global binary-first install channels

- **Intent:** Make the release binary the default way users obtain `aps` on
  Mac, Linux, and Windows — one semver across all channels.
- **Expected Outcome:** Install script defaults to `--global --binary` when no
  project scaffold is requested. `install.ps1` mirrors the behaviour on Windows
  (User PATH + `aps.exe`). crates.io publishes the `aps-cli` crate with
  installable binary (`cargo install aps-cli --version …` / `cargo binstall`).
  Scoop manifest (`aps.json`) installs pinned releases. Release workflow
  documents the bump checklist (tag → GitHub assets → crates.io → Scoop).
  Installer file manifests include the full bash `lib/` set (including
  `audit.sh`) when bash fallback is selected.
- **Validation:** Smoke `aps --version` and `aps lint --help` on all five
  TUI-006 targets; `cargo publish --dry-run` clean; Scoop manifest install in
  CI or documented manual check; `VERSION=x.y.z curl …/install | bash -s --
--global --binary` installs exactly that release
- **Confidence:** medium
- **Dependencies:** TUI-006, INSTALL-010
- **Files:** scaffold/install, scaffold/install.ps1, cli/Cargo.toml,
  `.github/workflows/release.yml`, docs/installation.md
- **Status:** Draft

### INSTALL-016: Runtime project config discovery (alternate `plans_dir`)

- **Intent:** Let a global `aps` on PATH operate on the correct plan tree and
  toolchain pin without `--plans`, direnv, or vendored project CLI.
- **Expected Outcome:** Shared discovery helper: walk up from cwd for
  `.aps/config.yml` (under `tooling_root`, default `.aps/`). Project-scoped
  commands (`lint`, `next`, `start`, `complete`, `graph`, `audit`) default
  `--plans` to `plans_dir` from config (fallback `plans/`). Explicit `--plans`
  still wins. `cli_version` check: warn on mismatch; `--strict` exits non-zero
  for CI. Rust and bash implementations share behaviour; bash parity fixtures
  extended. MCP `APS_PLANS` remains an override for non-standard layouts.
- **Validation:** Fixture repo with `plans_dir: docs/plans/` — `aps lint` and
  `aps next` hit the alternate tree without flags; mismatching `cli_version`
  fails under `--strict`; `./test/run.sh` and `cargo test` green; monorepo
  package-local config discovery documented as follow-up to MONO module
- **Confidence:** medium
- **Dependencies:** INSTALL-014, TUI-009, ORCH-001
- **Files:** cli/src/config.rs, cli/src/lint.rs, cli/src/next.rs, cli/src/main.rs,
  lib/lint.sh, lib/orchestrate.sh, lib/audit.sh, lib/rules/common.sh,
  docs/usage.md, test/fixtures/, test/run.sh
- **Status:** Complete: 2026-06-15
- **Results:** Shared discovery walks up from cwd for `.aps/config.yml` and
  defaults the plan root to its `plans_dir` (fallback `plans/`); explicit
  `--plans`/target and `APS_PLANS` still win, in that order. Rust:
  `config::{discover_project, default_plans, check_cli_version}` plus a global
  `--strict` flag wired into `lint` and `next` (the only project-scoped Rust
  commands today). Bash: `aps_find_config`/`aps_config_get`/`aps_default_plans`/
  `aps_check_cli_version` in `lib/rules/common.sh`, wired into `lint`, `next`,
  `start`, `complete`, `graph`, and `audit` with a `--strict` flag each. A
  `cli_version` pin mismatch warns; `--strict` (or `APS_STRICT=1`) exits
  non-zero (Rust code 2, bash 1) for CI. The top-scalar reader tolerates both
  the flat and nested config shapes. New Rust tests (discover/strict/nested)
  and bash Test 38 cover alternate `plans_dir`, the version warning, `--strict`
  failure, the `APS_PLANS` override, and the bare-repo fallback. docs/usage.md
  documents the resolution order. cargo test (85) + bash suite green; fmt +
  clippy + markdownlint clean. Monorepo package-local nested-index discovery
  remains a MONO-module follow-up.

### INSTALL-017: Migration path from vendored CLI to global binary

- **Intent:** Give existing users a safe, documented path off root `bin/` +
  `lib/`, `.aps/bin/` bash copies, and direnv-only activation.
- **Expected Outcome:** `aps doctor` (or `aps setup doctor`) reports: global
  binary presence/version, `cli_version` match, leftover vendored CLI paths,
  stale direnv `.envrc` entries. Migration doc covers: (1) install global
  binary, (2) add `cli_version` to `.aps/config.yml`, (3) run `aps upgrade`
  to back up/remove vendored `bin/`, `lib/`, `.aps/lib/` per INSTALL-013, (4)
  optional `direnv allow` removal. `scaffold/update --global` remains for bash
  fallback users. anvil-001-style CI can switch from git-SHA checkout to
  `cli_version` pin + release install.
- **Validation:** Fixture “bloated v1 project” migrates with dry-run + apply;
  doctor flags incomplete `~/.aps/lib/` (e.g. missing `audit.sh`); migration
  doc reviewed against nxrust/anvil-001 pain points from dogfooding
- **Confidence:** high
- **Dependencies:** INSTALL-013, INSTALL-014, INSTALL-015
- **Files:** cli/src/setup.rs or new `cli/src/doctor.rs`, docs/installation.md,
  docs/usage.md, scaffold/upgrade, test/fixtures/
- **Status:** Draft

### INSTALL-018: Binary-first project init (no default local CLI vendoring)

- **Intent:** Align `curl | bash` and bash `aps init` with the Rust path: project
  repos get plans + config, not a second CLI tree.
- **Expected Outcome:** Default init/install-into-project does not copy
  `bin/aps` + `lib/` unless `--local-cli` / `--bash` is set. TTY installer
  picker offers “Install CLI globally” before “Initialize this repo”. Non-TTY
  documents two-step: global install then `aps init`. `aps setup cli` copies
  the running release binary to `~/.aps/bin` (already exists in Rust). Project
  layout diagram updated: `.aps/config.yml` required; `.aps/bin/` optional.
- **Validation:** Fresh `aps init` produces no root `bin/` or `lib/`; plans
  and `.aps/config.yml` present; `aps lint` works when global binary on PATH;
  `--local-cli` still vendors bash runtime for air-gap
- **Confidence:** high
- **Dependencies:** INSTALL-011, INSTALL-014, INSTALL-015
- **Files:** scaffold/install, lib/scaffold.sh, cli/src/scaffold.rs,
  docs/installation.md, test/run.sh
- **Status:** Draft

## Execution Strategy

### Wave 1: Foundations (no dependencies)

- INSTALL-001: Directory structure (Complete)
- INSTALL-002: Config schema (Complete)
- INSTALL-009: aps-rules.md split + project-context.md

### Wave 2: Core installer (depends on Wave 1)

- INSTALL-003: Shell-prompt install wizard
- INSTALL-004: Commands → skills

### Wave 3: Optional packages (depends on Wave 2)

- INSTALL-005: APS Planner agent
- INSTALL-006: Librarian agent

### Wave 4: Stretch (depends on Wave 2)

- INSTALL-007: Multi-tool support
- INSTALL-008: v1 → v2 migration

### Wave 5: Footprint cleanup (follow-up)

- INSTALL-010: Split install/init/setup/agent/upgrade entrypoints
- INSTALL-011: Minimal default `aps init`
- INSTALL-012: `aps setup` picker and shortcuts
- INSTALL-013: Safe upgrade cleanup

### Wave 6: Global binary + project contract (depends on Wave 5, TUI-006)

Parallel after INSTALL-014 lands:

- INSTALL-014: `.aps/config.yml` project contract (`cli_version`, paths)
- INSTALL-015: Global binary-first install channels (release, crates.io, Scoop)
- INSTALL-016: Runtime config discovery (alternate `plans_dir`, `--strict`)

Sequential cleanup:

- INSTALL-018: Binary-first init (depends on 014, 015)
- INSTALL-017: Migration path + `aps doctor` (depends on 013, 014, 015, 018)

## Notes

- The current `aps-planning/` contains: SKILL.md, reference.md, examples.md,
  hooks.md, and scripts/. Under the new layout:
  - SKILL.md + reference.md + examples.md → `.claude/skills/aps-planning/`
  - scripts/ → `.aps/scripts/`
  - hooks.md → repo docs (human reference, not installed to projects)
- Skills go to `.claude/skills/` (not `.aps/`) because that's the cross-tool
  compatible path (Claude Code, Copilot, OpenCode all check it). Codex users
  get an additional copy at `.agents/skills/`.
- Agents install to `.claude/agents/` (Claude Code convention). No equivalent
  exists for Codex/Copilot — their "agents" are skills with more capability.
- The Planner agent defaults to Opus (deep reasoning for planning); the
  Librarian defaults to Sonnet (fast, cheaper for repo scanning).
- `config.yml` enables the update script to be non-interactive — it reads
  existing choices and refreshes the appropriate files.
- The multi-select includes: Claude Code, Codex, Copilot, Gemini, OpenCode,
  Other/Generic. OpenCode and Copilot are "free" if Claude Code is selected
  (they read `.claude/skills/`). Codex and Gemini share `.agents/skills/` but
  both require explicit install commands after file placement.
- Follow-up direction: the full layout below is an explicit opt-in result, not
  the default fresh-project footprint. The default should stay close to APS as a
  markdown specification: planning files first, optional tooling second.
- **D-034 default footprint:** global `aps` on PATH + committed
  `.aps/config.yml` + `plans/` (path from `plans_dir`). No root `bin/`,
  no direnv requirement. See INSTALL-014..018.
- **Alternate `plans_dir`:** monorepos may set `plans_dir: packages/foo/plans/`
  in `.aps/config.yml`; federated nested indexes remain MONO module scope.

### Resulting Project Layout (full install, all options)

```
.aps/
├── config.yml                          # Project contract (cli_version, paths)
├── bin/aps                             # Optional vendored/pinned CLI only
├── lib/                                # Optional bash CLI internals
├── scripts/                            # Hook scripts
│   ├── init-session.sh
│   ├── check-complete.sh
│   ├── pre-tool-check.sh
│   ├── post-tool-nudge.sh
│   └── enforce-plan-update.sh
└── .session-baseline                   # Ephemeral (gitignored)

.claude/
├── skills/
│   └── aps-planning/                   # Skill (Claude Code, Copilot, OpenCode)
│       ├── SKILL.md
│       ├── reference.md
│       └── examples.md
└── agents/                             # Optional (Claude Code)
    ├── aps-planner.md
    └── aps-librarian.md

.github/
└── agents/                             # Optional (Copilot)
    ├── aps-planner.md
    └── aps-librarian.md

.opencode/
└── agents/                             # Optional (OpenCode)
    ├── aps-planner.md
    └── aps-librarian.md

.codex/
├── config.toml                         # Agent entries merged (Codex)
└── agents/
    ├── aps-planner.toml
    └── aps-librarian.toml

.agents/
└── skills/
    └── aps-planning/                   # Codex + Gemini (requires install/link)
        ├── SKILL.md
        ├── reference.md
        └── examples.md

plans/                                  # User content (unchanged)
├── aps-rules.md
├── index.aps.md
├── modules/
├── execution/
└── decisions/
```
