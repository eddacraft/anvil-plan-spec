# Orchestration Patterns: BMAD, Overseer, and APS

> Research conducted: 2026-02-25
> Status: Complete
> Context: Exploring whether APS should offer native orchestration as an
> optional capability, informed by how BMAD and Overseer approach the problem.

## Executive Summary

BMAD Method and Overseer represent two fundamentally different approaches to
AI-agent orchestration. BMAD uses **prompt engineering** — the LLM itself is
the execution engine, guided by structured files it reads at runtime. Overseer
uses a **programmatic engine** — a Rust binary with SQLite state, exposed via
MCP. APS currently sits closer to BMAD's philosophy (markdown is truth, no
runtime dependencies) but lacks the programmatic affordances that would let
agents self-drive through a plan.

**Recommendation:** Offer an optional ORCHESTRATE module that layers
lightweight CLI tooling on top of existing APS markdown specs — giving agents
programmatic `next`, `start`, `complete` operations without introducing a
database or breaking APS's portable-markdown philosophy.

---

## 1. BMAD Method v6 — Deep Analysis

### What BMAD Actually Is

BMAD is a **prompt engineering framework**, not a software orchestration system.
The entire "orchestration" works by loading files into the LLM context window:

1. **Agent definitions** (YAML → compiled to XML-wrapped markdown) give the LLM
   a persona, menu, and behavioral constraints
2. **`workflow.xml`** is the "core OS" — XML instructions telling the LLM how
   to process YAML workflow configs step-by-step
3. **Handoffs are manual** — the user invokes each slash command; the LLM reads
   the next file and follows instructions
4. **State is `sprint-status.yaml`** — a flat YAML file updated by agents as
   instructed by their workflow steps
5. **IDE integration** generates platform-native command files (slash commands
   for Claude Code, rules for Cursor, etc.) that instruct the LLM to read
   specific files from `_bmad/`

Key repository: <https://github.com/bmad-code-org/BMAD-METHOD>

### BMAD Orchestration Mechanics

**The BMad Master Agent** (`src/core/agents/bmad-master.agent.yaml`) is a
meta-agent that greets the user, lists available tasks/workflows from CSV
manifests, and routes to specialized agents. It does not execute code — it is
a prompt personality loaded into an LLM context window.

**The Workflow Engine** (`src/core/tasks/workflow.xml`) processes every YAML
workflow through a 3-step flow:

1. Load and initialize — resolve config references, system variables, ask user
   for unknowns
2. Process each instruction step — handles `action`, `check`, `ask`, `goto`,
   `invoke-workflow`, `invoke-task`, `invoke-protocol`, `template-output` tags
3. Completion — confirm outputs saved, report status

**Handler dispatch** (not automated agent-to-agent handoffs):

- `workflow="path.yaml"` → load workflow.xml, pass the YAML as config
- `exec="path.md"` → read the markdown file and follow all instructions
- `action` → inline operations (list files, etc.)

**The `/bmad-help` router** (`src/core/tasks/help.md`) reads a CSV catalog and
uses phase/sequence ordering to recommend what the user should do next, checking
which artifacts already exist.

### BMAD State Management

**`sprint-status.yaml`** is the central state file:

```yaml
development_status:
  epic-1: in-progress
  1-1-user-authentication: done
  1-2-account-management: ready-for-dev
  1-3-plant-data-model: backlog
  epic-2: backlog
  2-1-personality-system: backlog
```

Status machines:

- **Epic:** backlog → in-progress → done
- **Story:** backlog → ready-for-dev → in-progress → review → done

Each workflow agent updates this file as part of its instructions. There is no
programmatic enforcement — the LLM follows instructions to update YAML fields.

### BMAD Agent Architecture

Every agent YAML has: `metadata` (id, name, title, icon, module, capabilities),
`persona` (role, identity, communication_style, principles), optional
`critical_actions`, and `menu` items. The compiler (`tools/cli/lib/agent/compiler.js`)
converts YAML to XML-wrapped markdown, injecting activation steps and handler
logic from shared components.

Notable: the Dev agent has strict `critical_actions` enforcing TDD, sequential
task execution, and honest reporting — behavioral constraints enforced purely
through prompting.

### BMAD Context Packaging (Epic Sharding)

Large documents (PRD, Architecture, Epics) can be split into smaller files via
`shard-doc.xml` (splits on `##` headings, generates `index.md`). The
`discover_inputs` protocol handles both formats transparently:

