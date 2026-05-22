<!-- APS: See docs/workflow.md → "Completion and Archival" for guidance -->
<!--
Filename: plans/completed.aps.md (parallel to plans/index.aps.md)

Purpose: Historical roll-up of all shipped work. Tasks tables are copied
here when a module hits Done so the index stays focused on active work
without losing the task-by-task record.

When to update:
- A module transitions to Complete → copy its work-item table here under
  the appropriate release and module-area heading.
- A release is cut → add a new top-level section (## v[version]).

What lives in plans/completed/v[release]-[module].md:
- Long-form implementation notes, wave reports, post-mortems.
- Keep this index terse; link to those files when they exist.
-->

# Completed Work Archive

| Field   | Value      |
| ------- | ---------- |
| Scope   | ALL        |
| Status  | Active     |
| Updated | YYYY-MM-DD |

## Purpose

Historical record of all completed task-level work, grouped by release and
module area. Individual module specs in `plans/modules/` (or
`plans/archive/modules/` once archived) remain the authoritative description
of intent and decisions. This file preserves the **task tables** for
reference and traceability after a module has been compacted.

Detailed wave-level implementation notes — when they exist — live in
`plans/completed/v[release]-[module].md` and are linked from the relevant
section below.

---

## v[version] — [release theme]

_Brief one-line summary of what shipped in this release. Link the release
narrative:_ [`plans/releases/v[version].md`](./releases/v[version].md).

### [Module area heading]

<!--
One table per module (or per logical area when a release groups several
small modules). Copy the work-item table from the module spec verbatim —
do NOT rewrite the descriptions; this is an archive.
-->

| Task     | Module | Description     | Status   |
| -------- | ------ | --------------- | -------- |
| AUTH-001 | auth   | Login flow      | Complete |
| AUTH-002 | auth   | Session refresh | Complete |

### [Another module area]

| Task   | Module | Description       | Status   | Priority |
| ------ | ------ | ----------------- | -------- | -------- |
| UI-001 | ui     | Component library | Complete | high     |
| UI-002 | ui     | Theming           | Complete | medium   |

**Implementation notes:** [`plans/completed/v[version]-ui.md`](./completed/v[version]-ui.md)

---

## v[previous-version] — [previous theme]

_Repeat the structure above for each shipped release. Newest release at the
top; older releases below._

### [Module area]

| Task     | Module | Description | Status   |
| -------- | ------ | ----------- | -------- |
| CORE-001 | core   | …           | Complete |

---

## Conventions

- **Status column stays `Complete`.** This file is an archive; anything not
  Complete belongs in `plans/index.aps.md`.
- **Don't rewrite history.** When a module ships, copy its task table as-is.
  If a task was later renamed or scoped down, note it inline rather than
  rewriting the original row.
- **Releases match `plans/releases/v[version].md`.** Use the same version
  string so the narrative and the task roll-up cross-reference cleanly.
- **Module specs stay in `plans/modules/`** by default. Move them into
  `plans/archive/modules/` only when the module is unlikely to be revisited
  and the active `modules/` directory needs to stay readable.
