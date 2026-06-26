---
name: aps-escalation-queue
description: >-
  Structured, dependency-ordered escalation queue for the APS loop's unattended
  runs — the agent's batched "inbox for the human" so a night's blockers clear
  in one ordered, dependency-safe sitting. Each parked item is one closed
  question with a default. Use when running the loop unattended and a checkpoint
  or blocker is reached: "park this for the human", "add to the escalation
  queue", "clear the escalation queue", "what's blocking the loop", "process
  parked decisions".
---

# APS Escalation Queue

When the APS loop runs unattended, a checkpoint (design, ready, merge) or a
blocker is not a stop — it is a **park**. The agent writes the decision it needs
to a single queue file and keeps working on other available items. The human
later clears the whole queue in one ordered sitting. Clearing is a **transition,
not a conversation**: the human edits one line per item; the loop consumes the
resolutions on its next wake.

This is the unattended delivery mode for the loop's three checkpoints and for
any blocker the loop hits. The inline delivery mode (ask-now) is the
interactive run; see `aps-loop`.

## The file

A single file: `plans/execution/escalation.queue.md`. Two sections: `## Open`
(parked, awaiting the human) and `## Resolved` (audit trail — never deleted).

The whole design target is **20 items cleared in 10 minutes**. Everything below
serves that: a fixed schema makes clearing mechanical, one closed question makes
each decision a single word, priority puts the load-bearing items first, and the
"if you do nothing" line tells the human which items they can safely skip.

## Entry schema (fixed — every field, every time)

A fixed schema is what makes clearing mechanical. Each parked item carries:

```markdown
### ESC-014 · merge-approval · [PRIORITY]

- work item: WI-031 "rate-limit middleware" (left at: VERIFY passed, awaiting LAND)
- blocking-since: 2026-06-21T02:14Z
- what I tried (≤3):
  - implemented, full validation green (see evidence)
  - council pass, no critical/major findings
  - cannot self-merge: merge gate is human-owned
- evidence: <link to diff / verify dossier / failing test / finding>
- THE DECISION I NEED: Merge WI-031 to main now?
  - default: **defer** (stays Merged-pending; nothing ships)
  - alt: approve
- if you do nothing: the rate-limit fix does not ship; the abuse path stays open
- resume token: cycle=42 phase=LAND sha=abc1234 branch=wi-031-ratelimit
- confidence: high
```

Field rules:

- **id** — `ESC-NNN`, a monotonic sequence number, never reused.
- **class** — one of: `ready-gate`, `design-signoff`, `merge-approval`,
  `out-of-scope`, `verification-blocked`, `contract-change`, `safety-rail`,
  `low-confidence`.
- **work item** — the APS item ID, title, and the exact status it was left at,
  so the loop can resume without re-deriving.
- **blocking-since** — UTC timestamp; drives staleness and ordering ties.
- **what I tried** — at most three bullets. Forces the agent to have actually
  attempted the work before escalating.
- **evidence** — links to the verify artifact, diff, failing test, or finding
  the human needs to confirm the decision. No naked assertions.
- **THE DECISION I NEED** — exactly **one closed question** with a **default
  marked** and at most a couple of named alternatives. Never open-ended.
- **if you do nothing** — the downstream cost of inaction. This is what
  separates load-bearing from cosmetic blockers at a glance.
- **resume token** — enough state (cycle, phase, baseline SHA, branch) for the
  loop to rehydrate and continue from where it parked.
- **confidence** — high / medium / low; feeds priority ordering.

## One closed question per item

Each item asks **exactly one** closed question, always with a default the human
can accept in a single word. If a blocker needs two decisions, it is two
entries. The human's whole job is: read the question, read "if you do nothing",
type one word. No item should ever require the human to compose prose to clear
it.

## Priority ordering

The `## Open` section is kept sorted so the most consequential decisions are at
the top:

1. `safety-rail` — always first; these gate everything.
2. `merge-approval` that is **shipped-blocking** (its "if you do nothing" is a
   real downstream cost) — next.
3. `contract-change`, `verification-blocked`, `ready-gate` — middle, by
   blocking-since (oldest first).
4. `design-signoff` and `out-of-scope` — below those.
5. `low-confidence` design questions — **sink to the bottom**; they are the
   most skippable.

Within a tier, oldest `blocking-since` first. Re-sort on every append and every
clear.

## Atomic append (parallel-wave safe)

Appends must be safe for concurrent writers (parallel-wave agents):

- Write **one complete entry per append** — never a partial entry.
- A worker appends with a **collision-free provisional key** it owns —
  `work-item-ID + monotonic-heartbeat-seq` — never a shared `ESC-NNN` counter
  (deriving `ESC-NNN` from the current max at write time is a TOCTOU race: two
  concurrent appenders read the same max). The **single-writer conductor**
  assigns the canonical monotonic `ESC-NNN` when it next drains/re-sorts, so the
  human-facing numbers stay stable and unique without any cross-writer lock.
- An append never rewrites another agent's entry. Numbering and re-sorting are a
  separate, single-writer step (the conductor), not part of an append.

## Batch-clear via swap-file

The human resolves items in place; the loop clears them without racing live
appends:

1. The loop renames `escalation.queue.md` to `escalation.queue.processing`.
2. It immediately writes a **fresh empty** `escalation.queue.md` (`## Open`
   empty, `## Resolved` preserved or seeded), so parallel agents can keep
   appending new blockers without contention.
3. It processes resolutions out of `.processing`, then discards it.

Agents waiting on a parked decision **poll for their specific work-item-ID
resolution**, never for queue length — a shorter queue does not mean _their_
item cleared.

## Never delete — move to Resolved

Cleared items are never removed. They move from `## Open` to `## Resolved` with
the human's decision and a resolved timestamp appended. `## Resolved` is the
audit trail of every autonomous-to-human handoff: who decided what, when, and
on what evidence.

## Clearing is a transition, not a conversation

The human clears an item by editing **one line** into the entry:

```markdown
- DECISION: accept default
```

or

```markdown
- DECISION: approve
```

On its next wake the loop **consumes resolved items first**, before selecting
new work. For each resolved item it rehydrates from the resume token
(cycle/phase/SHA/branch), applies the decision, moves the entry to `##
Resolved`, and continues. No back-and-forth; the human never has to be online
when the loop acts on the answer.

## LAND is serialized in unattended mode

Only **one item past VERIFY at a time** may clear. This guarantees
interdependent merge approvals cannot be cleared out of dependency order:

- If a `merge-approval` entry depends on a prerequisite that is still `## Open`
  or unresolved, clearing the dependent merge is **refused** — the loop leaves
  it parked and surfaces the prerequisite's id.
- The human can still queue both decisions in one sitting; the loop simply
  applies them in dependency order, prerequisite first.

This is what makes "clear it all in one sitting" safe even when merges depend on
each other.

## The failure mode this bounds

A conservative agent can flood the queue and give the human "a second job
triaging". Three mechanisms keep clearing fast and stop the flood from
mattering:

- **Priority ordering** puts the few load-bearing items at the top.
- **One closed question + default** makes each decision a single word.
- **"If you do nothing"** lets the human skip cosmetic blockers without reading
  the detail.

An agent that parks a cosmetic item must still write its "if you do nothing"
line honestly — which is exactly the cost that makes it skippable. The queue
absorbs over-escalation instead of punishing the human for it.

## Cross-references

- `plans/designs/2026-06-18-canonical-aps-loop.design.md` — full design and
  rationale (change #8 / gap G7)
- `aps-loop` — the canonical loop this serves; checkpoints and unattended mode
