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

This is **binary-first** (D-034): `--cli` installs the prebuilt native
`aps` binary to `~/.aps/bin` and adds it to your shell PATH. It falls back to
the bash/PowerShell CLI (`bin/aps` + `lib/`) only when no release binary exists
for your platform, the download fails, or you force it with `--bash`. No
project files are created -- use `aps init` inside a project directory for that.

To use a custom location, set `APS_HOME`:

```bash
curl -fsSL .../install | APS_HOME=/opt/aps bash -s -- --global
```

To update a global installation, reinstall the binary the same way you
installed it:

```bash
cargo binstall aps-cli                                  # prebuilt binary
cargo install aps-cli                                   # build from source
curl -fsSL .../scaffold/install | bash -s -- --cli      # install script

# Bash/PowerShell runtime only (air-gapped installs) — refresh in place:
curl -fsSL .../scaffold/update | bash -s -- --global
```

> Note: `aps update` (the binary subcommand) reconciles a **project's**
> generated files — it is not how you upgrade the global binary itself.

To uninstall: remove `~/.aps/` and the PATH line from your shell config.

## Release Channels

The native `aps` binary is built for five targets (Linux x86_64/aarch64,
macOS x86_64/aarch64, Windows x86_64) and published to GitHub releases. Every
channel references the **same semver** as the release tag (D-036), and a
project pins its toolchain with `cli_version` in `.aps/config.yml` — not a
channel-specific pin.

```bash
# 1. Install script (binary-first) — pin an exact release with VERSION:
VERSION=0.4.0 curl -fsSL .../scaffold/install | bash -s -- --cli

# 2. cargo-binstall — fetch the prebuilt binary from GitHub releases (no build):
cargo binstall aps-cli

# 3. cargo install — build from source (requires the Rust toolchain):
cargo install aps-cli
```

On Windows, install via the script (`--cli` pulls `aps.exe`) or **Scoop**:

```powershell
scoop install https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/packaging/scoop/aps.json
```

> **crates.io status:** `aps-cli` publishes to crates.io — `cargo install
> aps-cli` builds from source and `cargo binstall aps-cli` fetches the prebuilt
> release binary. The install script and Scoop remain available as alternatives.

Maintainers: the release bump checklist (tag → GitHub assets → crates.io →
Scoop) lives in the header of `.github/workflows/release.yml`.

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

**Updated files** (only those a project actually has — a minimal install has no
vendored CLI to refresh):