- `FULL_LOAD` — load all files in sharded directory
- `SELECTIVE_LOAD` — load specific shard by template variable
- `INDEX_GUIDED` — load index, analyze structure, intelligently select docs

Story files are "ultimate context engines" — self-contained docs aggregating
context from epics, architecture, previous stories, git history, and web
research so a Dev agent can implement without other documents.

### BMAD v6 Key Changes

1. **Step-file architecture** — each workflow step is a separate `.md` file
   loaded just-in-time (saves tokens)
2. **Direct slash command invocation** — workflows invocable without agent menus
3. **Smart input discovery** — `discover_inputs` protocol with 3 load strategies
4. **Cross-file reference validation** — ~483 references across ~217 files
5. **Module system** — core + extension modules (bmm, tea, bmgd, cis)
6. **20+ IDE/CLI platform support** — Claude Code, Cursor, Gemini CLI, etc.

### BMAD's Key Insight

The LLM itself is the best orchestrator — give it structured prompts, file
conventions, and behavioral constraints, and it will execute a complex agile
workflow. No daemon, no database, no API needed. The "integration" is just
where to put the command files so the IDE discovers them.

---

## 2. Overseer — Deep Analysis

### What Overseer Is

Overseer is a **programmatic task execution engine** — a Rust binary with
SQLite state management, exposed to LLM agents via MCP (Model Context Protocol).

Repository: <https://github.com/rust-syndicate/overseer>

### Overseer Architecture

```
┌─────────────────────────────────────────┐
│            MCP Interface                 │
│  (codemode: single "execute" tool)       │
├─────────────────────────────────────────┤
│         TaskExecutionEngine              │
│  nextReady() → start() → complete()      │
├─────────────────────────────────────────┤
│          PlanRepository                  │
│  plans/milestones/tasks (SQLite)         │
├─────────────────────────────────────────┤
│         GitIntegration                   │
│  branch-per-task, auto-commit            │
└─────────────────────────────────────────┘
```

### Overseer State Management

State lives in SQLite with a strict lifecycle:

```
Plan:     Draft → Active → Completed → Archived
Milestone: Pending → Active → Completed
Task:      Pending → Ready → InProgress → Completed → Verified
```

The engine enforces transitions programmatically. `nextReady()` resolves
dependencies and returns the next task to execute.

### Overseer's MCP Codemode Pattern

Overseer exposes a **single MCP tool** called `execute` that accepts natural
language commands. The LLM sends requests like:

```
"List all ready tasks"
"Start task 5"
"Complete task 5 with summary: implemented auth flow"
```

The server parses intent and routes to the appropriate operation. This
"codemode" pattern reduces the tool surface area while keeping natural language
flexibility.

### Overseer Context System

Each task gets **progressive context**: its own description, parent milestone
context, plan-level context, and relevant milestone learnings. Learnings are
captured at task completion and propagated to related milestones.

### Overseer VCS Integration

Branch-per-task with auto-commit:

- Starting a task creates `task/plan-<id>/task-<id>` branch
- Completing a task commits and optionally merges
- Clean separation of work streams

### Overseer's Key Insight

Programmatic `nextReady()` and state enforcement catch mistakes the LLM won't.
When agents can call `next()` to self-drive through a dependency graph, they
need less human intervention between steps.

---

## 3. Comparison Matrix

| Dimension             | BMAD                        | Overseer                         | APS (current)                 |
| --------------------- | --------------------------- | -------------------------------- | ----------------------------- |
| **Engine**            | LLM reads files             | Rust binary + SQLite             | LLM reads markdown specs      |
| **State store**       | `sprint-status.yaml`        | SQLite database                  | Markdown Status fields        |
| **State enforcement** | Prompt instructions         | Programmatic                     | None (human reviews)          |
| **Dispatch**          | User picks slash command    | Agent calls `nextReady()`        | Human-directed                |
| **Dependencies**      | Implicit (phase ordering)   | Explicit (DAG in SQLite)         | Explicit (Dependencies field) |
| **Context**           | Epic sharding → story files | Progressive (parent + learnings) | Re-read spec (5-Op Rule)      |
| **VCS**               | None                        | Branch-per-task                  | None                          |
| **Portability**       | Any LLM that reads files    | MCP-capable tools only           | Any tool (pure markdown)      |
| **Runtime deps**      | None                        | Rust binary + SQLite             | None                          |
| **Learning**          | Previous story intelligence | Milestone learnings              | Compound (solution docs)      |
| **Platform support**  | 20+ IDEs via command files  | MCP clients                      | Any tool via AGENTS.md/skills |

