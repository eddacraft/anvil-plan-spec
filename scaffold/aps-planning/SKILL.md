---
name: aps-planning
description: >-
  APS awareness, truth validation, and reconciliation. Loads plan context,
  validates selected work against current project truth before implementation,
  and reconciles plan state at task boundaries.
---

# APS Planning Skill

## Activation Guard

If `plans/index.aps.md` does not exist in the current project, do nothing.
This skill is inert in projects without APS plans. Do not mention APS, do not
offer to create plans, do not load any context. Silently skip all sections
below.

## Session Start -- Load Context

When a session begins and `plans/index.aps.md` exists, build APS awareness in
working memory. Do this quietly -- do not print output to the user unless they
ask about plan status.

### Step 1: Read the index

Read `plans/index.aps.md`. Extract the Modules table. Identify active modules
-- those whose status is NOT `Done`, `Complete`, `Merged`, `Released`, `Shipped`,
`Released/Shipped`, or `Archived`. Note the plan title and any current-window
notes.

### Step 2: Read active modules

For each active module, read its `.aps.md` file from `plans/modules/`. Extract
all work items with their fields:

- **ID** (e.g., `AUTH-001`)
- **Title**
- **Status** (the native APS CLI currently transitions `Draft`, `Ready`,
  `In Progress`, `Complete`, and `Blocked`; it recognises `Proposed` as `Draft`
  and `Done` as `Complete`. This catalogue additionally follows ADR-0013:
  `Merged` / `Released` / `Shipped` / `Released/Shipped` are post-merge
  lifecycle states — not selectable for implementation; `Merged` is interim
  and advances to `Complete` only on release or ship evidence)
- **Files** (from the `Files:` field, if present)
- **Validation** (from the `Validation:` field)
- **Priority** (if specified)
- **Dependencies** (if specified)

Skip work items with inactive/completed statuses: `Done`, `Complete`, `Merged`,
`Released`, `Shipped`, `Released/Shipped`, or `Archived`.

### Step 3: Build the file-to-item map

From all non-Complete work items that have a `Files:` field, build a mapping of
file paths to work item IDs. This map is used during passive awareness to
recognise when edited files relate to planned work.

Example:

```
src/auth/login.ts -> AUTH-001, AUTH-003
src/db/migrations/ -> DB-002
packages/core/src/policy.ts -> POL-001
```

### Step 4: Check for cached context

Look for `plans/aps-project.md` in the project (the primary, harness-neutral
location per ADR-0008; a per-harness binding may map it to a harness-specific
path). If it exists, read it for additional project-specific APS context
(module relationships, conventions, recent decisions). If missing, note its
absence but do not block on it.

### Step 5: Store APS Context Block

Hold this compact summary in working memory (not displayed to user):

```
## APS Context
Active: [MODULE_ID (X/Y items done), ...]
In-progress: [ITEM-NNN: title, ...] or (none)
File map: [N tracked paths]
Next suggested: [ITEM-NNN (reason)]
```

Where:

- **Active** lists each active module with completion ratio
- **In-progress** lists work items currently being worked on
- **File map** is the count of tracked file paths
- **Next suggested** is the highest-priority Ready item with no unmet
  dependencies, or the most impactful Draft item if nothing is Ready

## APS Truth Validation

Run this mode when `dev-loop` asks for an APS gate, when
`planning-workflow` needs a readiness decision, when the user asks if a plan is
current, or when scope appears stale, ambiguous, or cross-cutting.

Steps:

1. Confirm the user goal maps to exactly one primary APS work item. If not,
   return `needs-plan-update` and hand off to `planning-workflow`.
2. If ownership, scope, behaviour, or architecture is unclear, hand off to
   `planning-workflow`.
3. Confirm the module and work item status allow implementation.
4. Check dependencies and cross-reference callouts.
5. Read referenced files from `Files:` plus directly related tests, schemas,
   docs, ADRs, workflows, and feature flag definitions.
6. Compare expected outcome and validation commands against current project
   truth.
7. Identify drift: already-completed work, stale assumptions, moved files,
   changed APIs, invalid commands, missing dependencies, release-state mismatch,
   documentation authority conflicts, or scope conflicts.

Return this report before branch or code:

```markdown
## APS Truth Validation

- Module:
- Work item:
- Status:
- Project truth checked:
- Drift found:
- Decision: valid | needs-plan-update | blocked
- Required APS updates:
- Implementation notes:
```

Implementation MUST NOT begin from a stale, ambiguous, unauthorised, or blocked
APS item.

## During Work -- Passive Awareness

While the user works on code, maintain quiet awareness of APS relevance
without interrupting their flow.

### Recognise Relevance

When the user edits, reads, or discusses a file that appears in the
file-to-item map:

