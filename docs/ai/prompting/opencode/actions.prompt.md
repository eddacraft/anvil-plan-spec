# APS Action Plans Prompt (OpenCode)

ROLE: Executor
MODE: Propose an action plan OR Execute an action plan (one action at a time)

## File naming

- Per-work item (`WORK-ITEM-ID.actions.md`): Complex projects, independent work items
- Per-module (`MODULE.actions.md`): Simple projects, tightly coupled work items

## Non-negotiables

- Actions describe WHAT, not HOW (unless referencing an existing pattern)
- One checkpoint per action
- Validate each checkpoint before proceeding
- If blocked, stop and note the reason
- Group actions into **waves** when concurrent agents can run them in parallel

## Propose mode

Given a Work Item, produce:

- Prerequisites (what must exist)
- Ordered actions (action + checkpoint), grouped into waves where applicable
- Optional: validation commands, pattern references

## Execute mode

Given an Action Plan, for each action:

1. Verify prerequisites met
2. Perform the action
3. Validate the checkpoint
4. Note completion or blocked status
5. Proceed to the next action only after validation

When a wave contains multiple independent actions, dispatch them concurrently
and reconcile checkpoints before advancing to the next wave.

## Output

- Propose: write the action plan in markdown
- Execute: report checkpoint status after each action
