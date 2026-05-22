# Compound Module

| ID       | Owner  | Priority | Status   |
| -------- | ------ | -------- | -------- |
| COMPOUND | @aneki | medium   | Complete |

**Last reviewed:** 2026-05-22

## Purpose

Capture reusable learnings from completed APS work so knowledge compounds
across projects instead of staying buried in one-off plans. Define the
"Learn" phase artifacts that close the compound-engineering loop.

## Background

Survey of [anvil-001](https://github.com/eddacraft/anvil-001) — the largest
production APS deployment — found three patterns that solve the
knowledge-compounding problem and that APS doesn't yet ship:

1. **`plans/completed/`** — a directory of detailed historical implementation
   reports (one per major wave/release). Read-only artifacts that preserve
   the play-by-play.
2. **`plans/completed.aps.md`** — a roll-up index of all shipped work
   grouped by release and module area, with task tables preserved for
   traceability. The "history of what we built" in one place.
3. **`plans/releases/v0.3.0-beta.md`** — narrative release planning docs
   (theme, what ships, success criteria). Richer than `CHANGELOG.md` because
   it tells the story of _why_ a release is being cut.

The orchestration CLI already captures per-work-item learnings inline (via
`aps complete --learning`, per ORCH D-002). COMPOUND turns those inline
learnings into a navigable corpus.

## In Scope

- `templates/solution.template.md` — schema for a solution doc
- `templates/completed-index.template.md` — schema for the completed
  roll-up (grouped by release/module, task tables preserved)
- `templates/release.template.md` — schema for a narrative release doc
- Documentation: when to create a solution doc, when to archive a module,
  how to write a release narrative
- Optional CLI helper: `aps archive <MODULE>` that moves a Done module's
  detailed task tables into `plans/completed/` and updates `completed.aps.md`
- Cross-linking conventions (module → solution, work item → solution)

## Out of Scope

- Hosted knowledge base
- AI model training datasets
- Automatic semantic search across solutions
- Replacing per-module decision records (`plans/decisions/`) — solutions
  document _recurring_ patterns; ADRs document one-off choices

## Interfaces

**Depends on:**

- ORCH (Complete) — `aps complete --learning` is the primary input channel
  into the solution library

**Exposes:**

- `templates/solution.template.md`
- `templates/completed-index.template.md`
- `templates/release.template.md`
- `plans/decisions/` cross-link conventions
- Optional `aps archive <MODULE>` command
- "Learn" phase guidance in `docs/workflow.md`

## Decisions

- **D-001:** Should `aps archive` ship in v0.3? — **deferred.** Land the
  doc-only patterns first (templates + workflow guidance). Add the CLI
  helper in a follow-up if manual archival proves painful.
- **D-002:** `completed.aps.md` location — _decided: at the plan root
  (`plans/completed.aps.md`), parallel to `plans/index.aps.md`_. Matches
  anvil-001's layout and keeps both "what we're doing" and "what we've
  done" at the same level.
- **D-003:** Solutions vs ADRs — _decided: ADRs document the one-off
  decision (`use JWT`); solutions document the recurring pattern
  (`token-refresh under network partitions`). Cross-link liberally._

## Work Items

### COMPOUND-001: Solution library workflow

- **Intent:** Make post-work learning capture repeatable and discoverable
- **Expected Outcome:** `templates/solution.template.md` lands with fields
  for Symptom, Root Cause, Solution, Prevention, Related (work items, PRs,
  similar solutions). `docs/workflow.md` "Learn Phase" section is updated
  to reference the template and explain when to write a solution vs an ADR.
  At least one solution doc exists in this repo's `docs/solutions/` as
  proof of concept.
- **Validation:** `aps lint plans/`, `markdownlint`, and a worked example
  showing how `aps complete --learning "..."` feeds into a solution doc.
- **Confidence:** high
- **Files:** templates/solution.template.md, docs/workflow.md,
  docs/solutions/, plans/decisions/
- **Status:** Complete: 2026-05-15 — solution template already existed;
  Learn Phase docs now reference it and proof-of-concept solution doc added

### COMPOUND-002: Completed-work archive pattern

- **Intent:** Adopt anvil-001's `completed/` + `completed-index` pattern
  so Done modules don't bloat `modules/` and historical work stays
  navigable.
- **Expected Outcome:** `templates/completed-index.template.md` lands with
  release-grouped, module-grouped task-table structure (anvil-001 shape).
  `docs/workflow.md` "Completion and Archival" section documents the
  convention: when a module hits Done, its task table moves into
  `completed.aps.md`, detailed implementation notes (if any) move into
  `completed/<release>-<module>.md`. The module spec itself stays in
  `modules/` as a concise historical record (matches DOGFOOD scope).
- **Validation:** Apply the pattern to one Complete module in this repo
  (e.g., scaffold or validation). `aps next` still resolves correctly
  (archived modules don't trip the parser).
- **Confidence:** medium
- **Dependencies:** COMPOUND-001
- **Files:** templates/completed-index.template.md, scaffold/plans/,
  docs/workflow.md
- **Status:** Complete: 2026-05-22 — template landed; `plans/completed.aps.md`
  seeded from this repo's shipped work (v0.2 + v0.3 modules); workflow.md
  "Completion and Archival" rewritten to reference the pattern.

### COMPOUND-003: Release narrative convention

- **Intent:** Provide a place for the narrative around a release (theme,
  what ships, success criteria) that's richer than `CHANGELOG.md` entries.
- **Expected Outcome:** `templates/release.template.md` ships with sections
  for Release Theme, What Ships (grouped by area), Success Criteria, Risks,
  and a link back to the modules covered. `docs/workflow.md` references
  `plans/releases/` as the standard location. This repo's own v0.3.0 cut
  uses the template as proof.
- **Validation:** `plans/releases/v0.3.0.md` exists and passes lint;
  CHANGELOG entry remains terse and references the release doc.
- **Confidence:** medium
- **Dependencies:** COMPOUND-001
- **Files:** templates/release.template.md, plans/releases/,
  docs/workflow.md, CHANGELOG.md
- **Status:** Complete: 2026-05-22 — release template landed; new
  `docs/workflow.md` "Release Narrative" section explains when/how to use it;
  `plans/releases/v0.3.0.md` authored as the proof-of-concept narrative;
  CHANGELOG v0.3.0 entry now links to the narrative.

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified (ORCH Complete)
- [x] Decisions resolved (D-002, D-003); D-001 explicitly deferred
- [x] Work items defined with validation
- [x] Prior-art surveyed (anvil-001)
