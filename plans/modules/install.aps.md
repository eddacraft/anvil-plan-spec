# Install v2 Module

| ID      | Owner  | Priority | Status   |
| ------- | ------ | -------- | -------- |
| INSTALL | @aneki | high     | Complete |

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

## Out of Scope

- Changing APS spec format or templates
- MCP server (separate TASKS module concern)
- Tool-specific plugin/extension development
- TUI wizard (future replacement for shell-prompt wizard)

## Interfaces

**Depends on:**

- SCAFFOLD (Complete) — evolves the existing scaffold
- VAL (Complete) — CLI moves into `.aps/` but linting logic unchanged

**Exposes:**

- `scaffold/install` — new interactive installer (replaces current)
- `scaffold/update` — migration-aware updater
- `.aps/config.yml` — project configuration schema
- `.claude/skills/aps-planning/` — skill files (cross-tool compatible)
- `.agents/skills/aps-planning/` — Codex/Gemini-compatible copy (optional)
- `.claude/agents/aps-planner.md` — optional agent (Claude Code)
- `.claude/agents/aps-librarian.md` — optional agent (Claude Code)

## Decisions

- **D-011:** `.aps/` as tooling root — _decided: yes, consolidate all
  APS-owned files (CLI, scripts, config, ephemeral) under `.aps/`_
- **D-012:** CLI location — _decided: `.aps/bin/aps` with PATH hint_
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

### Resulting Project Layout (full install, all options)

```
.aps/
├── config.yml                          # Install choices
├── bin/aps                             # CLI linter
├── lib/                                # CLI internals
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
