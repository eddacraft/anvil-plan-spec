# CLI Usage

The `aps` CLI has two layers of functionality:

- **Authoring** — scaffold new projects, lint specs (`init`, `update`, `migrate`, `lint`)
- **Orchestration** — drive specs through the work-item lifecycle from the command line (`next`, `start`, `complete`, `graph`)

You can ignore orchestration entirely and edit markdown by hand — the CLI is
additive. But once a plan gets non-trivial, `aps next`/`start`/`complete` are
faster and harder to get wrong than manual status edits.

## Command Index

```bash
aps init [dir]              # Create APS structure in a new project
aps update [dir]            # Reconcile generated templates + skill (add missing, refresh)
aps migrate [dir]           # Move a project onto the global binary (remove vendored bloat)
aps lint [file|dir]         # Validate APS documents
aps next [module]           # Show the next ready work item
aps start <ID>              # Mark a Ready work item as In Progress
aps complete <ID>           # Mark an In Progress work item as Complete
aps graph [module]          # Show work items + dependency arrows
aps rollup                  # Roll-up table for a federated (nested) parent
aps audit [module]          # Audit plan state against reality
aps doctor                  # Diagnose migration state (global binary vs vendored)
aps --help                  # Top-level help
aps <cmd> --help            # Per-command help
```

