# Progress Journal Module

| ID   | Owner  | Priority | Status |
| ---- | ------ | -------- | ------ |
| PROG | @aneki | high     | Draft  |

**Last reviewed:** 2026-08-07

## Purpose

Separate high-churn work-item **progress** from cold module **intent** so
standing and high-throughput backlogs can drain without turning a shared module
file (and index count cells) into a merge mutex. Introduce an opt-in
`Standing` module type whose durable status history lives in create-only git
journal shards under `plans/progress/`, with effective status and counts always
computed.

## Background

Default APS keeps `- **Status:**` inside the module file. That works for small
vertical slices. Under multi-item drain and standing intakes (anvil-001 CIB:
~291 items in one module; ~898 items across ~94 modules) every start/complete
and recon pass rewrites the same hot file. Index `Progress` columns accumulate
hand-maintained `N/M` counts and recon diaries that `index-counts` cannot keep
honest.

Related designs already draw useful boundaries without solving progress storage:

- **Conductor** — horizontal coordination across modules, not status isolation
- **TEAM** — transient claims/leases; explicitly rejects tracked claim files as
  the default because of plan churn and merge races
- **COMPOUND** — archives completed history; not a live progress log

The contract is drafted in
[progress journal design](../designs/2026-08-07-progress-journal.design.md).

## In Scope

- `Type: Standing` + `Progress: journal` metadata contract (opt-in)
- Create-only journal shards under `plans/progress/<module-id>/`
- Deterministic fold (last-write-wins effective status; computed counts)
- Standing-aware `next` / status / progress CLI behaviour (no module rewrite
  for pure status flips in journal mode)
- Recon-oriented shard proposal from delivery evidence
- Lint for markers, shard schema, terminal evidence, orphan events, discouraged
  index `N/M` patterns
- Templates and docs for Standing modules and recon ownership
- Bootstrap / migration recipe from in-file Status to journal
- Fixtures at standing-backlog scale (many items, concurrent shard adds)

## Out of Scope

- Mandatory journals for every module (vertical in-file status remains default)
- Per-work-item progress files in v1
- Hosted APS service or non-git progress store
- Replacing TEAM claims, conductor modules, or COMPOUND archives
- Definition-file compaction for 290-item modules (separate COMPOUND-adjacent
  lever; journal solves status races only)
- Auto-merging recon PRs or expanding recon authority beyond plan progress
- Expanding the canonical work-item status machine for coordination views

## Interfaces

**Depends on:**

- ORCH — item resolution, dependency graph, `next` / `start` / `complete` flow
- VAL / lint — plan-tree validation surface for new rules
- SPEC — status vocabulary ownership for post-merge tokens
- MONO — plan-tree-local paths for nested `plans/progress/`
- INTEGRATIONS — export/rollup projections for computed counts
- TEAM — claims stay out of the journal (boundary; soft dependency)
- COMPOUND — archive remains post-completion narrative, not live progress

**Exposes:**

- [Progress journal design](../designs/2026-08-07-progress-journal.design.md)
- Standing + journal module markers and shard schema
- Effective status fold and computed progress views
- Standing-aware orchestration behaviour (recon-owned durable status)
- Lint codes for progress contract (exact IDs at implementation)
- Migration recipe for CIB-shaped standing backlogs

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Design exists and records D-001–D-007 plus open questions
- [x] Work items are defined with observable validation
- [ ] PROG-000 design decisions are approved
- [ ] Design Q-001 through Q-006 resolved or explicitly deferred with defaults
- [ ] ORCH / SPEC / TEAM / COMPOUND boundaries have no duplicated owner

## Work Items

### PROG-000: Ratify the progress journal contract

- **Status:** Draft
- **Intent:** Freeze the Standing + journal contract before changing templates,
  lint, or CLI behaviour.
- **Expected Outcome:** Design status is Accepted (or equivalent); D-001–D-007
  are confirmed; each open question Q-001–Q-006 has an answer or a documented
  v1 default; owning boundaries with ORCH, SPEC, TEAM, COMPOUND, and MONO are
  explicit; this module's Ready Checklist promotion criteria are clear.
- **Validation:** Design metadata Status is no longer Draft; module Decisions
  match the design; no unresolved Q blocks PROG-001 without a written default;
  `aps lint plans` stays clean on the plan-only ratification PR.
- **Design Source:** plans/designs/2026-08-07-progress-journal.design.md,
  plans/modules/progress-journal.aps.md
- **Confidence:** medium
- **Dependencies:** None

### PROG-001: Parse shards and fold effective status

- **Status:** Draft
- **Intent:** Make journal progress machine-readable and deterministic.
- **Expected Outcome:** APS can load `plans/progress/<id>/` shards, validate
  required metadata and Events rows, and fold last-write-wins status per item
  with Draft (or documented default) for defined items never seen in events.
- **Validation:** Fixtures cover ordered multi-shard fold, tie-break by
  filename, compensating events, orphan item warnings, missing evidence on
  terminal rows, and ≥200 synthetic items without incorrect fold; cross-CLI
  parity where status parsing already has parity tests.
- **Non-scope:** CLI UX polish; recon evidence discovery
- **Files:** `cli/src/parser.rs`, `cli/src/` progress fold module (new),
  `lib/` parity paths if required, `test/fixtures/`
- **Confidence:** medium
- **Dependencies:** PROG-000

