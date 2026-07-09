# Crosscutting / Conductor Modules

| ID   | Owner  | Priority | Status      |
| ---- | ------ | -------- | ----------- |
| COND | @aneki | medium   | In Progress |

**Last reviewed:** 2026-07-02

## Purpose

Introduce a new module type — **crosscutting / conductor** — for concerns
that don't fit cleanly inside a single domain module because they span,
coordinate, or sequence work _across_ modules.

The classic APS module is a vertical slice (auth, ingestion, billing). It
owns its work items end-to-end. But some concerns are horizontal: release
cuts, security audits, performance budgets, migration waves, observability
rollouts. Forcing them into a vertical module either duplicates work items
across domains or creates a sprawling "misc" module.

A conductor module solves this by explicitly modelling cross-module
coordination as its own first-class artifact.

## Background

The trigger for this module came from two converging patterns:

1. **Release planning** (see `release-planning.aps.md`) — release plans
   reference completed work from many modules. They aren't a vertical slice;
   they're a horizontal cut.
2. **ORCH-005 Conductor agent** (in `orchestrate.aps.md`) — already proposes
   an _agent_ that coordinates work across modules. The current trial
   extends that pattern from "an agent role" to "a module type."
3. **Anvil release coordination** — anvil-001's v0.3.0-beta release plan
   touches RCLI, KERN, RATS, PORT, and others. The plan is the conductor;
   the modules are the players.

### Crosscutting vs Vertical Modules

| Aspect           | Vertical (classic)                  | Crosscutting / Conductor          |
| ---------------- | ----------------------------------- | --------------------------------- |
| Owns work items? | Yes                                 | Optional — may reference others   |
| Boundary         | Domain (auth, billing)              | Concern (release, security audit) |
| Lifecycle        | Draft → Ready → Complete → Archived | Often recurring or rolling        |
| Dependencies     | On other modules                    | On work items across modules      |
| Status meaning   | Feature complete                    | Concern addressed in this slice   |

## In Scope

- Define what a crosscutting / conductor module is — frontmatter, required
  sections, lifecycle differences
- Add a module type marker (`Type: Conductor` in metadata table) so the
  linter and tooling can distinguish them
- Template: `templates/conductor.template.md` modelled on the trial
- Linter rules: conductor modules can reference work items from other
  modules without W003 warnings; index entry shows them in a separate
  section
- Documentation explaining when to reach for conductor vs vertical module
- Reference implementations:
  - `release-planning.aps.md` (this repo) — coordinates release cuts
  - Future candidates: security audit, perf budget, migration wave

## Out of Scope

- Replacing or deprecating vertical modules — both coexist
- Auto-creating conductor modules from work-item patterns
- Conductor-specific agents (those already exist as ORCH-005's Conductor
  agent — scoped separately)
- Full graph database / dependency engine — conductor modules use the same
  markdown referencing as everything else

## Interfaces

**Depends on:**

- VAL (Complete) — needs new linter rules for `Type: Conductor` modules
- ORCH (In Progress) — dependency parser must handle cross-module references
  cleanly, since conductor modules are essentially graphs of cross-module
  references
- REL (Draft) — release planning is the trial use case for the conductor
  pattern

**Exposes:**

- `templates/conductor.template.md` — conductor module template
- `Type: Conductor` metadata convention
- Linter rules suppressing W003 for conductor → cross-module references
- Documentation on when to use this module type

## Decisions

- **D-001:** Naming — _decided 2026-06-19: "Conductor". Shipped as the `Type`
  value, template name, docs, and index section throughout COND-002…005; the
  name held up in use and echoes the ORCH-005 Conductor agent. Alternatives
  (Crosscutting, Aspect, Coordinator) rejected — "Crosscutting" survives only
  as the descriptive subtitle._
- **D-002:** Should conductor modules own work items? — _decided 2026-06-19:
  yes, optionally. The release-planning trial owns REL-001…005 _and_ references
  work items in other modules; the template and linter (W002) support the
  hybrid. Allowing both keeps the type useful._
- **D-003:** Type marker location — _decided 2026-06-18 (COND-002): add a
  `Type` column to the metadata table after `ID`. `Type: Conductor` is opt-in;
  vertical modules omit it (default). ID stays the first column so the parser
  still reads it; Status is found by column name, so position is free._
