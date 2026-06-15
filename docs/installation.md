# Installation

## Quick Install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install | bash
```

In an interactive terminal this shows a **mode picker** before writing any
files:

1. **Install the APS CLI** on this machine
2. **Initialize APS planning** in this repository
3. **Initialize this repository for an AI agent** (minimal layer + next steps)
4. **Upgrade** an existing APS project
5. **Add a tool integration**

Pick non-interactively with a flag (required when there is no terminal, e.g.
in CI):

```bash
curl -fsSL .../scaffold/install | bash -s -- --cli       # CLI only, machine-wide
curl -fsSL .../scaffold/install | bash -s -- --init      # scaffold this repo
curl -fsSL .../scaffold/install | bash -s -- --agent     # minimal agent bootstrap
curl -fsSL .../scaffold/install | bash -s -- --upgrade   # upgrade in place
curl -fsSL .../scaffold/install | bash -s -- --setup claude-code   # add one integration
```

`--init` is **minimal by default**: it writes planning content (`plans/` with
rules, templates, `project-context.md`, `issues.md`) and the `.aps/config.yml`
project contract — nothing else. The global `aps` binary on your PATH drives
the repo. Opt into a heavier footprint with flags:

```bash
curl -fsSL .../scaffold/install | bash -s -- --init --local-cli   # also vendor the bash CLI into .aps/
curl -fsSL .../scaffold/install | bash -s -- --init --hooks       # also install hook scripts
```

Add hooks, agents, or tool skills any time after init with `aps setup`
(see [usage](usage.md)). To pick AI agent ports interactively, run `aps init`
from the installed CLI for the Ratatui onboarding wizard.

## Global Install

Install the APS CLI system-wide so `aps` is available in any directory:

```bash
# Linux/macOS
curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install | bash -s -- --global

# Windows (PowerShell) — `iex` doesn't forward args, so wrap the script in a scriptblock
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install.ps1))) --global
```

This installs only the CLI (`bin/aps` + `lib/`) to `~/.aps/` and adds it
to your shell PATH. No project files are created -- use `aps init` inside
a project directory for that.

To use a custom location, set `APS_HOME`:

```bash
curl -fsSL .../install | APS_HOME=/opt/aps bash -s -- --global
```

To update a global installation:

```bash
curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/update | bash -s -- --global

# Or from the installed CLI:
aps update --global
```

To uninstall: remove `~/.aps/` and the PATH line from your shell config.

## Install Options

```bash
# Install in a specific directory
curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install | bash -s -- ./my-project

# Install a specific version
curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install | VERSION=v0.3.0 bash
```

By default `--init` does not install hooks or a project-local CLI. Pass
`--hooks` to add hook scripts under `.aps/scripts/`, or `--local-cli`
(alias `--bash`) to vendor the bash CLI into `.aps/bin` + `.aps/lib` for
air-gapped or pinned-toolchain projects. In non-interactive mode (piped
without a terminal), the minimal default is used.

## Add Integrations (`aps setup`)

After the minimal `aps init`, add optional pieces with `aps setup`. Run it
with no argument for an interactive picker, or name a component to install
exactly one thing:

```bash
aps setup                 # interactive picker
aps setup cli             # vendor the bash CLI into .aps/bin + .aps/lib
aps setup hooks           # install hook scripts into .aps/scripts
aps setup agent           # minimal plans + agent next-steps file
aps setup claude-code     # add a tool integration (skill + agents)
aps setup all --yes       # full footprint (CLI + hooks + Claude Code)
```

Tool names accepted by `aps setup <tool>`: `claude-code`, `copilot`,
`codex`, `opencode`, `gemini`, `generic`. The `all` flow installs a bulky
footprint and asks for confirmation first (skip it with `--yes`). Every
other shortcut writes only the component you name. The native `aps` binary
ships a Ratatui picker for the same flows; the bash CLI uses a numbered
prompt.

## Update Existing Project

If you already have APS installed and want to pull the latest templates,
rules, and skill files:

```bash
curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/update | bash
```

Your specs are preserved -- the updater only replaces templates, rules,
the CLI, and skill files. It does not touch your `index.aps.md`, module
specs, or action steps.

**Updated files:**

- `bin/aps` + `lib/` (CLI: lint, init, update, migrate, next, start, complete, graph)
- `plans/aps-rules.md` (agent guidance — APS-managed)
- `plans/modules/.module.template.md`, `.simple.template.md`, `.index-monorepo.template.md`
- `plans/execution/.actions.template.md`
- `aps-planning/` (skill + scripts)
- Agent definitions for any AI tools you selected during `aps init`
  (`.claude/agents/`, `.github/agents/`, `.opencode/agents/`, etc.)

**Preserved:** your `plans/index.aps.md`, module specs, `plans/project-context.md`,
`plans/issues.md`, action plans, and anything under `plans/decisions/` or
`plans/designs/`.

## Project Config Contract (`.aps/config.yml`)

`aps init` writes `.aps/config.yml`, the per-repo contract the global `aps`
binary reads by walking up from the current directory. Its contract fields:

```yaml
cli_version: 0.4.0-dev   # toolchain semver, stamped from the running binary
plans_dir: plans/        # where plan documents live
docs_dir: docs/          # where generated docs live
tooling_root: .aps/      # APS-owned tooling root
```

- **`cli_version`** pins the toolchain a project expects. `aps init` stamps it
  from the running binary; `aps init --from <config>` replays an existing pin,
  and warns + inherits the current version when an older config predates the
  field.
- **`plans_dir` / `docs_dir` / `tooling_root`** are runtime defaults, not just
  init metadata — a monorepo can set `plans_dir: packages/foo/plans/`. Explicit
  flags (`--plans`, …) override them.
- Unknown keys are ignored for forward compatibility, so newer fields never
  break an older CLI.

Both the native binary and the bash CLI write these same contract keys.

## Upgrade (Remove Generated Bloat)

Older or heavier installs scatter generated files across the repo (root
`bin/` + `lib/`, a v1 `aps-planning/` skill dir, `.claude/commands/`, and a
vendored `.aps/bin` + `.aps/lib`). `aps upgrade` removes that bloat safely so
the repo can run on the global `aps` binary instead.

```bash
aps upgrade            # dry run — shows what would be backed up and removed
aps upgrade --apply    # back up to .aps/backup/<timestamp>/, then remove
aps upgrade --apply --yes   # non-interactive

