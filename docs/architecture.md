# Architecture and decision map

This describes the current source tree, not a new design approval. A release
tag may precede changes on main.

## Layers and data flow

1. Human-approved Markdown plans feed the parser and dependency graph.
2. Lint, next, graph, rollup, and export produce read views.
3. Start and complete change plan status; start generates a context brief.
4. Init, setup, update, and migrate consume flags/configuration and scaffold
   templates, skills, hooks, and generated agent envelopes.
5. The optional MCP stdio server routes allowlisted CLI arguments to the same
   plan files, not a second planning database.

Markdown is the durable source. No database or hosted coordination service is
required. Reading/editing the format needs no Rust, Node, or agent subscription.

## Source map

| Concern | Implementation | Check |
| --- | --- | --- |
| Native dispatch and flags | cli/src/main.rs | clap surface tests, native help |
| Parsing, selection, status changes | parser.rs, next.rs, orchestrate.rs in cli/src/ | Rust tests and test/orchestrate*.sh |
| Lint parity | cli/src/lint.rs, lib/rules/, lib/Lint.psm1, lib/lint.sh | test/cli-parity.sh, test/ps-parity.ps1 |
| CLI peers | bin/aps, bin/aps.ps1 | independent implementations, shared fixtures |
| Init, setup, updates | scaffold.rs, setup.rs, wizard.rs, update.rs, migrate.rs, managed.rs in cli/src/; shell peers | test/run.sh, native journeys |
| Binary self-update | cli/src/self_update.rs, scaffold/install*, scaffold/update* | self-update tests, release assets |
| Release records | cli/src/release.rs | release tests; REL-005 tracks pending ports |
| MCP transport | mcp/src/index.ts, mcp/src/route.ts | pnpm --dir mcp test |
| Installation content | templates/, scaffold/ | scaffold tests, agent generation |

The native CLI uses clap, crossterm, ratatui, and published eddacraft-tui.
cli/scaffold and cli/templates are source-tree symlinks used by embedded
assets; Cargo packaging dereferences them. Do not create divergent copies.

## Ownership and side effects

- plans/: durable intent, status, decisions, and evidence, reviewed in Git.
- .aps/config.yml: consumer settings, including the project CLI version pin.
- plans/project-context.md: consumer-owned context, not a generated status view.
- .aps/context/: disposable focused briefs produced by start.
- Managed skill markers: track installed payload ownership/freshness so updates
  can distinguish APS-managed content from local edits.

Start checks module/item status and dependencies, rewrites Markdown, and writes
a context package. It does **not** create an atomic cross-process claim.
Complete records status and learning; it does **not** run or independently
verify the validation command. Human/tool policy and contributor evidence
still determine whether completion is justified.

Audit may execute validation commands through bash (including in the native
Windows build); its --no-run mode is the bash-free inspection route. Native
PowerShell execution of validation commands is not implemented. Do not confuse
a successful Windows --no-run smoke with full audit execution support. MCP uses execFile with allowlisted commands rather
than arbitrary shell text, but start/complete still mutate files under the
server's OS permissions. It is not a sandbox or approval service. Point it at
a trusted plan tree; see [MCP setup](../mcp/README.md).

## Decisions and current limitations

The [roadmap decisions](../plans/index.aps.md#decisions) and owning modules
record current contracts:

- D-034–D-036: binary-first distribution, project config, version alignment.
- D-039: Rust/bash/PowerShell lockstep; shared tests are the behavioural gate.
- D-042–D-044: managed skill freshness and one project CLI version stamp.
- D-045: harness expansion; scaffold installation is not proof of every
  third-party harness's runtime discovery behaviour.
- D-047: model/reasoning hints, not an embedded model runtime.

[ADR-001](../plans/decisions/001-use-checkpoint-based-steps.md) records the
checkpoint-based breakdown (historically steps, now actions).
Designs live under [plans/designs](../plans/designs/); read each status and
owning module before treating a proposal as approved. The earlier
[architecture review](plans/2026-03-08-aps-v2-architecture-review.md) is a
historical Draft, not this implementation map.

Known incomplete surfaces:

- [REL-005](../plans/modules/release-planning.aps.md): native release commands
  exist; bash/PowerShell ports and shared closeout fixtures remain incomplete.
- [TEAM](../plans/modules/team-coordination.aps.md): atomic claims, leases,
  handoffs, and coordination projections remain Draft.
- [PROG](../plans/modules/progress-journal.aps.md): progress journals remain Draft.
- [CLI-002](../plans/modules/cli-redesign.aps.md): team-aware vocabulary depends
  on TEAM; a profile setting alone does not supply coordination.

## Validation and distribution

[CONTRIBUTING](../CONTRIBUTING.md#local-development-checks) lists local commands.
Windows is a first-class native target: CI builds the shipped x64 GNU zip and
runs the release-shaped journey on windows-latest under both PowerShell 7 and
Windows PowerShell 5.1. The independent fallback-script parity harness uses
PowerShell 7 on Linux. Neither Linux pwsh nor redirected Windows CI proves
the interactive Warp-specific CIB-010 gate. Standard native workflows need no
WSL; audit execution has the bash limitation described above; [native contributor checks](../CONTRIBUTING.md#windows-native-powershell)
separate Windows source-build prerequisites from binary-only use. GitHub releases, crates.io,
and Scoop distribute the CLI. Node is needed for contributor tooling or the
optional MCP server, not the native CLI or Markdown format.
