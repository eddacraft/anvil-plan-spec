# APS Roadmap

| Field | Value |
|-------|-------|
| Status | Active |
| Owner | @aneki |
| Created | 2025-12-31 |
| Updated | 2026-02-19 |

## Problem

APS needs continued development to:

1. **Lower adoption barriers** — New users should get value in minutes, not hours
2. **Improve tooling** — Validation, search, and integration with existing workflows
3. **Build ecosystem** — Examples, templates, and community contributions

## Success Criteria

- [x] New user can try APS in under 5 minutes
- [x] CLI validates APS documents in CI pipelines
- [x] Documentation covers common workflows with examples
- [ ] Knowledge compounds across projects via solution library

## Constraints

- No runtime dependencies — APS remains pure markdown
- No vendor lock-in — Specs stay portable across tools
- No breaking changes without migration path

## Modules

### Current (v0.2 — Usability)

| Module | Purpose | Status |
|--------|---------|--------|
| [scaffold](./modules/scaffold.aps.md) | One-command setup for new projects | Complete |
| [templates](./modules/templates.aps.md) | Reduce friction, mark optional fields | Complete |
| [docs](./modules/docs.aps.md) | Workflow guide, improved onboarding | Complete |
| [validation](./modules/validation.aps.md) | CLI tool to validate APS documents | Complete |

### Current (v0.3 — Distribution)

| Module | Purpose | Status |
|--------|---------|--------|
| [install](./modules/install.aps.md) | Interactive install, `.aps/` layout, multi-tool | Complete |
| [agents](./modules/agents.aps.md) | APS Planner + Librarian agents, multi-harness | Complete |

### Near Term

| Module | Purpose | Status |
|--------|---------|--------|
| [orchestrate](./modules/orchestrate.aps.md) | CLI orchestration, dependency resolution, state machine | Ready |
| [tui](./modules/tui.aps.md) | Ratatui TUI customization wizard for project setup | Ready |
| [spec](./modules/spec.aps.md) | Canonical vocabulary + schema (status reconciliation) | Draft |
| [compound](./modules/compound.aps.md) | Review/Learn phase tooling (solution library, completed archive) | Ready |
| [tasks](./modules/tasks.aps.md) | Claude Code Tasks integration | Draft |
| [examples](./modules/examples.aps.md) | Additional worked examples | Draft |
| [prompts](./modules/prompts.aps.md) | Tool-specific prompt variants | Draft |
| [integrations](./modules/integrations.aps.md) | JSON export, GitHub sync | Draft |

### Long Term

| Module | Purpose | Status |
|--------|---------|--------|
| ecosystem | GitHub Action, VS Code extension | Proposed |

## Non-Goals

These are explicitly out of scope:

- **Execution engines** — APS describes intent; it doesn't run code
- **Vendor plugins** — No Jira/Linear/Notion plugins (specs are portable markdown)
- **AI training** — Not a dataset for model fine-tuning
- **Hosted services** — No cloud component; everything runs locally

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Scope creep into PM territory | High | Maintain non-goals, reject out-of-scope requests |
| Template changes break existing specs | Medium | Keep field names stable, add optional markers |
| Tooling complexity undermines simplicity | Medium | CLI stays optional; markdown-first always |

## Decisions

- **D-001:** Rename "Leaf" to "Module" — *decided: yes, improves clarity*
- **D-002:** Use `ID` instead of `SCOPE` in templates — *decided: yes, less confusing*
- **D-003:** Add aps-rules.md as portable agent guide — *decided: yes, travels with templates*
- **D-004:** Adopt compound engineering philosophy — *decided: yes, planning lifecycle*
- **D-005:** Quickstart as default entry point — *decided: no, keep template choices*
- **D-006:** Tool-specific prompt variants — *decided: yes, need variants or stubs pointing to generic AGENTS.md*
- **D-007:** Validation approach — *decided: standalone CLI first, then GitHub Action wrapper*
- **D-008:** Solution docs organization — *decided: per-project with monorepo support*
- **D-009:** npm init module — *decided: merged into scaffold module, no separate npm package*
- **D-010:** Claude Code Tasks integration — *decided: yes, APS as planning layer + Tasks as execution layer*
- **D-011:** `.aps/` as tooling root — *decided: yes, CLI + scripts + config + ephemeral under `.aps/`*
- **D-012:** CLI location — *decided: `.aps/bin/aps` with PATH hint (direnv or shell)*
- **D-013:** Skill format per tool — *decided: `.claude/skills/` as cross-tool path (Claude Code + Copilot + OpenCode auto-discover), `.agents/skills/` for Codex + Gemini (both require explicit install/link cmd); instruction files per tool (AGENTS.md, GEMINI.md)*
- **D-014:** Agent model defaults — *decided: Planner on Opus, Librarian on Sonnet*
- **D-015:** Commands deprecated — *decided: yes, fold `/plan` and `/plan-status` into skill*
- **D-016:** Agent scope split — *decided: Planner = planning + execution + status + waves; Librarian = archiving + cross-refs + orphans*
- **D-017:** Agent path references — *decided: agents reference `plans/` and `.aps/scripts/`, not `.aps/config.yml`*
- **D-018:** Shared core vs per-tool rewrite — *decided: shared core prompt, tool-specific frontmatter/packaging*
- **D-019:** Agent format per tool — *decided: 4/5 tools have native agent mechanisms (Claude Code `.claude/agents/`, Copilot `.github/agents/`, OpenCode `.opencode/agents/`, Codex `.codex/config.toml` + TOML overlays); Gemini is skill-only. Port to each tool's native format, not just skills.*
- **D-022:** External planning repo reversed — *decided: plans move back to main repo*
- **D-023:** Commands fully dropped — *decided: skills only, no `.claude/commands/` shipped*
- **D-024:** aps-rules.md split — *decided: `aps-rules.md` (APS-managed) + `project-context.md` (user-owned)*
- **D-025:** designs/ and issues.md into plans/ — *decided: single planning content root*
- **D-026:** Promote `spec` module from Long Term to Near Term — *decided: yes, status vocabulary divergence with anvil-001 needs formal resolution; see [spec.aps.md](./modules/spec.aps.md) D-026 for the open Draft↔Proposed / Complete↔Done question*
- **D-027:** Promote `compound` from Draft to Ready — *decided: yes, anvil-001 surveyed prior art (completed/ archive, releases/ narrative, completed-index roll-up) makes the work concrete; see [compound.aps.md](./modules/compound.aps.md)*