---

## 4. The Gap APS Could Fill

### What BMAD gets right and wrong

BMAD nails the philosophy: no runtime dependencies, files are truth, any LLM
can participate. But it has no programmatic safety net. When the Dev agent is
told "pick the next story from sprint-status.yaml," it reads a YAML file and
makes a judgment call. If it misreads a dependency, skips a blocked story, or
marks something done that isn't — nothing catches it. The entire workflow
depends on prompt quality and the LLM paying attention. At scale (19+ agents,
50+ workflows), this is a real fragility. BMAD compensates with extremely
detailed prompts and `critical_actions` constraints, but it's guardrails on
guardrails with no ground truth check.

### What Overseer gets right and wrong

Overseer nails the programmatic layer: `nextReady()` resolves a real dependency
graph, state transitions are enforced, learnings propagate. But it introduces
a database that becomes the real source of truth, with markdown as a lossy
export. If the SQLite state and the plan description drift, you have two
sources of truth and no clear winner. It also requires a Rust binary and MCP —
not portable, not inspectable by humans, not versionable in git. The database
solves the "LLM makes mistakes" problem but creates a "humans can't see the
state" problem.

### Where APS sits today

APS has the specs right — modules, work items, dependencies, status fields,
action plans. All in portable markdown, all git-versioned. But there's a
practical gap when an agent is actually executing:

1. **"What's next?" requires scanning.** An agent starting a session has to
   read every module, find items where status=Ready, check each item's
   dependencies against other items' statuses, and pick one. That's 3-5 file
   reads and a mental join across them. LLMs do this unreliably — they miss
   items, misread statuses, or pick something whose dependency isn't actually
   complete yet.

2. **State transitions aren't enforced.** Nothing prevents an agent from
   marking AUTH-003 as Complete when AUTH-002 (its dependency) is still Draft.
   Nothing prevents skipping "In Progress" and going straight to Complete.
   APS describes valid states in aps-rules.md, but the LLM has to remember
   and follow the rules every time.

3. **Context is scattered.** When starting AUTH-003, an agent needs: the work
   item intent, the module scope, relevant decisions, what was learned from
   AUTH-001 and AUTH-002, and the file paths involved. Today the agent has to
   manually assemble this from multiple files. BMAD solved this with
   self-contained "story files." Overseer solved it with progressive context
   injection. APS has no equivalent.

4. **Learnings don't compound.** When an agent finishes a work item and
   discovers something useful ("the token library requires explicit algorithm
   whitelisting"), there's nowhere structured to capture that insight so the
   next work item benefits. The Compound module proposes solution docs, but
   that's for cross-project knowledge — there's no in-plan learning loop.

### The gap in one sentence

**APS has the data for orchestration but no programmatic interface to it.** The
dependency graph exists in markdown. The state machine exists in aps-rules.md.
The context exists across module files. But there's no `nextReady()` to query
the graph, no `start()` to enforce transitions, no `context()` to assemble
a briefing, and no `learn()` to capture insights.

Neither BMAD nor Overseer fills this gap well. BMAD doesn't have the
programmatic layer at all. Overseer has it but abandons markdown as truth.

### Recommendation: CLI on markdown

The synthesis is to build a thin CLI that treats markdown as a queryable
database. The parser already exists (VAL module). The data model already exists
(APS spec format). The gap is a handful of commands that parse the specs and
return structured answers.

```
┌──────────────────────┐
│  APS Markdown Specs   │  Source of truth (always, git-versioned)
└──────────┬───────────┘
           │ parse + write back
     ┌─────┼─────────────────┐
     │     │                 │
┌────▼───┐ ┌──▼────┐  ┌──────▼───┐
│Conductor│ │aps CLI│  │MCP Server│
│ Agent   │ │(shell)│  │(optional)│
│(prompt) │ │       │  │          │
└─────────┘ └───────┘  └──────────┘
     │          │            │
     └──────────┼────────────┘
                │
     ┌──────────▼───────────┐
     │  Agent Execution      │
     │  (any tool, any LLM)  │
     └──────────────────────┘
```

What this gives you:

- **`aps next`** — resolves the dependency graph and returns the next ready
  item. No more scanning. No more missed dependencies. An agent or human calls
  one command and gets a definitive answer.
- **`aps start <ID>`** — enforces the Ready → In Progress transition, rejects
  invalid moves, optionally assembles a context package and creates a VCS
  branch. The agent gets everything it needs in one call.
- **`aps complete <ID>`** — enforces In Progress → Complete, rejects skipped
  states, prompts for an optional learning. The state machine is now real, not
  aspirational.
- **`aps learn <ID> "insight"`** — attaches a learning to the work item and
  propagates it to the module. When the next item starts, its context package
  includes prior learnings.
- **`aps graph`** — renders the dependency graph with status. Humans get
  visibility; agents get structure.

What this preserves:

- **Markdown stays canonical.** The CLI reads `.aps.md` files and writes
  changes back to them. There is no database, no separate state store, nothing
  that can drift. `git diff` shows every state change.
- **Fully optional.** The CLI is additive. Agents and humans can always work
  directly with markdown. The CLI just makes it faster and less error-prone.
  Projects that don't want the CLI lose nothing.
- **Tool-agnostic.** The CLI runs in any shell. An optional MCP server wraps
  it for MCP-capable agents. A Conductor agent wraps it in prompts for tools
  that don't have MCP. Every tool gets something.
- **Progressive enhancement.** Phase 1 is `next` + `start` + `complete`
  (the core loop). Phase 2 adds context packaging and graph visualization.
  Phase 3 adds MCP and the Conductor agent. Each phase is independently
  useful.

What this avoids:

- **No database.** Overseer's SQLite is powerful but creates drift risk. We
  skip it entirely — markdown is structured enough to query directly.
- **No daemon.** BMAD's event-driven proposals (agents listening for file
  changes) add complexity APS doesn't need. Explicit commands are simpler.
