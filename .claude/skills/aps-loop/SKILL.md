---
name: aps-loop
description: >-
  Canonical APS development loop: interactive planning and design fill a Ready
  backlog, autonomous implementation drains it, with Ready as the human-owned
  membrane. Autonomous-first with three named checkpoints (design, ready, merge).
  Use after an APS plan exists: "run the APS loop", "work through the plan",
  "execute the approved plan", "implement and keep the plan current".
---

# APS Loop

The canonical, harness-agnostic development loop. **Planning fills a backlog,
execution drains it, and `Ready` is the membrane between them.** A human
authorises the _what_ by letting a work item cross the membrane; the loop owns
the _how_ on the other side. Because everything past the membrane lives in a
throwaway branch or worktree, the loop drives autonomously and stops only at the
named checkpoints.

This skill is the canonical spec; runtimes (Claude, Codex) run it directly. The
full design rationale is
`plans/designs/2026-06-18-canonical-aps-loop.design.md`.

If `plans/index.aps.md` does not exist, this skill does not apply. Route to
`planning-workflow` to decide whether APS should be introduced.

## Four principles

1. **Isolation makes autonomy safe.** In-branch work is reversible — drive
   through it; reserve human stops for irreversible, approach, and authority
   boundaries.
2. **Evidence gates every transition.** No state advances without a fresh
   verification artifact. Autonomy is anchored to proof, not self-reported
   confidence.
3. **APS is the source of truth; the run-journal is a durable resume record.**
   On resume, trust-but-verify by re-running validation.
4. **Safety rails are immutable** and override every toggle and track.

## Topology — two loops, one membrane

```
PLANNING (fills) :  INTAKE -> DESIGN --(1)design-- AUTHORISE --(2)ready--> || Ready backlog ||   <- membrane (human-owned)
EXECUTION (drains): SELECT -> ISOLATE -> BUILD -> VERIFY -> REVIEW -> LAND --(3)merge--> LEARN (repeat)
PRE   :  PROBE+  self-validating capability manifest (once per repo)
FEED  :  HYGIENE (scheduled, report-only) + discoveries -> INTAKE ;  lessons/ folded in at LEARN
RAILS :  immutable, checked before execution, override every toggle and track
```

## Canonical phases

| Phase     | Loop | What                                                           | Where it runs                         |
| --------- | ---- | -------------------------------------------------------------- | ------------------------------------- |
| PROBE+    | pre  | Detect test/lint/build/typecheck/CI; self-validating manifest  | `aps-probe`                           |
| INTAKE    | plan | Request -> Draft work item in a module; classify fast/full     | `planning-workflow`                   |
| DESIGN    | plan | Approach + design doc — **checkpoint (1)** (toggle, full only) | `planning-workflow`, `brainstorming`  |
| AUTHORISE | plan | Design -> lean APS actions + validation — **checkpoint (2)**   | `planning-workflow`, `writing-plans`  |
| SELECT    | exec | Self-select next safe Ready item                               | this loop                             |
| ISOLATE   | exec | Worktree/branch + green baseline + smoke test                  | `dev-workflow`, `using-git-worktrees` |
| BUILD     | exec | Implement action-by-action, TDD where tests exist              | `dev-workflow`                        |
| VERIFY    | exec | Re-run full validation after last edit -> evidence dossier     | `verification-before-completion`      |
| REVIEW    | exec | Council; fixes re-enter BUILD->VERIFY                          | `local-review-council`, `council`     |
| LAND      | exec | Fenced commit/PR/CI/merge -> set `Merged` — **checkpoint (3)** | `aps-landing`, `dev-workflow`         |
| LEARN     | exec | Reconcile APS, record lessons, log discoveries                 | this loop, `aps-planning`             |

PROBE+ (`aps-probe`), the gate policy (`aps-gate-policy`), the fast/full router
(`aps-task-router`), the safety rails (`aps-safety-rails`), fenced landing +
reconciliation (`aps-landing`), the escalation queue (`aps-escalation-queue`),
and the trust-window resume model (`aps-resume`) are the loop's leaves. This
skill orchestrates them; each is single-purpose and usable on its own.

## The three checkpoints

