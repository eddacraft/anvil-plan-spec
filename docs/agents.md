# APS Agents

APS provides three distributable agents that automate planning, execution, and
repository hygiene. Agent definitions are ported across Claude Code, Codex,
GitHub Copilot, OpenCode, and Grok — pick the variant for your tool below.

## Agents Overview

| Agent             | Purpose                                                  | Model  | Invocation                        |
| ----------------- | -------------------------------------------------------- | ------ | --------------------------------- |
| **APS Planner**   | Planning, scoping modules, drafting work items           | Opus   | `@aps-planner` or Task dispatch   |
| **APS Conductor** | Driving execution of Ready work items, wave coordination | Opus   | `@aps-conductor` or Task dispatch |
| **APS Librarian** | Archiving, cross-refs, orphan detection, repo hygiene    | Sonnet | `@aps-librarian` or Task dispatch |

### APS Planner

The Planner scopes and shapes work:

- **Initialize** — bootstrap `plans/` in new projects
- **Plan** — create indexes, modules, work items, action plans
- **Status** — scan artefacts and report current state

Use the Planner when starting new work or checking progress.

### APS Conductor

The Conductor drives execution of authored plans:

- **Execute** — pick up Ready work items and implement them via `aps start` / `aps complete`
- **Waves** — analyse dependencies and coordinate parallel execution across agents
- **Learning capture** — fold post-implementation learnings back into the work item

Use the Conductor when you have Ready work items and want them implemented.

### APS Librarian

The Librarian keeps your repo organized:

- **Audit** — scan for orphaned files, broken references, stale docs
- **Archive** — move completed modules to `plans/archive/`
- **Cross-refs** — verify all internal links resolve correctly
- **Filing** — identify stray planning docs and suggest proper locations

Use the Librarian after completing features, during cleanup sessions, or when
the repo feels disorganized.

## Planner vs Conductor vs Librarian

| Task                             | Agent     |
| -------------------------------- | --------- |
| "Create a plan for feature X"    | Planner   |
| "Draft a module for payments"    | Planner   |
| "What's the status of our work?" | Planner   |
| "Execute AUTH-001"               | Conductor |
| "Run the next ready work item"   | Conductor |
| "Coordinate this wave"           | Conductor |
| "Clean up after the auth module" | Librarian |
| "Are our docs consistent?"       | Librarian |
| "Archive completed specs"        | Librarian |

## Agents vs Skill

APS includes both **agents** (active dispatch) and a **skill** (passive
guidance):

- **Skill** (`aps-planning/SKILL.md`) — teaches the agent APS conventions.
  Always active. Provides behavioral nudges (plan before building, update
  specs as you work). Lightweight, no model cost.
- **Agents** (`aps-planner`, `aps-conductor`, `aps-librarian`) — perform
  specific APS tasks when dispatched. Use tool calls and reasoning. Consume
  model tokens.

Use the skill for day-to-day guidance. Use agents when you need active help
with planning or cleanup.

## Installation

### Claude Code

**Install:**

```bash
mkdir -p .claude/agents
cp scaffold/agents/claude-code/aps-planner.md .claude/agents/
cp scaffold/agents/claude-code/aps-conductor.md .claude/agents/
cp scaffold/agents/claude-code/aps-librarian.md .claude/agents/
```

The easiest path: run `aps init` and select Claude Code at the agent-port
prompt — the wizard installs all three for you. Manual `cp` is shown here for
completeness.

**Usage:**

Dispatch via the Agent tool or Task tool within Claude Code:

```
# Ask the planner to create a plan
> Use @aps-planner to plan the authentication module

# Ask the planner for status
> Use @aps-planner to report the current plan status

# Ask the librarian to audit
> Use @aps-librarian to scan for orphaned files and broken references
```

The Planner runs on Opus (deep reasoning). The Librarian runs on Sonnet
(fast, cheaper). Both are configured in the agent frontmatter.

### Copilot

**Install:**

```bash
mkdir -p .github/agents
cp scaffold/agents/copilot/aps-planner.md .github/agents/
cp scaffold/agents/copilot/aps-conductor.md .github/agents/
cp scaffold/agents/copilot/aps-librarian.md .github/agents/
```

**Usage:**

Invoke in Copilot Chat by mentioning the agent:

```
@aps-planner create a plan for the payments module
@aps-librarian check for stale docs in the repo
```

Copilot auto-discovers agents in `.github/agents/`. No model selection is
available — Copilot uses its default model.

