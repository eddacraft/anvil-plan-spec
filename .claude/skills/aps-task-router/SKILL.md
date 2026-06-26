---
name: aps-task-router
description: >-
  Fast/full classifier for the APS loop. Routes trivial work to a cheap fast
  path while keeping every authority gate and safety rail intact. Use when
  deciding how much ceremony a work item needs: "classify this task",
  "is this fast or full", "does this need a design doc", "route this work item",
  "can I skip the worktree for this", "fast path or full path".
---

# APS Task Router

Leaf of the canonical APS loop (module 12). It answers one question per work
item: **fast track or full track?** The fast track keeps trivial work cheap by
skipping ceremony that buys nothing on small, established changes. It never
buys that speed by skipping authority or safety — those are independent of the
track and apply identically on both.

> **Self-assessment without a fan-out scan is a wish, not a gate.**

## The classification

A task is **FAST** only if **ALL** of these hold:

- touches **≤3 files**;
- **no** interface / schema / API change;
- **no** new dependencies;
- **no** security-sensitive paths (auth, secrets, permissions, data validation);
- the approach is **already established** in the codebase;
- **no** cross-module dependencies.

If any one fails, the task is **FULL**. There is no partial fast track.

**Any doubt -> FULL.** Misclassifying full work as fast only ever costs you a
worktree you didn't strictly need; it never costs a merge or a rail, because
those are reached the same way on both tracks. So the safe error is always to
round up.

## The fan-out gate (non-negotiable)

The file count is **not** the test — it is a precondition you confirm with a
scan, not a guess. Before declaring FAST, run a **dependency fan-out check**: a
symbol-usage / call-graph scan of everything the task will touch.

1. List the symbols (functions, types, exports, config keys) the change edits.
2. Scan the repo for every usage of each symbol.
3. If usages reach **outside the declared file set**, or the touched-file count
   crosses the ≤3 threshold once fan-out is included, the task is **automatically
   FULL** — regardless of how small the original edit looked.

A task that edits one file but whose symbol is consumed across modules is a
FULL task wearing a fast disguise. The scan is what strips the disguise. Skipping
the scan and self-reporting "this is small" is exactly the failure this leaf
exists to close.

## What the fast track changes (content, never authority)

On the FAST track:

- **Skip DESIGN** (checkpoint 1) — no design doc.
- **Skip REVIEW** — no council pass.
- **Skip the worktree** — work in place; the change is small and reversible.
- Write a **reduced work item**: `Intent` + `Expected Outcome` + `Validation`.
  No action-plan file, no design doc.
- **`Expected Outcome` is REQUIRED even on the fast path.** It is the minimum a
  fresh reviewer needs to judge the diff; dropping it is not a permitted saving.

## What the fast track NEVER changes

The fast track reduces the **content** required to cross a gate — never **who**
crosses it.

- **NEVER bypasses checkpoint (2), the Ready membrane.** Authority is
  **human-owned on both tracks.** Fast trims what the human reads; it does not
  remove the human.
- **NEVER bypasses checkpoint (3), merge.** The one irreversible boundary holds
  on both tracks.
- **NEVER bypasses the safety rails.** They are immutable and independent of the
  track — checked before execution, overriding every classification.

These three are orthogonal to the fast/full decision. The router cannot reach
them, weaken them, or skip them.

## Hygiene-class exemption

Items from the scheduled hygiene feeder carry a **predetermined scope**. They:

- skip classification noise (the fast/full decision and its fan-out scan) and
  the design gate;
- still cross checkpoint (2) the Ready membrane;
- still respect the safety rails in full.

Hygiene work is pre-scoped, not pre-authorised: the scope is settled, the
authority and the rails are not.

## The failure mode this guards

The threat is an **optimistic agent that classifies work FAST to dodge
ceremony** — talking itself into "small enough" to avoid design, review, and a
worktree. Three properties close it:

1. **The fan-out scan** turns "this looks small" into an evidence check that a
   cross-module symbol fails objectively.
2. **Any doubt -> FULL** removes the incentive to argue at the margin — the
   cheap, safe default is rounding up.
3. **Rails and the Ready/merge checkpoints are independent of the track**, so
   even a successfully-gamed classification wins only a skipped worktree, never
   a skipped authority gate or a skipped rail.

Gaming the router is therefore low-value by construction: the prize is small and
the protected boundaries are unreachable from here.

## Cross-references

- `plans/designs/2026-06-18-canonical-aps-loop.design.md` — full design and
  rationale (changes #3 and #5, gap G4)
- `aps-loop` — the canonical loop this leaf serves; checkpoints (1)(2)(3),
  the safety rails, and the hygiene feeder are defined there
