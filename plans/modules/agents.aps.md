# Agents Module

| ID    | Owner  | Priority | Status   |
| ----- | ------ | -------- | -------- |
| AGENT | @aneki | high     | In Progress |

**Last reviewed:** 2026-07-16

## Purpose

Build distributable APS agents for each AI tool harness. The core agent
logic (planning lifecycle, repo hygiene) is the same across tools — what
changes is the packaging format. Claude Code, Copilot, and OpenCode use agent
markdown files with frontmatter. Codex uses a multi-agent system with TOML
configuration. Grok Build ships no bespoke files — it auto-discovers the
Codex-shared `.agents/skills/` and the `AGENTS.md` family (D-040).

## Background

APS defines five conceptual roles (AGENTS.md): Planner, Implementer, Executor,
Reviewer, Librarian. Two exist as personal agents in code-env:

- `anvil-plan-spec.md` — Planner + Executor (planning, status, execution,
  wave coordination)
- `librarian.md` — Librarian (archiving, cross-refs, orphan detection)

These need to be:

1. Refined for distribution (strip personal assumptions, adapt to `.aps/`)
2. Ported to other tool harnesses (same capability, different packaging)

### Agent Mechanism Per Tool

| Tool            | Agent Mechanism                       | Format                                                                                                                             |
| --------------- | ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| **Claude Code** | `.claude/agents/<name>.md`            | Frontmatter (name, description, model, tools) + system prompt                                                                      |
| **Codex**       | Multi-agent TOML + skills             | Standalone `.codex/agents/<name>.toml` roles; also `.agents/skills/` for passive guidance                                       |
| **Copilot**     | `.github/agents/<name>.md`            | Frontmatter (name, description) + system prompt — very similar to Claude Code format                                               |
| **Grok**        | none (auto-discovery)                 | Reads `AGENTS.md` family; discovers `.agents/skills/<name>/SKILL.md` and `.claude/` assets natively (D-040)                        |
| **OpenCode**    | `.opencode/agents/<name>.md` + skills | Frontmatter (description, mode, model, tools, permission) + system prompt; also skills at `.opencode/skills/` or `.claude/skills/` |

The agent _content_ (system prompt, decision tree, APS knowledge) is largely
shared. What differs is frontmatter, file location, and tool-specific
affordances.

**Codex Multi-Agent Detail (Updated 2026-07-16)**

Codex has a richer agent mechanism beyond skills. The multi-agent system uses
standalone TOML role files under `.codex/agents/`:

```toml
name = "aps-planner"
description = "Plans and administers APS work."
developer_instructions = """
Role instructions here.
"""
```

Codex auto-discovers each TOML file as one role. `name`, `description`, and
`developer_instructions` are required; model, reasoning, sandbox, MCP, and
skill settings are optional overrides that otherwise inherit from the parent
session. Users ask Codex to delegate to a role and use `/agent` to inspect or
switch threads.

For APS, the Codex port produces both:

1. A skill (`.agents/skills/aps-planning/`) for passive guidance
2. Standalone role configs (`.codex/agents/*.toml`) for active dispatch

**Copilot Custom Agent Detail (Researched 2026-02-19)**

Copilot supports custom agents stored at `.github/agents/<name>.md` (repo
level) or in the `.github-private` repo (org level). Format is YAML
frontmatter (name, description, optional tools list) with a system prompt body
— structurally identical to Claude Code's agent files. This means the Claude
Code agent can be adapted with minimal changes: different file location,
frontmatter adjusted for Copilot's supported fields.

**OpenCode Agent Detail (Researched 2026-02-19)**

OpenCode has a rich agent system with two categories: primary agents (switched
via Tab) and subagents (invoked via `@mention` or Task tool). Agent files live
at `.opencode/agents/<name>.md` (project) or `~/.config/opencode/agents/`
(global). Format is markdown with YAML frontmatter:

```yaml
---
description: Agent purpose
mode: subagent # primary | subagent | all
model: anthropic/claude-opus-4-20250514
steps: 50
tools:
  write: true
  bash: true
permission:
  edit: "ask" # ask | allow | deny
---
System prompt here.
```

Key differences from Claude Code: `mode` field (primary vs subagent), `steps`
limit, granular `tools` and `permission` maps (per-tool allow/ask/deny), and
`model` uses `provider/model-id` format. The APS port should produce subagent
mode agents (users invoke them deliberately, not as the default primary agent).

## In Scope