- **No lock-in.** The CLI doesn't require MCP, doesn't require a specific
  AI tool, doesn't require a specific LLM. It's shell commands that parse
  markdown.

---

## 5. Applicable Patterns from BMAD

These BMAD patterns could enhance APS without changing its philosophy:

### 5.1 Step-File Architecture

BMAD's step-files load one step at a time into the LLM context, saving tokens.
APS action plans already do this (actions are sequential), but the pattern
could be more explicit — each action could reference external files for detailed
context rather than inlining everything.

### 5.2 Context Packaging

BMAD's story files are "ultimate context engines" — self-contained docs that
let an agent start fresh. APS could generate similar context packages when
starting a work item: pulling the item's intent, module scope, parent
decisions, and relevant learnings into a focused briefing.

### 5.3 Smart Input Discovery

BMAD's `discover_inputs` protocol handles both whole docs and sharded docs
with three load strategies. APS could use similar patterns when modules grow
large — sharding by section and loading selectively.

### 5.4 Platform-Native Command Generation

BMAD generates command files for 20+ platforms. APS already does this via
the AGENT module (Claude Code, Codex, Copilot, OpenCode, Gemini). The BMAD
approach confirms this is the right pattern for tool-agnostic distribution.

### 5.5 Behavioral Constraints via Prompting

BMAD's `critical_actions` (Dev agent must follow TDD, never skip tasks, never
lie about tests) are enforced purely through prompting. APS's aps-rules.md
serves the same purpose. The BMAD pattern validates that prompt-based
behavioral constraints work at scale.

---

## 6. Applicable Patterns from Overseer

### 6.1 Dependency Resolution (`nextReady()`)

The most valuable Overseer pattern. APS work items already have `Dependencies`
fields — a CLI could parse these and return the next item whose dependencies
are all Complete and whose status is Ready. This removes guesswork from agents.

### 6.2 Learning Propagation

Overseer captures learnings at task completion and propagates them to related
milestones. APS could capture learnings in work item metadata and surface them
when starting related items — similar to BMAD's "previous story intelligence."

### 6.3 State Machine Enforcement

Overseer's programmatic state transitions prevent invalid moves (e.g.,
completing a task that was never started). A CLI `aps complete` command could
validate transitions before updating the markdown.

### 6.4 Codemode MCP Pattern

Overseer's single-tool MCP interface (`execute` with natural language) is
elegant. An APS MCP server could expose similar natural-language-routed
operations rather than a large tool surface area.

---

## Sources

- [BMAD Method Repository](https://github.com/bmad-code-org/BMAD-METHOD)
- [BMAD Method Documentation](https://docs.bmad-method.org)
- [BMAD Party Mode](https://docs.bmad-method.org/explanation/party-mode/)
- [Overseer Repository](https://github.com/rust-syndicate/overseer)
