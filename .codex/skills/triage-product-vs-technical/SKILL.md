---
name: triage-product-vs-technical
description: >-
  Before an autonomous workflow pauses for the user, classifies the pending
  question as PRODUCT (must surface, in plain English) or TECHNICAL
  (auto-resolve, log the decision, continue). Use inside any unattended or
  low-touch run — the fable5-aps-loop checkpoint step, autonomous planning,
  or execution agents — whenever a question, blocker, or grey area is about
  to interrupt the user. Keeps autonomous runs moving by stopping only for
  decisions a non-developer must actually make.
---

# Triage: Product vs Technical

Consult this skill before any autonomous agent surfaces a question, blocker,
grey area, or pause to the user. Classify the pending question and either:

- surface it to the user in plain English (PRODUCT), or
- resolve it autonomously and record the decision in the decisions log
  (TECHNICAL).

## When to apply

- An autonomous loop (such as `fable5-aps-loop`) hits something that looks
  checkpoint-worthy and must decide whether to stop or proceed.
- A planning or execution agent is about to ask the user a question
  mid-run.
- A workflow is configured for minimal user involvement (non-developer
  stakeholder, unattended run, low-touch autopilot).

Structural checkpoint rules (for example the plan-evolution authority table
in `fable5-aps-loop`) take precedence: a change that table marks as a
checkpoint is always surfaced. This skill classifies everything else — the
ad-hoc questions and grey areas that arise during work.

## Classification rules

A pending question is **PRODUCT** (surface to user) when answering it
changes any of:

1. **What the user sees or experiences** — the visible surface, the
   wording, the flow, the feedback.
2. **What scenarios are supported** — does it work offline, on a slow
   network, on multiple devices, after a crash.
3. **Who the feature is for** — audience changes, persona shifts,
   accessibility expectations.
4. **What outcome counts as success** — KPIs, target metrics, acceptable
   latency from the user's perspective.
5. **Trade-offs between user needs** — speed vs accuracy, breadth vs
   depth, simple vs powerful.
6. **What's explicitly out of scope** — refusing a use case is a product
   decision.

A pending question is **TECHNICAL** (auto-resolve, do not surface) when
answering it changes any of:

1. **How something is implemented** — architecture, framework, library,
   data structure, pattern.
2. **Internal naming, file layout, module structure** — engineers'
   ergonomics.
3. **Test strategy** — unit vs integration, coverage targets, mocking
   approach.
4. **Retry/timeout/concurrency values** — without user-visible latency
   consequences.
5. **Error-handling approach** — how to log, how to retry, what to do on
   partial failure.
6. **Performance trade-offs invisible to the user** — caching, indexing,
   query optimisation.
7. **Refactoring scope** — does this clean up adjacent code or stay
   surgical.
8. **Anything an experienced engineer would decide without asking a
   product owner.**

## Edge cases — when technical IS product

A technical decision becomes product-level when:

- it changes user-visible latency in a meaningful way ("this approach makes
  the page take 2s vs 200ms");
- it changes which user scenarios actually work (offline support, slow
  connections);
- it has user-facing cost implications (a more expensive infrastructure
  choice that ends up affecting pricing);
- it prevents a user-stated requirement (the user said "must work offline"
  and the technical choice precludes it);
- it introduces a privacy or data-handling consideration the user should
  know about;
- it permanently locks in a choice that's hard to reverse later.

When in doubt: **classify as PRODUCT.** A false positive wastes a user
moment. A false negative buries a real product question.

## Output: PRODUCT classification

When PRODUCT, ask the user — one question at a time, using the harness's
question mechanism where one exists. Reframe in plain English:

- Ban technical vocabulary: schema, endpoint, framework, library, deploy,
  config, migration, service, API, validation, async, sync, queue, cache,
  retry.
- Frame as scenario: "What should happen when \_\_\_?"
- Frame as outcome: "Does this need to also work when \_\_\_?"
- Show the user the trade-off in their terms, not engineering terms.

Examples:

| Raw agent question                     | Product reframe                                                                                                                                                                               |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| "Should we use Stripe or Square?"      | If both have been pre-approved by the user → TECHNICAL. If neither has → PRODUCT, frame as: "Do you have a preferred payment provider, or should I pick the one that's fastest to integrate?" |
| "Sync or async checkout confirmation?" | "When the customer clicks Pay, do they need to see confirmation right away, or is a 'processing — we'll email you' screen acceptable?"                                                        |
| "Soft delete or hard delete?"          | "If an admin deletes a customer record by mistake, should they be able to recover it for a window of time?"                                                                                   |
| "What's the cache TTL?"                | TECHNICAL — auto-resolve unless user-visible staleness is in question                                                                                                                         |

## Output: TECHNICAL classification

When TECHNICAL, do NOT pause. Resolve autonomously and append the decision
to the run's decisions log, `plans/execution/decisions.md`:

```yaml
- decision: "Use Phoenix LiveView for the dashboard rather than React"
  rationale: "Project's existing dashboards use LiveView; convention match; no user-visible difference"
  reversibility: "easy — could swap later if needed"
  confidence: "high"
  surfaced: false
  reason_not_surfaced: "Pure technical — no user-visible consequence"
```

Continue with the work. The decision is logged for audit, surfaced in batch
in the run's final report, and reviewable by anyone later. A decision that
proves load-bearing should be promoted to a full ADR in `plans/decisions/`.

## Output: borderline cases

When you classify with low confidence (60–80%), still resolve technically
but flag the entry:

```yaml
- decision: "Use eventual consistency for profile sync across devices"
  rationale: "Vast majority of users won't notice; matches existing pattern"
  reversibility: "medium — would require schema/replication change"
  confidence: "low"
  surfaced: false
  reason_not_surfaced: "Borderline — likely invisible to users, but flagging for human review"
  needs_review: true
```

Entries with `needs_review: true` are listed in the run's final report as
"decisions worth a human pass."

## Anti-patterns to avoid

1. **Surfacing technical questions in product clothing** — "Are you OK with
   the data being slightly stale?" is still a technical question. Don't.
2. **Auto-resolving things that change what the user sees** — if the
   trade-off is between two visible behaviours, that's product, even if
   engineers think it's technical.
3. **Asking the user about defaults** — if the user hasn't specified
   preferences and the choice is genuinely invisible, pick the conventional
   default and move on.
4. **Surfacing decisions with low confidence and no user-visible
   consequence** — log them, don't ask.
5. **Bundling multiple unrelated questions into one user pause** — surface
   one product question at a time.

## Procedure

1. Read the pending question / pause reason.
2. Classify against the rules above.
3. If PRODUCT: reframe in plain English → ask the user → integrate the
   answer.
4. If TECHNICAL: pick the best option using project context (existing
   patterns, the project instruction file, sensible defaults) → append to
   `plans/execution/decisions.md` → proceed.
5. If borderline: TECHNICAL with the `needs_review: true` flag.

## Calibration

Re-tune periodically by reading recent decisions-log entries. If many
`needs_review: true` items turn out to have been product-level in
retrospect, the classifier is too aggressive. If users frequently say "you
didn't need to ask me that," it's too conservative.

Target: fewer than one user pause per work item, except when the user has
explicitly requested more involvement.

## Cross-references

- `fable5-aps-loop` — the checkpoint step this skill refines
- `product-only-interview` — companion for non-developer requirement
  gathering
- `plans/aps-rules.md` — decisions and design-doc conventions
