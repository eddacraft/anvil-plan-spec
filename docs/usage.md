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
aps update [dir]            # Refresh templates, skill, and tool files
aps migrate [dir]           # Convert v1 layout to v2 (.aps/)
aps lint [file|dir]         # Validate APS documents
aps next [module]           # Show the next ready work item
aps start <ID>              # Mark a Ready work item as In Progress
aps complete <ID>           # Mark an In Progress work item as Complete
aps graph [module]          # Show work items + dependency arrows
aps audit [module]          # Audit plan state against reality
aps --help                  # Top-level help
aps <cmd> --help            # Per-command help
```

Every command accepts `--plans <dir>` if your plans aren't at the default
`plans/` location.

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

#### Warning codes

| Code | Scope          | Description                                                                                                             |
| ---- | -------------- | ----------------------------------------------------------------------------------------------------------------------- |
| W001 | Work Item      | ID does not match `PREFIX-NNN` pattern (e.g., `AUTH-001`)                                                               |
| W003 | Work Item      | Dependency references an ID not found anywhere in the plan tree (work items and decisions both resolve cross-file)      |
| W004 | Module / Index | Section exists but is empty (`## Purpose`, `## In Scope`, `## Overview`, `## Problem & Success Criteria`, `## Modules`) |
| W005 | Module         | Status is `Ready` but no work items are defined                                                                         |
| W010 | Issues         | Issue entry missing `Status`, `Discovered`, or `Severity` field                                                         |
| W011 | Issues         | Question entry missing `Status`, `Discovered`, or `Priority` field                                                      |
| W012 | Issues         | Issue ID does not match `ISS-NNN` format or uses wrong casing                                                           |
| W013 | Issues         | Question ID does not match `Q-NNN` format or uses wrong casing                                                          |
| W017 | Module         | Active module (`Ready` / `In Progress`) has no `**Last reviewed:**` field, or it is older than `APS_STALE_DAYS` (60)    |
| W018 | Work Item      | Complete item has no `**Validation:**` field inside a still-active module — completion cannot be audited                |
| W019 | Index          | Link in `## Modules` points to a non-existent file (warning so seed plans stay clean; `aps audit` gates it as A004)     |

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

These are the install-time commands. See [installation.md](installation.md) for
the full lifecycle (scaffold, update, v1→v2 migration).

## Orchestration

The orchestration commands read and rewrite your `.aps.md` files in place.
Markdown stays the single source of truth — there's no separate database.

### State machine

```text
Draft ──→ Ready ──→ In Progress ──→ Complete
              ↑           │
              └───────────┘  (reset only by manual edit)
```

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
the plan text. Items in `Complete`, `Draft`, or `Blocked` modules are skipped.

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

## Windows (PowerShell)

The PowerShell port currently mirrors the lint/init/update surface:

```powershell
.\bin\aps.ps1 lint plans\
.\bin\aps.ps1 lint plans\modules\auth.aps.md
.\bin\aps.ps1 lint plans\ --json
```

For orchestration on Windows, use WSL or Git Bash with the bash CLI. A native
PowerShell port of `next`/`start`/`complete`/`graph` is on the roadmap.

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
