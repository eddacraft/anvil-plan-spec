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

`--init` is the full scaffold — plans, templates, skill, and a local copy of
the CLI. After it, run `aps init` to launch the Ratatui-based onboarding
wizard: it scaffolds your first plan, picks which AI agent ports to install
(Claude Code / Codex / Copilot / OpenCode / Gemini), and writes
`plans/project-context.md`.

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

The installer prompts you to set up Claude Code hooks and PATH
configuration. In non-interactive mode (piped without a terminal), it
uses sensible defaults.

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