# Or via curl, without a local CLI (the dry run is agent-safe — it writes nothing):
curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/upgrade | bash
curl -fsSL .../scaffold/upgrade | bash -s -- --apply --yes
```

`upgrade` **dry-runs by default** and never deletes user content: `plans/**`,
`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, and settings files are protected. Every
removed path is copied to `.aps/backup/<timestamp>/` first. Hook paths in
`.claude/settings.local.json` are rewritten from `aps-planning/scripts/` to
`.aps/scripts/` (with a backup) when kept. Files that can't be classified as
APS-generated (e.g. a `lib/` mixing APS and your own scripts) are listed for
manual review and left untouched.

## Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install.ps1 | iex
```

To install a specific version:

```powershell
$env:APS_VERSION='v0.3.0'; irm https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install.ps1 | iex
```

Or follow the [Manual Setup](#manual-setup) steps below.

## Manual Setup

If you prefer to set things up by hand:

1. Copy `bin/aps` and `lib/` into your project (or `aps-install --global`)
2. Copy the contents of `scaffold/plans/` into your project's `plans/`
3. Copy `scaffold/aps-planning/` for the Claude Code skill
4. Copy agent definitions from `scaffold/agents/<tool>/` for any AI tools you use
5. Edit `plans/index.aps.md` to define your plan's scope and modules
6. Create modules by copying templates (remove the leading dot from filenames)
7. Add Work Items when a module is ready for implementation

## What Gets Installed

The install script creates the following structure in your project:

```text
bin/
└── aps                              # CLI (lint, init, update, migrate,
                                     #      next, start, complete, graph)

lib/                                 # CLI internals (parsers, rules, output)

plans/
├── aps-rules.md                     # Agent guidance (APS-managed)
├── project-context.md               # Project-specific context (you own this)
├── index.aps.md                     # Your main plan
├── issues.md                        # Dev-time discoveries (ISS-NNN / Q-NNN)
├── modules/
│   ├── .module.template.md          # Module template
│   ├── .simple.template.md          # Simple feature template
│   └── .index-monorepo.template.md  # Index for monorepos
├── execution/
│   └── .actions.template.md         # Action plan template
└── decisions/                       # ADRs (optional, empty by default)

aps-planning/
├── SKILL.md                         # Planning skill (core rules)
├── reference.md                     # APS format reference
├── examples.md                      # Real-world examples
├── hooks.md                         # Hook configuration guide
└── scripts/                         # Hook install + session scripts
```

`aps init` may additionally install agent definitions for each AI tool you
opt into (`.claude/agents/`, `.github/agents/`, `.opencode/agents/`,
`.codex/config.toml` overlays, or Gemini skills) — the prompt during `init`
controls this.

`.aps/context/` is created on first `aps start` and holds ephemeral context
packages — it's added to `.gitignore` automatically.

## Next Steps

After installation:

1. Edit `plans/index.aps.md` to define your plan
2. Copy templates to create modules (remove the leading dot)
3. Point your AI agent at `plans/aps-rules.md`, or run `aps next` to start working

For a full walkthrough, see the [Getting Started](getting-started.md) guide.
For the command reference, see [usage.md](usage.md).
