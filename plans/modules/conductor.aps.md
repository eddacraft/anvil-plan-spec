# Crosscutting / Conductor Modules

| ID | Owner | Priority | Status |
|----|-------|----------|--------|
| COND | @aneki | medium | Draft (Trialing) |

## Purpose

Introduce a new module type — **crosscutting / conductor** — for concerns
that don't fit cleanly inside a single domain module because they span,
coordinate, or sequence work *across* modules.

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
   an *agent* that coordinates work across modules. The current trial
   extends that pattern from "an agent role" to "a module type."
3. **Anvil release coordination** — anvil-001's v0.3.0-beta release plan
   touches RCLI, KERN, RATS, PORT, and others. The plan is the conductor;
   the modules are the players.

### Crosscutting vs Vertical Modules

| Aspect | Vertical (classic) | Crosscutting / Conductor |
|--------|-------------------|--------------------------|
| Owns work items? | Yes | Optional — may reference others |
| Boundary | Domain (auth, billing) | Concern (release, security audit) |
| Lifecycle | Draft → Ready → Complete → Archived | Often recurring or rolling |
| Dependencies | On other modules | On work items across modules |
| Status meaning | Feature complete | Concern addressed in this slice |

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
- ORCH (Ready) — dependency parser must handle cross-module references
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

- **D-001:** Naming — *proposed: "Conductor" as the type name. Alternatives
  considered: Crosscutting, Aspect, Coordinator. Conductor connects naturally
  with the existing Conductor *agent* concept (ORCH-005) and is more
  concrete than "Crosscutting".*
- **D-002:** Should conductor modules own work items? — *proposed: yes,
  optionally. A release conductor may have its own work items (REL-001
  through REL-005) plus references to work items in other modules. Allowing
  both keeps the type useful.*
- **D-003:** Type marker location — *proposed: add `Type` column to the
  metadata table. `Type: Conductor` opt-in; vertical modules omit (default).*
- **D-004:** Linter handling — *proposed: when `Type: Conductor` is set,
  W003 (cross-module dependency) downgrades to info or suppresses entirely.
  Conductor modules legitimately reference IDs from other files.*
- **D-005:** Index treatment — *proposed: separate "Conductor / Crosscutting"
  table in `index.aps.md` so the type is visible at a glance.*
- **D-006:** Lifecycle — *proposed: same Draft/Ready/Active/Complete states,
  with an extra "Recurring" state for conductor modules that don't naturally
  complete (perf budgets, ongoing security posture).*

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [ ] Trial yields enough evidence to confirm D-001 through D-006
- [ ] D-001 (naming) confirmed
- [ ] D-002 (work item ownership) resolved
- [ ] D-003 (type marker) resolved
- [ ] D-004 (linter behaviour) resolved
- [ ] D-005 (index treatment) resolved
- [ ] D-006 (recurring state) resolved
- [ ] Work items defined with validation

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

### COND-002: Define `Type: Conductor` schema and template

- **Intent:** Codify the trial pattern as something other projects can adopt
- **Expected Outcome:** `templates/conductor.template.md` with: metadata
  table including `Type: Conductor`, sections for Coordinated Modules,
  Cross-Module Work Items, Status Roll-up, Decisions, Notes
- **Validation:** Template instantiates cleanly; produces a valid module
  spec when filled in for the release-planning trial
- **Confidence:** medium
- **Dependencies:** COND-001

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

## Notes

- The whole module is a deliberate trial. If after running release-planning
  through this lens the pattern feels forced, the right outcome is to
  delete this module and treat releases as a regular module that happens
  to reference others. Status `Draft (Trialing)` reflects that.
- The name **Conductor** was chosen over **Crosscutting** because it
  echoes the existing ORCH-005 Conductor agent and reads more concretely.
  Could be revisited in D-001.
- Open question for the trial: does a conductor module benefit from a
  rendered status roll-up (Complete: 12/15 work items across modules) or
  is that just noise? Answer should fall out of REL trial usage.
