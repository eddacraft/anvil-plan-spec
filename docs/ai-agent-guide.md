# AI Agent Implementation Guide

> This guide is for LLMs and AI agents working with APS.
> For human-readable docs, see the [README](../README.md).
>
> If you're an AI assistant being asked to work with APS, read this carefully.
> These instructions guide autonomous implementation without human intervention.

## Quick Decision Tree

```
Is there a plans/ directory?
├─ NO  → Initialize APS using scaffold (see Initialization below)
├─ YES → Does plans/index.aps.md exist?
   ├─ NO  → Create index using index.prompt.md
   ├─ YES → Read index, identify user's request type:
      ├─ Planning request → Generate/update specs (see Planning Workflow)
      ├─ Implementation request → Execute work items (see Execution Workflow)
      └─ Question → Read relevant specs, answer from context
```

## Core Principles for AI

1. **NEVER implement without a work item** — If no work item exists, create one first or ask
2. **ALWAYS read before you write** — Check existing specs and code patterns
3. **Work items are permission** — Status must be "Ready" before execution
4. **Checkpoints over instructions** — Write what should exist, not how to create it
5. **One work item at a time** — Complete and validate before moving to next

## Initialization Workflow

When user asks to "set up APS" or "initialize planning":

1. **Check for plans/ directory**
   - If missing: Create plans/ structure
   - Copy templates from templates/ (or use scaffold)

2. **Create plans/aps-rules.md**
   - Copy from [scaffold/plans/aps-rules.md](../scaffold/plans/aps-rules.md)
   - This becomes the agent's reference guide

3. **Create plans/index.aps.md**
   - Use [docs/ai/prompting/index.prompt.md](ai/prompting/index.prompt.md)
   - Fill with user's project context
   - List initial modules (no work items yet)

4. **Validate structure**
   - Checkpoint: `ls plans/` shows index.aps.md and aps-rules.md
   - Tell user: "APS initialized. Create modules when ready to plan."

## Planning Workflow

When user asks to plan a feature or break down work:

1. **Read context**

   ```bash
   # Must read before planning
   cat plans/index.aps.md              # Understand project scope
   cat plans/aps-rules.md              # Recall APS conventions
   ls plans/modules/                   # See existing modules
   ```

2. **Determine scope**
   - Small feature (1-3 work items)? → Use simple.template.md
   - Medium feature (4-8 work items)? → Use module.template.md
   - Large initiative (multiple modules)? → Update index, create leaf modules

3. **Create/update module spec**
   - Use [docs/ai/prompting/module.prompt.md](ai/prompting/module.prompt.md)
   - Name: `plans/modules/[NN-name].aps.md` (numbered by dependency order)
   - Fill: Problem, Success Criteria, Interfaces, Boundaries
   - **Do NOT write work items yet** — leave empty unless scope is crystal clear

4. **Add work items only when module is Ready**
   - Use [docs/ai/prompting/work-item.prompt.md](ai/prompting/work-item.prompt.md)
   - Each work item needs: Intent, Expected Outcome, Validation command
   - Work item IDs: `[MODULE-PREFIX]-NNN` (e.g., AUTH-001, CORE-001)

5. **Validate plan**
   - Checkpoint: Module file exists in plans/modules/
   - Checkpoint: Work items have validation commands
   - Tell user: "Plan ready. Mark work items as Ready when approved."

## Execution Workflow

When user asks to implement a feature or execute a work item:

1. **Find the work item**

   ```bash
   # Locate the spec
   grep -r "work item ID" plans/modules/
   cat plans/modules/[module-name].aps.md
   ```

2. **Verify execution authority**
   - Prefer `aps start <ID>` — it validates that the item is Ready, that all
     dependencies are Complete, and that the owning module is Ready or In
     Progress. On success it writes the new Status and generates a focused
     context package at `.aps/context/<ID>.md`.
   - If `aps start` rejects the item, do not bypass it by editing the markdown.
     Surface the reason to the user.
   - If APS isn't available, fall back to reading the file: status must be
     "Ready" and every listed dependency must be `Complete`.

3. **Understand the outcome**
   - Read: Intent (what and why)
   - Read: Expected Outcome (observable result)
   - Read: Validation command (how to verify)
   - Explore codebase patterns (use existing conventions)

4. **Execute or create action plan**
   - **Simple work item (< 4 changes)?** → Implement directly
   - **Complex work item (≥ 4 changes)?** → Create action plan first

   If creating action plan:
   - Use [docs/ai/prompting/actions.prompt.md](ai/prompting/actions.prompt.md)
   - Create: `plans/execution/[WORKITEM-ID].actions.md` using the full work item ID (e.g., `plans/execution/AUTH-001.actions.md`)
   - Write actions with observable checkpoints (max 12 words each)
   - **NO implementation detail** — just what should exist
   - Identify independent actions and group into **waves** for parallel execution
   - Assign **Agent** types when different actions need different expertise

