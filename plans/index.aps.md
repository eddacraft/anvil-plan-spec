# APS Roadmap

| Field   | Value      |
| ------- | ---------- |
| Status  | Active     |
| Owner   | @aneki     |
| Created | 2025-12-31 |
| Updated | 2026-07-16 |

## Problem

APS needs continued development to:

1. **Lower adoption barriers** — New users should get value in minutes, not hours
2. **Improve tooling** — Validation, search, and integration with existing workflows
3. **Build ecosystem** — Examples, templates, and community contributions

## Success Criteria

- [x] New user can try APS in under 5 minutes
- [x] CLI validates APS documents in CI pipelines
- [x] Documentation covers common workflows with examples
- [x] Knowledge compounds across projects via solution library

## Constraints

- No runtime dependencies — APS remains pure markdown
- No vendor lock-in — Specs stay portable across tools
- No breaking changes without migration path

## Modules

### Current (v0.2 — Usability)

| Module                                    | Purpose                               | Status   |
| ----------------------------------------- | ------------------------------------- | -------- |
| [scaffold](./modules/scaffold.aps.md)     | One-command setup for new projects    | Complete |
| [templates](./modules/templates.aps.md)   | Reduce friction, mark optional fields | Complete |
| [docs](./modules/docs.aps.md)             | Workflow guide, improved onboarding   | Complete |
| [validation](./modules/validation.aps.md) | CLI tool to validate APS documents    | In Progress |

### Shipped (v0.3 — Distribution)

| Module                              | Purpose                                                   | Status                  |
| ----------------------------------- | --------------------------------------------------------- | ----------------------- |
| [install](./modules/install.aps.md) | Global binary install, project config contract, migration | Complete |
| [agents](./modules/agents.aps.md)   | APS Planner + Librarian agents, multi-harness             | In Progress             |

### In Progress (v0.4 — Orchestration & UX)

| Module                                      | Purpose                                                 | Status   |
| ------------------------------------------- | ------------------------------------------------------- | -------- |
| [orchestrate](./modules/orchestrate.aps.md) | CLI orchestration, dependency resolution, state machine | Complete |
| [tui](./modules/tui.aps.md)                 | Ratatui TUI customization wizard for project setup      | Complete |
| [dogfood](./modules/dogfood.aps.md)         | Keep this repo's own APS plans accurate and validated   | Complete |
| [conductor](./modules/conductor.aps.md)     | New module type for cross-module concerns               | Complete |

### Conductor / Crosscutting (Adopted)

<!-- Modules listed here carry `Type: Conductor` (they coordinate work across
     vertical modules). The `conductor` module that *introduces* the type is a
     normal feature module and lives under v0.4 above. `aps lint` enforces the
     Type marker on entries in this section (W006). -->

| Module                                                                  | Purpose                                                     | Status      |
| ----------------------------------------------------------------------- | ----------------------------------------------------------- | ----------- |
| [release-planning](./modules/release-planning.aps.md)                   | Release plan template, scaffold, `aps lint` rules, and docs | In Progress |
| [continuous-improvement](./modules/continuous-improvement-backlog.aps.md) | Standing intake for small cross-module correctness fixes    | In Progress |

### Compound-Engineering (Complete)

| Module                                | Purpose                                                          | Status   |
| ------------------------------------- | ---------------------------------------------------------------- | -------- |
| [compound](./modules/compound.aps.md) | Review/Learn phase tooling (solution library, completed archive) | Complete |

Task-level history rolls up in [`plans/completed.aps.md`](./completed.aps.md);
release narratives live in [`plans/releases/`](./releases/).

### Near Term

| Module                                        | Purpose                                                   | Status   |
| --------------------------------------------- | --------------------------------------------------------- | -------- |
| [spec](./modules/spec.aps.md)                 | Canonical vocabulary + schema (status reconciliation)     | Complete |
| [tasks](./modules/tasks.aps.md)               | Claude Code Tasks integration                             | Complete |
| [examples](./modules/examples.aps.md)         | Additional worked examples                                | Ready    |
| [prompts](./modules/prompts.aps.md)           | Tool-specific prompt variants                             | Ready    |
| [integrations](./modules/integrations.aps.md) | JSON export, GitHub Action, lint/rollup CI surface        | Ready    |
| [monorepo](./modules/monorepo.aps.md)         | Nested index.aps.md plans, federated lint + orchestration | Complete |
| [package-views](./modules/package-views.aps.md) | CLI tooling for the tagged monorepo tier (`Packages:` lint, next filter, generated views) | Complete |
| [ci-parity](./modules/ci-parity.aps.md)       | Behavioural pwsh + cross-CLI parity checks in CI          | Complete |

