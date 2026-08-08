# Progress Journal and Standing Modules

| Field   | Value |
| ------- | ----- |
| Date    | 2026-08-07 |
| Status  | Draft |
| Modules | [progress-journal](../modules/progress-journal.aps.md) |
| Scope   | Separate high-churn progress from intent; create-only recon journals; drop index counts |
| Related | [team-coordination](./2026-07-19-team-coordination.design.md), [conductor modules](../../docs/conductor-modules.md), [checkout/check-in (superseded)](../../docs/plans/2026-03-10-checkout-checkin.design.md) |

## Problem

APS co-locates three different kinds of data in the same git-tracked markdown:

1. **Intent** — purpose, work-item definitions, validation, decisions (changes rarely)
2. **Progress** — per-item status, completion dates, learnings (changes on every item)
3. **Derived views** — index `N/M` counts, progress prose, roll-up tables (should be computed)

For small vertical modules this is fine. Under autonomous multi-item drain and
standing backlogs it becomes a mutex:

- Every `aps start` / `aps complete` rewrites the shared module file
- Parallel feature PRs and recon PRs conflict on the same status lines
- Index rows accumulate hand-maintained `N/M` counts and recon diaries
- PR volume is inflated by plan-state noise that is not product change

**Production evidence (anvil-001, surveyed 2026-08-06):**

| Metric | Value |
| ------ | ----- |
| Modules | ~94 |
| Work items | ~898 |
| CIB alone | ~291 items |
| Index practice | `Progress` cells with `N/M` plus multi-paragraph recon prose |
| Operating model | Feature PRs often **do not** edit shared standing modules; recon catches up later |

The CIB index row already documents that `index-counts` can maintain only the
leading `N/M` and cannot keep adjacent prose honest. Progress has become a
journal trapped inside the worst possible place: a shared module file and an
index table cell.

Conductor modules solve **cross-module coordination**, not high-churn status
isolation. TEAM coordination separates **claims/leases** (transient) from
intent, and explicitly rejects tracked claim files as a default because of
plan churn and merge races. Neither design owns durable progress history.

## Desired Outcome

Projects can opt high-churn modules into a contract where:

- Intent stays cold in the module file
- Progress is git-tracked, append-oriented, and recon-owned
- The index holds module lifecycle only — **no stored counts**
- Counts and effective status are always computed from the journal fold
- Small vertical modules keep today's in-file status behaviour unchanged
- Module **titles** remain free-form; tooling keys off a `Type` / progress policy

A consumer like anvil-001 CIB can drain dozens of items without serialising
every status flip through one module mutex or inflating the official PR
stream with index-count ticks.

## Constraints

- APS markdown (and related plan-tree files) remain the durable source of truth
- No hosted APS service or database is required
- Git remains the source of truth for progress (not local-only caches)
- Minimal vertical module template gains **no** mandatory new fields
- Opt-in only: default modules continue `Draft → Ready → In Progress → Complete`
  with in-file `- **Status:**` lines
- Canonical work-item status vocabulary is not expanded for coordination
  (TEAM still owns claims; this design owns durable progress events)
- Markdown remains human-reviewable; machine fold must be deterministic
- Design must scale to hundreds of items per standing module without
  exploding file counts (reject per-work-item progress files for v1)

## Design

### 1. Five planes (extends TEAM's four)

| Plane | Canonical information | Storage |
| ----- | --------------------- | ------- |
| **Intent** | Modules, work-item definitions, deps, outcomes, decisions | `plans/modules/*.aps.md` (cold) |
| **Progress** | Durable status transitions with evidence | Create-only journal shards under `plans/progress/` (hot) |
| **Coordination** | Actors, claims, leases, handoffs | TEAM store (transient; not this design) |
| **Delivery** | Branches, PRs, CI, merge evidence | Git / provider systems |
| **Visibility** | Effective status, counts, next queue | Projection only (`aps status`, `aps next`, rollup, export) |

Progress is durable and versioned in git. It is not coordination (claims can
expire) and not intent (definitions do not change when status flips).

Visibility never becomes a second source of truth: index cells and dashboards
must not store authoritative `N/M` that diverge from the fold.

### 2. Module types and progress policy

| Type (metadata) | Role | Progress default |
| --------------- | ---- | ---------------- |
| _(omit)_ | Vertical domain module | In-file `- **Status:**` (today) |
| `Conductor` | Cross-module concern | Unchanged (conductor rules) |
| `Standing` | Never-terminal intake / high-churn backlog | **Journal required** |

Optional alias: `Stream` may mean the same progress rules for finite but
high-throughput waves. Prefer a single `Standing` type plus a free-form title
unless Stream proves necessary.

