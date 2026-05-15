# APS Interactive Onboarding Design

**Date:** 2026-02-27
**Status:** Draft

## Problem

After running `curl | bash`, users land in a project with scaffolded files
and a "Next steps" message. There is no guided path from "I just installed"
to "I'm productively using APS." The current flow:

- Dumps everything at once regardless of what the user needs
- Doesn't adapt to user type (solo dev vs team vs AI agent operator)
- Doesn't know which AI tools the user works with
- Leaves manual steps: editing settings, installing hooks, configuring agents

## Goal

Replace the current `aps init` with an interactive TUI wizard that:

1. Adapts to the user's context (profile, scope, tooling)
2. Scaffolds exactly what they need
3. Leaves zero manual steps — working setup on completion
4. Shares visual identity with the Anvil product family (TUI from Anvil-001)

## Success Criteria

- [ ] User answers 3 questions and gets a fully working APS setup
- [ ] All selected AI tool integrations are installed and configured
- [ ] `aps lint` passes on the generated structure
- [ ] Non-interactive fallback works for CI and piped environments
- [ ] Existing `curl | bash` install remains as a lightweight alternative

## Wizard Flow

### Step 1: Profile (single-select)

Determines template defaults, hook suggestions, and guidance tone.

```
What are you using APS for?

  > Solo dev — personal project
    Team adoption — rolling out for a team
    AI agent setup — planning layer for AI tools
```

### Step 2: Scope (single-select)

Determines which index and module templates get scaffolded.

```
What's the scope of your first plan?

  > Small feature (1-3 work items)        -> quickstart template
    Module with boundaries                 -> module template
    Multi-module initiative                -> index + module templates
    Monorepo (multiple packages/apps)      -> monorepo index
```

### Step 3: AI Tooling (multi-select)

Determines which agents, hooks, skills, and commands get installed.

```
Which AI tools do you use? (space to toggle, enter to confirm)

  [x] Claude Code
  [ ] GitHub Copilot
  [ ] Codex
  [ ] OpenCode
  [ ] Gemini
  [ ] All of the above
  [ ] None / manual only
```

### Step 4: Scaffold + Verify (automated)

Downloads and installs based on selections, then verifies.

```
==> Installing APS

  > bin/aps + lib/              done
  > plans/ (simple template)    done
  > aps-planning/ (skill)       done
  > .claude/commands/           done
  > .claude/agents/             done
  > Hooks                       done
  > aps lint plans/             passed
```

### Step 5: Summary (results dashboard)

Shows what was installed, per-platform next steps, and doc links.

```
==> Ready!

  Installed:
    Plans       quickstart template in plans/
    CLI         bin/aps (lint, init, update)
    Skill       aps-planning/ (planning skill + hooks)
    Commands    .claude/commands/ (plan, plan-status)
    Agents      .claude/agents/ (aps-planner, aps-librarian)
    Hooks       .claude/settings.local.json

  Next steps:
    Run /plan in Claude Code to start planning

  Docs: https://github.com/EddaCraft/anvil-plan-spec
```

## What Gets Installed Per Selection

### By Profile

| Profile        | Effect                                                        |
| -------------- | ------------------------------------------------------------- |
| Solo dev       | Defaults to quickstart template, skips team-oriented guidance |
| Team adoption  | Defaults to index template, includes review workflow guidance |
| AI agent setup | Defaults to module template, emphasises tool integration      |

### By Scope

| Scope                   | Templates Installed                                 |
| ----------------------- | --------------------------------------------------- |
| Small feature           | `quickstart.template.md` as `plans/index.aps.md`    |
| Module with boundaries  | `module.template.md` in `plans/modules/`            |
| Multi-module initiative | `index.template.md` + `module.template.md`          |
| Monorepo                | `index-monorepo.template.md` + `module.template.md` |

All scopes also get `aps-rules.md`, `execution/.steps.template.md`, and
`decisions/.gitkeep`.

### By AI Tooling

| Tool           | What Gets Installed                                                                                                                      |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Claude Code    | `aps-planning/` skill, `.claude/commands/`, `.claude/agents/aps-planner.md` + `aps-librarian.md`, hooks in `.claude/settings.local.json` |
| GitHub Copilot | `.github/copilot/agents/aps-planner.md` + `aps-librarian.md`                                                                             |
| Codex          | `codex.toml` snippet, agents in codex format                                                                                             |
| OpenCode       | `.opencode/agents/aps-planner.md` + `aps-librarian.md`                                                                                   |
| Gemini         | `.gemini/skills/aps-planner/SKILL.md` + `aps-librarian/SKILL.md`                                                                         |
| None           | Plans + CLI only, no integrations                                                                                                        |

