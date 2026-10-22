# Work Surfaces Module — GitHub, Linear, Jira, Slack, Teams and BYO Agents

| ID   | Owner  | Priority | Status |
| ---- | ------ | -------- | ------ |
| SURF | @aneki | medium   | Draft  |

**Last reviewed:** 2026-10-23

## Purpose

Make APS easy to adopt from existing structured work trackers — GitHub Issues
and optional Projects, Linear and Jira Cloud — and conversational team
surfaces — Slack and Microsoft Teams — without turning APS into a mandatory
project manager, agent execution engine or hosted service.

GitHub is the technical reference; Linear is the usability benchmark; Jira
Cloud tests enterprise interoperability. Slack and Teams are alternative
chat overlays usable with any tracker or with an APS item alone. Compare
options empirically and let an adopting team *optionally* select a preferred
provider through a separately authorised, reversible trial. No winner
is chosen in advance.

## Background

APS already supplies Git-native plans, an optional MCP server,
`aps export --json` and a multi-harness planning interface. Rather
than recreate issue trackers, chat, coding agents or execution loops, this
module documents portable references, authority boundaries, usable
native/MCP workflows and conditional integrations.

Public APS must support:

- GitHub issues and optional Projects, with direct connections to PRs and CI.
- Linear's ordinary issue workflow and its optionally available agent APIs.
- Jira Cloud's varied workflow schemes and Atlassian Rovo MCP.
- Slack or Teams discussions involving existing agents, without requiring
  a custom APS chat bot.
- Independent native agents, custom orchestrators, human delivery, or none.
- eddaskills as an *internal eddacraft option*, **not** a public APS runtime,
  required dependency or preferred implementation. Anvil also stays optional.

Existing module ownership:

- [INTEGRATIONS](./integrations.aps.md) — current versioned export and Action.
- [ORCH](./orchestrate.aps.md) — work item readiness, context and transitions.
- [SPEC](./spec.aps.md) — canonical statuses and model/reasoning hints.
- [TEAM](./team-coordination.aps.md) (Draft) — actor identity, atomic
  claims, leases and handoffs.
- [PROG](./progress-journal.aps.md) (Draft) — journalled progress.
- [AGENT](./agents.aps.md) — existing APS agent and skill scaffolds.

See [the work-surfaces design](../designs/2026-10-22-work-surfaces.design.md)
for vendor capability constraints, comparison, security and adoption gates.

## Desired Outcomes

- Any supported tracker issue can link an APS item or module, with
  Git-approved scope and validation remaining canonical.
- Normal provider interfaces display meaningful progress, decisions and
  independently verifiable evidence without duplicating acceptance contracts.
- Team chat in Slack or Teams can coordinate different agents and reference
  the same APS item. A mention is not a work claim or permission grant.
- A task can reference GitHub, Linear and Jira simultaneously, then switch
  or restore its preferred tracker without changing APS identity or review.
- The same public APS format works with no external provider, eddaskills,
  Anvil, account or new execution framework.
- A comparable real-user pilot identifies a preferred tracker *if one earns
  that recommendation*, or preserves the incumbent if differences are small.
- An adopting team may trial the chosen work surface with explicit human
  permission, bounded cohort, evidence and rollback.
- API-backed provider adapters are conditional on measured value, not an
  automatic outcome of opening this plan.

## In Scope

- Portable work-surface reference, relationship and handoff semantics
- Manual/MCP-first GitHub, Linear and Jira Cloud reference journeys
- Slack and Teams conversation/thread examples for existing coding agents
- Optional provider-native rich experiences, only when access allows
- Issue-to-module and issue-to-item linkage; no issue explosion
- Reviewed-revision binding and attributed evidence/status projection
- Multi-provider association and reversible relinking
- Matched user journeys and a predeclared weighted 100-point tracker scorecard
- Separate, lightweight comparison of Slack versus Teams chat ergonomics
- Safety gates, optional preferred-provider decision and scoped adoption pilot
- Optional richer APS export and provider-specific adapters where justified
- Provider-neutral failure/permissions conformance for implemented adapters

