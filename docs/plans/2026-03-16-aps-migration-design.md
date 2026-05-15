# APS v2 Migration & Onboarding Overhaul Design

**Date:** 2026-03-16
**Status:** Draft

## Problem

APS v0.3 shipped most of its planned features (agents, multi-tool support,
interactive init) but diverged from the planned `.aps/` directory consolidation.
The current install produces a scattered layout:

- `bin/aps` + `lib/` at project root
- `aps-planning/` at project root (skill files + hook scripts)
- `.claude/commands/` (deprecated format, should be skills only)
- No `config.yml` to record install choices
- `designs/` lives at root, separate from `plans/`
- `aps-rules.md` mixes APS format rules with project-specific context

Additionally, the internal planning repo experiment (aps-closed) didn't work.
Plans are moving back into the main repo and need reconciliation with reality.

## Goal

1. Consolidate all APS-owned tooling under `.aps/` as originally designed
   (D-011, D-012)
2. Ship a shell-prompt install wizard (profile, scope, AI tools) that produces
   the new layout with `config.yml`
3. Provide a migration path for existing v1 installs
4. Clean up the planning content structure (`designs/` and `issues.md` into
   `plans/`, split `aps-rules.md`)
5. Move plans back into anvil-plan-spec and reconcile with shipped state

## Success Criteria

- [ ] Fresh `aps init` produces `.aps/` layout with config.yml
- [ ] `aps migrate` converts v1 layout to v2 without data loss
- [ ] `aps update` reads config.yml and refreshes without re-prompting
- [ ] No `bin/`, `lib/`, `aps-planning/`, or `.claude/commands/` at project root
- [ ] `aps-rules.md` contains only APS format rules (updatable)
- [ ] `project-context.md` contains project-specific context (user-owned)
- [ ] `plans/designs/` and `plans/issues.md` are part of the scaffold
- [ ] Shell-prompt wizard works in interactive terminals; flags work for CI
- [ ] Agent bootstraps `project-context.md` on first run after install

## Sequencing: aps-rules.md and designs/ Migration

The new `aps-rules.md` (which references `plans/designs/` instead of root
`designs/`) ships **only with the v2 layout**. It is never installed into a
v1 project via `aps update`. The sequencing:

1. `aps update` on v1 projects: continues shipping the current `aps-rules.md`
   with `designs/` at root. No change until user runs `aps migrate`.
2. `aps migrate`: moves files to v2 layout, then installs the new
   `aps-rules.md` that references `plans/designs/`.
3. `aps init` (fresh install): produces v2 layout with new `aps-rules.md`
   from the start.

This ensures agents never see a `plans/designs/` reference while the files
are still at root `designs/`.

## Global vs Per-Project Install

The global install (`aps init --global` or `scaffold/install --global`)
places the CLI at `~/.aps/bin/aps` and adds it to PATH. This is unchanged
by this design — `~/.aps/` is the global tooling root, `.aps/` (in a project)
is the per-project tooling root. They serve different purposes:

- **`~/.aps/`** (global): CLI only, no config, no project files. Used when
  the user wants `aps` available system-wide without per-project install.
- **`.aps/`** (per-project): CLI + config + scripts. Created by `aps init`
  or `aps migrate` inside a project directory.

When both exist, the per-project `.aps/bin/aps` takes precedence (via direnv
`PATH_add .aps/bin` or explicit path). The global install is a convenience
for users who work across many projects.

## Directory Layout: Before and After

### Before (v1 — current)

```
bin/
  aps
  lib/
    output.sh
    lint.sh
    scaffold.sh
    rules/

aps-planning/
  SKILL.md
  reference.md
  examples.md
  hooks.md
  scripts/

plans/
  aps-rules.md            # mixed: APS rules + project context
  index.aps.md
  modules/
  execution/
  decisions/

designs/                   # separate from plans/

.claude/
  commands/                # deprecated
    plan.md
    plan-status.md
  agents/                  # if installed
```

### After (v2 — `.aps/` layout)