## Non-Interactive Fallback

When TTY is not available (CI, piped input, `--non-interactive` flag):

- Use smart defaults: solo dev, small feature, no AI tools
- Accept overrides via flags: `--profile team --scope monorepo --tools claude,copilot`
- Silent operation with exit code for success/failure

The existing `curl | bash` install script remains as the lightweight,
no-dependency alternative. Both paths produce the same end state.

## Architecture

```
aps-cli/
  src/
    main.rs                     Entry point (clap)
    commands/
      init.rs                   Init wizard command
      lint.rs                   Lint (port from bash or native)
      update.rs                 Update command
    tui/
      widgets/                  Ratatui widgets (shared EddaCraft TUI)
        select.rs               Single-select prompt
        multi_select.rs         Checkbox multi-select
        confirm.rs              Y/N dialog
        spinner.rs              Progress indicator
        header.rs               Branded header
        results_dashboard.rs    Summary panel
      views/
        init/
          wizard.rs             Wizard orchestrator
          profile_step.rs       Step 1: profile selection
          scope_step.rs         Step 2: scope selection
          tooling_step.rs       Step 3: AI tools multi-select
          summary_step.rs       Step 5: results dashboard
      theme.rs                  EddaCraft shared theme
      tty.rs                    TTY/fallback detection
  Cargo.toml
```

Built with Rust using Ratatui for terminal UI. Components should match the
visual language of the Anvil product family (shared EddaCraft theme,
keyboard conventions). Compiled to a single static binary via
`cargo build --release` for each target platform.

## Keyboard Conventions (from Anvil-001)

| Action          | Keys              |
| --------------- | ----------------- |
| Navigate        | Arrow keys or j/k |
| Select/confirm  | Enter             |
| Toggle checkbox | Space             |
| Go back         | Esc or left arrow |
| Quit            | q or Ctrl+C       |

## Decisions

| Decision              | Choice                                        | Notes                                                                                                              |
| --------------------- | --------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| TUI framework         | **Ratatui** (Rust)                            | Shared EddaCraft TUI library, same product family as Anvil. Supersedes OpenTUI (Bun/Zig) decision from 2026-02-27. |
| Distribution          | **Single binary** via `cargo build --release` | Cross-compile for linux-x64, linux-arm64, darwin-arm64, darwin-x64, windows-x64. Zero runtime deps for end users.  |
| Where source lives    | TBD                                           | APS is public, Anvil is private. Need to decide whether aps-cli lives in APS repo or elsewhere.                    |
| Shared TUI components | TBD                                           | Depends on source location. Ratatui widgets may be extracted as a shared crate or vendored.                        |

### Why Ratatui

- Same product family as Anvil (shared EddaCraft TUI built on Ratatui)
- Rust compiles to true static binaries — no runtime, no VM, no native addon issues
- Ratatui is the dominant Rust TUI framework with active ecosystem
- Rich widget library (List, Table, Paragraph, Tabs, Gauge, etc.)
- Crossterm backend handles cross-platform terminal compatibility
- `cargo build --release` cross-compilation via `cross` or `cargo-zigbuild`

## Relationship to Existing Install

The `curl | bash` installer (`scaffold/install`) remains as-is. It serves
users who want no runtime dependencies and works in non-interactive
environments. The TUI wizard is the premium path for interactive setup.

Both produce the same file structure. The wizard just makes smarter choices
about what to include.

## Build & Cross-Compilation

The CLI compiles to standalone binaries via `cargo build --release`.
Cross-compilation uses `cross` or `cargo-zigbuild` for multi-platform
targets:

```bash
# Native build
cargo build --release

# Cross-compile (using cross)
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target aarch64-apple-darwin
cross build --release --target x86_64-apple-darwin
cross build --release --target x86_64-pc-windows-gnu
```

Binaries are published as GitHub release assets. The `curl | bash`
installer can optionally download the binary instead of the bash CLI.

## Prior Art

- **Anvil** (`anvil init`) — TUI wizard built with Rust/Ratatui (shared
  EddaCraft TUI). Direct inspiration for UX patterns and shared theme.
- **Superpowers** — Per-platform install instructions (Claude Code, Cursor,
  Codex, OpenCode). Inspiration for the multi-tool selection model.