## Out of Scope

- Bundling eddaskills, Anvil or another agent runtime in public APS
- Building, selecting or operating coding models, multi-agent supervisors,
  verification engines or a hosted APS agent execution service
- Replacing GitHub, Linear, Jira, Slack, Teams, CI or project-board UIs
- Required hosted APS tenant, service, database, new subscription or secrets
- Core APS template fields for particular vendor issue IDs or chat threads
- Assuming any closed issue, @mention, agent reply, PR merge or project
  field update is APS approval, exclusive claim or verified completion
- Universal bidirectional plan sync or mirroring every agent action to chat
- Automatic mass ticket creation or migration of a live production backlog
- Reimplementing TEAM leases, PROG journals or a messaging bus
- Depending on provider previews or restricted agent APIs for basic use
- Treating Jira Data Center as validated by Jira Cloud coverage

## Interfaces and Constraints

**Depends on:**

- INTEGRATIONS-002 — present versioned APS export (Complete)
- ORCH — core readiness and dependency flow (Complete)
- SPEC — canonical work-item state vocabulary and routing hints
- TEAM/PROG — *future* optional projections only; not initial pilot blockers

**Exposes (subject to SURF-000 ratification):**

- Git/Git-revision-bound portable external references and link resolution
- Provider-owned work metadata vs APS-owned intent/evidence boundaries
- Git/CLI-only, GitHub, Linear, Jira, Slack and Teams worked patterns
- Provider-specific capability profiles without forced uniformity
- Measured pilot, recommendation and human-approved adoption protocol
- Optional separately installed adapters and reusable conformance tests

**Constraints:**

- Git/Markdown is always readable and authorable without providers.
- No new mandatory network calls, agent, permission or account in APS core.
- Native provider workflows and MCP are tested before custom adapter code.
- No chat mention or ticket assignment upgrades APS authorisation.
- A status projection must state its source, evidence and unknowns.
- Teams app installation/RSC, Slack Code, Linear agents, GitHub preview agents
  and Jira Forge/A2A are *optional* availability-dependent enhancements.
- APS CLI/lint surface changes require D-039 Rust/bash/PowerShell parity.
- A team adoption or external write requires separate express approval.

## Ready Checklist

- [x] Problem, audiences and staged adoption path documented
- [x] Existing module ownership and no-harness boundary documented
- [x] GitHub/Linear/Jira and Slack/Teams capability limitations recorded
- [x] Comparable rubric, safety gates and opt-in adoption process designed
- [x] Testable proposed work items and dependencies identified
- [ ] SURF-000 contract, open questions and v1 defaults ratified
- [ ] Test accounts, incumbent tracker/chat, permissions and pilot cohort
      recorded before the real-user trials
- [ ] Accountable owner promotes selected work items from Draft before build

## Work Items

### SURF-000: Ratify the work-surface and selection contract

- **Intent:** Fix the smallest portable APS/tracker/chat boundary before
  building provider-specific code or selecting a team's work surface.
- **Expected Outcome:** Review and ratify authority, approved Git revision,
  external link identity, ownership of ticket fields, evidence projections,
  cross-provider semantics, failure handling, scorecard and adoption gates.
  Explicitly confirm GitHub reference / Linear UX / Jira portability and
  Slack+Teams overlays are testing roles, not preselected winners.
- **Validation:** Design questions have documented v1 decisions; every
  scenario in its failure matrix yields a definite safe result; no provider,
  agent runtime, new APS hosting or vendor field becomes a mandatory core
  dependency; `aps lint plans` remains clean.
- **Confidence:** high
- **Dependencies:** None
- **Status:** Draft

### SURF-001: Specify portable references and execution handoffs

