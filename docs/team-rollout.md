# Rolling APS out to a team

APS the *format* works the same for one person or ten. What a team needs on
top are conventions the format alone doesn't answer: who owns what, how plan
changes get reviewed, how concurrent edits avoid conflicts, and how everyone
ends up running the same linter. This guide is those conventions, with
concrete commands. The worked example
[`examples/team-payments/`](../examples/team-payments/) shows the end state —
a multi-owner plan honestly mid-execution.

## Day one, in order

1. **One person runs the init and owns the index.**

   ```bash
   aps init --profile team --scope module --tools claude-code,copilot,grok
   ```

   Pick the tools your team actually uses (`claude-code`, `copilot`,
   `codex`, `opencode`, `grok`, `generic`). Commit everything it generates.

2. **Pin the toolchain.** `aps init` stamps `cli_version` into
   `.aps/config.yml`. Everyone installs that same version (installer, Scoop,
   or crates.io — see [installation.md](./installation.md)), and CI runs
   with `--strict` so a drifted local CLI fails loudly instead of linting
   differently:

   ```bash
   aps lint plans/ --strict
   ```

3. **Turn on central enforcement before the second person touches the
   plan.** One line of workflow YAML — see
   [integrations.md](./integrations.md) for the Action and its plan-status
   PR comment:

   ```yaml
   - uses: eddacraft/anvil-plan-spec@v0.7.0
     with:
       plans-dir: plans
   ```

   The Action's tag pins the linter for CI the same way `cli_version` pins
   it locally. Bump both together.

## Ownership

- **The index has one owner** (usually the lead). Structural changes — new
  modules, decisions, success-criteria edits — go through them.
- **Each module file has one owner**, named in the module's metadata table
  and the index's Modules table. The owner is the merge authority for that
  file; others propose changes via PR, not by editing directly.
- **Cross-module work gets a conductor module** (`Type: Conductor`) instead
  of being duplicated into two owners' files —
  [`payments-launch.aps.md`](../examples/team-payments/modules/payments-launch.aps.md)
  is the shape. See [conductor-modules.md](./conductor-modules.md).

## Reviewing plan changes

Treat plan edits like code, with one refinement (team-payments D-002):

- **Scoped changes ride the code PR.** Flipping `PAY-002` to `In Progress`,
  filling its Learning line, checking an acceptance box — these land in the
  same PR as the implementation they describe. The plan and the code stay
  in lockstep because they merge together.
- **Structural changes get their own PR.** New modules, new decisions,
  re-scoped success criteria — small, plan-only PRs the index owner reviews.
  They're cheap to review and they don't hold code hostage.

The lint Action keeps both honest; the rollup comment gives reviewers the
plan state without leaving the PR.

## Avoiding merge conflicts

Status edits are single-line field changes, so git merges them cleanly as
long as two people don't edit the *same* item:

- **One status change per PR** — the item you're implementing, nothing else.
- **Owners edit their own module files.** Two people never have a reason to
  touch the same file concurrently; cross-cutting status lives in the
  conductor.
- **Don't reflow or reformat plan files in feature PRs** — formatting churn
  is what actually causes conflicts. If a file needs tidying, that's a
  plan-only PR.
- Statuses use the canonical vocabulary (`Draft / Ready / In Progress /
  Complete / Blocked`); `aps start` / `aps complete` make the edit for you:

  ```bash
  aps start PAY-003
  aps complete PAY-003 --learning "stripe replays webhooks out of order"
  ```

## Windows and PowerShell teams

The native `aps` binary is the full command surface on Windows — install via
Scoop and every command above works identically in PowerShell:

```powershell
scoop install https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/packaging/scoop/aps.json
```

The bundled PowerShell script CLI (`bin/aps.ps1`) is a lint-focused fallback
for locked-down environments that can't run the binary; it does not carry
`next`/`start`/`complete`/`export`. If that's your environment, say so
upstream — it moves the PowerShell port up the roadmap.

## Visibility for people who won't run a CLI

- The Action's **plan-status PR comment** shows the ready queue (or the
  federated rollup) on every PR.
- `aps export --json` ([shape](./integrations.md#json-export-aps-export---json))
  feeds dashboards or a scheduled job without anyone parsing markdown.

## Growing out of one index

Start with **one index and tagged modules** even as a team — it's the
default tier for a reason. Move a package to its own nested plan only when
it has a genuinely independent owner and lifecycle; the federated tier and
the migration path are in [monorepo.md](./monorepo.md).