### PROG-002: Standing-aware status, progress, and orchestration

- **Status:** Draft
- **Intent:** Stop durable status flips from rewriting Standing module files.
- **Expected Outcome:** Journal-mode modules expose effective status via fold;
  `aps progress` (or agreed verb) prints computed counts only; `next` uses fold
  not stale in-file Status; `start` / `complete` follow the ratified Q-001
  default (recon-only, manual shard, or queued) without treating module Status
  as authoritative under `Progress: journal`.
- **Validation:** Fixture module with journal shards: pure status completion
  leaves the module file byte-identical; concurrent create-only shards merge
  cleanly in a scripted git race; default vertical module behaviour unchanged.
- **Confidence:** medium
- **Dependencies:** PROG-001

### PROG-003: Lint the Standing and journal contract

- **Status:** Draft
- **Intent:** Keep Standing markers, shard schema, and index hygiene honest.
- **Expected Outcome:** Lint warns or errors (per ratified severity) on Standing
  without journal (and vice versa), authoritative-looking in-file Status under
  journal mode, malformed shards, terminal events without evidence, orphan
  event IDs, and discouraged index `N/M` Progress cells for journal modules.
- **Validation:** Invalid and valid fixtures in the lint corpus; `aps lint`
  exit behaviour documented; markdownlint still clean on scaffolded examples.
- **Confidence:** medium
- **Dependencies:** PROG-001

### PROG-004: Templates and docs for Standing + recon

- **Status:** Draft
- **Intent:** Make the opt-in contract teachable without forcing it on verticals.
- **Expected Outcome:** Standing (or progress) template/docs describe markers,
  shard layout, recon ownership, index-without-counts, and boundaries vs
  Conductor / TEAM / COMPOUND; team-rollout and workflow docs state when feature
  PRs omit status and when recon lands shards.
- **Validation:** Scaffold or docs links resolve; a reader can create a Standing
  module and first shard from docs alone; minimal vertical template unchanged.
- **Files:** `templates/`, `docs/`, `scaffold/` as needed
- **Confidence:** high
- **Dependencies:** PROG-000

### PROG-005: Bootstrap and migration recipe

- **Status:** Draft
- **Intent:** Move an existing in-file-status standing backlog onto journal
  without rewriting history by hand.
- **Expected Outcome:** Documented (and preferably CLI-assisted) bootstrap emits
  one write-once shard from current in-file Status lines; dual-read or cutover
  follows Q-003; index Progress/`N/M` cells are dropped for the migrated module;
  recipe is proven on a CIB-shaped fixture (not necessarily live anvil-001).
- **Validation:** Bootstrap fixture round-trips: fold after bootstrap matches
  prior in-file statuses for all items; re-running bootstrap does not mutate
  existing shards (new compensating/bootstrap policy documented).
- **Confidence:** medium
- **Dependencies:** PROG-001, PROG-002, PROG-004

### PROG-006: Prove the standing-drain journey

- **Status:** Draft
- **Intent:** Validate the whole contract under parallel drain, not only unit
  fixtures.
- **Expected Outcome:** Worked example or harness models many completions via
  recon shards, concurrent recon branches, feature PRs that omit module status
  edits, computed counts, and an untouched default vertical module path.
- **Validation:** Journey runs from documented commands; lint clean; two recon
  branches adding distinct shards merge without conflict; no false requirement
  to edit index counts; design success criteria checklist is satisfied or
  explicitly waived with rationale.
- **Confidence:** medium
- **Dependencies:** PROG-002, PROG-003, PROG-004, PROG-005

## Execution Strategy

1. PROG-000 ratifies the design and freezes v1 defaults for open questions.
2. PROG-001 lands parse + fold as the pure core.
3. PROG-002 and PROG-003 attach orchestration and lint to that core.
4. PROG-004 teaches the opt-in path; PROG-005 migrates existing standing shape.
5. PROG-006 is the release gate for the progress-journal capability.

## Decisions

- **D-001:** Create-only shards under `plans/progress/<module-id>/` — _proposed
  in design; confirm at PROG-000._
- **D-002:** Draft/Ready intent-side; journal owns In Progress and terminal /
  post-merge tokens — _proposed in design; confirm at PROG-000._
- **D-003:** Recon owns durable progress commits for Standing+journal —
  _proposed in design; confirm at PROG-000._
- **D-004:** Index drops stored counts; counts always computed — _proposed in
  design; confirm at PROG-000._
- **D-005:** Opt-in only; vertical in-file status remains default — _proposed
  in design; confirm at PROG-000._
- **D-006:** Shards immutable; corrections are new events — _proposed in
  design; confirm at PROG-000._
- **D-007:** Free-form module titles; tooling keys off Type/Progress — _proposed
  in design; confirm at PROG-000._

## Open Questions

See design Q-001–Q-006 (CLI verbs and complete behaviour; post-merge token
plane; bootstrap dual-read; nested plans; TEAM claim events; shard immutability
CI). Track answers in the design and mirror them here at PROG-000.

## Notes

- Pilot consumer after APS lands the contract: anvil-001
  `continuous-improvement-backlog` (not in this repo's execution scope).
- PROG-000 is planning/design ratification only. PROG-001+ remain unauthorised
  until PROG-000 completes.
- This module is a **vertical feature module** that _introduces_ `Type:
  Standing`. It is not itself a Standing journal module.