- **Intent:** Let any tracker, chat or executor refer to the correct APS
  item and reviewed Git revision without owning that plan's approval.
- **Expected Outcome:** An optional versioned contract covers repo, plan
  root, federated child, item/module ID, reviewed Git revision and external
  provider/workspace/object identity. It permits many-to-many links
  (GitHub, Linear, Jira, Slack, Teams) without storing provider credentials
  or making new fields mandatory in APS Markdown.
- **Validation:** Fixtures link one APS module to GitHub/Linear/Jira issues,
  and a single APS work item to all three issues plus Slack and Teams
  conversations; rename/relink, duplicate tenant IDs, missing permissions,
  cross-repo/nested plans and stale approval revisions yield unambiguous
  results. The canonical APS scope is never rewritten by relinking.
- **Confidence:** medium
- **Dependencies:** SURF-000
- **Status:** Draft

### SURF-002: Prove the zero-adapter Linear MCP journey

- **Intent:** Let a team plan and execute APS-backed work from Linear using
  its official MCP and an executor it already has.
- **Expected Outcome:** A repeatable worked example covers creating/selecting
  an issue, linking it to an existing item or a reviewed plan PR, delegating
  to a native Linear agent or another coding tool, recording PR/test evidence,
  and reflecting blockers without false Complete claims. A module-level
  issue and an individual-item issue are both demonstrated. No custom
  Linear app, hosted controller or eddaskills installation is required.
- **Validation:** A fresh-project walkthrough succeeds with Linear MCP,
  the public APS CLI/Markdown and an independent native executor; record
  observed steps/time and manual data re-entry. An issue closed ahead of
  actual APS completion leaves the APS contract unchanged and visibly
  discrepant. Read-only MCP can browse without granting write access.
- **Confidence:** medium
- **Dependencies:** SURF-001
- **Status:** Draft

### SURF-003: Prove the Slack shared-team conversation journey

- **Intent:** Let a team involve different coding agents from one familiar
  chat space while preserving a single approved APS contract.
- **Expected Outcome:** A worked example uses a shared team channel for
  intake, threads for small tasks and optionally one Slack Code channel per
  longer task where already available. It demonstrates referencing APS,
  selecting an existing coding agent by mention, using a separate verifier,
  surfacing decisions and links, and summarising back to the originating
  conversation. The baseline requires no APS Slack bot or partner-only API.
- **Validation:** Walkthrough completes on ordinary Slack messages/threads
  with existing agents and without Slack Code API privileges. A second agent
  does not acquire exclusive ownership by mention; a changed requirement
  is proposed for Git review rather than silently implemented. The original
  team channel stays navigable across at least three concurrent tasks.
- **Confidence:** medium
- **Dependencies:** SURF-001
- **Status:** Draft

### SURF-004: Compare real usability across tracker and chat options

- **Intent:** Determine whether the pattern reduces administration and
  which *structured* work tracker a real team could productively adopt.
- **Expected Outcome:** A reproducible pilot runs the same small bug,
  multi-item feature and merged-not-shipped item using GitHub Issues/Projects,
  Linear and Jira Cloud, with a Git+CLI-only control. A separate paired
  comparison tests Slack and Teams as conversational overlays. Capture a
  product/lead, developer and reviewer; agent switching, provider outage,
  explicit claim uncertainty, setup time, manual re-entry, evidence
  findability, permissions, actual cost and participant clarity.
- **Validation:** Publish raw measurements plus a confidence-labelled,
  predeclared weighted 100-point tracker scorecard with hard safety gates.
  Both chat overlays work with any linked tracker and APS-only references,
  or limitations are explicitly reported. No false Complete, silent scope
  rewrite or unauthorised execution occurs. Findings identify whether
  extra APS export support or any provider adapter is worthwhile.
- **Confidence:** medium
- **Dependencies:** SURF-002, SURF-003, SURF-010, SURF-011, SURF-013, SURF-017
- **Status:** Draft

