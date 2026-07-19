# Team Coordination Module

| ID   | Owner  | Priority | Status |
| ---- | ------ | -------- | ------ |
| TEAM | @aneki | high     | Draft  |

**Last reviewed:** 2026-07-19

## Purpose

Make APS safe and legible for projects where multiple humans each run
independent agentic processes from separate worktrees or clones. Add an
optional coordination plane for actor identity, atomic claims, leases,
handoffs, conflict-aware selection, and delivery visibility without replacing
APS markdown, Git, pull requests, or provider permissions.

## Background

The v0.7 team rollout gives teams ownership conventions, pinned tooling,
central lint enforcement, JSON export, and a multi-owner example. ORCH provides
dependency-aware `next`, `start`, `complete`, context packaging, and an MCP
wrapper. TASKS provides vendor-specific wave and assignment prompts.

Those surfaces coordinate planned work, but not competing execution processes.
`aps start` records only `In Progress`; it does not name a sponsor, actor, run,
workspace, or expiry, and it cannot make one winner observable when separate
processes race. The superseded check-out/check-in and dashboard designs contain
useful prior art, but transient coordination should not expand the canonical
work-item status vocabulary or create mandatory template fields.

## In Scope

- Portable owner / sponsor / actor / run terminology and authority boundaries
- Atomic, expiring work-item claims across local worktrees and separate clones
- Claim renewal, release, stale detection, explicit takeover, and handoff
- Effective team status projected over plan, claim, and delivery evidence
- Conflict-aware `next` and wave planning using dependencies and file hints
- Optional Git/provider adapters for branch, PR, review, CI, and merge evidence
- Versioned JSON projection for team dashboards and external consumers
- Adversarial multi-process and multi-clone fixtures

## Out of Scope

- Running implementation agents or becoming an execution engine
- General agent-to-agent chat, messaging, or prompt transcript storage
- A required hosted APS service, database, or identity provider
- Replacing Git hosting, pull requests, CI, CODEOWNERS, or merge queues
- Automatically merging, deploying, releasing, or expanding task authority
- Bidirectional Jira, Linear, Notion, or project-board synchronisation
- Mandatory team fields in the minimal APS templates

## Interfaces

**Depends on:**

- ORCH — item resolution, dependency enforcement, context packaging, state flow
- INTEGRATIONS — versioned export and provider-adapter boundary
- AGENT — portable agent instructions and human-control rules
- TASKS — wave planning and assignment guidance to generalise
- MONO — path-qualified identity across nested plan trees

**Exposes:**

- [Team coordination design](../designs/2026-07-19-team-coordination.design.md)
- Optional claim/lease CLI surface (names settled by TEAM-000)
- Optional `coordination` projection in JSON status/export
- Team status view: available, claimed, review, blocked, stale, conflicting
- Provider-neutral handoff and delivery-evidence record

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Existing ORCH/TASKS/INTEGRATIONS overlap is identified
- [x] Work items are defined with observable validation
- [ ] TEAM-000 design decisions are approved
- [ ] Q-001 shared transport is resolved by evidence
- [ ] Q-002 declared/effective status semantics are resolved
- [ ] Q-003 actor trust and takeover policy is resolved

## Work Items

### TEAM-000: Ratify the team coordination contract

- **Status:** Draft
- **Intent:** Define the smallest portable contract for multi-human,
  multi-process execution before changing APS commands or templates.
- **Expected Outcome:** The design fixes the owner/sponsor/actor/run model,
  claim lifecycle, authority boundary, effective status, failure semantics,
  transport contract, handoff evidence, and progressive fallback behaviour.
- **Validation:** Every row in the design's Failure and Validation Matrix has
  one unambiguous expected result; Q-001 through Q-003 are answered; ORCH,
  INTEGRATIONS, AGENT, TASKS, and MONO boundaries have no duplicated owner.
- **Design Source:** plans/designs/2026-07-19-team-coordination.design.md,
  plans/issues.md, plans/modules/team-coordination.aps.md
- **Confidence:** medium
- **Dependencies:** None

### TEAM-001: Prove atomic claims in a shared local repository

- **Status:** Draft
- **Intent:** Prevent duplicate execution when processes share a common Git
  directory but run in separate worktrees.
- **Expected Outcome:** A prototype claim store supports claim, renew, release,
  expiry, and explicit takeover; exactly one of two simultaneous claimants wins
  the same item, while independent items remain parallel.
- **Validation:** An automated race harness launches independent processes and
  proves single-winner, non-holder rejection, crash/stale recovery, no partial
  claim, and no plan-file mutation by the losing process.
- **Non-scope:** Production remote/provider integration
- **Confidence:** medium
- **Dependencies:** TEAM-000

### TEAM-002: Prove shared claims across independent clones

- **Status:** Draft
- **Intent:** Give separate humans and their agents one visible coordination
  state without a hosted APS database.
- **Expected Outcome:** Two independent clones coordinate through the selected
  transport with compare-and-set claim creation, renewal, release, expiry, and
  auditable takeover; degraded/offline behaviour is explicit.