One mechanism: produce the artifact plus **a single closed question with a
default**, then either **ask inline** (interactive run) or **park in the
escalation queue** (unattended run; cleared in one batch).

1. **(1) DESIGN** — is this the right approach? Toggleable; default-on
   (collaborative). Full-path only.
2. **(2) READY (the membrane)** — does this work item have execution authority?
   **Human-owned on both fast and full tracks.** The fast path reduces the
   _content_ required to cross, never _who_ crosses.
3. **(3) MERGE** — should this enter shared history? The one irreversible
   boundary; sets `Merged` (not Complete).

## Operating stance

Proceed autonomously for reversible implementation, validation, review, APS
bookkeeping, drift correction, and plan evolution within the approved scope. Ask
the user only for product intent, scope changes, safety decisions, destructive or
irreversible actions, or input no tool can discover.

Before reporting progress, audit each claim against evidence from this session:
files changed, commands run, review results, or APS updates. If something is not
verified, say so explicitly.

Do not end a turn with a plan or promise when the next action is safe and within
scope. Do the work, record the evidence, then report the outcome.

## The execution loop

```
Orient -> Select -> Validate -> Implement -> Verify -> Review -> Land -> Reconcile -> Evolve -> Repeat
```

1. **Orient.** Load APS context via `aps-planning`. Read relevant module files,
   `plans/execution/lessons/` if present, and the tail of
   `plans/execution/loop-journal.md` if present. Resume interrupted iterations
   from the journal rather than starting fresh (re-derive by re-running
   validation; trust-window resume is LOOP-009).
2. **Select.** Choose the highest-priority `Ready` item with no unmet
   dependencies. For independent action waves, use `parallel-agents` where
   available and keep working while they run. If nothing is Ready, route to
   **Evolve** — replanning is the work.
3. **Validate.** Run the APS truth gate. Treat drift as replanning input, not a
   reason to stop. Correct stale, blocked, ambiguous, or already-done items
   through `planning-workflow` / `aps-planning`, then continue.
4. **Implement (ISOLATE + BUILD).** Route one ready item through `dev-workflow`:
   isolated branch or worktree, TDD where practical, focused changes. Do not
   widen scope while implementing (see safety rails).
5. **Verify.** Run the item's validation command and repo-mandated checks. Have a
   fresh-context reviewer compare the diff against the item's Expected Outcome.
   Failures route back to implementation or `systematic-debugging`.
6. **Review.** `local-review-council` during implementation; `council` for
   milestones or high-risk changes. Fixes re-enter Implement -> Verify. Address
   critical and major findings before Land.
7. **Land.** Checkpoint (3). Fenced commit/PR/CI/merge, then set the item to
   `Merged` — **not** Complete (see lifecycle). Fencing detail in `aps-landing`.
8. **Reconcile + Evolve (LEARN).** Update APS status with validation evidence,
   add discovered files to `Files:`, append the journal entry, fold lessons in,
   and make the plan true for the next cycle within the authority table.

## APS lifecycle (per ADR-0013)

The loop uses Anvil's full canonical lifecycle:

```
Draft -> Proposed -> Ready -> In Progress -> Merged -> Released/Shipped -> Complete
                              (start)        (merge)   (release record)    (tag/ship)
```

- `Merged` is **interim** — a merged PR is not done.
- `Complete` is **tag-gated** — a cleanup agent advances
  `Merged -> Released/Shipped -> Complete` only on release/ship evidence.
- `Blocked` is a side-state; `Committed` is legacy wording for `Merged`.
- **A-now:** the loop and cleanup agent edit these back-end statuses directly
  (the `scripts/aps-cleanup.sh` pattern); `aps lint` tolerates the vocabulary.
- **B-later:** when the bundled `aps` CLI gains `aps merge` + tag-gated complete,
  swap direct edits for CLI calls — same states, no model change.

## Autonomy posture and gates

`aps-gate-policy` makes autonomy a toggleable `GatePolicy` with presets
(collaborative / design-once / autopilot / spike), where `designGate` is the
single required toggle; its default (`collaborative`) is the authority table
below plus the three checkpoints — design and merge held by a human, reversible
in-scope execution autonomous. `aps-task-router` decides which track a task takes
— the fast path skips DESIGN, REVIEW, and the worktree; any doubt -> full, and
the fast path never bypasses checkpoint (3) or the safety rails.