- APS Planner agent for Claude Code (`.claude/agents/` format)
- APS Librarian agent for Claude Code
- Codex standalone agent roles (`.codex/agents/*.toml`)
- Copilot custom agents (`.github/agents/` format)
- OpenCode agents (`.opencode/agents/` format)
- Grok Build coverage via the Codex-shared `.agents/skills/` payload (D-040)
- Shared core prompt that all harness variants derive from
- Documentation on when/how to use each agent per tool
- Testing in fresh projects

## Out of Scope

- Implementer/Reviewer agents (general-purpose, not APS-specific)
- Agent-to-agent communication protocols
- MCP server integration (separate TASKS module)
- Tool-specific UI integrations beyond skill/agent files

## Interfaces

**Depends on:**

- INSTALL (Ready) — agents are packaged and distributed by the installer

**Exposes:**

- `scaffold/agents/claude-code/aps-planner.md` — Claude Code agent
- `scaffold/agents/claude-code/aps-librarian.md` — Claude Code agent
- `scaffold/agents/codex/aps-planner.toml` — Codex agent role config
- `scaffold/agents/codex/aps-librarian.toml` — Codex agent role config
- `scaffold/agents/copilot/aps-planner.md` — Copilot custom agent
- `scaffold/agents/copilot/aps-librarian.md` — Copilot custom agent
- `scaffold/agents/opencode/aps-planner.md` — OpenCode agent
- `scaffold/agents/opencode/aps-librarian.md` — OpenCode agent
- `scaffold/aps-planning/` — shared skill payload installed to
  `.agents/skills/aps-planning/` (Codex + Grok, D-040)

## Decisions

- **D-016:** Agent scope — Planner covers planning + execution + status +
  waves. Librarian covers archiving + cross-refs + orphans. No overlap.
  _decided: yes_
- **D-017:** Should agents reference `.aps/` paths or `plans/` paths?
  _decided: agents reference `plans/` (user content) and `.aps/scripts/`
  (tooling). They don't need to know about `.aps/config.yml`._
- **D-018:** Shared core vs per-tool rewrite — _decided: write a shared core
  prompt, then wrap it in tool-specific frontmatter/packaging. Minimises
  drift between harness variants._

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] Decisions resolved
- [x] Work items defined with validation

## Work Items

### AGENT-001: Build APS Planner agent (Claude Code) — Complete 2026-02-21

- **Intent:** Create the primary Planner agent for Claude Code
- **Expected Outcome:** `scaffold/agents/claude-code/aps-planner.md` with
  frontmatter (name, description, model: opus, tools) and system prompt
  covering: project init, index/module/work-item creation, status tracking,
  work item execution, wave-based parallel coordination. Derived from
  code-env's `anvil-plan-spec.md`, adapted for `.aps/` layout.
- **Validation:** Place agent in test project with `plans/`; dispatch via Task
  tool; agent reads plans, reports status, creates a module spec
- **Confidence:** high
- **Files:** scaffold/agents/claude-code/aps-planner.md

### AGENT-002: Build APS Librarian agent (Claude Code) — Complete 2026-02-21

- **Intent:** Create the primary Librarian agent for Claude Code
- **Expected Outcome:** `scaffold/agents/claude-code/aps-librarian.md` with
  frontmatter (name, description, model: sonnet, tools) and system prompt
  covering: archiving completed modules, orphan detection, cross-reference
  maintenance, stale doc flagging. Derived from code-env's `librarian.md`,
  adapted for `.aps/` layout.
- **Validation:** Place agent in test project with completed modules; dispatch
  via Task tool; agent identifies archivable modules and orphaned files
- **Confidence:** high
- **Files:** scaffold/agents/claude-code/aps-librarian.md

### AGENT-003: Port agents to Codex format — Complete 2026-02-21

- **Intent:** Make APS agents available to Codex users via multi-agent roles
- **Expected Outcome:** Two deliverables per agent:
  1. Agent role TOML configs — `scaffold/agents/codex/aps-planner.toml` and
     `scaffold/agents/codex/aps-librarian.toml` containing
     `developer_instructions` (derived from shared core prompt),
     `sandbox_mode`, and model config. Installer merges `[agents.aps-planner]`
     and `[agents.aps-librarian]` entries into user's `.codex/config.toml`
     with `config_file` pointing to the TOML overlays.
  2. Codex config snippet — example `[agents.*]` blocks for documentation and
     installer to use when writing `.codex/config.toml`.
     Core planning/librarian logic identical to Claude versions, adapted for
     Codex's `developer_instructions` field and TOML format.
