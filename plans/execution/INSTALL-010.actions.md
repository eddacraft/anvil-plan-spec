# Action Plan: INSTALL-010

| Field      | Value                                                   |
| ---------- | ------------------------------------------------------- |
| Source     | [./modules/install.aps.md](../modules/install.aps.md)   |
| Work Item  | INSTALL-010 — Split install, init, setup, agent, upgrade |
| Created by | @aneki / AI                                             |
| Status     | In Progress                                             |

## Prerequisites

- [x] INSTALL-003 (shell-prompt install wizard) Complete
- [x] INSTALL-008 (v1→v2 migration) Complete
- [x] INSTALL-009 (aps-rules split) Complete
- [x] Installer already has install_global / install_agent / install_binary

## Waves

| Wave | Actions | Gate                                                     |
| ---- | ------- | -------------------------------------------------------- |
| 1    | 1, 2    | bash installer dispatches modes + TTY picker; tests pass |
| 2    | 3       | PowerShell parity                                        |
| 3    | 4, 5    | docs + advertised command updated; full suite green      |

## Actions

### 1. Mode dispatch in `scaffold/install`

- **Checkpoint:** `--cli`, `--init`, `--agent`, `--upgrade`, `--setup <tool>`
  each route to one path; `--global`/`--binary` still work
- **Validate:** `bash -n scaffold/install`

### 2. TTY mode picker + demote default

- **Checkpoint:** With no mode flag in a TTY, a picker is shown before any
  file is written; non-TTY with no mode prints usage and exits non-zero
- **Validate:** `./test/run.sh`

### 3. PowerShell parity in `scaffold/install.ps1`

- **Checkpoint:** Same flags and picker behavior on the PS path
- **Validate:** reviewed against bash (no pwsh on host; CI shell job covers
  bash syntax)

### 4. Update advertised command + docs

- **Checkpoint:** `docs/installation.md` and `README.md` show the picker and
  the per-mode flags
- **Validate:** `pnpm exec markdownlint docs/installation.md README.md`

### 5. Tests

- **Checkpoint:** Coverage for each mode flag and the non-TTY-no-mode guard
- **Validate:** `./test/run.sh`

## Completion

- [x] All checkpoints validated
- [x] Work item marked complete in `install.aps.md`

**Completed by:** @aneki / AI — 2026-06-14