```
.aps/
  config.yml               # install choices, read by updater
  bin/aps                   # CLI
  lib/                      # CLI internals
  scripts/                  # hook scripts
    init-session.sh
    pre-tool-check.sh
    post-tool-nudge.sh
    check-complete.sh
    enforce-plan-update.sh

plans/
  aps-rules.md             # APS-managed, safe to update
  project-context.md       # user-owned, never overwritten
  index.aps.md
  modules/
  execution/
  decisions/
  designs/                 # moved in from root
  issues.md                # planning-adjacent tracker

.claude/skills/aps-planning/   # Claude Code, Copilot, OpenCode
  SKILL.md
  reference.md
  examples.md

.claude/agents/                # Claude Code (optional)
  aps-planner.md
  aps-librarian.md

# Additional tool-specific dirs per selection:
# .github/agents/             (Copilot)
# .opencode/agents/           (OpenCode)
# .codex/agents/              (Codex)
# .agents/skills/aps-planning (Codex, Gemini)
```

## Install Wizard Flow (Shell Prompts)

Three questions, then scaffold + verify.

### Step 1: Profile (single-select)

```
What are you using APS for?

  1) Solo dev — personal project
  2) Team adoption — rolling out for a team
  3) AI agent setup — planning layer for AI tools
```

Determines template defaults and guidance tone.

### Step 2: Scope (single-select)

```
What's the scope of your first plan?

  1) Small feature (1-3 work items)     -> quickstart template
  2) Module with boundaries             -> module template
  3) Multi-module initiative            -> index + module templates
  4) Monorepo (multiple packages/apps)  -> monorepo index
```

Determines which index and module templates get scaffolded.

### Step 3: AI Tooling (multi-select)

```
Which AI tools do you use? (comma-separated, e.g. 1,2,4)

  1) Claude Code
  2) GitHub Copilot
  3) Codex
  4) OpenCode
  5) Gemini
  6) None / manual only
```

Determines which agents, skills, and hooks get installed.

### Step 4: Scaffold

Produces `.aps/` layout based on selections. Writes `config.yml`.

### Step 5: Agent Context Bootstrap

Post-install message directs user to run their agent to populate
`project-context.md`. For Claude Code: "Run /plan in Claude Code to set up
your project context." For other tools: points to AGENTS.md.

If no agent is available, `project-context.md` ships as a template with
TODO markers.

### Step 6: Verify

`aps lint plans/` confirms scaffold is valid.

### Non-Interactive Fallback

When TTY is not available or `--non-interactive` flag is set:

- Defaults: solo dev, small feature, no AI tools
- Override via flags: `--profile team --scope monorepo --tools claude,copilot`
- Silent operation with exit code

## config.yml Schema

```yaml
# .aps/config.yml — written by installer, read by updater
aps:
  version: "0.3.0" # APS release version that was installed
  config_schema: 1 # config.yml schema version (for future compat)
  installed: "2026-03-16" # date of initial install
  updated: "2026-03-16" # date of last aps update

project:
  type: simple # simple | monorepo
  monorepo_tool: ~ # pnpm | turbo | lerna | nx
  planning: internal # internal | external
  planning_repo: ~ # path or URL (null if internal)
  profile: solo # solo | team | agent

tools:
  - name: claude-code
    skill: .claude/skills/aps-planning
    hooks: full # full | minimal | none
    agents:
      - aps-planner
      - aps-librarian
  - name: codex
    skill: .agents/skills/aps-planning
    instruction_file: AGENTS.md
```

**Version field semantics:**