- **Validation:** Agent role appears in Codex; `/agent spawn aps-planner`
  starts agent thread; agent reads plans and reports status. Fallback: skill
  at `.agents/skills/aps-planning/` still works as passive guidance.
- **Confidence:** medium
- **Dependencies:** AGENT-001, AGENT-002
- **Superseded by:** AGENT-008 updates the packaging contract to standalone
  auto-discovered roles.

### AGENT-004: Port agents to Copilot, OpenCode, and Gemini formats — Complete 2026-02-21

- **Intent:** Make APS agents available to Copilot, OpenCode, and Gemini users
  using each tool's native agent or skill format
- **Expected Outcome:** Three format variants:
  1. **Copilot** — `scaffold/agents/copilot/aps-planner.md` and
     `scaffold/agents/copilot/aps-librarian.md` with YAML frontmatter (name,
     description). Nearly identical to Claude Code format; installs to
     `.github/agents/`. Minimal adaptation needed.
  2. **OpenCode** — `scaffold/agents/opencode/aps-planner.md` and
     `scaffold/agents/opencode/aps-librarian.md` with frontmatter (description,
     mode: subagent, model, tools, permission). Installs to
     `.opencode/agents/`. Planner gets `mode: subagent` so users invoke it
     deliberately via `@aps-planner`.
  3. **Gemini** — `scaffold/agents/gemini/aps-planner/SKILL.md` and
     `scaffold/agents/gemini/aps-librarian/SKILL.md`. Skill format only
     (Gemini has no agent mechanism). Installs to `.gemini/skills/` or
     `.agents/skills/` with post-install `gemini skills link` instruction.
     Core logic identical to Claude versions across all three.
- **Validation:** Copilot agent discoverable at `.github/agents/`; OpenCode
  agent appears as subagent; Gemini skill links correctly
- **Confidence:** high
- **Dependencies:** AGENT-001, AGENT-002

### AGENT-005: Create agent documentation — Complete 2026-03-24

- **Intent:** Help users understand what each agent does and how to use it in
  their tool of choice
- **Expected Outcome:** Documentation covering: what each agent does, per-tool
  usage (dispatch command for Claude Code, skill invocation for Codex,
  activation for Gemini), when to use agent vs. passive skill, model cost
  implications
- **Validation:** Documentation exists with per-tool examples
- **Confidence:** high
- **Dependencies:** AGENT-001, AGENT-002, AGENT-003, AGENT-004

### AGENT-006: Test agents across harnesses — Complete 2026-03-28

- **Intent:** Verify agents work correctly in each tool's environment
- **Expected Outcome:** Test plan covering: Claude Code Task dispatch, Codex
  `/agent spawn`, Copilot agent discovery, OpenCode `@mention` invocation,
  Gemini skill link. Each agent performs its core function (planner creates
  plan, librarian audits repo) without errors in the tool's native format.
- **Validation:** Tests pass on clean projects per tool
- **Confidence:** medium
- **Dependencies:** AGENT-001, AGENT-002, AGENT-003, AGENT-004, INSTALL-003
- **Results:** Automated format/content validation complete for all 5 harnesses.
  build.sh idempotent, all 14 files correct. Fixed stale OpenCode model IDs
  (→ claude-opus-4-6 / claude-sonnet-4-6) and added Codex vendor comments.
  Manual end-to-end tests documented in docs/plans/2026-03-15-agent-cross-harness-test-plan.md — require
  respective tool installs. Claude Code agents validated live.

### AGENT-007: Retire Gemini scaffolding, add Grok (D-040) — In Progress

- **Intent:** Land the D-040 harness-set revision in the shipped tooling:
  Gemini out of init/setup/wizard/installers in all three CLIs; Grok in,
  riding the Codex-shared `.agents/skills/` assets (Grok Build discovers
  `.agents/skills/` and the `AGENTS.md` family natively — no bespoke files).
- **Expected Outcome:** `--tools` accepts `grok` and rejects `gemini`
  everywhere a tool list is parsed (Rust `config.rs`/`wizard.rs`/`setup.rs`/
  `scaffold.rs`, bash `lib/scaffold.sh`, `scaffold/install.ps1`); Grok install
  path reuses the Codex `.agents/skills/` payload with a Grok post-install
  note; `scaffold/agents/gemini/` handwritten skills and `build.sh` Gemini
  section removed; `GEMINI.md` remains on the migrate "protected" list; docs
  (README, getting-started, installation, agents, ai-agent-guide) name the
  new set.
