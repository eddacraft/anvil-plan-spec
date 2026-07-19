# Team Coordination Plane

| Field   | Value                                                        |
| ------- | ------------------------------------------------------------ |
| Date    | 2026-07-19                                                   |
| Status  | Draft                                                        |
| Modules | [team-coordination](../modules/team-coordination.aps.md)      |
| Scope   | TEAM-000 — define actor, claim, lease, handoff, and authority |

## Problem

APS can guide one human or one agent through a dependency graph, and its team
rollout conventions support multiple module owners. It does not yet coordinate
a project where several humans each run independent agent processes from
separate worktrees or clones.

Today, `aps start` changes a work item from `Ready` to `In Progress`, but the
operation does not identify the human sponsor, executing process, run, branch,
or lease. Two processes can read the same Ready item and both believe they
claimed it. The plan also cannot distinguish active work from an abandoned
agent session, express a safe handoff, or connect a work item to delivery
evidence without relying on team convention.

The earlier
[check-out/check-in design](../../docs/plans/2026-03-10-checkout-checkin.design.md)
identified these failure modes, but coupled transient execution state to an
expanded work-item status machine. The preserved
[dashboard design](../../docs/plans/2026-03-10-dashboard.design.md) also
anticipated actor, staleness, branch, commit, and review views. This design
revives those needs while keeping APS's canonical lifecycle and minimal
templates stable.

## Desired Outcome

A team can run multiple agentic processes concurrently and answer, from one
portable interface:

- What work is available and safe to claim?
- Who is accountable for it, and which process is executing it?
- Which claim won if processes raced?
- Is the actor still active, blocked, waiting for review, or safe to replace?
- What branch, pull request, validation, and merge evidence belongs to it?
- Which otherwise-independent items are likely to collide on files or owners?

The coordination capability must remain optional. Projects that use only APS
markdown and the current `next` / `start` / `complete` flow continue to work.

## Constraints

- APS markdown remains the durable source of planning intent.
- No hosted APS service or database is required.
- The minimal index and module templates gain no mandatory team fields.
- Provider-specific review and merge concepts stay behind adapters.
- Claims authorise bounded execution only; they do not grant merge, release,
  deployment, destructive-operation, or scope-expansion authority.
- Coordination must degrade visibly when no shared transport is configured.
- The design must work for common-directory worktrees and for separate clones.

## Design

### 1. Separate the four planes

| Plane        | Canonical information                                      |
| ------------ | ---------------------------------------------------------- |
| Intent       | Modules, work items, dependencies, outcomes, decisions     |
| Coordination | Actors, runs, claims, leases, activity, handoffs            |
| Delivery     | Branches, commits, pull requests, CI, reviews, merge state   |
| Visibility   | Effective ready queue, active work, conflicts, stale alerts |

The intent plane stays in tracked APS markdown. Coordination is transient,
shared operational state: it can expire or be garbage-collected without
changing what the project intends to build. Delivery evidence belongs to Git
and provider systems, with APS storing or deriving only the links needed for
traceability. Visibility is a projection over the other three planes, not a
new source of truth.

### 2. Actor and authority model

The current module `Owner` remains the durable accountable human or team. Team
coordination introduces four distinct identities:

| Term      | Meaning                                                         |
| --------- | --------------------------------------------------------------- |
| Owner     | Durable accountability and review authority for a module        |
| Sponsor   | Human authorising this execution within an approved work item    |
| Actor     | Human or agent process currently performing the work             |
| Run       | Unique invocation/session of an actor, used for recovery/auditing |

An actor identity is descriptive unless a transport can bind it to an
authenticated provider identity. A self-asserted local actor name is useful
for coordination but is not an access-control boundary.

A claim inherits only the authority already present in the work item and the
sponsor's explicit instruction. It cannot silently authorise broader work,
external communication, merging, deployment, release, or destructive action.

### 3. Claim record

The conceptual claim record is versioned independently from the APS document:

```yaml
schema: aps-claim/v1
plan: root
item: PAY-003
sponsor: human:@alice
actor:
  kind: agent
  harness: codex
  id: worker-2
run: 019f-example
base: a089dbd
workspace: feat/pay-003
claimed_at: 2026-07-19T10:00:00+08:00
expires_at: 2026-07-19T12:00:00+08:00
last_activity: 2026-07-19T10:20:00+08:00
file_hints:
  - src/payments/export.rs
```

