# CLI Command-Surface Redesign Map — CLI Module

| Field   | Value                                              |
| ------- | -------------------------------------------------- |
| Date    | 2026-07-23                                         |
| Status  | Approved                                            |
| Modules | [cli-redesign](../modules/cli-redesign.aps.md)     |
| Scope   | CLI-001 (anchor); informs CLI-002, CLI-003, CLI-006 |

> This is the CLI-001 deliverable: a deliberate map of the whole `aps` command
> surface — every verb, its noun, its help text, and its grouping — read for
> two audiences at once (solo and team) **before** any behaviour changes. No
> rename lands without this map approved (module Ready checklist).

## Problem

The `aps` surface grew feature-by-feature — `init`, `setup`, `update`,
`migrate`, `upgrade`, `lint`, `next`, `start`, `complete`, `graph`, `rollup`,
`audit`, `export`, `doctor` — with no deliberate pass over how the whole thing
reads. Three problems compound:

1. **A tangled maintenance-verb family.** `update`, `migrate`, and `upgrade`
   overlap in meaning, and — critically — `migrate` names two *different*
   actions in different CLIs while the word `upgrade` carries three meanings.
2. **A profile-blind surface.** `.aps/config.yml` already records
   `profile: solo | team | agent-operator`, but the profile only steers which
   templates `init` scaffolds. At runtime every profile sees the identical
   command set — a solo user sees team-shaped affordances they never use, and a
   team gets no first-class claim / handoff / who-has-what verbs.
3. **Three-way drift.** Under D-039 the Rust, bash, and PowerShell CLIs are
   equal shipping targets, yet the command *set* itself has diverged: PowerShell
   exposes 3 of 13 commands, bash lacks `doctor`, and `migrate` means different
   things in Rust vs bash.

This map inventories the real surface, names every overlap with evidence, and
proposes a coherent redesign with an alias/migration path for every rename.

## Current surface (inventory)

Verified against `cli/src/main.rs` (Rust, canonical), `bin/aps` + `lib/*.sh`
(bash), and `bin/aps.ps1` + `lib/*.psm1` (PowerShell) at `main @ 5c978e5`.

| Command            | Meaning (as-built)                                                            | Rust | bash | pwsh |
| ------------------ | ----------------------------------------------------------------------------- | :--: | :--: | :--: |
| `init`             | Create APS structure (TUI wizard / flags / replay)                            |  ✅  |  ✅  |  ✅  |
| `setup [target]`   | Add optional pieces: `cli`, `init`, `agent`, `hooks`, `upgrade`, `all`, tool  |  ✅  |  ✅  |  —   |
| `update [dir]`     | Reconcile generated footprint (templates, skill, tool files)                  |  ✅  |  ✅  |  ✅  |
| `migrate [dir]`    | **Rust:** move project onto global binary + strip vendored bloat (dry-run)    |  ✅  |  ⚠️  |  —   |
| `migrate [dir]`    | **bash:** convert v1 layout → v2 (`.aps/` consolidation)                       |  —   |  ⚠️  |  —   |
| `upgrade [dir]`    | **bash:** safely remove generated bloat (dry-run by default)                  |  —   |  ✅  |  —   |
| `lint [file\|dir]` | Validate APS documents under `plans/`                                         |  ✅  |  ✅  |  ✅  |
| `next [module]`    | Resolve the next Ready work item                                              |  ✅  |  ✅  |  —   |
| `start <ID>`       | Mark a Ready item In Progress + write context package                         |  ✅  |  ✅  |  —   |
| `complete <ID>`    | Mark an In Progress item Complete                                             |  ✅  |  ✅  |  —   |
| `graph [module]`   | Show work items and dependency arrows                                         |  ✅  |  ✅  |  —   |
| `rollup`           | Markdown roll-up table for a federated (nested) parent                        |  ✅  |  ✅  |  —   |
| `audit [module]`   | Audit plan state against reality (runs Validation)                            |  ✅  |  ✅  |  —   |
| `export`           | Machine-readable JSON snapshot (`aps-export/v1`)                              |  ✅  |  ✅  |  —   |
| `doctor`           | Diagnose migration state (global binary, `cli_version`, leftover CLI)         |  ✅  |  —   |  —   |

Legend: ✅ present · — absent · ⚠️ present but **different meaning** across CLIs.

Totals: **Rust 13**, **bash 13** (different set — has `upgrade`, lacks `doctor`;
`migrate` differs), **PowerShell 3** (`init`, `update`, `lint`).