- Note the matching work item(s) internally
- Do NOT announce the match unprompted
- If the user asks "what item is this for?" or similar, cite the work item ID,
  title, and status
- If the user's changes clearly advance a work item's Expected Outcome, note
  this in working memory for later reconciliation

### Flag Completion Opportunities

At natural pauses -- after a commit, after completing a logical chunk of work,
or when the user asks "what's next?" -- you may surface a brief suggestion if
a work item appears to be complete:

> It looks like AUTH-001 (Add login endpoint) may be done -- the validation
> command is `pnpm vitest run src/auth/__tests__/login.test.ts`. Want me to
> run it and mark it complete?

Rules:

- **Never auto-update** work item status. Always ask first.
- Keep suggestions to one sentence plus the validation command.
- Do not repeat a suggestion the user has already declined or deferred.
- Maximum one suggestion per natural pause.

### Track Unplanned Work

If the user creates or modifies files within an active module's scope but those
files are not tracked by any work item:

- Note the untracked work internally
- At a natural pause, offer to add it as a new work item:

> You've added `src/auth/mfa.ts` which is in the AUTH module scope but isn't
> covered by any work item. Want me to draft a work item for MFA support?

Rules:

- Only offer for files clearly within an active module's scope
- Do not offer for test files, config files, or minor refactors
- One offer per untracked cluster of changes, not per file

## Commit and PR Integration

Read `references/commit-pr-integration.md` when writing a commit or PR for
APS-tracked work — it holds the `APS:` commit-trailer and PR-section rules.
This is passive -- only add references when the file-to-item map produces
matches.

## Session Boundaries -- Reconciliation

Reconciliation syncs the APS plan files with actual project state.

### Triggers

Run reconciliation when any of these occur:

- A commit touches files in the file-to-item map
- A PR is created from a branch with tracked changes
- A branch is completed (merged or closed)
- The user explicitly requests it ("reconcile plan", "update plan status", "plan status")

### Reconciliation Steps

Perform reconciliation inline (foreground). This means:

1. For each recently changed file, check if its work item's Expected Outcome
   and Validation are now satisfied.
2. For work items that appear complete, verify by running the Validation
   command if one exists.
3. Draft status updates -- do NOT write them until the user approves, unless a
   currently active loop skill (`dev-loop` or `land-branch`)
   has already granted explicit authority to reconcile the current work item.
4. Identify any new files that should be added to existing work items' Files fields.
5. Identify any unplanned work that warrants new Draft work items.
6. Check for dependency changes -- are any Blocked items now unblocked?

Read `references/reconciliation-report.md` when producing a reconciliation
report — it holds the report format and the optional background-reconciliation
recipe. After presenting the report, ask if the user wants to apply the
proposed changes.

Exception: when called by `dev-loop` / `land-branch` for the current
ReadyItem after verified PR/merge evidence, apply only that item's status,
`Files:`, and evidence updates under the loop's authority. Broader plan, module,
ADR, dependency, or new-work changes still require a user checkpoint.

## Plan Status Query

When the user asks for plan status ("what's the plan?", "plan status", "what's next?", "show plan"):

1. Run reconciliation inline (see above)
2. Produce the full APS status report
3. Ask if they want to apply proposed changes

When the installed CLI is current for the project, prefer its read-only
surfaces over reimplementing them:

- `aps next` for selection;
- `aps graph` for dependencies;
- `aps audit` for validation-backed reconciliation; and
- `aps doctor` for project/toolchain health.

Use `--strict` when version-pin drift must block the operation.

For authorised canonical transitions, prefer `aps start <ID>` and `aps complete
<ID>`. ADR-0013's post-merge states remain direct, evidence-backed plan
bookkeeping until the CLI ships equivalent merge/release transitions; do not
force them through `aps complete`.

## Plan Creation / Modification

When the user asks to create or modify a plan:

1. If `plans/index.aps.md` does not exist, prefer `aps init` so the current APS
   release writes the project contract and canonical templates. If the CLI is
   unavailable, point the user to the current APS installer; only hand-create
   a minimal `plans/` tree when they explicitly want a tool-free bootstrap.
2. If plans exist, help the user add modules, work items, or update existing entries
3. Always ask before writing — show proposed content first

## What This Skill Does NOT Do

- **Interrupt mid-flow** -- never break the user's concentration with APS info
- **Auto-edit plan files** -- always ask before writing status changes, except
  current-item reconciliation explicitly delegated by an active loop skill
- **Run validation during active work** -- validation runs only at session
  boundaries or on explicit request
- **Slow down the commit/PR workflow** -- reconciliation never gates commits or pushes
- **Activate on projects without `plans/index.aps.md`** -- the activation
  guard ensures complete silence in non-APS projects
