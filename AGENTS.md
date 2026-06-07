# AGENTS.md — Anvil-Plan-Spec (APS) Collaboration Rules

This repo defines the Anvil Plan Spec format — templates, prompts, and examples
for planning and task authorisation in AI-assisted development.

## What this repo is

APS is a **specification format**, not executable code. It contains:

- Templates for plans, modules, work items, action plans, and **design documents**
- Prompts for AI agents
- Examples and documentation
- A CLI for validation (linting)

## Execution authority

- **Specs describe intent.**
- **Tasks authorise execution.**
- If there is no task, do not implement changes unless explicitly instructed.

## Rules for AI contributing to this repo

When modifying templates, prompts, or documentation:

- **Keep templates minimal** — avoid over-prescription
- **Maintain consistency** — field names, structure, and terminology
- **Update examples** when template structure changes
- **Run markdownlint** before committing (`npx markdownlint-cli "**/*.md"`)
  - CI automatically runs markdown linting on all PRs

## Keeping the plans honest

This repo plans itself with APS ([plans/index.aps.md](plans/index.aps.md)).
Plan updates are part of the change, not an afterthought — include them in
the same commit/PR as the work they describe.

### When a plan update is required

Update the matching plan whenever a change affects **templates, prompts,
examples, installer behaviour, or validation behaviour** (the dogfood
module's scope). Concretely:

- Executing a work item → its status, and `Results:` for non-trivial items
- Discovering new work → add a `Draft` work item to the owning module
- Making a judgment call → record it in the module's `## Decisions`
- Hitting a bug or open question → log it in
  [plans/issues.md](plans/issues.md) (`ISS-NNN` / `Q-NNN`)
- Finishing a module's last item → mark the module Complete, update the
  index table, roll the task table into `plans/completed.aps.md`

### How to mark work item status

Prefer the CLI — it enforces the state machine and stamps dates:

```bash
./bin/aps next                      # what's ready
./bin/aps start DOGFOOD-003         # Ready → In Progress
./bin/aps complete DOGFOOD-003 --learning "optional insight"
```

Hand-editing `- **Status:**` is fine too (Draft → Ready → In Progress →
Complete; terminal states carry a date, e.g. `Complete: 2026-06-08`).
Module status in the metadata table is always hand-edited.

### What validation to run

```bash
./bin/aps lint plans                # plan structure (errors fail CI-style)
./test/run.sh                       # full CLI test suite
npx markdownlint-cli "**/*.md"      # markdown style (CI-enforced)
./bin/aps audit --no-run            # optional: status-vs-reality drift
```

`aps lint plans` must pass with no errors before committing plan changes;
keep it warning-free where practical — warnings are drift signals.

## Execution layer

- **Work items** define outcomes (what to achieve)
- **Actions** define what to do (not how)
- Action plans live in `execution/[WORKITEM-ID].actions.md`
- Each action has a checkpoint (observable completion state)

See: [docs/ai/prompting/actions.prompt.md](docs/ai/prompting/actions.prompt.md)

## Prompting entry points

Use:

- [docs/ai/prompting/index.prompt.md](docs/ai/prompting/index.prompt.md)
- [docs/ai/prompting/module.prompt.md](docs/ai/prompting/module.prompt.md)
- [docs/ai/prompting/work-item.prompt.md](docs/ai/prompting/work-item.prompt.md)
- [docs/ai/prompting/actions.prompt.md](docs/ai/prompting/actions.prompt.md)

OpenCode/Claude Opus variants:

- [docs/ai/prompting/opencode/](docs/ai/prompting/opencode/)

## Roles (conceptual)

- **Planner**: completes index/module docs, identifies decisions
- **Implementer**: executes one work item at a time
- **Executor**: follows action plans to complete work items, validates checkpoints
- **Reviewer**: flags anti-patterns and boundary issues, suggests alternatives
- **Librarian**: updates ADR links/pattern references and keeps docs consistent