- **Validation:** A bare-remote fixture races two clones for one item, permits
  two independent claims, simulates transport loss, and confirms recovery
  without contradictory winners.
- **Confidence:** low
- **Dependencies:** TEAM-000, TEAM-001

### TEAM-003: Add handoff and delivery-evidence projection

- **Status:** Draft
- **Intent:** Let an agent stop, transfer, or submit work without losing the
  concise state another actor or reviewer needs.
- **Expected Outcome:** A provider-neutral handoff records checkpoints,
  workspace, commits, validation, blockers, and takeover safety; optional
  adapters project PR, review, CI, and merge evidence without owning it.
- **Validation:** Fixture covers clean handoff, uncommitted handoff, blocked
  escalation, failed validation, and provider-unavailable fallback; no secret
  or transcript content enters the record.
- **Confidence:** medium
- **Dependencies:** TEAM-000, TEAM-001

### TEAM-004: Expose effective team status and export

- **Status:** Draft
- **Intent:** Make active, stale, conflicting, and review-ready work visible to
  humans, conductors, CI, and dashboards.
- **Expected Outcome:** A CLI view and versioned JSON projection combine
  declared plan status, claims, and available delivery evidence into available,
  claimed, in-review, blocked, stale, and conflicting buckets.
- **Validation:** Deterministic fixture output covers local/shared/off modes,
  nested plans, stale claims, unavailable providers, and consumers that ignore
  the optional coordination object.
- **Confidence:** medium
- **Dependencies:** TEAM-001, TEAM-002, INTEGRATIONS-002

### TEAM-005: Make selection and waves conflict-aware

- **Status:** Draft
- **Intent:** Recommend the next safe work across a team rather than merely the
  first dependency-ready item.
- **Expected Outcome:** Team-aware selection excludes live claims, respects
  dependencies, warns on overlapping file hints and constrained owners, and
  permits explicit action-level cooperation without silently sharing an item.
- **Validation:** A fixture with competing claims, independent work, nested
  plans, overlapping file hints, and action waves produces deterministic
  recommendations and never treats advisory hints as hard dependencies.
- **Confidence:** low
- **Dependencies:** TEAM-001, TEAM-004, TASKS-001

### TEAM-006: Validate the multi-human, multi-agent journey

- **Status:** Draft
- **Intent:** Prove the coordination contract through a realistic team journey,
  not only command-level fixtures.
- **Expected Outcome:** A worked example and harness model at least two humans,
  four agent runs, concurrent claims, a contested item, a crash/takeover, a
  handoff, a pull request review, and completion without plan-file conflicts.
- **Validation:** The journey runs from fresh setup using documented commands;
  APS lint stays clean; every claim and delivery event traces to one work item;
  a file-only project still follows the existing solo workflow unchanged.
- **Confidence:** medium
- **Dependencies:** TEAM-002, TEAM-003, TEAM-004, TEAM-005

## Execution Strategy

1. TEAM-000 resolves the contract and promotion gate.
2. TEAM-001 proves the smallest local atomic primitive.
3. TEAM-002 and TEAM-003 establish distributed coordination and recovery.
4. TEAM-004 and TEAM-005 build visibility and safe selection on proven state.
5. TEAM-006 validates the whole user journey before any release claim.

## Decisions

- **D-001:** Coordination plane — _proposed: transient actor/claim state is
  separate from durable APS intent; no required database or plan-file lock._
- **D-002:** Status vocabulary — _proposed: keep Draft / Ready / In Progress /
  Complete / Blocked; project Claimed / Review / Stale / Conflicting views._
- **D-003:** Identity split — _proposed: module Owner is accountable; Sponsor
  authorises; Actor executes; Run identifies one invocation._
- **D-004:** Claim shape — _proposed: exclusive, renewable, expiring item lease
  with explicit handoff and audited takeover._
- **D-005:** Conflict strictness — _proposed: same-item/dependency conflicts are
  hard; file-hint and owner-capacity conflicts are advisory._
- **D-006:** Provider boundary — _proposed: provider adapters enrich delivery
  evidence but never become the source of APS plan intent._
- **D-007:** Template impact — _proposed: no mandatory template fields; team
  coordination is optional, configured progressive enhancement._

## Open Questions

- [Q-001](../issues.md#q-001-which-shared-claim-transport-should-team-mode-use)
- [Q-002](../issues.md#q-002-how-do-claims-affect-the-effective-and-declared-work-item-status)
- [Q-003](../issues.md#q-003-how-are-actor-identity-lease-expiry-and-takeover-trusted)

## Notes

- Historical prior art:
  [check-out/check-in](../../docs/plans/2026-03-10-checkout-checkin.design.md)
  and [dashboard](../../docs/plans/2026-03-10-dashboard.design.md).
- TEAM-000 is planning/design work only. TEAM-001 and later remain Draft until
  the design gate and open questions are explicitly approved.
