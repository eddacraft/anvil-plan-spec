<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!--
Conductor (crosscutting) module template.

Use this INSTEAD of module.template.md when the concern spans, coordinates, or
sequences work ACROSS modules rather than owning a single vertical domain.
Examples: release cuts, security audits, performance budgets, migration waves,
observability rollouts. See docs/conductor-modules.md for when to reach for
this type vs a vertical module.

What makes a module a conductor:
- `Type: Conductor` in the metadata table (vertical modules omit Type).
- It coordinates other modules; owning its own work items is OPTIONAL — it may
  reference work item IDs that live in other module files.
- Its status means "concern addressed in this slice", not "feature complete".
- Lifecycle may be recurring (perf budget, security posture) rather than a
  one-shot Draft → Ready → Complete.

ID: Use 2-6 uppercase chars (REL, SEC, PERF, MIGR).
File naming: name.aps.md (e.g. release-planning.aps.md).
-->

# [Conductor Module Title]

| ID  | Type      | Owner     | Priority | Status |
| --- | --------- | --------- | -------- | ------ |
| REL | Conductor | @username | medium   | Draft  |

<!--
Keep ID as the first column. `Type: Conductor` marks this as a conductor module
so tooling treats cross-module references as legitimate. `aps lint` validates
the IDs in the Coordinated Modules / Cross-Module Work Items sections against
the plan tree and flags typos (W002); it does not warn on valid cross-file refs.
Status values: Draft / Ready / In Progress / Complete / Blocked, plus
**Recurring** for conductors that never naturally complete (ongoing budgets,
security posture).
-->

## Purpose

[Why this crosscutting concern exists and why it doesn't fit a single vertical
module — one paragraph max. Name the concern it conducts: release, audit,
budget, migration.]

## In Scope

- [What this conductor coordinates across modules]

## Out of Scope _(optional)_

- [What belongs to the vertical modules themselves, not the conductor]

## Coordinated Modules

<!--
The modules this conductor spans. This is the "players" list — the conductor is
the score that sequences them. Reference each by its module file; note what the
conductor needs from it (a completed work item, a gate, a dependency).
-->

| Module                         | Role in this concern        | Status   |
| ------------------------------ | --------------------------- | -------- |
| [auth](./modules/auth.aps.md)  | [what it contributes]       | Complete |
| [api](./modules/api.aps.md)    | [what it contributes]       | In Progress |

## Cross-Module Work Items

<!--
Work items from OTHER modules that this conductor depends on or bundles.
Reference them by their existing ID — do not redefine them here. The conductor
tracks their roll-up; the owning module remains the source of truth.
-->

- [AUTH-003](./modules/auth.aps.md) — [why this conductor needs it]
- [API-007](./modules/api.aps.md) — [why this conductor needs it]

## Work Items _(optional — conductors may own their own)_

<!--
A conductor MAY own coordinating work items (e.g. "extract the template",
"add lint rules") in addition to referencing others. Same shape as a vertical
module: Intent, Expected Outcome, Validation.
-->

### REL-001: [Coordinating work item title]

- **Intent:** [What this achieves — one sentence]
- **Expected Outcome:** [Observable/testable result]
- **Validation:** `[test command]`
- **Confidence:** medium
- **Dependencies:** REL-XXX _(optional)_

## Status Roll-up

<!--
The conductor's headline: how much of the coordinated concern is done. Express
it as a count across modules and a one-line readout. Update as referenced work
items land. This is what makes a conductor scannable at a glance.
-->

- **Concern:** [release v0.3.0 / Q3 security audit / API latency budget]
- **Progress:** [N/M] coordinated work items Complete
- **Readout:** [one line — what's shipped, what's left, blocking item if any]

## Decisions _(optional)_

- **D-001:** [Decision] — [rationale]

## Notes _(optional)_

- [Lifecycle note: is this conductor one-shot or Recurring?]
- [Trial/source reference if extracted from a real instance]