### SURF-005: Publish multi-provider, tool-neutral adoption recipes

- **Intent:** Let people use APS in familiar tracker/chat tools in minutes
  without adopting an unrelated agent execution framework.
- **Expected Outcome:** Public quickstarts cover GitHub Issues and optional
  Projects, Linear official MCP, Jira Cloud Rovo MCP, Slack channels/threads,
  Teams standard-channel threads/group chat, and pure Git+CLI. They cover
  issue-to-module, issue-to-item, changed scope, agent selection, reporting
  evidence and reversible tracker relinking. Preview/tenant access caveats
  are visibly documented.
- **Validation:** A newcomer with existing provider access can follow each
  baseline recipe in 15 minutes or less (test target). No recipe asks them
  to install eddaskills, Anvil, an APS hosted controller or a mandatory bot.
  All links resolve; APS and Markdown linters pass; advanced restricted
  features are never presented as baseline requirements.
- **Confidence:** high
- **Dependencies:** SURF-004, SURF-012
- **Status:** Draft

### SURF-006: Add a richer APS handoff projection only if needed

- **Intent:** Remove proven context-lookup friction for external agents
  without forcing a new schema or runtime on simple Markdown users.
- **Expected Outcome:** If SURF-004 demonstrates a concrete gap, a versioned
  additive read representation exposes one reviewed item's intent, expected
  outcome, validation, dependencies, plan revision and evidence references;
  missing facts remain explicitly unknown. Otherwise, document why the
  existing export plus Markdown is sufficient and do not add code.
- **Validation:** Decision cites SURF-004 evidence. If implemented, fixtures
  cover nested plans, version/revision mismatch, unknown evidence,
  read-only/no-provider operation and byte-stable projection; every affected
  Rust, bash and PowerShell CLI surface passes the D-039 parity contract.
  No read operation changes APS status or launches agents.
- **Confidence:** medium
- **Dependencies:** SURF-004
- **Status:** Draft
- **Non-scope:** Claims, runtime sessions, provider credentials

### SURF-007: Build an optional reliable Linear adapter if justified

- **Intent:** Automate only the proven repetitive parts of Linear/APS linking
  and visibility while preserving independent source ownership.
- **Expected Outcome:** A separately installable adapter, if approved by
  pilot results, supports stable external IDs, idempotent link/update,
  configurable status projection, explicit review on changed acceptance
  intent, permission-scoped access, reconciliation after outages, and
  a no-write dry-run. Native Linear agent delegation/API use is a separately
  gated enhancement, not required for baseline operation.
- **Validation:** Recorded fixtures cover duplicate/out-of-order webhook
  events, conflicting issue edits, removed access, renamed issues, nested
  projects, module vs item granularity and merged-but-not-shipped work.
  Replaying input creates no extra issues or runs. APS plan text never
  changes based solely on external tickets or agent activity; unconfigured
  installs remain fully functional without the adapter.
- **Confidence:** low
- **Dependencies:** SURF-001, SURF-004
- **Status:** Draft
- **Approval Gate:** Only promote after a recorded SURF-004 go decision

### SURF-008: Add optional Slack app/Code integration if justified

- **Intent:** Reduce repetitive chat reporting without making an APS bot,
  model selector or Slack Code API a prerequisite for collaboration.
- **Expected Outcome:** If validated and separately approved, a Slack app
  links originating threads, APS items, activity and evidence, respects
  workspace/tenant visibility, and presents actionable human questions.
  Slack Code channel properties, views and commands are optional enhancements
  only where documented provider access is actually available.
- **Validation:** Standard-thread flow still works without the app or beta
  APIs; repeated mentions/events do not duplicate sessions; two unrelated
  tasks stay isolated; private context is never posted across channels;
  revocation and unknown provider state fail visibly; users can switch
  coding agents without changing APS. A normal APS-only path is unaffected.
