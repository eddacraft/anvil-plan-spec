# Create Tasks from APS Module

This variant defers to the [generic module prompt](../module.prompt.md); only Claude Code-specific differences follow.

Use this prompt to generate Claude Code Tasks from an APS module spec.

---

## Prompt

```
Read the APS module at: {MODULE_PATH}

For each work item with canonical `- **Status:** Ready`:

1. Create a Task with format: "{ID}: {title}"
   - Example: "AUTH-001: User registration flow"

2. Set blockedBy from the Dependencies field
   - If Dependencies says "AUTH-001", the task is blocked by AUTH-001

3. Include in task description:
   - Intent (one line)
   - Validation command
   - Key files (if listed)

After creating all tasks, show me:
1. The task list with dependencies
2. Wave breakdown (which tasks can run in parallel)
3. Recommended agent assignment if tasks span multiple domains
```

---

## Example Output

Given a module with work items AUTH-001, AUTH-002 (depends on 001), AUTH-003 (depends on 002):

```
Tasks created:
- Task #1: AUTH-001: User registration flow
- Task #2: AUTH-002: Email verification (blockedBy: #1)
- Task #3: AUTH-003: Password reset (blockedBy: #2)

Wave Breakdown:
- Wave 1: #1 (AUTH-001) - no dependencies
- Wave 2: #2 (AUTH-002) - after Wave 1
- Wave 3: #3 (AUTH-003) - after Wave 2

Agent Assignment:
- Single agent recommended (all AUTH domain)
- Execute sequentially: #1 → #2 → #3
```

---

## Variations

### Multiple Modules

```
Read APS modules at:
- plans/modules/01-core.aps.md
- plans/modules/02-auth.aps.md
- plans/modules/03-ui.aps.md

Create Tasks from all Ready work items. Show cross-module dependencies
and recommend agent assignment to minimize file conflicts.
```

### Action Plan Expansion

```
Read the APS module at: {MODULE_PATH}
Also read the action plan at: plans/execution/{WORK_ITEM_ID}.actions.md

Create a parent Task for the work item, then sub-tasks for each action
in the plan. Sub-tasks should be sequential (each blocked by previous).
```