- `plans/aps-rules.md` (agent guidance — APS-managed)
- `plans/modules/.module.template.md`, `.simple.template.md`, `.index-monorepo.template.md`
- `plans/execution/.actions.template.md`
- The vendored CLI (`.aps/bin/aps` + `.aps/lib/`) **only if** you installed it
  with `--local-cli`; otherwise the global `aps` binary is updated separately
  (see [Global Install](#global-install))
- `aps-planning/` skill + scripts, and agent definitions for any AI tools you
  added (`.claude/agents/`, `.github/agents/`, `.opencode/agents/`, etc.) —
  only when present

**Preserved:** your `plans/index.aps.md`, module specs, `plans/project-context.md`,
`plans/issues.md`, action plans, and anything under `plans/decisions/` or
`plans/designs/`.

## Project Config Contract (`.aps/config.yml`)

`aps init` writes `.aps/config.yml`, the per-repo contract the global `aps`
binary reads by walking up from the current directory. Its contract fields:

```yaml
cli_version: 0.4.0       # toolchain semver, stamped from the running binary
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

## Migrate (Remove Generated Bloat)

Older or heavier installs scatter generated files across the repo (root
`bin/` + `lib/`, a v1 `aps-planning/` skill dir, `.claude/commands/`, and a
vendored `.aps/bin` + `.aps/lib`). `aps migrate` diagnoses the project and
removes that bloat safely so the repo can run on the global `aps` binary
instead.

```bash
aps migrate            # dry run — diagnoses, shows what would be removed
aps migrate --apply    # back up to .aps/backup/<timestamp>/, then remove
aps migrate --apply --yes   # non-interactive

# Or via curl, without a local CLI (the dry run is agent-safe — it writes nothing):
curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/upgrade | bash
curl -fsSL .../scaffold/upgrade | bash -s -- --apply --yes
```

`migrate` **dry-runs by default** and never deletes user content: `plans/**`,
`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, and settings files are protected. Every
removed path is copied to `.aps/backup/<timestamp>/` first. Hook paths in
`.claude/settings.local.json` are rewritten from `aps-planning/scripts/` to
`.aps/scripts/` (with a backup) when kept. Files that can't be classified as
APS-generated (e.g. a `lib/` mixing APS and your own scripts) are listed for
manual review and left untouched.

## Migrating to the Global Binary

Projects that adopted APS before the binary-first model carry a vendored bash
CLI (root `bin/` + `lib/`, or `.aps/bin` + `.aps/lib`) and often a direnv
`PATH_add bin` entry. Move them onto the single global `aps` binary like so:

1. **Diagnose** — `aps doctor` reports the global binary version, whether the
   project's `cli_version` matches, any leftover vendored CLI trees, an
   incomplete `~/.aps/lib/` (e.g. a runtime missing `audit.sh`), and stale
   direnv entries. It only reads — safe to run anywhere.

   ```bash
   aps doctor
   ```

2. **Install the global binary** (if not already on PATH) — see
   [Release Channels](#release-channels):

   ```bash
   curl -fsSL .../scaffold/install | bash -s -- --cli     # binary-first
   ```

3. **Pin the toolchain** — add `cli_version` to `.aps/config.yml` so the repo
   declares the release it expects (see
   [Project Config Contract](#project-config-contract-apsconfigyml)). `aps init`
   stamps it on new projects; for older configs, add it by hand.

4. **Remove the vendored bloat** — run `aps migrate` (dry-run first), which
   backs up and removes root `bin/`, `lib/`, and `.aps/lib/`, rewrites stale
   hook paths, pins `cli_version`, and drops the direnv `PATH_add bin` per the
   rules above. `aps migrate --apply` performs it.

5. **Drop direnv activation (optional)** — `aps migrate --apply` removes the
   `PATH_add bin` line for you; re-run `direnv allow` to refresh the cache.

**Bash-only / air-gapped users** can stay on the vendored runtime: keep it
fresh with `scaffold/update --global`, and re-run `aps doctor` — an incomplete
`~/.aps/lib/` is reported as a problem.

**CI** can switch from a git-SHA checkout of the bash CLI to a pinned release:
install the binary for the `cli_version` in `.aps/config.yml`, then run `aps`
directly (config discovery finds `plans_dir` — no `--plans` needed). Add
`--strict` to fail the job on toolchain drift. See
[usage → CI Integration](usage.md#ci-integration).

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

If you prefer to set things up by hand (binary-first, the same shape `aps init`
produces):

1. Install the global `aps` binary (see [Release Channels](#release-channels)),
   or vendor the bash CLI by copying `bin/aps` + `lib/` into `.aps/` only if you
   need an air-gapped / pinned toolchain
2. Copy the contents of `scaffold/plans/` into your project's `plans/`
3. Add `.aps/config.yml` with `cli_version`, `plans_dir`, `docs_dir`, and
   `tooling_root` (the project contract — see above)
4. Edit `plans/index.aps.md` to define your plan's scope and modules
5. Create modules by copying templates (remove the leading dot from filenames)
6. Add Work Items when a module is ready for implementation
7. Add tool skills/agents (Claude Code, Codex, …) later with `aps setup <tool>`

## What Gets Installed

`aps init` is **binary-first and minimal** (D-034 / INSTALL-018): the global
`aps` binary on your PATH is the CLI, so a fresh project gets only planning
content plus the project contract — no vendored CLI tree, no root `bin/` or
`lib/`:

```text
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
├── decisions/                       # ADRs (optional, empty by default)
└── designs/                         # Technical designs (optional)

.aps/
└── config.yml                       # Project contract (cli_version, paths) — required
```

Everything else is **opt-in** — a default `aps init` writes none of it:

```text
.aps/bin/aps + .aps/lib/   # Vendored bash CLI — only with --local-cli / --bash
                           #   (air-gapped or pinned toolchains)
.aps/scripts/              # Hook scripts — only with --hooks
.claude/skills/, .claude/agents/, .github/agents/, .opencode/agents/,
.codex/config.toml, .gemini/ …   # Tool integrations — via `aps setup <tool>`
```

Add any of these after init with `aps setup` (see
[Add Integrations](#add-integrations-aps-setup)). `.aps/config.yml` is the only
required `.aps/` file; `.aps/bin/` is optional and present only when you vendor
the CLI.

`.aps/context/` is created on first `aps start` and holds ephemeral context
packages — it's added to `.gitignore` automatically.

## Next Steps

After installation:

1. Edit `plans/index.aps.md` to define your plan
2. Copy templates to create modules (remove the leading dot)
3. Point your AI agent at `plans/aps-rules.md`, or run `aps next` to start working

For a full walkthrough, see the [Getting Started](getting-started.md) guide.
For the command reference, see [usage.md](usage.md).