Every command accepts `--plans <dir>` if your plans aren't at the default
`plans/` location. The orchestration commands (`next`, `start`, `complete`,
`graph`, `audit`) additionally accept `--child <name>` to scope a
[federated nested plan](#nested-plans-federated-orchestration) to one child.

### Project config discovery

Project-scoped commands (`lint`, `next`, `start`, `complete`, `graph`,
`audit`) resolve their plan root automatically. When you don't pass `--plans`
(or a lint target), `aps` walks up from the current directory for the nearest
`.aps/config.yml` and uses its `plans_dir` — so a repo with
`plans_dir: docs/plans/` just works from any subdirectory, no flag needed.

Resolution order: explicit `--plans` / target → `APS_PLANS` environment
variable (the MCP/manual override) → discovered `plans_dir` → `plans/`.

`aps` also compares the project's `cli_version` pin to the running binary and
prints a warning on a mismatch. Add `--strict` (or set `APS_STRICT=1`) to turn
that mismatch into a non-zero exit — useful in CI to enforce the pinned
toolchain:

```bash
aps lint --strict          # fail CI if the toolchain version drifts
```

## Authoring

### `aps lint`

```bash
aps lint                          # Lint plans/
aps lint plans/modules/auth.aps.md
aps lint . --json                 # Machine-readable output
```

Errors cause a non-zero exit code. Warnings are informational.

#### Error codes

| Code | Scope     | Description                                                                           |
| ---- | --------- | ------------------------------------------------------------------------------------- |
| E001 | Module    | Missing `## Purpose` section                                                          |
| E002 | Module    | Missing `## Work Items` section                                                       |
| E003 | Module    | Missing ID/Status metadata table                                                      |
| E004 | Index     | Missing `## Modules` section                                                          |
| E005 | Work Item | Missing required field (`**Intent:**`, `**Expected Outcome:**`, or `**Validation:**`) |
| E010 | Issues    | Missing `## Issues` section                                                           |
| E011 | Issues    | Missing `## Questions` section                                                        |
| R001 | Release   | Release file is not named `v<version>.md` (e.g., `v0.3.0.md`)                         |
| R002 | Release   | Missing release header table with `Target` and `Status` fields                        |
| R003 | Release   | Missing `## Release Theme` section                                                    |
| R004 | Release   | Missing `## What Ships` section                                                       |

Release rules apply to files under `plans/releases/` (the `v<version>.md`
narratives). `README.md` and the `.release.template.md` are not linted.

#### Warning codes

| Code | Scope          | Description                                                                                                             |
| ---- | -------------- | ----------------------------------------------------------------------------------------------------------------------- |
| W001 | Work Item      | ID does not match `PREFIX-NNN` pattern (e.g., `AUTH-001`)                                                               |
| W002 | Module         | Conductor (`Type: Conductor`) references a work-item ID in `## Coordinated Modules` / `## Cross-Module Work Items` that resolves nowhere in the plan tree (likely a typo) |
| W003 | Work Item      | Dependency references an ID not found anywhere in the plan tree (work items and decisions both resolve cross-file)      |
| W004 | Module / Index | Section exists but is empty (`## Purpose`, `## In Scope`, `## Overview`, `## Problem & Success Criteria`, `## Modules`) |
| W005 | Module         | Status is `Ready` but no work items are defined                                                                         |
| W006 | Index          | Module listed under a `### Conductor / Crosscutting` index subsection but its file is not marked `Type: Conductor`       |
| W010 | Issues         | Issue entry missing `Status`, `Discovered`, or `Severity` field                                                         |
| W011 | Issues         | Question entry missing `Status`, `Discovered`, or `Priority` field                                                      |
| W012 | Issues         | Issue ID does not match `ISS-NNN` format or uses wrong casing                                                           |
| W013 | Issues         | Question ID does not match `Q-NNN` format or uses wrong casing                                                          |
| W017 | Module         | Active module (`Ready` / `In Progress`) has no `**Last reviewed:**` field, or it is older than `APS_STALE_DAYS` (60)    |
| W018 | Work Item      | Complete item has no `**Validation:**` field inside a still-active module — completion cannot be audited                |
| W019 | Index          | Link in `## Modules` points to a non-existent file (warning so seed plans stay clean; `aps audit` gates it as A004)     |
| W020 | Index          | Work-item ID defined in more than one child tree of a federated (nested-plans) monorepo — collisions make `<name>:<ID>` cross-tree references ambiguous (warning; each child tree stays independently valid) |
| W021 | Index          | Module ID defined in more than one child tree of a federated monorepo — a warning because IDs remain bare per tree, but orchestration resolves each status within its owning child |

> **Nested plans (monorepos).** When `aps lint` is pointed at a federated
> **parent** `index.aps.md` (one with a `## Child Plans` section), it follows
> those links and validates every child plan tree as one plan. Cross-tree
> dependencies are written `<child-name>:<ID>` (e.g. `core:AUTH-001`) and
> resolve against the named child when it is in scope; a child linted on its own
> treats such a reference as an intentional external link and stays clean. See
> the [nested-plans design](../plans/designs/2026-06-27-nested-plans.design.md).

#### JSON output

Pass `--json` for machine-readable results — handy for CI summaries:

```json
{
  "files": [
    {
      "path": "plans/modules/auth.aps.md",
      "type": "module",
      "errors": [],
      "warnings": [
        { "code": "W003", "message": "Dependency 'VAL-002' not found in this file", "line": 105 }
      ]
    }
  ],
  "summary": { "files": 2, "errors": 0, "warnings": 1 }
}
```

### `aps init` / `update` / `migrate`

These are the install-time commands:

- **`aps init`** scaffolds a new project (TUI wizard or flags).
- **`aps update [dir]`** reconciles the generated footprint — adds any missing
  core templates, refreshes existing ones, and reconciles the skill when
  installed. It reports each file as added / updated / unchanged / skipped, and
  never touches your plan content. (`aps setup upgrade` refreshes only the files
  already present; `update` is the one that also adds what's missing.)
- **`aps migrate [dir]`** moves a project off the vendored bash CLI onto the
  global binary: it runs the `aps doctor` diagnosis, then (with `--apply`) backs
  up and removes vendored CLI bloat, rewrites stale hook paths, pins
  `cli_version`, and drops a stale direnv `PATH_add bin`. Dry-runs by default.

See [installation.md](installation.md) for the full lifecycle.

## Orchestration

The orchestration commands read and rewrite your `.aps.md` files in place.
Markdown stays the single source of truth — there's no separate database.

### State machine

```text
Draft ──→ Ready ──→ In Progress ──→ Complete
              ↑           │
              └───────────┘  (reset only by manual edit)
```

**Canonical status vocabulary:** `Draft`, `Ready`, `In Progress`, `Complete`,
`Blocked`.

**Accepted aliases** (normalized internally, never rewritten in your files):

| Alias      | Canonical | Notes                                      |
| ---------- | --------- | ------------------------------------------ |
| `Proposed` | `Draft`   | anvil-001 and other adopters               |
| `Done`     | `Complete`| Terminal / compacted items                 |

Lint also treats `Merged`, `Released`, and `Shipped` as terminal completion
states (see work-item compaction). Orchestration maps `Done` to `Complete` for
dependency checks; `Proposed` maps to `Draft` (module not actionable until
`Ready`).

| Command        | Transition enforced                                     |
| -------------- | ------------------------------------------------------- |
| `aps next`     | None — read-only                                        |
| `aps start`    | Ready → In Progress (all dependencies must be Complete) |
| `aps complete` | In Progress → Complete                                  |
| `aps graph`    | None — read-only                                        |
| `aps audit`    | None — read-only (but executes Validation commands)     |

Invalid transitions are rejected with a clear error. The CLI never silently
forces a state change.

### `aps next` — find the next ready work item

```bash
$ aps next
AUTH-003: Implement token refresh
Module: AUTH | Dependencies: AUTH-001, AUTH-002, CORE-001 | Status: Ready
File: plans/modules/auth.aps.md

$ aps next auth          # Scope to one module
$ aps next --plans docs/plans
```

`next` walks every work item across every module, picks the first whose status
is `Ready` and whose dependencies (work-item IDs _and_ module IDs) all resolve
to Complete. Decision dependencies (`D-NNN`) are treated as resolved inline in
the plan text. Items in `Complete`, `Draft` (including `Proposed`), or
`Blocked` modules are skipped.

### `aps start <ID>` — claim a work item

```bash
$ aps start AUTH-003
Marked AUTH-003 as In Progress
Suggested branch: work/auth-003
File: plans/modules/auth.aps.md
Context package: .aps/context/AUTH-003.md
```

`start` validates that:

1. The work item exists and is currently `Ready`.
2. Every dependency is `Complete`.
3. The owning module is `Ready` or `In Progress` (you can't start items in a
   Draft module).

On success it:

- Rewrites the `- **Status:**` line (creates one if absent) to `In Progress`.
- Suggests a branch name (`work/<id>`) — branch creation is left to you per
  [ORCH D-003](../plans/modules/orchestrate.aps.md). APS doesn't manage git.
- Assembles a context package at `.aps/context/<ID>.md` containing the work
  item, module scope, decisions, dependency learnings, and related file paths.
  The directory is gitignored by default — regenerated on each `start`.

Running `aps start` on an item that's already `In Progress` is a no-op warning,
not an error. Running it on a `Complete` item is rejected.

### `aps complete <ID>` — close out a work item

```bash
$ aps complete AUTH-003
Marked AUTH-003 as Complete: 2026-05-12
File: plans/modules/auth.aps.md

$ aps complete AUTH-003 --learning "Token refresh needs retry on network errors"
Marked AUTH-003 as Complete: 2026-05-12
Learning recorded for AUTH-003
```

`complete` requires the item to be `In Progress`. It stamps Status as
`Complete: <UTC date>`. With `--learning "..."` it inserts a `- **Learning:**`
line immediately after `- **Validation:**` (per ORCH D-002). Learnings travel
with the work item — they show up in dependency learnings for downstream items
when those items `start`.

### `aps graph [module]` — see the dependency graph

```bash
$ aps graph auth
AUTH-001 [Complete] Create users
  <- none
AUTH-002 [Complete] Verify credentials
  <- AUTH-001[Complete]
AUTH-003 [Complete] Add token refresh
  <- AUTH-001[Complete] AUTH-002[Complete] CORE-001[Complete]
AUTH-004 [Ready] Add session audit log
  <- AUTH-003[Complete]
```

Useful for spotting blocked chains or auditing what's in flight. The arrow
reads as "blocked by". An empty `<- none` means no upstream dependencies.

### `aps audit [module]` — check plan state against reality

Plans drift: items get marked Complete without passing their own validation,
Draft items quietly ship, review dates go stale. The audit formalizes the
completion-audit pattern from anvil-001 (which found 8 overstated and 19
understated items across 191 by hand):

```bash
$ aps audit
APS Audit

Complete-item verification:
  AUTH-001     PASS     npm test -- auth.test.ts
  AUTH-002     FAIL     npm test -- session.test.ts
  AUTH-003     PARTIAL  Validation is not a runnable command

Findings:
  A001  AUTH-002     overstated: Validation failed: npm test -- session.test.ts
  A002  PAY-004      understated: Draft but files exist: src/pay/refund.ts
  A003  UI-002       stale: module last reviewed 2026-01-10 (89 days ago, threshold 60)
  A004  index        broken-link: ./modules/ghost.aps.md (line 41)

Findings: 4 (23 items audited)
```

| Code | Meaning                                                              |
| ---- | -------------------------------------------------------------------- |
| A001 | Overstated — `Complete` item whose `Validation` command fails        |
| A002 | Understated — `Draft` item whose `Files` already exist with content  |
| A003 | Stale — `Ready` item in a module with no recent `**Last reviewed:**` |
| A004 | Broken link — index `## Modules` link points to a non-existent file  |

Options: `--json` for machine-readable output, `--no-run` to skip executing
validation commands (verification reports `PARTIAL`; no A001 findings),
`--stale-days N` to tune the threshold (also via `APS_STALE_DAYS`). Exit
code is non-zero when there are findings, so it slots into CI as a deeper,
optional companion to `aps lint`.

> **Warning:** by default the audit _executes_ backtick `Validation` commands
> found in Complete work items, with full shell semantics (a timeout applies:
> `APS_AUDIT_TIMEOUT`, default 60s, and a notice is printed to stderr). The
> trust boundary is the plan file itself — anyone who can edit a Validation
> field controls what runs. Only run it on plans you trust, or pass
> `--no-run`. **In CI, use `--no-run` for pull-request-triggered jobs** —
> running with execution enabled on PR-modified plans hands code execution
> to the PR author. Reserve execution for trusted branches.

### Nested plans: federated orchestration

When you point an orchestration command at a **federated parent**
`index.aps.md` (one with a `## Child Plans` section), it treats the whole
nested tree as one queue — the same traversal `aps lint` uses. `aps next`,
`start`, `complete`, `graph`, and `audit` all see the parent plus every child
plan reachable through `## Child Plans`:

```bash
aps next  --plans packages          # next ready item across all child plans
aps graph --plans packages          # dependency graph spanning every tree
aps audit --plans packages --no-run # audit every child plan from the root
```

- **Scope to one child** with `--child <name>`, where `<name>` is the
  path-derived child name (the directory above its `plans/`, e.g. `core`):

  ```bash
  aps next --child core --plans packages
  aps audit --child api --plans packages --no-run
  ```

- **Cross-tree dependencies** written as `<name>:<ID>` (e.g. `core:AUTH-001`,
  per [D-002](../plans/modules/monorepo.aps.md)) are resolved against the named
  child, gate `aps next`/`start`, and appear as prefixed edges in `aps graph`.

- **Mutations target the owning child file.** `aps start`/`complete` rewrite
  the work item in the child module file it actually lives in, never a parent
  or a same-ID sibling. Accepts a bare ID, a `<name>:<ID>` ref, or a bare ID
  plus `--child`.

- **Ambiguous bare IDs are rejected.** If the same ID is defined in more than
  one child tree (the [W020](#aps-lint) case), `start`/`complete` refuse the
  bare ID and ask you to disambiguate with `<name>:<ID>` or `--child <name>`.

**Keeping the parent roll-up current.** A federation root carries a
`## Roll-up` table summarising each child (modules complete/total, next ready
item, overall status). The root index stays hand-authored, so the table does
not regenerate itself — `aps rollup` prints the current rows for you to paste:

```bash
$ aps rollup --plans packages
| Child | Modules (complete/total) | Next ready item | Status |
| ----- | ------------------------ | --------------- | ------ |
| core  | 0/1                      | AUTH-001        | Ready  |
| api   | 0/1                      | —               | Ready  |
```

The refresh ritual: at session end (or whenever a child's state changes), run
`aps rollup` and copy the table body into the parent's `## Roll-up` section.
Keeping it a manual paste keeps the root a readable, reviewable artifact rather
than generated output.

See the [nested-plans design](../plans/designs/2026-06-27-nested-plans.design.md)
and [monorepo guide](monorepo.md) for the full convention.

### `aps doctor` — diagnose migration state

`aps doctor` checks whether a project is cleanly on the global binary or still
carries a vendored bash CLI. It is read-only and prints one line per check:

```text
$ aps doctor
aps doctor — migration diagnostics

  [ok  ] global binary: aps 0.4.0 at /home/you/.aps/bin/aps
  [warn] cli_version: project pins 0.3.0 but this binary is 0.4.0 — install the pinned release or update the pin
  [warn] vendored CLI: leftover vendored CLI under /repo: bin/aps, lib — run `aps migrate` to back up and remove
  [ok  ] global runtime: /home/you/.aps/lib is complete
  [warn] direnv: /repo/.envrc still adds ./bin to PATH — drop it once you run on the global binary
```

Checks: global binary presence/version, `cli_version` match, leftover vendored
CLI trees (`bin/aps`, `lib/`, `.aps/bin`, `.aps/lib` — only when the bash CLI
marker is present, so an unrelated project `lib/` is never flagged), an
incomplete global `~/.aps/lib/` runtime (a missing file such as `audit.sh` is a
**problem** → non-zero exit), and a stale direnv `PATH_add bin` entry. See
[Migrating to the Global Binary](installation.md#migrating-to-the-global-binary)
for the full walkthrough.

### Driving a plan from end to end

```bash
aps next                              # Discover what to work on
aps start AUTH-003                    # Claim it
git switch -c work/auth-003           # Optional — follow the suggestion
# ...implement, test, commit...
aps complete AUTH-003 --learning "..."
aps next                              # Loop
```

That loop is the intended day-to-day flow for solo and AI-driven work. For
team workflows where multiple people might race on the same plan, treat the
markdown rewrites as commits — push and pull to coordinate.

## MCP Server

For agents that speak MCP, an optional server in [`mcp/`](../mcp/) exposes
the orchestration commands as a single `aps` tool over stdio. The agent sends
either a direct command (`"next auth"`) or a natural-language request
(`"what's the next ready work item in the auth module?"`); the server routes
it to an allowlisted CLI invocation and returns the result.

```bash
cd mcp && pnpm install      # one-time setup (Node >= 22.18)
```

Register it with your MCP-capable tool, e.g. for Claude Code:

```json
{
  "mcpServers": {
    "aps": {
      "command": "node",
      "args": ["/path/to/anvil-plan-spec/mcp/src/index.ts"],
      "env": { "APS_PLANS": "/path/to/your/project/plans" }
    }
  }
}
```

Environment variables: `APS_BIN` overrides the `aps` executable (defaults to
the sibling `bin/aps`, then `$PATH`); `APS_PLANS` sets the plan root passed
to every command. The server is optional — everything it does is also
available via the CLI or by editing markdown directly.

## Windows

On Windows, use the native `aps.exe` from the PowerShell installer or Scoop for
the cross-platform command surface:

```powershell
aps init
aps lint plans\
aps next
aps doctor
```

The legacy PowerShell script remains available for lint/init/update fallback
use cases:

```powershell
.\bin\aps.ps1 lint plans\
.\bin\aps.ps1 lint plans\modules\auth.aps.md
.\bin\aps.ps1 lint plans\ --json
```

Commands that still depend on the bash runtime should be run from WSL or Git
Bash. See [installation.md](installation.md#windows-details) for the recommended
PowerShell and Scoop install paths.

## CI Integration

Add `.github/workflows/lint-aps.yml` to your project. The canonical workflow
lives at [`docs/ci-lint-example.yml`](ci-lint-example.yml); the snippet below
keeps it inline for convenience.

```yaml
name: Lint APS Documents

on:
  push:
    paths: ["plans/**/*.aps.md", "plans/**/*.actions.md"]
  pull_request:
    paths: ["plans/**/*.aps.md", "plans/**/*.actions.md"]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install APS CLI
        run: |
          curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install \
            | bash -s -- --global

      - name: Lint APS documents
        run: aps lint plans/

      - name: Upload results on failure
        if: failure()
        run: aps lint plans/ --json > aps-lint-results.json

      - name: Upload artefact
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: aps-lint-results
          path: aps-lint-results.json
```

Using the installer keeps `lib/` in sync as new validation rules ship. If you
need to vendor the CLI without network access at build time, mirror `bin/aps`
plus the entire `lib/` directory into your repo and run `./bin/aps lint plans/`.