- **D-004:** Linter handling — _decided 2026-06-18 (COND-003): W003 needed no
  change — it already resolves cross-module dependency IDs against the
  plan-tree index, so legitimate cross-file references never warned. Instead,
  conductor awareness adds **W002**: for `Type: Conductor` modules, the
  `## Coordinated Modules` and `## Cross-Module Work Items` sections are
  validated against the tree and unresolved IDs (typos) are flagged. Cleaner
  than downgrading W003, and it covers the conductor-specific surface that was
  previously unchecked._
- **D-005:** Index treatment — _decided 2026-06-19 (COND-004): a separate
  `### Conductor / Crosscutting` subsection under `## Modules` lists
  `Type: Conductor` modules so the type is visible at a glance. The subsection
  is for conductor _instances_; the `conductor` feature module that introduces
  the type lives with the other feature modules. `aps lint` enforces this with
  W006 (a listed module must carry `Type: Conductor`). Added to
  `templates/index.template.md` as an optional subsection._
- **D-006:** Lifecycle — _decided 2026-06-19 (COND-005): same
  Draft/Ready/In Progress/Complete states, plus a `Recurring` status for
  conductor modules that don't naturally complete (perf budgets, ongoing
  security posture). Documented in `templates/conductor.template.md`,
  `docs/conductor-modules.md`, and `aps-rules.md`._

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] Trial yields enough evidence to confirm D-001 through D-006
- [x] D-001 (naming) confirmed
- [x] D-002 (work item ownership) resolved
- [x] D-003 (type marker) resolved — COND-002
- [x] D-004 (linter behaviour) resolved — COND-003
- [x] D-005 (index treatment) resolved — COND-004
- [x] D-006 (recurring state) resolved — COND-005
- [x] Work items defined with validation

## Work Items

### COND-001: Run release-planning trial as the first conductor module

- **Intent:** Validate the conductor pattern against a real use case before
  generalising
- **Expected Outcome:** `release-planning.aps.md` evolves with `Type:
Conductor` metadata; references work items from other modules without
  warnings; serves as the canonical example
- **Validation:** Release planning module ships, references hold, no
  awkwardness emerges from the type marker
- **Confidence:** high
- **Dependencies:** REL-001
- **Status:** Complete: 2026-06-18 — release-planning shipped (REL-001–004)
  and now carries `Type: Conductor`. All three validation criteria hold:
  module ships; cross-module references (`ORCH-001`, `COMPOUND-003`) resolve
  clean in-tree; the `Type` column is lint-safe (whole-tree lint passes with
  the marker applied). The linter is agnostic to the marker — semantic
  awareness (allowed values, rule relaxation) is COND-003's scope. Trial
  outcome: adopt (resolves D-029).

### COND-002: Define `Type: Conductor` schema and template

- **Intent:** Codify the trial pattern as something other projects can adopt
- **Expected Outcome:** `templates/conductor.template.md` with: metadata
  table including `Type: Conductor`, sections for Coordinated Modules,
  Cross-Module Work Items, Status Roll-up, Decisions, Notes
- **Validation:** Template instantiates cleanly; produces a valid module
  spec when filled in for the release-planning trial
- **Confidence:** medium
- **Dependencies:** COND-001
- **Status:** Complete: 2026-06-18 — shipped `templates/conductor.template.md`
  with a `Type: Conductor` metadata table (ID kept first so the parser still
  reads it; Status discoverable by column name) plus Coordinated Modules,
  Cross-Module Work Items, optional owned Work Items, Status Roll-up,
  Decisions, and Notes sections. Validated by instantiating it as the
  release-planning trial and running `aps lint` — produces a valid module
  with no warnings. Confirms D-003 (Type column in the metadata table).
  Linter W003 suppression is COND-003's scope; index treatment is COND-004's.

### COND-003: Linter support for conductor modules

- **Intent:** Stop warning about legitimate cross-module references in
  conductor modules
- **Expected Outcome:** `lib/rules/module.sh` reads `Type:` from metadata.
  When `Type: Conductor`, W003 is suppressed for cross-file dependency
  references. New W004 surfaces if a conductor references a non-existent
  ID anywhere in the plans tree.
- **Validation:** `aps lint` returns 0 warnings against
  `release-planning.aps.md`; correctly flags a typo'd cross-module ID
