# Agent Assignment for Multi-Agent Execution

This variant defers to the [generic action plan prompt](../actions.prompt.md); only Claude Code-specific differences follow.

Use this prompt to assign Claude Code Tasks to parallel agents.

---

## Prompt

```
I have created Claude Code Tasks for these Ready APS work items:

{PASTE WORK ITEMS WITH FILES/PACKAGES}

Assign these to parallel agents with goals:
1. **Minimize file conflicts** - agents shouldn't edit same files
2. **Respect dependencies** - blocked tasks go to same agent as blocker
3. **Balance workload** - distribute tasks roughly evenly
4. **Domain coherence** - keep related work together

Output an agent assignment table:
| Agent | Focus Area | Tasks (in order) | Key Files |
|-------|------------|------------------|-----------|
```

---

## Example

Input:

```
AUTH-001: User registration
  Files: src/auth/register.ts, src/db/users.ts

AUTH-002: Email verification (deps: AUTH-001)
  Files: src/auth/verify.ts, src/email/

CORE-001: Config parsing
  Files: src/config/, src/utils/env.ts

UI-001: Login page (deps: AUTH-001)
  Files: src/pages/login.tsx, src/components/forms/

UI-002: Dashboard (deps: UI-001)
  Files: src/pages/dashboard.tsx
```

Output:

```
| Agent | Focus Area | Tasks (in order) | Key Files |
|-------|------------|------------------|-----------|
| Agent A | Auth + Email | AUTH-001 → AUTH-002 | src/auth/, src/db/users.ts, src/email/ |
| Agent B | Core | CORE-001 | src/config/, src/utils/ |
| Agent C | UI | UI-001 → UI-002 | src/pages/, src/components/ |

Execution timeline:
- Wave 1: All agents start (A: AUTH-001, B: CORE-001, C: waits)
- Wave 2: A continues (AUTH-002), C starts (UI-001 unblocked by AUTH-001)
- Wave 3: C continues (UI-002)

Potential conflict: None - file domains are isolated
```

---

## Monorepo Variant

For monorepo projects with package tags:

```
Work items have package tags (Packages field). Assign agents by package
to completely isolate file access:

| Agent | Package | Tasks |
|-------|---------|-------|

This ensures zero file conflicts since packages are independent.
```

---

## Shared Task List Setup

After assignment, provide the command to start each agent:

```bash
# Terminal 1 - Agent A (Auth)
CLAUDE_CODE_TASK_LIST_ID=project-sprint1 claude -p "Work on AUTH tasks: AUTH-001, AUTH-002"

# Terminal 2 - Agent B (Core)
CLAUDE_CODE_TASK_LIST_ID=project-sprint1 claude -p "Work on CORE tasks: CORE-001"

# Terminal 3 - Agent C (UI)
CLAUDE_CODE_TASK_LIST_ID=project-sprint1 claude -p "Work on UI tasks: UI-001, UI-002"
```