- **Validation:** `test/run.sh` agent-init case covers
  `--tools claude-code,copilot,codex,opencode,grok`; `gemini` as a tool value
  errors with a pointer to D-040; cargo test green.
- **Confidence:** high
- **Dependencies:** D-040
- **Files:** cli/src/, lib/scaffold.sh, scaffold/agents/, scaffold/install.ps1,
  test/run.sh, docs/

### AGENT-008: Update Codex agents to standalone role discovery — Complete 2026-07-16

- **Intent:** Stop the APS scaffold from emitting malformed Codex agent roles
  or obsolete registration snippets.
- **Expected Outcome:** Every generated and installed `.codex/agents/*.toml`
  file defines non-empty `name`, `description`, and
  `developer_instructions`; Codex discovers the three roles without a config
  merge; generators, bash and Rust scaffold paths, and user docs emit no
  `codex-config-snippet.toml`.
- **Validation:** `bash scaffold/agents/build.sh` is idempotent;
  `./test/run.sh`, `cargo test --manifest-path cli/Cargo.toml`,
  `./bin/aps lint plans`, and markdownlint pass; a fresh Codex scaffold has
  exactly three valid role TOMLs and no registration snippet.
- **Identified From:** Codex 0.144.5 warnings against an APS-generated project
  after the standalone role schema made `name`, `description`, and
  `developer_instructions` mandatory.
- **Confidence:** high
- **Dependencies:** AGENT-003
- **Files:** scaffold/agents/, lib/scaffold.sh, cli/src/scaffold.rs,
  cli/src/setup.rs, cli/src/update.rs, cli/src/wizard.rs, test/run.sh, docs/agents.md,
  docs/ai/prompting/codex/README.md
- **Results:** The shared generator now emits complete standalone Codex roles;
  bash and native Rust updates refresh installed roles and remove both legacy
  snippet locations, and fresh scaffolds omit the snippet. End-to-end tests
  enforce the required schema across canonical and installed files.
  Documentation and plan decisions now describe automatic discovery.

## Execution Strategy

### Wave 1: Claude Code agents (parallel, no dependencies)

- AGENT-001: APS Planner (Claude Code)
- AGENT-002: APS Librarian (Claude Code)

### Wave 2: Documentation (depends on Wave 1)

- AGENT-005: Agent documentation (partial — Claude Code section)

### Wave 3: Port to other harnesses (depends on Wave 1)

- AGENT-003: Codex multi-agent port (TOML config)
- AGENT-004: Copilot + OpenCode agents, Gemini skill

### Wave 4: Final docs + testing (depends on Wave 3)

- AGENT-005: Agent documentation (complete — all tools)
- AGENT-006: Cross-harness testing

## Notes

- The key architectural decision is **shared core, tool-specific wrapper**.
  The APS knowledge (hierarchy, templates, workflow, decision tree) lives
  in a shared core. Each harness variant wraps it in appropriate frontmatter
  and adjusts for tool-specific features.
- **Four tools now have real agent mechanisms** (Claude Code, Codex, Copilot,
  OpenCode). Only Gemini is skill-only. This is a significant finding — the
  original assumption was that only Claude Code had agents.
- **Adaptation effort varies by tool:**
  - Copilot: minimal — nearly identical to Claude Code (`.md` + frontmatter)
  - OpenCode: moderate — same `.md` format but richer frontmatter (mode,
    tools map, permission map, model as `provider/id`)
  - Codex: significant — entirely different format (TOML config, agent roles,
    `developer_instructions` field, concurrent thread model)
  - Gemini: minimal — skill only, no agent adaptation needed
- The Planner is the heavier agent (Opus for Claude Code, model varies per
  tool). The Librarian is lighter (Sonnet). Tools with model selection:
  Claude Code (model field), OpenCode (model field), Codex (model in TOML).
  Copilot and Gemini don't expose model choice in agent/skill config.
- Agents should NOT duplicate the SKILL.md content. The passive skill handles
  behavioral guidance (plan-before-code, update specs). The agents handle
  active dispatch (create a plan for me, audit the repo).
- OpenCode agents should use `mode: subagent` so they're invoked deliberately
  (via `@aps-planner`) rather than being the default primary agent.
