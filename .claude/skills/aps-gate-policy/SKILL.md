---
name: aps-gate-policy
description: >-
  The toggle layer that tunes how autonomous the APS loop runs. Configures the
  three checkpoints (design, ready, merge) via a GatePolicy object and presets
  (collaborative / design-once / autopilot / spike), with designGate the single
  required toggle. Use when deciding how much the loop self-approves: "set the
  gate policy", "run the loop on autopilot", "make design autonomous", "flip the
  design gate", "configure autonomy budget", "how autonomous should this run".
---

# APS Gate Policy

The APS loop has three named checkpoints — **(1) design**, **(2) ready
membrane**, **(3) merge**. This skill is the toggle layer that decides whether
each is held by a human or self-approved by the loop. It makes the loop more or
less autonomous **without** changing the loop's shape: a gate set to `auto`
still runs, still produces its artifact, and still records an approval. Only the
approver changes.

This policy NEVER overrides the immutable safety rails (see `aps-safety-rails`)
and NEVER changes who owns the Ready membrane on the fast track — that stays
human. It only flips the named gates.

## The GatePolicy object

```
GatePolicy {
  designGate : "human" | "auto"          // checkpoint (1) — REQUIRED toggle
  readyGate  : "human" | "auto"          // checkpoint (2) — the Ready membrane
  mergeGate  : "human" | "auto"          // checkpoint (3) — entry to shared history
  autonomyBudget : {
    unit       : "tokens" | "wallclock" | "usd"  // what is being metered
    limit      : number                          // ceiling in that unit per run
    onExhaust  : "park"                           // at the ceiling, park (escalate)
    resetScope : "run"                            // counter resets each loop run
  }
  landPolicy : "pr-only" | "pr-then-merge" | "direct-merge"
}
```

### The three gate toggles

- **designGate** — checkpoint (1), the _one required_ toggle.
  - `"human"` (default) = collaborative design: a human approves the approach.
  - `"auto"` = the loop self-approves design, using the design checklist as an
    internal rubric instead of asking. Full-path only; the fast path has no
    design gate to flip.
- **readyGate** — checkpoint (2), execution authority crossing the membrane.
  `"human"` (default) = a human marks the item `Ready`; `"auto"` lets the loop
  self-grant `Ready` **on the full track** (e.g. `autopilot`). This toggle is
  independent of the fast/full track: the fast track's Ready membrane is always
  human regardless of `readyGate` (see "What this skill does not change").
- **mergeGate** — checkpoint (3), the one irreversible boundary into shared
  history.

A gate set to `"auto"` is **not deleted**. It still runs, still emits its
artifact (design doc / readiness record / land record), and writes a
**self-approval** entry to the run journal. The audit trail of an auto gate is
identical in structure to a human approval — the journal just records who
approved.

### autonomyBudget

A spend ceiling for one run, so an unattended loop cannot run away.

- **unit** — what is metered: `tokens`, `wallclock` time, or `usd`.
- **limit** — the numeric ceiling in that unit.
- **onExhaust** — `park`: on hitting the limit the run parks (escalates) rather
  than continuing; the human resumes deliberately.
- **resetScope** — `run`: the counter is per loop run and resets when a new run
  begins. Budget does not carry across runs.

### landPolicy

How landing reaches shared history:

- `"pr-only"` — open a PR and stop; a human or external check merges.
- `"pr-then-merge"` — open a PR, then merge it once checks pass.
- `"direct-merge"` — merge straight to the target branch (throwaway repos only).

`landPolicy` describes _how_ to land; `mergeGate` decides _whether_ the loop may
cross checkpoint (3) without a human. The safety rails still forbid merging
contract- or migration-class changes autonomously regardless of either field.

## Presets (sugar over the object)

Presets are named GatePolicy values. Pick a preset, or set fields directly.

| Preset            | designGate | readyGate | mergeGate | landPolicy    | Notes                                                                 |
| ----------------- | ---------- | --------- | --------- | ------------- | --------------------------------------------------------------------- |
| **collaborative** | human      | human     | human     | pr-only       | DEFAULT — all gates human                                             |
| **design-once**   | human      | auto      | auto      | pr-then-merge | approve approach, then drive                                          |
| **autopilot**     | auto       | auto      | auto      | pr-only       | never auto-merges contract/migration-class changes; rails still apply |
| **spike**         | auto       | auto      | auto      | direct-merge  | throwaway repos only                                                  |

`autopilot` self-approves every gate but **never** auto-merges
contract/migration-class changes — those still escalate, and every safety rail
still applies. `spike` is for disposable repos where direct-merge is acceptable.

## Resolution order

The effective policy is resolved later-wins:

1. **Built-in default** — the `collaborative` preset.
2. **Project policy** — a policy block in `plans/project-context.md` (a preset
   name or explicit field values).
3. **Invocation-time override** — a preset or fields passed when the loop is
   invoked.

Each later layer overrides only the fields it sets; unset fields fall through to
the layer below. The fully **resolved** policy is recorded in the run journal at
the start of the run, so every run documents exactly how autonomous it was and
why each gate was held or self-approved.

## What this skill does not change

- It does not weaken the safety rails — they are immutable and orthogonal to
  every gate (`aps-safety-rails`).
- It does not change _who_ owns the Ready membrane on the **fast track** — that
  is human regardless of `readyGate`. The fast path reduces the _content_
  required to cross, never the authority.
- It does not add or remove checkpoints. It only sets each named gate to `human`
  or `auto` and records the resolved choice.

## Cross-references

- `plans/designs/2026-06-18-canonical-aps-loop.design.md` — full design and
  rationale (design change #7 / gap G3; GatePolicy section)
- `aps-loop` — the canonical loop this policy tunes (the three checkpoints)
- `aps-safety-rails` — the immutable rails this policy never overrides