5. **Implement step-by-step**
   - If action plan has **waves**: dispatch wave actions in parallel, wait for gate before next wave
   - If no waves: execute one action at a time
   - Validate checkpoint after each action
   - If checkpoint fails: Debug, fix, validate again
   - If blocked: Document blocker, ask user

6. **Validate completion**

   ```bash
   # Run the validation command from work item
   [validation command from spec]
   ```

7. **Mark complete**
   - Run `aps complete <ID>` — it requires the item to be In Progress and stamps
     `Status: Complete: YYYY-MM-DD`.
   - If you discovered something worth carrying forward, capture it inline:
     `aps complete <ID> --learning "..."`. The learning attaches to the work
     item and surfaces in dependency learnings for downstream items.
   - Update action plan checkboxes (if used).
   - Report to user: "Work item [ID] complete. Validation passed."

## File Reading Priority

When starting work, read in this order:

1. **plans/aps-rules.md** — Your reference guide
2. **plans/index.aps.md** — Project overview
3. **plans/modules/[relevant].aps.md** — Specific module
4. **plans/execution/[WORKITEM-ID].actions.md** — Action plan (if exists)
5. **Codebase patterns** — Explore similar implementations

## Common Scenarios

| User Says                          | You Do                                                    |
| ---------------------------------- | --------------------------------------------------------- |
| "Set up planning for this project" | Initialization Workflow → Create index                    |
| "Plan the auth module"             | Planning Workflow → Create module spec, NO work items yet |
| "Break down the auth work"         | Planning Workflow → Add work items to existing module     |
| "Implement AUTH-001"               | Execution Workflow → Verify Ready, execute, validate      |
| "What's the plan for payments?"    | Read plans/modules/_payment_.aps.md, summarize            |
| "Is this project using APS?"       | Check for plans/ dir and aps-rules.md                     |

## Anti-Patterns to Avoid

| NEVER Do This                                | DO This Instead                                   |
| -------------------------------------------- | ------------------------------------------------- |
| Implement without a work item                | Create work item or ask for approval              |
| Write implementation details in action plans | Write observable checkpoints only (12 words max)  |
| Execute work item with status "Proposed"     | Ask user to approve (change status to Ready)      |
| Create work items in every module at once    | Create work items per module as it becomes Ready  |
| Guess validation commands                    | Use language/framework conventions or ask         |
| Skip reading aps-rules.md                    | Always read first — it contains your instructions |

## Prompt Entry Points

Use these prompts to generate APS documents:

| Document Type              | Prompt File                                                               |
| -------------------------- | ------------------------------------------------------------------------- |
| Index (project root)       | [docs/ai/prompting/index.prompt.md](ai/prompting/index.prompt.md)         |
| Module (bounded area)      | [docs/ai/prompting/module.prompt.md](ai/prompting/module.prompt.md)       |
| Work Item (execution unit) | [docs/ai/prompting/work-item.prompt.md](ai/prompting/work-item.prompt.md) |
| Action Plan (actions)      | [docs/ai/prompting/actions.prompt.md](ai/prompting/actions.prompt.md)     |

**Tool-specific variants:**

- [docs/ai/prompting/opencode/](ai/prompting/opencode/) — OpenCode-tuned prompts
- [docs/ai/prompting/claudecode/](ai/prompting/claudecode/) — Claude Code
  Tasks coordination (tasks from modules, wave planning, agent assignment,
  status sync)

**First-class agent definitions** ship for Claude Code, Codex, GitHub Copilot,
and OpenCode; Grok Build auto-discovers the shared `.agents/skills/` payload.
`aps init` installs them for the tools you select. See
[docs/agents.md](agents.md) for installation, invocation, and model defaults
for each port, plus the APS-aware agents (planner, conductor, librarian).

## Self-Check Questions

Before implementing, ask yourself:

- [ ] Have I read plans/aps-rules.md this session?
- [ ] Does a work item exist for this change?
- [ ] Is the work item status "Ready"?
- [ ] Do I understand the Expected Outcome?
- [ ] Do I know the Validation command?
- [ ] Have I explored similar code patterns?
- [ ] Am I writing checkpoints (not implementation steps)?

## When Uncertain

1. **Read more context** — Check aps-rules.md, explore modules/
2. **Ask the user** — "Should I create a work item for this?"
3. **Propose a plan** — "I'll create AUTH-001 with outcome X. Proceed?"

**Remember:** Humans approve, AI executes. When in doubt, ask.
