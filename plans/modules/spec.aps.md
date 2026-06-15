# Spec Module

| ID   | Owner  | Priority | Status |
| ---- | ------ | -------- | ------ |
| SPEC | @aneki | medium   | Complete |

**Last reviewed:** 2026-06-15

## Purpose

Own the canonical APS format itself — the vocabulary, schema, and versioning
rules that every downstream tool (lint, orchestrate CLI, agents, third-party
parsers) must agree on.

So far the spec has lived implicitly across the templates, the linter, and
`aps-rules.md`. As real-world adopters (notably [anvil-001](https://github.com/eddacraft/anvil-001))
ship their own parsers, divergences in vocabulary start to appear. This module
exists to formalize those decisions in one place so the spec can be referenced,
versioned, and migrated cleanly.

## Background

Survey of the largest production APS deployment (anvil-001, 85 modules, 191
work items) found:

- **Status vocabulary divergence.** anvil-001 canonicalizes on
  `Proposed / Ready / In Progress / Done / Blocked`. APS canonicalizes on
  `Draft / Ready / In Progress / Complete / Blocked`. anvil-001's parser
  treats `Draft → Proposed` and `Complete → Done` as legacy aliases.
- **Schema dispersion.** anvil-001 maintains its own Zod schema
  (`packages/aps/src/types/index.ts`) because no canonical machine-readable
  schema ships with APS.
- **No version stamp.** Spec files have no way to declare which APS revision
  they target. anvil-001 inferred the spec from documentation.

Status-value frequency in anvil-001 (across 70 modules with Status fields):
`Draft 74 / Ready 22 / In Progress 16 / Done 13 / Proposed 11 / Complete 6 /
Blocked 1`. The legacy `Draft` is still dominant because migrations are
incremental — both vocabularies exist side-by-side today.

## In Scope

- Canonical vocabulary for module and work-item Status
- Documenting accepted aliases (if any) and the parser's normalization rules
- Optional `Last reviewed:` metadata convention (77/85 anvil-001 modules use it)
- Versioning strategy for the spec itself (header convention, validator
  behaviour on unknown versions)
- A machine-readable schema artifact (JSON Schema preferred over Zod for
  portability) — generated from the canonical definition

## Out of Scope

- Implementing the schema artifact (that's downstream — VAL would consume it)
- Project-specific extensions (anvil-001's `.anvilrc`, completed/ archive
  layout, etc.) — those live in their respective modules
- Multi-language parser ports (anvil-001's TS parser stays project-local for
  now; we can publish later if demand exists)

## Interfaces

**Depends on:**

- VAL (validation) — lint must honour whatever the spec says
- AGENT (agents) — `aps-rules.md` must teach the canonical vocabulary

**Exposes:**

- Authoritative status vocabulary (consumed by lint, orchestrate, agents,
  templates)
- Schema artifact (future: `schema/aps.schema.json`)
- Migration guidance for downstream parsers

## Decisions

- **D-026:** Status vocabulary — _decided 2026-06-15: **Approach A** — canonical
  APS remains `Draft / Ready / In Progress / Complete / Blocked`. Lint and
  orchestrate accept `Proposed → Draft` and `Done → Complete` as aliases and
  never rewrite the form the author chose. Revisit at v1.0 for a more
  opinionated stance._

- **D-027:** `Last reviewed:` metadata — **proposed.** Make the field
  optional but documented. Lint warns when a module has been in `Ready` or
  `In Progress` for more than N days without an updated `Last reviewed:`
  line. Default N is configurable (start at 60 days).

- **D-028:** Spec version header — **deferred.** Defer until SPEC-001 lands;
  needed before we ever break compatibility.

## Work Items

### SPEC-001: Resolve status vocabulary

- **Intent:** Settle the Draft↔Proposed and Complete↔Done divergence so every
  downstream tool agrees on a single canonical schema.
- **Expected Outcome:** D-026 is resolved (A or B). If A: `lib/rules/` and
  `lib/orchestrate.sh` accept aliases, `docs/usage.md` documents both, and
  `aps-rules.md` enumerates the full set. If B: templates, lint, orchestrate,
  tests, and `aps-rules.md` all use Proposed/Done; `aps migrate --status-vocab`
  ships; the CHANGELOG documents the breaking change.
- **Validation:** Round-trip an anvil-001 module through `aps lint`,
  `aps next`, `aps start`, `aps complete` without status normalization
  surprises. Existing fixtures keep passing.
- **Confidence:** high (once D-026 lands)
- **Dependencies:** D-026
- **Files:** lib/orchestrate.sh, lib/rules/module.sh, lib/rules/workitem.sh,
  scaffold/plans/aps-rules.md, docs/usage.md, test/fixtures/, CHANGELOG.md
- **Status:** Complete: 2026-06-15 — D-026 Approach A. `orch_normalize_status`
  and Rust `normalize_status` map `Proposed→Draft`, `Done→Complete`; docs and
  `scaffold/plans/aps-rules.md` document canonical + alias set; orchestrate
  fixture tests cover Proposed modules and Done dependencies.

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] D-026 resolved (Approach A)
- [ ] D-027 resolved (`Last reviewed:` semantics)
- [x] Work items defined with validation

## Notes

The roadmap previously listed `spec` as a Long Term module (formal versioning,
JSON Schema). It's promoted to Near Term because the vocabulary divergence is
already affecting a production deployment.
