# Release Planning Module

| ID | Owner | Priority | Status |
|----|-------|----------|--------|
| REL | @aneki | medium | Draft |

## Purpose

Add release planning as a first-class APS concern. Provide templates, a
`plans/releases/` location, and lightweight CLI support so projects can plan,
track, and ship releases using the same markdown-first approach as modules
and work items.

The pattern is already in production trial in `EddaCraft/anvil-001`
(`plans/releases/v0.3.0-beta.md`) — this module is about extracting the
working pattern, codifying it as an APS addon, and shipping it via the
scaffold/installer.

## Background

Release planning sits awkwardly between modules and external tooling:

- Modules are scoped to a domain concern (auth, ingestion). Releases cut
  across modules — they are slices of completed work bundled into a versioned
  artifact.
- GitHub releases / CHANGELOG.md are downstream — they record what shipped,
  not what was planned.
- Roadmaps describe direction but not the bundle of work that constitutes a
  specific version.

The anvil-001 trial uses a single markdown file per release with:

- Header: target version, branch, previous release, commit count, date
- "Release Theme" — the narrative
- "What Ships" — table of completed modules and their work items
- Cross-references to module spec files

This produces a useful planning artifact that doubles as draft release notes.
The pattern is generic enough to extract.

## In Scope

- Canonical `plans/releases/` directory layout
- `release.template.md` — release plan template aligned with anvil-001 trial
- Index entry / linking convention so releases reference completed modules
  and work items
- `aps lint` rules for release files (front-matter table, required sections)
- Scaffold integration: `plans/releases/` created on `aps init` (optional via
  prompt)
- Release status flow (Planning → Cut → Shipped → Archived)
- Optional `aps release` subcommand: `new`, `status`, `notes` (generates
  draft release notes by enumerating completed work items since last release)
- Documentation and example based on the anvil-001 v0.3.0-beta plan

## Out of Scope

- Actual release automation (changelog generation, tagging, publishing) —
  those belong in cargo-dist, semantic-release, or project-specific tooling
- Versioning policy (semver vs calver) — APS records what the project
  decides; it doesn't prescribe
- Cross-project release coordination (monorepo release trains) — single
  project for v1
- Backporting / hotfix workflow

## Interfaces

**Depends on:**

- VAL (Complete) — extend linter with release file rules
- ORCH (Ready) — `aps release notes` reuses dependency/work-item parser
- INSTALL (Complete) — installer wires up `plans/releases/` and template

**Exposes:**

- `templates/release.template.md` — release plan template
- `scaffold/plans/releases/.gitkeep` — directory scaffolded on init
- `aps release new <version>` — create new release file from template
- `aps release status [version]` — show release readiness (work items
  Complete vs In Progress vs Draft scoped to release)
- `aps release notes <version>` — emit markdown release notes draft

## Decisions

- **D-001:** Where do release plans live? — *proposed: `plans/releases/`
  with one file per version (`v0.3.0-beta.md`). Matches anvil-001 trial.*
- **D-002:** How do releases reference work items? — *proposed: free-form
  links + tables (anvil-001 trial style). Linter optionally validates that
  referenced IDs exist. Decision pending more usage.*
- **D-003:** Should release files be in the index? — *open. Options:
  (a) separate "Releases" section in `index.aps.md`,
  (b) standalone `plans/releases/index.md`,
  (c) no index, files discovered by glob.*
- **D-004:** CLI support depth — *proposed: defer `aps release` subcommand
  until trial usage stabilizes. Ship template + linter rules first.*

## Ready Checklist

- [ ] Purpose and scope are clear
- [x] Dependencies identified
- [ ] D-001 (location) — confirmed via trial
- [ ] D-002 (work item references) — needs more anvil-001 mileage
- [ ] D-003 (indexing approach) resolved
- [ ] D-004 (CLI scope) resolved
- [ ] Work items defined with validation

## Work Items

### REL-001: Extract release.template.md from anvil-001 trial

- **Intent:** Codify the working release plan pattern as a reusable APS
  template
- **Expected Outcome:** `templates/release.template.md` modelled on
  `EddaCraft/anvil-001/plans/releases/v0.3.0-beta.md`. Sections: header
  table (version, branch, previous, date), Release Theme, What Ships, Risks,
  Rollout, References. Annotated with `<!-- guidance -->` comments.
- **Validation:** Template renders correctly when copied into a fresh
  project; matches anvil-001 trial structure
- **Confidence:** high
- **Dependencies:** —

### REL-002: Add `plans/releases/` to scaffold

- **Intent:** New projects get a place for release plans without manual setup
- **Expected Outcome:** `scaffold/plans/releases/` directory with README
  pointing at the template; `aps init` creates `plans/releases/` (optional
  via wizard prompt for solo profile, default-on for team profile)
- **Validation:** Fresh `aps init` with default flags produces
  `plans/releases/` with README; install wizard prompt gates the creation
- **Confidence:** high
- **Dependencies:** REL-001

### REL-003: Lint rules for release plan files

- **Intent:** Catch malformed release plans before they pollute the planning
  surface
- **Expected Outcome:** `lib/rules/release.sh` checking: file matches
  `plans/releases/v*.md`, has header table with required fields, has Release
  Theme + What Ships sections. Wired into `aps lint`. New error codes
  `R001`–`R004`.
- **Validation:** Linter flags missing sections in a malformed release file;
  passes the anvil-001 v0.3.0-beta.md
- **Confidence:** high
- **Dependencies:** REL-001

### REL-004: Document the release planning workflow

- **Intent:** Users understand when and how to create release plans
- **Expected Outcome:** `docs/release-planning.md` covering: when to start a
  release plan (cut date target), how to enumerate work items, how to handle
  scope changes, status flow, hand-off to release tooling. Includes the
  anvil-001 trial as the worked example.
- **Validation:** Doc reviewed; example matches actual anvil-001 release
- **Confidence:** high
- **Dependencies:** REL-001

### REL-005: `aps release` CLI subcommand (optional, deferred)

- **Intent:** Reduce friction for creating and tracking release plans
- **Expected Outcome:** `aps release new <version>` copies template with
  date/version pre-filled; `aps release status [version]` summarises work
  item completion; `aps release notes <version>` enumerates Complete items
  since last release tag as a markdown draft
- **Validation:** All three commands work end-to-end on anvil-001's plans/
- **Confidence:** medium
- **Dependencies:** REL-001, REL-003, ORCH-001 (parser reuse)
- **Notes:** Deferred until trial usage validates the template and lint rules

## Execution Strategy

### Wave 1: Template + scaffold (parallel)

- REL-001: Extract template
- REL-002: Scaffold integration

### Wave 2: Validation + docs (depends on Wave 1)

- REL-003: Lint rules
- REL-004: Documentation

### Wave 3 (optional): CLI

- REL-005: `aps release` subcommand — deferred pending trial feedback

## Notes

- Trial source: `EddaCraft/anvil-001/plans/releases/v0.3.0-beta.md`. Any
  changes to the template should be validated against this file.
- The MVP is template + scaffold + lint. CLI support is gravy and should
  wait for the trial pattern to settle.
- This module is a candidate for the **Crosscutting / Conductor** module
  type being trialed (see `conductor.aps.md`). Releases naturally cut
  across multiple domain modules.