- **Confidence:** high
- **Dependencies:** COND-002
- **Status:** Complete: 2026-06-18 — implemented in the canonical Rust linter
  (`cli/src/lint.rs` + `cli/src/parser.rs`), not bash `lib/rules/module.sh`,
  since `aps lint` is the Rust CLI (same call REL-003 made). `PlanFile`
  gained `module_type()` / `is_conductor()` reading the `Type` column by name
  (ID stays first; Status still found by name). W003 already resolved
  cross-module deps via the plan-tree index, so legitimate cross-file
  references never warned — the real gap was the conductor's coordination
  sections, which went unvalidated. New **W002** scans `## Coordinated
  Modules` and `## Cross-Module Work Items` for work-item IDs and flags any
  that resolve nowhere in the tree (the spec's "W004" is already taken;
  W002 was free and sits beside W003). `release-planning.aps.md` now carries
  `Type: Conductor` and lints clean (0 warnings); a typo'd cross-ref
  (`INSTALL-099`) flags W002 while the real `INSTALL-014` does not. Covered
  by three unit tests (`detects_conductor_type_column`,
  `w002_flags_conductor_typo_refs_but_not_valid_ones`,
  `w002_skipped_for_vertical_modules`) and documented in `docs/usage.md`.

### COND-004: Index treatment for conductor modules

- **Intent:** Make conductor modules visible as a distinct category
- **Expected Outcome:** `templates/index.template.md` includes a "Conductor /
  Crosscutting" section. `aps lint` (index rules) is aware of the new
  section. Real `plans/index.aps.md` updated to surface release-planning
  and conductor modules in this section.
- **Validation:** Index renders the new section; linter accepts it; no
  warnings on the upgraded `plans/index.aps.md`
- **Confidence:** medium
- **Dependencies:** COND-002
- **Status:** Complete: 2026-06-19 — `templates/index.template.md` gains an
  optional `### Conductor / Crosscutting` subsection with guidance. The index
  linter is conductor-aware via new **W006**: a module listed under a
  `### Conductor / Crosscutting` subsection whose file is not `Type: Conductor`
  is flagged (inverse of COND-003's module check; reuses a shared
  `link_targets` helper with W019). Dogfood fix: the real index listed the
  **`conductor` feature module** under Conductor / Crosscutting, but that
  module introduces the type rather than being an instance — W006 caught it.
  Moved `conductor` to the v0.4 In Progress group; the section now lists only
  `release-planning` (a real `Type: Conductor` module) and lints clean. Unit
  test `w006_flags_non_conductor_in_conductor_index_section`; documented in
  `docs/usage.md`. Resolves D-005.

### COND-005: Documentation + worked example

- **Intent:** Help users decide between vertical and conductor module types
- **Expected Outcome:** `docs/conductor-modules.md` with: when to use
  conductor vs vertical, worked example using release-planning, the
  recurring vs one-shot lifecycle distinction. Plus a section in
  `aps-rules.md` covering the `Type` field.
- **Validation:** Doc reviewed, example matches the actual release-planning
  module
- **Confidence:** high
- **Dependencies:** COND-001, COND-002
- **Status:** Complete: 2026-06-19 — added `docs/conductor-modules.md`
  (conductor vs vertical decision guidance with a "remove the cross-module
  references — is there anything left?" test, anatomy of the `Type` marker and
  coordination sections, the release-planning worked example, the recurring vs
  one-shot lifecycle distinction, and how W002/W006 help). Added a "Module
  Types: Vertical and Conductor" section to `scaffold/plans/aps-rules.md`
  covering the `Type` field. Cross-linked from the README docs list. Worked
  example matches the real module: release-planning owns REL-001…005 _and_
  references cross-module work (`ORCH-001`, `COMPOUND-003`).

### COND-006: Identify additional candidates from existing modules

- **Intent:** Confirm the type generalises beyond release planning
- **Expected Outcome:** Audit existing draft modules (compound,
  integrations, prompts) for crosscutting traits; either re-categorise as
  conductor or confirm vertical. Document the assessment in this module's
  Notes section.
- **Validation:** At least one additional draft module re-categorised, or
  written justification for why none qualify
- **Confidence:** low
- **Dependencies:** COND-005
- **Status:** Complete: 2026-06-19 — audited compound, integrations, prompts
  (plus monorepo, examples, tasks, spec, dogfood). None re-categorised; all
  are vertical. Written justification recorded in Notes below. Outcome: the
  conductor type currently generalises to exactly one instance
  (release-planning), which is acceptable — the type earns its keep on the
  concern it models, not on headcount.

### COND-007: Backport W002 and W006 to the bash + PowerShell linters