Marker sketch (Standing):

```markdown
| ID  | Type     | Owner  | Priority | Status    | Progress |
| --- | -------- | ------ | -------- | --------- | -------- |
| CIB | Standing | @aneki | medium   | Recurring | journal  |
```

- `Type: Standing` — lifecycle may use `Recurring` (already allowed for
  conductors that never finish); completing every listed item does **not**
  complete the module
- `Progress: journal` — effective item status comes from the fold, not from
  authoritative in-file Status lines
- User-facing module **name/title** stays arbitrary ("Continuous Improvement
  Backlog", "Ops pen", etc.)

### 3. Create-only journal shards (not one growing file)

A single append-only file still conflicts at EOF when two recon branches both
append. At standing-module cadence that is common.

Layout:

```text
plans/
  modules/continuous-improvement-backlog.aps.md   # intent definitions
  progress/
    cib/                                          # module id / slug (lowercase)
      2026-08-06T142200Z-recon.md                 # write-once shard
      2026-08-06T180515Z-recon.md
      2026-08-07T091100Z-bootstrap.md
```

Rules:

1. Each shard is **write-once**. Never rewrite or amend a published shard.
2. Corrections are **new compensating events** in a new shard.
3. Parallel recon runs create **different filenames** → near-zero merge conflict.
4. Filename prefix is UTC `YYYY-MM-DDTHHMMSSZ` plus a kind token
   (`recon`, `bootstrap`, `manual`, optional short run id).
5. Directory name is the module ID lowercased (or a stable slug declared once).

### 4. Shard schema

Markdown tables for reviewability. Columns are fixed; unknown columns are
ignored by fold, not fatal.

```markdown
# Progress shard

| Field  | Value                |
| ------ | -------------------- |
| Module | CIB                  |
| Kind   | recon                |
| At     | 2026-08-06T14:22:00Z |
| Actor  | recon@ci             |
| Base   | a089dbd              |

## Events

| Item    | To     | Evidence | Note              |
| ------- | ------ | -------- | ----------------- |
| CIB-032 | Merged | #2269    | delivered on main |
| CIB-216 | Merged | #3470    |                   |
```

| Field | Required | Notes |
| ----- | -------- | ----- |
| Module | yes | Must match directory / module ID |
| Kind | yes | `recon` \| `bootstrap` \| `manual` \| … |
| At | yes | RFC3339 or `YYYY-MM-DDTHH:MM:SSZ`; fold sorts by this, then filename |
| Actor | recommended | Human or recon identity (descriptive, not ACL) |
| Base | recommended | Git SHA observed when recon ran |
| Item | yes | Existing work-item ID |
| To | yes | Canonical status token |
| Evidence | required for terminal states | PR `#N`, commit SHA, audit path, or `intake` / `bootstrap` |
| Note | optional | One line; not a substitute for evidence |

**Terminal states** (evidence required): `Complete`, `Merged`, `Released`,
`Shipped`, `Archived` (and project-local post-merge tokens already recognised
by APS). Non-terminal: `In Progress`, `Blocked`, and optional progress-side
`Ready` if a project chooses not to keep Ready as intent (see D-002).

### 5. Fold algorithm (effective status)

Deterministic fold for module `M`:

1. Load all shards under `plans/progress/<m>/`.
2. Sort by shard `At` ascending; break ties by filename ascending.
3. For each event in document order, set `status[item] = To`.
4. **Last write wins** per item ID.
5. Items defined in the module but absent from all events default to **`Draft`**
   (or project default — see D-002 for Ready).
6. Unknown item IDs in events are lint warnings (W-progress-orphan), not fold
   failures — recon may race ahead of a definition PR.

CLI / agent surfaces:

```text
aps status CIB-032           # fold → effective status
aps progress CIB             # computed N/M and breakdown (never stored)
aps next --module CIB        # uses fold + deps, not stale in-file Status
aps reconcile --module CIB   # propose a new shard from delivery evidence
```

Names are illustrative; CLI redesign may place them under existing verbs.

### 6. Intent vs progress field split

**Module file (Standing + journal):**

- Keeps: Purpose, scope, work-item Intent / Expected Outcome / Validation,
  Files, Dependencies, Decisions
- **New item intake** still edits the module (definition is intent)
- In-file `- **Status:**` is either omitted or treated as **non-authoritative**
  when `Progress: journal` is set (`aps lint` warns if present as if authoritative)

**Planning statuses as intent (recommended default):**

| State | Plane | Who writes |
| ----- | ----- | ---------- |
| Draft, Ready | Intent (module) or rare plan PR | Planner / human |
| In Progress, Complete, Merged, Blocked | Progress (journal) | Recon (primary); manual shard exceptional |

This matches anvil-001 practice: definitions and readiness are planned;
landed/merged truth is reconciled after delivery.

### 7. Recon job ownership

Feature / implementation PRs for Standing+journal modules:

- **May omit** plan status edits entirely
- **Must not** rewrite journal shards they did not create
- **May** add a work-item definition only when filing new intent

Recon (human-run or automation):

1. Observe delivery evidence (merged PRs, APS commit trailers, optional audit)
2. Diff against current fold
3. Emit **one new shard** with the batch of `To` transitions
4. Open a recon PR that adds only `plans/progress/...` files (and never
   hand-edited index counts)
5. Does not rewrite the module for pure status flips

Local `aps start` under this policy:

- Preferred: coordination claim only (TEAM), no durable progress event until
  recon — **or**
- Allowed later: a local `manual` shard for In Progress if projects need
  git-visible active work without waiting for recon

v1 recommendation: **recon owns terminal and post-merge transitions**; local
start need not mutate git progress.

### 8. Index contract (drop counts)

Index rows for Standing modules:

```markdown
| Module | ID  | Owner | Status    | Tags     |
| ------ | --- | ----- | --------- | -------- |
| [cib](./modules/continuous-improvement-backlog.aps.md) | CIB | @aneki | Recurring | standing |
```

- **No Progress column**
- **No stored `N/M`**
- Status = module lifecycle only (`Draft` / `In Progress` / `Recurring` / …)
- Optional docs link to `plans/progress/<id>/`; not a count cell

Computed views:

- `aps progress` / `aps status` / `aps export --json`
- Existing rollup PR comments
- Agent "what's next" via fold + `aps next`

Migration: delete or freeze legacy Progress prose; history remains in git.
Do not attempt to perfect old diary cells.

### 9. Lint and validation (sketch)

| Code | Rule |
| ---- | ---- |
| W-progress-type | Index/module claims Standing without `Progress: journal` (or vice versa) |
| W-progress-status | Authoritative-looking `- **Status:**` under Standing+journal |
| W-progress-shard | Shard missing Module/At/Kind or Events table |
| W-progress-evidence | Terminal `To` without Evidence |
| W-progress-orphan | Event Item not defined in module (warning) |
| E-progress-rewrite | CI policy: forbid modifying existing shard blobs (optional; git history + review may suffice in v1) |
| W-index-count | Discouraged pattern: `N/M` in index module tables when journal modules exist |

Exact codes assigned at implementation; keep warnings first where migration needs softness.

### 10. Compaction (out of band, second lever)

Journal shards answer **status races**. They do not shrink a 290-item
**definition** file.

Later (not v1 required):

- Archive completed **definitions** out of the live module into
  `completed.aps.md` / archive modules (COMPOUND territory)
- Optional monthly **snapshot shard** + cold storage of old shards for
  fold performance (fold may short-circuit on snapshot + newer shards)

v1 fold loads all shards; hundreds of small markdown files are acceptable.

### 11. Relationship to existing designs

| Design | Boundary |
| ------ | -------- |
| Conductor | Horizontal coordination of work owned elsewhere; not progress storage |
| TEAM claims | Transient exclusive execution rights; must not become the durable status log |
| Checkout/check-in (superseded) | Put more execution state into work-item status — rejected pattern |
| COMPOUND archive | Historical narrative and task tables after completion; journal is live progress |
| Context packages | Ephemeral handoff under `.aps/context/`; same "don't commit derived truth" lesson |
| In-file status (default vertical) | Unchanged for modules without Standing+journal |

## Alternatives Considered

| Alternative | Pros | Cons | Verdict |
| ----------- | ---- | ---- | ------- |
| Per-work-item progress files | Perfect isolation | ~900 files on anvil-001-scale; busy tree | Rejected for v1 |
| One sidecar per module (mutable status map) | Simple mental model | Still a mutex under parallel recon | Rejected as sole mechanism |
| Single append-only journal file | Simple | EOF merge conflicts on concurrent append | Rejected; use create-only shards |
| Status only in recon PRs, still in-module | Minimal schema change | Module remains conflict hotspot | Rejected as primary fix |
| Counts in index, bot-maintained | Familiar | Already failing honesty (index-counts vs prose) | Rejected; counts always computed |
| Local-only progress (not git) | Zero PR noise | Breaks shared SOT and multi-clone truth | Rejected (decision #2) |
| Expand work-item status machine | Richer semantics | Repeats superseded checkout design; chaff in intent files | Rejected |

## Implementation Notes

Suggested work breakdown **after** this design is ratified (not authorised by
this doc alone):

1. **Contract** — freeze Type/Progress markers, shard schema, fold rules, D-00x
2. **Parser + fold** — read shards; effective status API; tests at CIB-like scale
3. **CLI** — `progress` / reconcile proposal; Standing-aware `next` / `start` /
   `complete` behaviour (no module rewrite when journal mode)
4. **Lint** — markers, evidence, orphan warnings; discourage index `N/M`
5. **Templates + docs** — Standing template or conductor-adjacent docs;
   team-rollout recon guidance
6. **Dogfood / migration recipe** — bootstrap shard from existing Status lines;
   strip index Progress counts
7. **Optional later** — definition compaction; snapshot fold acceleration;
   Stream alias

**Owning module:** [progress-journal](../modules/progress-journal.aps.md) (`PROG`).
Ratify via PROG-000 before implementation work items.

**Pilot candidate:** anvil-001 `continuous-improvement-backlog` after APS
lands the contract.

## Decisions

- **D-001:** Progress storage for Standing modules is **create-only journal
  shards** under `plans/progress/<module-id>/`, not per-item files and not a
  single mutable sidecar. — Merge-friendly at recon cadence; git remains SOT;
  avoids file explosion at ~900 items.
- **D-002:** **Ready (and Draft) stay intent-side** by default; journal owns
  In Progress / Complete / Merged / Blocked (and post-merge lifecycle tokens).
  — Matches standing-backlog practice; keeps planning edits rare relative to
  recon.
- **D-003:** **Recon owns durable progress commits** for Standing+journal;
  feature PRs need not flip status. — Cuts plan-file conflicts and PR noise.
- **D-004:** **Index drops stored counts** (`N/M` and Progress diary cells);
  counts are computed only. — Ends index-counts honesty failure mode.
- **D-005:** Vertical modules **opt in**; default in-file status remains.
  — No forced migration for small modules.
- **D-006:** Published shards are **immutable**; corrections are new events.
  — Preserves audit trail and enables create-only merges.
- **D-007:** Module **display names stay free-form**; tooling keys off
  `Type` / `Progress`, not the title. — Avoids inventing a type just for
  naming ("pen", "pet").

## Open Questions

- [ ] **Q-001:** Exact CLI verbs and whether `aps complete` in journal mode
  writes a local manual shard, queues pending events, or is recon-only.
- [ ] **Q-002:** Whether `Merged` / `Released` / `Shipped` remain progress-fold
  tokens only, or still appear in module files for non-journal modules only
  (status vocabulary ownership with SPEC).
- [ ] **Q-003:** Bootstrap: one-shot import from in-file Status — required
  before journal mode activates, or soft dual-read period?
- [ ] **Q-004:** Nested monorepo plans — is `plans/progress/` always plan-tree
  local (per nested root), and how do federated rollups expose computed counts?
- [ ] **Q-005:** Should TEAM claim acquisition ever emit a progress event, or
  stay strictly out of the journal?
- [ ] **Q-006:** CI enforcement of shard immutability vs review convention only
  for v1.

## Non-Goals

- Hosted APS progress service or database
- Per-work-item progress files in v1
- Replacing TEAM claims, conductor modules, or COMPOUND archives
- Mandatory journal for every module
- Storing authoritative counts in `index.aps.md`
- Auto-merging recon PRs or expanding recon authority beyond plan progress
- Solving 290-item definition-file bulk (compaction is a separate lever)

## Success Criteria

- [ ] Standing+journal module can accept many terminal status flips via new
      shards without editing the module file
- [ ] Two concurrent recon branches adding different shards merge without
      conflict in the common case
- [ ] `aps progress` / fold counts match journal events; index has no `N/M`
      for those modules
- [ ] Default vertical modules unchanged in template and CLI behaviour
- [ ] Lint catches missing evidence on terminal events and Standing/progress
      marker mismatch
- [ ] Migration recipe documented for an existing standing backlog (CIB-shaped)

## References

- anvil-001 scale and CIB/index Progress practice (2026-08-06 survey)
- [Team Coordination Plane](./2026-07-19-team-coordination.design.md) — planes;
  rejection of tracked claim files as default
- [Conductor modules](../../docs/conductor-modules.md) — Type marker precedent
- [Context package regeneration](../../docs/solutions/planning/context-package-regeneration.md)
  — derived artefacts must not compete with source
- [Team rollout](../../docs/team-rollout.md) — status-on-code-PR convention
  (still valid for vertical modules; Standing+journal shifts terminal status
  to recon)