- `aps.version`: The APS release version (from the repo's `package.json`)
  that was last installed or updated. `aps update` compares this against the
  latest release to decide whether files need refreshing.
- `aps.config_schema`: Integer schema version for the config.yml format
  itself. Starts at 1. Incremented if the config structure changes in a way
  that requires migration logic in the updater.
- `aps.installed`: Date of initial installation. Informational, never changed
  after first write.
- `aps.updated`: Timestamp of the last `aps update` run. Informational.

**Canonical tool identifiers** (used in config.yml, CLI flags, and internally):

| Display Name   | Identifier    |
| -------------- | ------------- |
| Claude Code    | `claude-code` |
| GitHub Copilot | `copilot`     |
| Codex          | `codex`       |
| OpenCode       | `opencode`    |
| Gemini         | `gemini`      |
| None / manual  | `generic`     |

CLI flags use these identifiers: `--tools claude-code,copilot`

## aps-rules.md Split

### aps-rules.md (APS-managed)

Contains only APS format rules: hierarchy, naming conventions, status flows,
work item structure, action plan format. Same for every project. Updated by
`aps update`.

### project-context.md (user-owned)

Contains project-specific context: what the project is, team, tech stack,
conventions, active decisions. Populated by agent on first run or manually
by user. Never overwritten by `aps update`.

Template shipped by scaffold:

```markdown
# Project Context

## Overview

<!-- What is this project? What problem does it solve? -->

## Team

<!-- Who works on this? Roles and responsibilities. -->

## Tech Stack

<!-- Languages, frameworks, key dependencies. -->

## Conventions

<!-- Coding standards, branching strategy, review process. -->

## Active Decisions

<!-- Key architectural or process decisions in effect. -->
```

## Migration Path (v1 to v2)

`aps migrate` detects v1 layout and converts. Supports `--dry-run` to preview
changes without modifying files.

### Detection

v1 layout is detected by the presence of any of: `bin/aps` at project root,
`aps-planning/` directory, or `.claude/commands/plan.md`.

### File Moves

| v1 Location                       | v2 Location                                       | Action                                                                           |
| --------------------------------- | ------------------------------------------------- | -------------------------------------------------------------------------------- |
| `bin/aps`                         | `.aps/bin/aps`                                    | Move                                                                             |
| `bin/lib/` or `lib/`              | `.aps/lib/`                                       | Move                                                                             |
| `aps-planning/SKILL.md`           | `.claude/skills/aps-planning/SKILL.md`            | Move                                                                             |
| `aps-planning/reference.md`       | `.claude/skills/aps-planning/reference.md`        | Move                                                                             |
| `aps-planning/examples.md`        | `.claude/skills/aps-planning/examples.md`         | Move                                                                             |
| `aps-planning/hooks.md`           | Deleted                                           | Remove (hook scripts are the source of truth; hooks.md was human reference only) |
| `aps-planning/scripts/`           | `.aps/scripts/`                                   | Move                                                                             |
| `.claude/commands/plan.md`        | Deleted                                           | Back up to `.aps/backup/commands/` then remove                                   |
| `.claude/commands/plan-status.md` | Deleted                                           | Back up to `.aps/backup/commands/` then remove                                   |
| `designs/`                        | `plans/designs/`                                  | Move                                                                             |
| `plans/aps-rules.md` (mixed)      | `plans/aps-rules.md` + `plans/project-context.md` | Split (see below)                                                                |
| (none)                            | `.aps/config.yml`                                 | Create (inferred)                                                                |
| (none)                            | `plans/issues.md`                                 | Create from template                                                             |

### aps-rules.md Split Logic

The current `aps-rules.md` contains these section categories:

**APS-managed (stays in aps-rules.md):**

- APS Hierarchy (Index, Module, Work Item, Action Plan)
- Naming conventions (ID format, file naming)
- Status flows (Draft → Ready → In Progress → Complete)
- Work item structure (Intent, Expected Outcome, Validation)
- Action plan format (waves, checkpoints, steps)
- Decision logging format
- Template reference

**Project-specific (moves to project-context.md):**

- Session Start/End Ritual content
- Project-specific conventions
- Monorepo-specific configuration
- Any section referencing specific tech stack, team, or project names

The migration script splits on section headers. Sections that match
APS-managed patterns stay; everything else moves to `project-context.md`.
If the script can't determine the boundary (e.g., user has heavily
customized the file), it preserves the original as
`.aps/backup/aps-rules-original.md` and installs fresh copies of both files.

### config.yml Inference Logic

When creating `config.yml` from an existing v1 install, the migration
infers choices by checking for installed files:

| Check                                                                              | Inference                                 |
| ---------------------------------------------------------------------------------- | ----------------------------------------- |
| `plans/modules/` contains `*-monorepo*` or `index-monorepo*`                       | `project.type: monorepo`                  |
| `pnpm-workspace.yaml` exists                                                       | `project.monorepo_tool: pnpm`             |
| `turbo.json` exists                                                                | `project.monorepo_tool: turbo`            |
| `lerna.json` exists                                                                | `project.monorepo_tool: lerna`            |
| `nx.json` exists                                                                   | `project.monorepo_tool: nx`               |
| `.claude/agents/aps-planner.md` exists                                             | tool: `claude-code` with agents           |
| `.claude/skills/aps-planning/` exists                                              | tool: `claude-code` (or copilot/opencode) |
| `.github/agents/aps-planner.md` exists                                             | tool: `copilot`                           |
| `.opencode/agents/aps-planner.md` exists                                           | tool: `opencode`                          |
| `.codex/agents/aps-planner.toml` exists                                            | tool: `codex`                             |
| `.gemini/skills/aps-planner/` exists                                               | tool: `gemini`                            |
| `.agents/skills/aps-planning/` exists (no `.codex/` or `.gemini/` to disambiguate) | tool: `codex` (more common v1 install)    |
| None of the above tool markers                                                     | tool: `generic`                           |