### Long Term

| Module    | Purpose                          | Status   |
| --------- | -------------------------------- | -------- |
| ecosystem | GitHub Action, VS Code extension | Proposed |

## Non-Goals

These are explicitly out of scope:

- **Execution engines** — APS describes intent; it doesn't run code
- **Vendor plugins** — No Jira/Linear/Notion plugins (specs are portable markdown)
- **AI training** — Not a dataset for model fine-tuning
- **Hosted services** — No cloud component; everything runs locally

## Risks

| Risk                                     | Impact | Mitigation                                       |
| ---------------------------------------- | ------ | ------------------------------------------------ |
| Scope creep into PM territory            | High   | Maintain non-goals, reject out-of-scope requests |
| Template changes break existing specs    | Medium | Keep field names stable, add optional markers    |
| Tooling complexity undermines simplicity | Medium | CLI stays optional; markdown-first always        |

## Decisions

- **D-001:** Rename "Leaf" to "Module" — _decided: yes, improves clarity_
- **D-002:** Use `ID` instead of `SCOPE` in templates — _decided: yes, less confusing_
- **D-003:** Add aps-rules.md as portable agent guide — _decided: yes, travels with templates_
- **D-004:** Adopt compound engineering philosophy — _decided: yes, planning lifecycle_
- **D-005:** Quickstart as default entry point — _decided: no, keep template choices_
- **D-006:** Tool-specific prompt variants — _decided: yes, need variants or stubs pointing to generic AGENTS.md_
- **D-007:** Validation approach — _decided: standalone CLI first, then GitHub Action wrapper_
- **D-008:** Solution docs organization — _decided: per-project with monorepo support_
- **D-009:** npm init module — _decided: merged into scaffold module, no separate npm package_
- **D-010:** Claude Code Tasks integration — _decided: yes, APS as planning layer + Tasks as execution layer_
- **D-011:** `.aps/` as tooling root — _decided: yes, CLI + scripts + config + ephemeral under `.aps/`_
- **D-012:** CLI location — _decided: `.aps/bin/aps` with PATH hint (direnv or shell). Amended 2026-06-15: global release binary on PATH is primary; `.aps/bin/` optional (see install D-034)_
- **D-034:** Global binary-first install — _decided: default distribution is the release `aps` binary (Mac/Linux/Windows) via GitHub releases, install script, crates.io, and Scoop; `aps init` scaffolds project content only. See [install.aps.md](./modules/install.aps.md) INSTALL-014..018_
- **D-035:** `.aps/config.yml` project contract — _decided: `cli_version` pins the toolchain; `plans_dir` / `docs_dir` / `tooling_root` are runtime defaults discovered by global `aps`. Explicit flags override_
- **D-036:** Install-channel semver alignment — _decided: all distribution channels publish the same release version; per-project pin is `cli_version` in `.aps/config.yml`_
- **D-013:** Skill format per tool — _decided: `.claude/skills/` as cross-tool path (Claude Code + Copilot + OpenCode auto-discover), `.agents/skills/` for Codex + Gemini (both require explicit install/link cmd); instruction files per tool (AGENTS.md, GEMINI.md). Harness set amended by D-040 (Gemini out, Grok in; Grok discovers `.agents/skills/` natively)_
- **D-014:** Agent model defaults — _decided: Planner on Opus, Librarian on Sonnet_
- **D-015:** Commands deprecated — _decided: yes, fold `/plan` and `/plan-status` into skill_
- **D-016:** Agent scope split — _decided: Planner = planning + execution + status + waves; Librarian = archiving + cross-refs + orphans_
- **D-017:** Agent path references — _decided: agents reference `plans/` and `.aps/scripts/`, not `.aps/config.yml`_
- **D-018:** Shared core vs per-tool rewrite — _decided: shared core prompt, tool-specific frontmatter/packaging_
- **D-019:** Agent format per tool — _decided: 4/5 tools have native agent mechanisms (Claude Code `.claude/agents/`, Copilot `.github/agents/`, OpenCode `.opencode/agents/`, Codex `.codex/config.toml` + TOML overlays); Gemini is skill-only. Port to each tool's native format, not just skills. Harness set amended by D-040 (Gemini out, Grok in; Grok consumes the Codex-shared `.agents/skills/` assets)_
- **D-022:** External planning repo reversed — _decided: plans move back to main repo_
- **D-023:** Commands fully dropped — _decided: yes, skills only, no `.claude/commands/` shipped_
- **D-024:** aps-rules.md split — _decided: `aps-rules.md` (APS-managed) + `project-context.md` (user-owned)_
- **D-025:** designs/ and issues.md into plans/ — _decided: single planning content root_
- **D-026:** Promote `spec` module from Long Term to Near Term — _decided: yes; status vocabulary resolved via SPEC-001 (see D-037)_
- **D-037:** Status vocabulary aliases — _decided 2026-06-15: Approach A — canonical `Draft / Ready / In Progress / Complete / Blocked`; accept `Proposed→Draft` and `Done→Complete` as aliases without rewriting files. See [spec.aps.md](./modules/spec.aps.md)_
- **D-027:** Promote `compound` from Draft to Ready — _decided: yes, anvil-001 surveyed prior art (completed/ archive, releases/ narrative, completed-index roll-up) makes the work concrete; see [compound.aps.md](./modules/compound.aps.md)_
- **D-028:** Add release planning as an APS addon — _decided: yes, extract pattern from anvil-001 trial (`plans/releases/v0.3.0-beta.md`); see `release-planning.aps.md`_
- **D-029:** Introduce conductor / crosscutting module type — _decided 2026-06-18: adopt. Trial concluded via COND-001 — release-planning carries `Type: Conductor`, cross-module references resolve clean, the marker is lint-safe. Remaining work: COND-002 (schema/template) + COND-003 (linter awareness)_
- **D-030:** Deeper monorepo support via nested indexes — _decided: yes, as a new Draft module. Tagged modules (docs/monorepo.md) stay the default tier; nested `index.aps.md` plans are the federated tier for packages with independent owners/lifecycles. See [monorepo.aps.md](./modules/monorepo.aps.md) — its D-001..D-004 (child location, ID namespacing, child autonomy, coexistence) resolved 2026-06-26: co-located child plans, bare per-tree IDs with path-qualified cross-tree refs, standalone children, tags-default coexistence; module promoted to Ready (MONO-001 complete 2026-06-27, module now In Progress)_
- **D-038:** Promote `prompts` from Draft to Ready — _decided 2026-06-27: yes. Work broken out into PROMPTS-001 (normalize existing variants), PROMPTS-002 (variant-vs-stub policy), PROMPTS-003 (stubs for Copilot/Codex/Gemini, closing the D-006 coverage gap). See [prompts.aps.md](./modules/prompts.aps.md)_
- **D-039:** CLI three-way lockstep — _decided 2026-07-01: the `aps` CLI has three independent implementations of one command surface — Rust (`cli/src/`, the primary distributed binary), bash (`bin/aps` + `lib/*.sh`, the zero-dependency Unix reference/fallback), and PowerShell (`bin/aps.ps1` + `lib/*.psm1`, the Windows fallback). They do not call each other, so a rule added to one is absent from the others until ported by hand. **Policy:** all three stay in lockstep — a lint/`next`/orchestration change is not `Complete` until it lands in all three and the shared parity suite (`test/fixtures/**`) confirms identical behaviour. This supersedes orchestrate D-006's "bash is feature-frozen after parity" clause (bash is a maintained peer, not frozen) and extends the PowerShell-parity rule to Rust. See tui D-031, orchestrate D-006, and monorepo MONO-007 (the first parity debt this policy retires)._
- **D-040:** Harness set revision — Gemini out, Grok in — _decided 2026-07-16: the supported harness set becomes **Claude Code, Copilot, Codex, OpenCode, Grok** (amends the five fixed by D-013/D-019). xAI's Grok Build reads the `AGENTS.md` instruction-file family and discovers Agent Skills from `.agents/skills/` (and `.claude/` assets) natively, so it slots into the existing Codex-shared paths with no bespoke assets — no `GROK.md`, no handwritten skill copies. Gemini scaffolding (`.gemini/skills/`, the `gemini` tool option in init/setup/wizard) is removed from all three CLIs and the installers; existing user installs are untouched and `GEMINI.md` stays on the migrate "protected, never removed" list. See [prompts.aps.md](./modules/prompts.aps.md) PROMPTS-003 and [agents.aps.md](./modules/agents.aps.md)._
