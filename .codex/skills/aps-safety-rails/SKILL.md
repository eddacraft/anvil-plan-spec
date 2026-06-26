---
name: aps-safety-rails
description: >-
  The immutable never-do list for the APS loop: actions an agent never takes
  autonomously, checked by intent before execution and overriding every toggle,
  preset, and track. Use when running autonomously and an action might be
  destructive, irreversible, or authority-changing: "is this safe to do
  unattended", "check the safety rails", "can the loop force-push / drop a
  table / touch .env / change a public API / deploy", "what stops the agent",
  "park this as a safety-rail escalation".
---

# APS Safety Rails

The immutable floor of the canonical APS loop. Where the gate policy and the
fast/full router decide _how much process_ a task gets, the rails decide what an
agent **never does on its own** — no matter who approved, which preset is set,
or how trivial the task looks. They are the one part of the loop with no knob.

This leaf is conceptually an extension of git command guardrails, generalised
beyond git and made runtime-neutral: it describes the _policy_, not any one
harness's hook mechanism. A binding may enforce it with hooks, a wrapper, or
in-context discipline; the list below is identical regardless.

## The two invariants

1. **Immutable.** The list cannot be toggled, narrowed, or overridden. No
   preset weakens it — not even an all-auto autopilot or a throwaway-repo spike.
   Neither the fast track nor the full track can bypass a rail. A gate set to
   `auto` still cannot authorise a rail action.
2. **Checked before execution, by intent.** The loop simulates its intended
   action against the list _before_ running it and parks if the action would
   trip a rail. This is an intent-check, not after-the-fact detection: the rail
   stops the action from ever happening, rather than noticing it already did.

## The never-do list

An agent never does any of the following autonomously.

### Git and history

- force-push, history rewrite (rebase that rewrites shared history, amend of
  pushed commits), `--no-verify` (bypassing commit/push hooks), branch-protection
  bypass, or deleting tags.
- merge or push to a protected or default branch **without the merge checkpoint**
  (checkpoint 3). The merge checkpoint is the only authority for shared history.

### Destructive data

- schema-destructive migrations: `DROP`, or any non-additive column change
  (rename, retype, narrow, drop). Additive-only is allowed.
- data backfills, bulk updates, or deletes.
- anything irreversible without a verified backup.

### Secrets

- writing, committing, or reading-into-context the contents of `.env`,
  credential files, key material, or token stores.
- "fixing" or editing such files.
- scanning **may report** that a secret exists or is misplaced; it never reads
  the value into context and never edits the file.

### External contracts

- changing the shape of a public API, pricing, billing, or an auth/permission
  boundary, or altering third-party integration semantics.
- **including internal-library signature changes when the symbol is consumed
  cross-repo** — a private-looking change with external blast radius is still an
  external-contract change.
- these need a human even to **design**, not only to ship.

### Deletion without recovery

- deleting any worktree, branch, or file the loop did not itself create, unless a
  recoverable backup exists. The loop may freely clean up its own throwaway
  artifacts.

### Deploy and release

- deploying or promoting a release to any non-ephemeral environment. Ephemeral
  preview/scratch environments are not covered; staging and production are.

### Money and budget

- spending real money or calling paid external APIs beyond a declared budget.

### Scope

- editing files outside the work item's declared **Scope** beyond a small slack.
  Anchor this on the item's **Scope / Non-scope**, _not_ the best-effort `Files`
  field, which is informational only. A real scope expansion is a
  **stop-and-replan**: it re-crosses the Ready membrane, it does not silently
  widen the current build.

## When a rail would trip

The loop does not proceed. It:

1. **Parks** the action to the escalation queue with class `safety-rail`.
2. Records the intended action and which rail it would trip.
3. **Surfaces it plainly** — one closed question for the human, no burying it in
   a progress report.
4. Continues with other safe, in-scope work if any remains; otherwise stops.

A rail trip is a **stop**, full stop. It is independent of the gate policy and of
whether the run is attended or unattended — an unattended autopilot run parks
exactly the same actions a collaborative run would ask about inline. The rails
are the answer to "what is still safe when no human is watching."

## Relationship to the rest of the loop

- The **gate policy** governs _who approves_ reversible decisions (design, ready,
  merge). The rails govern _what is never delegated at all_. A rail outranks any
  gate.
- The **fast/full router** can drop process for low-risk work but can never drop
  a rail; if a task even appears to approach a rail, that is by itself a reason to
  treat it as full and human-owned.
- The **merge checkpoint (3)** is the single sanctioned path for the one
  irreversible boundary the loop is allowed to cross at all; everything else
  irreversible is on this list and off-limits autonomously.

## Cross-references

- `plans/designs/2026-06-18-canonical-aps-loop.design.md` — full design and
  rationale (design change #9 / gap G5; rails section under "Safety rails").
- `aps-loop` — the canonical loop this leaf belongs to (module 12); see its
  "Safety rails (immutable)" summary, which this leaf expands.