- **Confidence:** low
- **Dependencies:** SURF-001, SURF-004
- **Status:** Draft
- **Approval Gate:** Only promote after a recorded SURF-004 go decision

### SURF-009: Conformance-test approved provider adapters

- **Intent:** Keep optional GitHub, Linear, Jira, Slack and Teams adapters
  interchangeable, safe and truthful about actual execution.
- **Expected Outcome:** A reusable provider-adapter contract matrix covers
  stable identity, reviewed Git revision, tenant and agent permissions,
  event idempotency, status/evidence provenance, provider outages,
  relinking, privacy and provider-free fallback. Unsupported advanced
  app/agent capabilities degrade visibly rather than silently.
- **Validation:** For every adapter *actually approved and shipped*,
  adversarial tests verify duplicate/out-of-order events never create
  duplicate items/runs, a different tenant cannot access hidden data,
  forged chat instructions cannot expand scope, an issue/project state
  cannot silently mark APS Complete, and provider shutdown leaves Git/CLI
  viable. If no adapter is approved, record why and leave implementation
  unstarted.
- **Confidence:** medium
- **Dependencies:** SURF-001, SURF-004
- **Status:** Draft

## Execution and Adoption Strategy

**Phase A — prove value with what's already available**

1. Ratify the design and authority model (SURF-000).
2. Define a minimal work reference that can be documented without code
   (SURF-001).
3. Run Linear MCP and Slack conversation pilots in parallel (SURF-002/003).
4. Compare with provider-free operation and record measurable friction
   (SURF-004).
5. Publish the useful worked patterns (SURF-005). This can be the final
   release outcome.

**Phase B — optional engineering only after proof**

1. Decide if missing APS item-level export data warrants additive read
   support (SURF-006).
2. Approve Linear and/or Slack adapters individually (SURF-007/008), only
   where removal of manual work clearly outweighs maintenance/permission
   costs.
3. Apply provider-neutral failure/security conformance to anything actually
   shipped (SURF-009).

If SURF-004 shows the native MCP/agent workflow is already good enough,
record the result and close the optional branch as not planned. Do not
reinterpret a Draft adapter item as permission to build it.

## Decisions (proposed; ratify through SURF-000)

- **D-001:** Human work surfaces are optional; APS/Git owns approved intent.
- **D-002:** Public APS stays executor-neutral; no eddaskills or Anvil bundle,
  native-agent requirement, hosted controller or mandatory plugin.
- **D-003:** Start from Linear MCP and normal Slack threads; adapter code
  comes after measured usability evidence.
- **D-004:** Support issue-to-module and issue-to-item links; never mirror
  every APS checkpoint into external tracking.
- **D-005:** Agent mentions and optional Model/Reasoning fields are routing
  hints, not claim, authorisation or evidence of completed work.
- **D-006:** Adopt TEAM/PROG outputs when ratified, not their ownership.
- **D-007:** Provider-specific preview/beta APIs are optional enhancements;
  unsupported environments retain a complete baseline flow.

## Open Questions

- How is a portable link mapping stored without forcing provider fields into
  every APS module and how are link conflicts resolved?
- Which revision approval proof is sufficient for provider-originated work?
- Does the pilot justify a richer read-only APS item projection?
- How should providers display review/integration/release states without
  changing canonical status tokens?
- Which, if any, provider adapter is justified by measured manual effort?
- What warning is required when exclusive coordination is not available?

## Design and References

- [Portable work surfaces design](../designs/2026-10-22-work-surfaces.design.md)
- [Existing APS integration export](../../docs/integrations.md)
- [TEAM coordination module](./team-coordination.aps.md)
- [PROG progress module](./progress-journal.aps.md)
- [Public Linear MCP docs](https://linear.app/docs/mcp)
- [Slack Code developer docs](https://docs.slack.dev/ai/slack-code/)

All work items above remain Draft. This PR requests review of a plan, not
authorisation to implement provider adapters.
