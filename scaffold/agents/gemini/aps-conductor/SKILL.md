# APS Conductor Skill

> Coordinate APS work item execution through dependency-aware CLI workflows.

## What This Skill Does

This skill turns APS plans into an execution queue. It selects the next safe
work item, starts it with context, dispatches implementation when needed,
validates completion, and records learnings.

## When to Activate

Activate this skill when:

- The user asks what APS work is next
- The user wants to start or complete a work item
- Several APS work items need dependency-aware coordination
- An implementer needs a focused context package
- Completed work needs validation and learning capture

## Core Rules

1. APS markdown is the source of truth.
2. Prefer `aps` CLI commands when available.
3. Fall back to reading `plans/index.aps.md` and module files when needed.
4. Do not execute blocked work.
5. Do not mark work complete without validation evidence.
6. Keep git actions under explicit human control.

## CLI Workflow

Use whichever local command exists:

```bash
aps next [module]
aps graph [module]
aps start WORK-001
aps complete WORK-001 --learning "short insight"
```

For non-default plan roots, pass `--plans DIR` to every CLI command:

```bash
aps next --plans test/fixtures/orchestrate/plans
aps graph auth --plans test/fixtures/orchestrate/plans
aps start AUTH-003 --plans test/fixtures/orchestrate/plans
aps complete AUTH-003 --plans test/fixtures/orchestrate/plans --learning "short insight"
```

If `aps` is not on `PATH`, try `./bin/aps` or `.aps/bin/aps`.

Read `.aps/context/<ID>.md` after `aps start` before dispatching or executing
implementation work.

## Fallback Workflow

If the CLI is unavailable:

1. Read `plans/index.aps.md`.
2. Read active modules in `plans/modules/`.
3. Select the first `Ready` work item with complete dependencies.
   Missing work item status defaults to `Ready`; invalid explicit statuses fail
   closed.
4. Read its full work item and module context.
5. Dispatch or execute only that work item.
6. Run the Validation command.
7. Update status only after validation succeeds.

After completing a work item, run `aps next [module]` again before selecting
more work. Re-query plan state after every status change.

## Output Format

```markdown
## APS Conductor Status

- Next: WORK-001 - Title
- Module: MOD
- Dependencies: complete | blocked by ...
- Context: .aps/context/WORK-001.md | not generated
- Validation: command or method
- Recommended action: start | dispatch | validate | complete | block
```

Lead with blockers when work cannot proceed.