For ambiguous cases (e.g., `.claude/skills/` exists but could be any of
three tools), the migration defaults to `claude-code` since that's the
most common v1 install. Profile defaults to `solo` since v1 didn't track
this.

The generated config includes a comment: `# Inferred by aps migrate — review
and adjust if needed`.

### Post-Migration

- Update hook paths in `.claude/settings.local.json` (`.aps/scripts/` paths)
- Remove empty directories (`bin/`, `aps-planning/`, `.claude/commands/`)
- Print summary of what moved and what was backed up
- Run `aps lint plans/` to verify
- If `--dry-run`, print the above as a preview without modifying files

## AGENTS.md

The root `AGENTS.md` stays at the project root. It serves as the entry point
for Codex and Copilot (both read `AGENTS.md` as their instruction file). The
v2 layout does not move or replace it.

When tools are selected during install, APS appends an "APS Planning" section
to `AGENTS.md` (creating it if it doesn't exist) that points agents to
`plans/` and `.aps/`. This is the same behavior described in D-013.

## Agent Context Bootstrap Contract

After install, `project-context.md` ships as a template with TODO markers.
The post-install message tells the user how to populate it per their tool:

- **Claude Code:** "Run /plan in Claude Code to set up your project context"
- **Other tools:** "See AGENTS.md for how to populate plans/project-context.md"

The **aps-planner agent** (all tool variants) includes this behavior in its
system prompt:

1. On first invocation, check if `plans/project-context.md` contains TODO
   markers or is a template.
2. If so, read the project (package.json, README, AGENTS.md/CLAUDE.md, git
   log, directory structure) and populate the file with inferred context.
3. For anything it can't infer (team, conventions), leave a TODO marker and
   tell the user what to fill in.
4. If `project-context.md` is already populated, skip this step.

This behavior is part of the agent prompt, not the CLI. It requires no
changes to the bash scripts — the agent handles it naturally when dispatched.

## Relationship to TUI

This design uses shell prompts exclusively. The Anvil project is delivering
an EddaCraft-standard TUI (OpenTUI / Bun / Zig) that APS will adopt when
available. The shell-prompt wizard will be replaced by the TUI wizard; the
non-interactive flag path remains regardless.

No TUI work is in scope for this design.

## Relationship to Existing Onboarding Design

The `2026-02-27-onboarding-design.md` describes the TUI-based wizard flow.
This design implements the same wizard logic (profile, scope, tools) using
shell prompts as an interim step. The TUI design remains the target UX.

## New Decisions

| Decision | Choice                             | Notes                                                             |
| -------- | ---------------------------------- | ----------------------------------------------------------------- |
| D-022    | External planning repo reversed    | Plans move back to main repo. aps-closed deleted.                 |
| D-023    | Commands fully dropped             | Skills only. No `.claude/commands/` shipped. Supersedes D-015.    |
| D-024    | aps-rules.md split                 | `aps-rules.md` (APS-managed) + `project-context.md` (user-owned). |
| D-025    | designs/ and issues.md into plans/ | Single planning content root.                                     |

## Risks

| Risk                                                        | Impact | Mitigation                                                                                                     |
| ----------------------------------------------------------- | ------ | -------------------------------------------------------------------------------------------------------------- |
| Existing users have customized `aps-rules.md`               | Medium | Migration backs up original, splits by section headers, preserves unrecognized content in `project-context.md` |
| Hook path changes break active sessions                     | Low    | Migration updates `settings.local.json` automatically                                                          |
| Agent can't infer project context accurately                | Low    | Template with TODO markers as fallback                                                                         |
| Users resist `.aps/` hidden directory                       | Low    | Same convention as `.git/`, `.github/`, `.vscode/`                                                             |
| Users have customized `.claude/commands/` files             | Low    | Migration backs up to `.aps/backup/commands/` before deletion                                                  |
| New `aps-rules.md` references `plans/designs/` on v1 layout | High   | New rules only ship with v2 layout; `aps update` on v1 projects uses the old rules (see Sequencing section)    |
| config.yml inference guesses wrong tool                     | Low    | Generated config includes review comment; user can edit                                                        |