> PowerShell's minimal surface is a known strategic position, not an accidental
> regression: the native `aps.exe` (PowerShell installer or Scoop) is the
> primary Windows path and carries the **full** surface (`README.md:347-350`,
> `docs/installation.md:72`). The `.psm1` module is a deliberately-partial
> fallback for the subset that cannot execute the binary. It is inventoried here
> so the map is honest, and so CLI-002/CLI-006 can decide the parity bar per
> command rather than assume full three-way lockstep on every verb. See D-CLI-d.

## Findings (overlaps & inconsistencies)

**F1 — `migrate` names two different actions.** Rust `migrate` moves a project
onto the global binary and strips vendored bash-CLI bloat; bash `migrate`
converts a v1 layout to v2. Same word, incompatible semantics — a user reading
one CLI's help is misled about the other.

**F2 — "upgrade" carries three meanings, none of them the expected one.** bash
`upgrade` removes generated bloat (which is what Rust *`migrate`* does);
`setup upgrade` is a third, unrelated sub-target; and the meaning users expect
from `upgrade` — "fetch me a newer `aps`" — is **unclaimed**. The word most
likely to be typed for self-update is spent on bloat removal.

**F3 — `update` vs `migrate` vs `upgrade` are indistinguishable at a glance.**
Three near-synonyms front three different jobs (refresh assets / move onto
binary / remove bloat) with no noun that tells them apart.

**F4 — `doctor` exists only in Rust.** The read-only diagnosis verb — the safest
first command a confused user runs — is missing from bash and PowerShell.

**F5 — the surface is flat and ungrouped.** `aps --help` lists 13 peers with no
sections, so the daily planning loop (`next`/`start`/`complete`) reads at the
same weight as once-per-project maintenance (`init`/`migrate`).

**F6 — the surface does not flex by profile.** `profile:` is recorded and parsed
but never consulted at dispatch; solo and team see an identical surface.

## Design

### Principle: one noun per job, grouped by cadence, flexed by profile

Four maintenance verbs, four distinct nouns, zero collisions:

| Verb        | Owns exactly                                                        | Replaces / absorbs                              |
| ----------- | ------------------------------------------------------------------- | ----------------------------------------------- |
| `update`    | Refresh generated assets (templates, skill, tool files) in place    | (unchanged)                                     |
| `migrate`   | Make a project *current*: move onto the global binary, strip vendored bloat, fix stale paths, and (auto-detected) convert a v1 layout | Rust `migrate` **+** bash `upgrade` **+** bash v1→v2 `migrate` |
| `upgrade`   | **Reserved** for the binary self-update ("get me a newer `aps`")    | claims the word users expect (built in CLI-003) |
| `doctor`    | Read-only diagnosis; prints what `migrate`/`upgrade` *would* do      | promoted from Rust-only to all three CLIs       |

`migrate` becomes the single "bring this project up to date with the current
`aps`" verb — it absorbs bash's bloat-removal `upgrade` (that removal *is* part
of moving onto the binary) and folds v1→v2 conversion into an auto-detected
step. It stays **dry-run by default** with `--apply`, backing up every removed
path (existing Rust behaviour). `doctor` is `migrate` with writes disabled.

### Command groups (help layout)

`aps --help` renders in cadence-ordered sections rather than a flat list:

```
PLAN            the daily loop
  next          resolve the next Ready work item
  start <ID>    begin a Ready item (writes its context package)
  complete <ID> finish an In Progress item
  graph         show dependency arrows
  audit         check plan state against reality
  lint          validate APS documents
  rollup        roll-up table for a federated parent
  export        JSON snapshot (aps-export/v1)

PROJECT         set up & maintain a project
  init          create APS structure
  setup         add optional pieces (cli, hooks, agents, …)
  update        refresh generated assets in place
  migrate       bring this project onto the current binary (dry-run)
  upgrade       update the aps binary itself            [CLI-003]
  doctor        diagnose project & binary health (read-only)

TEAM            multi-human / multi-agent   [shown when profile: team]
  claim <ID>    take ownership of a work item           [reserved, CLI-002/TEAM]
  release <ID>  drop your claim                          [reserved, CLI-002/TEAM]
  handoff <ID>  pass an item to another actor            [reserved, CLI-002/TEAM]
  who           who holds what right now                 [reserved, CLI-002/TEAM]
```

### Profile-aware surfacing (shape only; mechanics are CLI-002 + TEAM)

The map fixes *how the surface flexes*; CLI-002 implements it and the TEAM
module owns the claim/lease mechanics behind the team verbs.