### OpenCode

**Install:**

```bash
mkdir -p .opencode/agents
cp scaffold/agents/opencode/aps-planner.md .opencode/agents/
cp scaffold/agents/opencode/aps-conductor.md .opencode/agents/
cp scaffold/agents/opencode/aps-librarian.md .opencode/agents/
```

**Usage:**

Agents are configured as subagents (`mode: subagent`). Invoke via `@mention`:

```
@aps-planner what's the next ready work item?
@aps-librarian archive completed modules
```

Switch to an agent as a primary with Tab, or invoke as subagent with
`@mention`. By default APS omits a `model` field so OpenCode uses each user's
configured provider. Explicit install preference `opus` / `sonnet` pins
OpenAI Codex-family IDs (`openai/gpt-5.6-sol` / `openai/gpt-5.6-terra`), not
Anthropic — or set `model:` in the agent frontmatter yourself.

### Codex

**Install:**

```bash
mkdir -p .codex/agents
cp scaffold/agents/codex/aps-planner.toml .codex/agents/
cp scaffold/agents/codex/aps-conductor.toml .codex/agents/
cp scaffold/agents/codex/aps-librarian.toml .codex/agents/
```

Codex discovers project roles automatically from `.codex/agents/`. Each file
is a complete standalone role definition; no `.codex/config.toml` registration
is required. `aps init` / `aps setup codex` write a `model` field from the
install model preference (default: `gpt-5.6-sol` for planner/conductor,
`gpt-5.6-terra` for librarian). Override the field in a role file if needed.

For an existing APS project, run `aps update` or `aps setup codex` to refresh
the standalone roles and remove the obsolete registration snippet.

**Usage:**

Ask Codex to delegate a task to the role you want:

```
Use the aps-planner agent to plan the user authentication module.
Use the aps-conductor agent to execute the next ready work item.
Use the aps-librarian agent to audit the repo for orphaned files.
```

Use `/agent` in the Codex CLI to inspect and switch between agent threads.

### Grok

**Install:**

Grok Build needs no bespoke install: it reads the `AGENTS.md` instruction-file
family and auto-discovers Agent Skills from `.agents/skills/` — the same
payload the Codex install places — and from `.claude/` when present (D-040).
Selecting `grok` in `aps init` (or running `aps setup grok`) installs the
shared `.agents/skills/aps-planning/` skill.

**Usage:**

The planning skill activates when you ask about planning or repo hygiene:

```
Plan the authentication module using APS
Scan the repo for broken cross-references
```

For custom foreground subagents, Grok Build supports `subAgents` entries in
its own config; APS ships no Grok-specific agent files — the shared core
prompts under `scaffold/agents/core/` are the source of truth if you want to
wire some up.

## Model Cost

Install-time generation maps a **preference tier** (`default` / `opus` /
`sonnet` in config) to concrete IDs per tool. With `default` (recommended):

| Role | Claude Code | OpenCode | Codex |
| ---- | ----------- | -------- | ----- |
| Planner / Conductor | `opus` | *(omit — user config)* | `gpt-5.6-sol` |
| Librarian | `sonnet` | *(omit — user config)* | `gpt-5.6-terra` |

- **Planner** and **Conductor** use the premium tier (where pinned) because
  planning and orchestration need deeper reasoning about architecture and
  trade-offs.
- **Librarian** uses the balanced tier because repo hygiene is mostly
  pattern-matching and file organisation.
- **OpenCode** is multi-provider: default leaves `model` unset so each
  person's provider/model setup wins. Explicit `opus` / `sonnet` pin
  OpenAI Codex-family IDs (`openai/gpt-5.6-sol` / `openai/gpt-5.6-terra`)
  only when requested.

Choosing `opus` or `sonnet` in `aps init` forces **every** role on that tool
to the premium or balanced tier (where the tool supports pins). Copilot
agents have no model field (n/a in the wizard). You can still hand-edit
installed agent files after install.

## Building Agent Variants

Agent **bodies** live in `scaffold/agents/core/`. The CLI generates tool
envelopes (frontmatter / TOML + model IDs) at install from those cores and
the selected model preference.

The build script still produces **default-preference goldens** for review:

```bash
bash scaffold/agents/build.sh
```

This regenerates Claude Code, Copilot, OpenCode, and Codex trees under
`scaffold/agents/`. Grok Build and Antigravity consume the shared
`.agents/skills/` payload directly, so they have no agent variant.