The schema must not contain credentials, prompts, model transcripts, or
secrets. Transport-specific authentication data stays outside the record.

### 4. Claim lifecycle

```text
Absent ──claim──> Active ──renew──> Active
                     ├──handoff──> Active (new actor/run)
                     ├──release──> Released
                     └──expire───> Stale ──takeover──> Active
```

- Claim creation is compare-and-set: exactly one contender wins.
- A failed contender receives the current holder and expiry, and changes no
  plan file.
- Only the current holder may renew or release a live claim.
- Handoff atomically closes the old actor's tenure and names the new one.
- Expiry makes a claim eligible for takeover; it does not mutate APS markdown.
- Takeover is explicit and leaves durable audit evidence in the coordination
  store.
- Cleanup removes terminal coordination records only after relevant delivery
  evidence has been retained or linked.

The canonical APS lifecycle remains:

```text
Draft -> Ready -> In Progress -> Complete
```

`Claimed`, `Stale`, `In review`, and `Conflicting` are effective coordination
views, not new work-item statuses. Whether a successful claim also changes the
actor's branch copy to `In Progress` is unresolved in
[Q-002](../issues.md#q-002-how-do-claims-affect-the-effective-and-declared-work-item-status).

### 5. Claim acquisition

A team-aware claim operation follows this sequence:

1. Resolve the item against a named plan-tree snapshot and verify dependencies.
2. Read the configured coordination store.
3. Atomically create the claim only if no live claim exists.
4. On success, generate the context package and optionally advise or create a
   workspace through a separate VCS integration.
5. On failure, report the holder, workspace, age, and expiry.
6. Re-query the effective plan state before selecting more work.

Claiming and VCS workspace creation are separate authority boundaries. Core APS
may advise a branch name; a configured adapter may create one only when project
policy explicitly permits it.

### 6. Coordination transport

The CLI should depend on a small claim-store contract rather than a provider:

| Candidate              | Strengths                                     | Limits / proposed role                         |
| ---------------------- | --------------------------------------------- | ---------------------------------------------- |
| Local exclusive file   | Simple and atomic in one filesystem           | Local worktrees only; local-mode adapter       |
| Git refs/objects        | Compare-and-set, auditable, outside worktree  | Prototype for local and remote Git coordination |
| Tracked claim files     | Visible in ordinary Git history               | Reject as default: plan churn and merge races  |
| GitHub state            | Authenticated identity and PR/CI visibility   | Optional provider adapter                      |
| Hosted APS database     | Strong shared transactions                    | Out of scope                                   |

The first prototype should test Git-backed compare-and-set semantics, including
an independent bare remote. TEAM-000 does not select the production transport;
that decision closes [Q-001](../issues.md#q-001-which-shared-claim-transport-should-team-mode-use).

### 7. Conflict model

| Conflict surface           | Default treatment                                      |
| -------------------------- | ------------------------------------------------------ |
| Same work item             | Hard exclusion: one live claim                         |
| Unmet dependency           | Hard exclusion: cannot claim                           |
| Same explicit action       | Hard exclusion when action-level claims are enabled    |
| Overlapping `Files` hints  | Advisory warning; hints are incomplete                 |
| Same module owner capacity | Advisory scheduling signal                             |
| Provider merge conflict    | Delivery adapter / merge queue owns resolution         |

Work-item claims are exclusive by default. Cooperative work on one item must be
declared through an action plan whose independent actions and checkpoints make
the split observable; it must not arise from two actors silently sharing one
claim.

Conflict-aware `next` and wave planning rank work using dependencies, claims,
file hints, module boundaries, and owner capacity. They warn rather than block
distinct items merely because their best-effort file hints overlap.

### 8. Handoff and delivery evidence

A handoff/check-in record carries only concise recovery and traceability data:

- Completed checkpoint and remaining checkpoint.
- Branch/worktree and relevant commits.
- Pull request and current review/CI state when an adapter is available.
- Validation commands already run and their outcome.
- Blocker, decision, or authority escalation required.
- Whether takeover is safe, unsafe, or requires human review.

Git providers continue to own code review, branch protection, merge queues,
and authenticated permissions. APS maps those facts to work items; it does not
reimplement or bypass them.

### 9. Visibility and export

A team projection should expose at least:

```text
Available:    12
Claimed:       5
In review:     3
Blocked:       2
Stale:         1
Conflicting:   2
```

Each active row links work item -> owner -> sponsor -> actor/run -> workspace ->
delivery evidence. The JSON export adds a versioned, optional `coordination`
object so existing consumers of plan intent do not break.

The useful command names remain design-level placeholders:

```text
aps claim TEAM-001
aps claim renew TEAM-001
aps claim release TEAM-001
aps handoff TEAM-001 --to human:@alice
aps team status
aps next --claim
```

TEAM-000 decides naming only after the state and failure contracts are stable.

### 10. Progressive enhancement

| Mode     | Behaviour                                                     |
| -------- | ------------------------------------------------------------- |
| Off      | Current APS workflow; `start` remains advisory across clones   |
| Local    | Atomic coordination across processes sharing a Git directory  |
| Shared   | Claims published through a remote Git coordination store      |
| Provider | Shared claims enriched with authenticated PR/CI/review state   |

Commands must state which mode is active. Falling back from Shared to Local or
Off must be explicit because it weakens duplicate-work prevention.

## Failure and Validation Matrix

| Scenario                                      | Required result                                      |
| --------------------------------------------- | ---------------------------------------------------- |
| Two processes claim the same Ready item       | Exactly one succeeds                                 |
| Two processes claim independent Ready items   | Both succeed                                         |
| Losing claimant retries                       | Current holder and expiry are shown                  |
| Non-holder renews or releases a claim         | Operation fails without mutation                     |
| Actor crashes                                 | Claim becomes stale; plan intent remains intact      |
| Authorised takeover occurs                    | New holder wins and takeover history is retained     |
| Distinct items have overlapping file hints    | Both may proceed, with a visible conflict warning    |
| Claim acquisition fails midway                | No partial claim and no plan-file mutation           |
| Shared transport is unavailable               | Explicit degraded-mode error or opt-in fallback      |
| Work lives in a nested child plan             | Path-qualified item identity remains unambiguous     |
| Handoff occurs with uncommitted work           | Recovery state says whether takeover is safe         |
| PR and CI exist                               | Delivery evidence projects without owning that state |

TEAM-001 and TEAM-002 must turn this matrix into an adversarial automated
harness before the module can become Ready for broader rollout.

## Decisions

- **D-001:** Separate coordination from durable intent — _proposed: claims,
  actors, and leases are an optional operational plane; APS markdown remains
  canonical for planned outcomes and durable status._
- **D-002:** Preserve the canonical work-item statuses — _proposed: do not add
  Checked Out, Checked In, Review, or Stale to the portable status vocabulary;
  derive them as effective views._
- **D-003:** Distinguish owner, sponsor, actor, and run — _proposed: ownership
  and execution identity are different authority concepts._
- **D-004:** Use expiring leases — _proposed: claims are renewable and
  explicitly recoverable, never permanent locks._
- **D-005:** Exclusive item, advisory file overlap — _proposed: hard exclusion
  is item/action scoped; best-effort file hints inform scheduling._
- **D-006:** Keep provider features behind adapters — _proposed: providers own
  authentication, reviews, CI, merge queues, and permissions._
- **D-007:** Progressive enhancement — _proposed: solo/file-only APS stays
  valid; team coordination is enabled by configuration and reports degradation._

## Open Questions

- [Q-001](../issues.md#q-001-which-shared-claim-transport-should-team-mode-use) —
  which shared transport can provide portable compare-and-set claims across
  separate clones?
- [Q-002](../issues.md#q-002-how-do-claims-affect-the-effective-and-declared-work-item-status) —
  when and where should a claimed item become durably `In Progress`?
- [Q-003](../issues.md#q-003-how-are-actor-identity-lease-expiry-and-takeover-trusted) —
  what trust and takeover policy is appropriate without making APS an identity
  provider?

## Implementation Notes

TEAM-000 is a design gate, not implementation authority. Once the decisions
and questions above are resolved, the module can promote the smallest
prototype slice to Ready. No template changes are required for the design
phase.