## Plan evolution authority

| Change                                               | Loop may apply autonomously |
| ---------------------------------------------------- | --------------------------- |
| Item statuses and validation evidence                | yes                         |
| `Files:` fields and drift corrections                | yes                         |
| Action plans within an approved item                 | yes                         |
| New `Proposed` items within the approved scope       | yes                         |
| Splitting, merging, or retiring items with rationale | yes                         |
| Index Problem, Success Criteria, Constraints         | no - checkpoint             |
| Accepted ADRs or project policy                      | no - checkpoint             |
| Deleting modules or abandoning a milestone           | no - checkpoint             |
| Destructive or irreversible actions                  | no - checkpoint             |

A checkpoint means record the proposal in the journal, surface it plainly to the
user (or the escalation queue when unattended), and continue with other available
work. End the turn only when no safe in-scope work remains.

## Safety rails (immutable)

Never do these autonomously — checked before execution, overriding every toggle
and track (full rail leaf: `aps-safety-rails`):

- force-push, history rewrite, `--no-verify`, branch-protection bypass;
- merge to protected/default branches without checkpoint (3);
- destructive migrations or data deletes;
- touch secrets / `.env` (scan-only; report, never edit);
- change external contracts (public API, pricing, auth, billing) — including
  internal-library signatures consumed cross-repo;
- delete anything without a recoverable backup;
- deploy/release to non-ephemeral environments;
- spend real money beyond a declared budget.

## Memory

Store durable execution learnings in `plans/execution/lessons/` when the project
uses that directory. Each lesson explains what changed future execution and why.
Prefer updating or deleting existing lessons over creating duplicates. Read
relevant lessons during Orient and fold them into new work item text during
Evolve — the loop should compound rather than rediscover.

## Journal

Append one entry per cycle to `plans/execution/loop-journal.md` when the project
uses APS execution files:

```markdown
## Cycle N - YYYY-MM-DD

- Item: ID - title
- Outcome: done | blocked | replanned | validation-failed
- Validation: command plus result summary
- Review: reviewer/council result summary
- Plan changes: items added, updated, split, retired, or unblocked
- Checkpoints raised: none, or the exact user-level decision needed
- Next: selected item or stop condition
```

The journal is the resume point and audit trail. Treat it as bookkeeping: commit
it with plan changes, never hidden inside feature work.

## Stop conditions

Stop, report, and end the turn when:

- no Ready work remains and no in-scope Proposed work can be promoted safely;
- every remaining item is blocked on user input or an external dependency;
- a product, scope, safety, or irreversible checkpoint is the only work left;
- validation fails after three focused attempts on one item (mark it `Blocked`);
- a user-specified cycle, time, or cost limit is reached.

The final report opens with outcomes: items completed, validation evidence, plan
changes made, and any checkpoint decision needed from the user.

## Bindings and leaves

- **Bindings:** none — Claude and Codex run this spec directly. Any future
  runtime binding references this skill rather than redefining the loop.
- **Leaves:** `aps-probe` (capability manifest), `aps-gate-policy` (gate policy +
  presets + designGate toggle), `aps-task-router` (fast/full routing),
  `aps-safety-rails` (immutable rails), `aps-landing` (fenced landing +
  single-writer reconciliation + wave locks), `aps-escalation-queue` (batched
  human queue), `aps-resume` (trust-window resume). Each is single-purpose and
  usable on its own.

## Cross-references

- `plans/designs/2026-06-18-canonical-aps-loop.design.md` - full design + rationale
- `plans/decisions/0013-aps-loop-state-model.md` - lifecycle decision (ADR-0013)
- `planning-workflow` - interactive planning and readiness gate
- `dev-workflow` - implementation of one ready item
- `aps-planning` - APS context, truth validation, and reconciliation
- `writing-plans` - implementation plans after design approval
- `parallel-agents` - independent action waves
- `systematic-debugging` - validation failures and regressions
- `verification-before-completion` - evidence before completion claims