- **solo** (default): `PLAN` + `PROJECT` groups. The `TEAM` group is hidden; team
  verbs still parse but print a one-line "enable with `profile: team`" hint.
- **team**: additionally surfaces the `TEAM` group and actor-aware output (e.g.
  `next`/`graph` annotate who holds an item).
- **agent-operator**: superset surface tuned for unattended/CI runs — same verbs,
  machine-friendly defaults (JSON-first, non-interactive). No hidden commands;
  only defaults differ.

The `TEAM` verbs above are **reserved vocabulary only** in CLI-001 — named here
so the surface reads coherently for a team, pinned with the TEAM module, and
built in CLI-002. This map does not implement claim/lease behaviour.

**TEAM-interface note (owed upstream).** `profile:` is a per-project setting in
`.aps/config.yml`; it means "this repo runs in team mode", not "you belong to
team X". A person working across several teams simply gets each repo's profile —
nothing in the CLI vocabulary blocks that, and the vocabulary choice (top-level
verbs vs an `aps team <sub>` namespace) does not model team membership either
way. What the CLI cannot answer on its own is whether a *single repo* must model
**multiple teams at once** (per-item team scoping, "who on which team"). That is
an actor/claim concern the **TEAM / team-coordination** module owns. CLI-002
must therefore treat "one repo = one team" as an assumption to confirm with TEAM,
not to bake into dispatch. Recorded here so the interface debt is visible rather
than silently inherited.

### Migration & aliases (no breaking changes — module constraint)

Every rename ships with an alias that still works and prints a one-line
deprecation pointer for one minor cycle:

| Old form                         | New form                          | Transition                                        |
| -------------------------------- | --------------------------------- | ------------------------------------------------- |
| bash `aps upgrade` (bloat strip) | `aps migrate`                     | `upgrade` aliases to `migrate`; warns; freed for CLI-003 self-update next cycle |
| bash `aps migrate` (v1→v2)       | `aps migrate` (auto-detects v1)   | same verb; v1 detection folded in; no user change |
| `aps setup upgrade`              | `aps setup refresh`               | old key aliases to new; warns                     |
| `doctor` (Rust-only)             | `doctor` (all three CLIs)         | additive; no alias needed                         |

No verb is renamed for taste; each rename resolves a documented collision (F1–F4)
and carries an alias, per the module's no-breaking-changes constraint.

## Decisions (approved 2026-07-23)

1. **D-CLI-a — `upgrade` reassignment — ACCEPTED.** Free `upgrade` from
   bloat-removal (→ `migrate`) and reserve it for binary self-update in CLI-003.
   Claims the word users expect and de-tangles F2.
2. **D-CLI-b — fold v1→v2 into `migrate` — ACCEPTED.** Retire bash's standalone
   v1→v2 `migrate`; the unified `migrate` auto-detects a v1 layout. v1 is legacy;
   one "make current" verb is simpler.
3. **D-CLI-c — team verb vocabulary — ACCEPTED (top-level, profile-gated).**
   `claim` / `release` / `handoff` / `who` as the reserved team surface (names
   only, pinned with TEAM). Team-neutral top-level verbs read better than an
   `aps team <sub>` namespace for actors who span multiple teams.
4. **D-CLI-d — PowerShell parity bar — ACCEPTED (binary-first), with an open
   sub-question.** PowerShell stays binary-first: the native `aps.exe`
   (PowerShell installer or Scoop) carries the full surface, and the `.psm1`
   module remains partial (init/update/lint) rather than porting every verb, so
   CLI-006 does not over-invest.
   - **Open sub-question (OQ-1):** the subset that genuinely *cannot* execute the
     unsigned binary (locked-down execution policy / code-signing mandates) falls
     back to `.psm1` and currently has **no** planning loop (`next`/`start`/
     `complete`). Decide before CLI-006 whether that subset gets a minimal
     planning port, a signed-binary distribution path, or an explicit
     "binary-required for the loop" support statement. Not blocking CLI-001;
     routed to CLI-003/CLI-006 planning.

## Out of scope

- Implementing profile-aware dispatch (CLI-002) or the claim/lease mechanics
  (TEAM / team-coordination).
- Building the binary self-update behind the reserved `upgrade` (CLI-003).
- Any change to the APS markdown spec, template fields, or status vocabulary.
- Renames beyond those resolving F1–F4; no taste-only renames.

## Validation

- `aps lint plans/` clean (design-doc rules W014/W015/W016 satisfied).
- Linked from [cli-redesign](../modules/cli-redesign.aps.md), satisfying the
  module's Ready-checklist item "command-redesign map (CLI-001) is drafted and
  reviewed".
