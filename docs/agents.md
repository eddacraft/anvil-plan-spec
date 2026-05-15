# APS Agents

APS provides two distributable agents that automate planning lifecycle and
repository hygiene tasks.

## Agents Overview

| Agent             | Purpose                                                 | Model  | Invocation                        |
| ----------------- | ------------------------------------------------------- | ------ | --------------------------------- |
| **APS Planner**   | Planning, execution, status tracking, wave coordination | Opus   | `@aps-planner` or Task dispatch   |
| **APS Librarian** | Archiving, cross-refs, orphan detection, repo hygiene   | Sonnet | `@aps-librarian` or Task dispatch |

### APS Planner

The Planner manages the full APS lifecycle:

- **Initialize** — bootstrap `plans/` in new projects
- **Plan** — create indexes, modules, work items, action plans
- **Status** — scan artefacts and report current state
- **Execute** — pick up Ready work items and implement them
- **Waves** — analyze dependencies and plan parallel execution

Use the Planner when starting new work, checking progress, or executing
planned work items.

### APS Librarian

The Librarian keeps your repo organized:

- **Audit** — scan for orphaned files, broken references, stale docs
- **Archive** — move completed modules to `plans/archive/`
- **Cross-refs** — verify all internal links resolve correctly
- **Filing** — identify stray planning docs and suggest proper locations

Use the Librarian after completing features, during cleanup sessions, or when
the repo feels disorganized.

## Planner vs Librarian

| Task                             | Agent     |
| -------------------------------- | --------- |
| "Create a plan for feature X"    | Planner   |
| "What's the status of our work?" | Planner   |
| "Execute AUTH-001"               | Planner   |
| "Clean up after the auth module" | Librarian |
| "Are our docs consistent?"       | Librarian |
| "Archive completed specs"        | Librarian |

## Agents vs Skill

APS includes both **agents** (active dispatch) and a **skill** (passive
guidance):

- **Skill** (`aps-planning/SKILL.md`) — teaches the agent APS conventions.
  Always active. Provides behavioral nudges (plan before building, update
  specs as you work). Lightweight, no model cost.
- **Agents** (`aps-planner`, `aps-librarian`) — perform specific APS tasks
  when dispatched. Use tool calls and reasoning. Consume model tokens.

Use the skill for day-to-day guidance. Use agents when you need active help
with planning or cleanup.

## Installation

### Claude Code

**Install:**

```bash
mkdir -p .claude/agents
cp scaffold/agents/claude-code/aps-planner.md .claude/agents/
cp scaffold/agents/claude-code/aps-librarian.md .claude/agents/
```

Or if you installed APS via the scaffold scripts, agents are available in
`scaffold/agents/claude-code/` within the APS repository.

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
cp scaffold/agents/opencode/aps-librarian.md .opencode/agents/
```

**Usage:**

Agents are configured as subagents (`mode: subagent`). Invoke via `@mention`:

```
@aps-planner what's the next ready work item?
@aps-librarian archive completed modules
```

Switch to an agent as a primary with Tab, or invoke as subagent with
`@mention`. The Planner uses `anthropic/claude-opus-4-20250514`; edit the
`model` field in the frontmatter to change.

### Codex

**Install:**

```bash
mkdir -p .codex/agents
cp scaffold/agents/codex/aps-planner.toml .codex/agents/
cp scaffold/agents/codex/aps-librarian.toml .codex/agents/
```

Then merge `scaffold/agents/codex/codex-config-snippet.toml` into your
`.codex/config.toml`:

```toml
[agents.aps-planner]
model = "o4-mini"
config_file = ".codex/agents/aps-planner.toml"

[agents.aps-librarian]
model = "o4-mini"
config_file = ".codex/agents/aps-librarian.toml"
```

**Usage:**

Spawn agent threads with the `/agent` command:

```
/agent spawn aps-planner
> Plan the user authentication module

/agent spawn aps-librarian
> Audit the repo for orphaned files
```

Agent threads run concurrently and can be managed with `/agent route` and
`/agent close`. Codex uses `o4-mini` by default; change the `model` field in
`.codex/config.toml` if needed.

### Gemini

**Install:**

```bash
mkdir -p .gemini/skills
cp -r scaffold/agents/gemini/aps-planner .gemini/skills/
cp -r scaffold/agents/gemini/aps-librarian .gemini/skills/
gemini skills link . --scope workspace
```

**Important:** Gemini skills are not auto-discovered — the `gemini skills link`
step is required. Without it, the copied files won't be available.

**Usage:**

Gemini has no agent mechanism — the planner and librarian are skills, not
agents. They activate when you ask about planning or repo hygiene:

```
Plan the authentication module using APS
Scan the repo for broken cross-references
```

The skill provides guidance but doesn't have the same dispatch model as
agents in other tools. For active orchestration, consider using Claude Code
or Codex.

## Model Cost

- **Planner** uses Opus (most capable, higher cost) because planning requires
  deep reasoning about architecture, dependencies, and trade-offs.
- **Librarian** uses Sonnet (fast, lower cost) because repo hygiene is
  pattern-matching and file organization — less reasoning-intensive.

You can change the model in each agent's frontmatter if you prefer different
cost/capability trade-offs.

## Building Agent Variants

The build script generates tool-specific agents from shared core prompts:

```bash
bash scaffold/agents/build.sh
```

This regenerates all tool variants (Claude Code, Copilot, OpenCode, Codex)
from `scaffold/agents/core/`. Gemini skills are handwritten since the SKILL.md
format differs structurally — the build script verifies they exist.