- **Intent:** Restore three-way lint lockstep (index D-038). The conductor
  rules W002 and W006 shipped in the Rust linter only (COND-003, COND-004),
  so a conductor typo or a mis-tagged index section passes `./bin/aps lint`
  (bash) and the PowerShell fallback while failing the primary Rust binary —
  a real cross-implementation behaviour split.
- **Expected Outcome:** `lib/rules/*.sh` (bash) and `lib/rules/*.psm1`
  (PowerShell) gain **W002** (a conductor module's `## Coordinated Modules` /
  `## Cross-Module Work Items` sections referencing a work-item ID that
  resolves nowhere in the tree) and **W006** (a module listed under a
  `### Conductor / Crosscutting` index subsection whose file is not
  `Type: Conductor`), matching the Rust semantics, codes, and exit behaviour.
- **Validation:** bash and PowerShell `aps lint` reproduce the Rust results
  on the COND-003/COND-004 fixtures — a typo'd conductor ref warns W002, a
  mis-tagged index entry warns W006, a clean plan is silent — verified against
  a fetched pwsh; string-parity guard added to `test/run.sh`.
- **Confidence:** high
- **Dependencies:** COND-003 (complete), COND-004 (complete)
- **Status:** Ready
- **Notes:** The mirror image of monorepo MONO-007 (which ports MONO-002's
  bash/PS federated-lint rules the other way, into Rust). Surfaced by the
  2026-07-02 bash-vs-Rust rule-set audit: the two linters were non-superset —
  Rust carried W002/W006 but not W020 or child-plan traversal; bash/PS the
  inverse. Small adjacent cleanup: drop the stale `E012` reference in
  `cli/src/audit.rs` — no linter emits E012.

## Execution Strategy

### Wave 1: Trial validation

- COND-001: Run release-planning as the trial conductor

### Wave 2: Codification (depends on Wave 1)

- COND-002: Schema + template
- COND-003: Linter support
- COND-004: Index treatment

### Wave 3: Adoption (depends on Wave 2)

- COND-005: Documentation
- COND-006: Audit existing modules

### Wave 4: Parity (depends on Wave 2)

- COND-007: Backport W002/W006 to bash + PowerShell

## Notes

- This module began as a deliberate trial. The trial concluded in favour of
  adopting the type (D-029): release-planning runs cleanly as a conductor, the
  `Type` marker is lint-safe, and the pattern did not feel forced. All work
  items COND-001…006 are Complete, so the module is now `Complete`.
- The name **Conductor** was chosen over **Crosscutting** because it
  echoes the existing ORCH-005 Conductor agent and reads more concretely.
  Could be revisited in D-001.
- Open question for the trial: does a conductor module benefit from a
  rendered status roll-up (Complete: 12/15 work items across modules) or
  is that just noise? Answer should fall out of REL trial usage.

### COND-006 audit: candidate modules assessed (2026-06-19)

Audited the draft/feature modules for crosscutting traits. Verdicts use the
test "if you removed every cross-module reference, would the module still
have a reason to exist?" — yes ⇒ vertical.

| Module       | Status   | Verdict    | Why                                                                                                   |
| ------------ | -------- | ---------- | ----------------------------------------------------------------------------------------------------- |
| compound     | Complete | Vertical   | Owns the learn-phase templates/archival end-to-end. Ships `release.template.md` but doesn't coordinate release work. |
| integrations | Draft    | Vertical   | Owns JSON export / GitHub sync end-to-end; exposes a data interface, doesn't sequence other modules.   |
| prompts      | Draft    | Vertical   | Owns prompt variants within its own scope; no cross-module references.                                 |
| monorepo     | Draft    | Borderline | Spans structural concerns and references VAL/ORCH/SCAFFOLD, but those are implementation _dependencies_; it owns the federated-trees feature end-to-end. |
| examples     | Draft    | Vertical   | Owns the worked-examples corpus.                                                                       |
| tasks        | Complete | Vertical   | Owns Claude Code task integration end-to-end; depends on AGENT/ORCH but coordinates nothing.           |
| spec         | Complete | Vertical   | Defines shared vocabulary others depend on, but doesn't coordinate their work.                         |
| dogfood      | Complete | Vertical   | Validates this repo's own planning surface; hygiene, not coordination.                                 |

**Conclusion:** no module re-categorised. `monorepo` is the closest call —
its scope is structural — but it _owns_ that concern rather than waving
traffic across autonomous domains, so it stays vertical. The conductor type
generalises to exactly one instance today (release-planning); that is a fine
outcome. Re-audit when a security-audit, perf-budget, or migration-wave
module is proposed — those are the natural next conductors.
