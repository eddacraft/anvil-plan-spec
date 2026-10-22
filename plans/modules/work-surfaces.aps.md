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


### SURF-010: Demonstrate GitHub Issues with optional Projects

- **Intent:** Provide the technical reference for APS-backed engineering
  where issues, Git/PR and CI already coexist.
- **Expected Outcome:** A reproducible GitHub MCP example links an issue to
  an approved APS item or module and uses an available native or independent
  coding agent, Git PR and CI review evidence. Project fields offer richer
  tracking *when available*, without being mandatory.
- **Validation:** Run item- and module-level paths with and without Projects;
  verify access restrictions, setup time, manual re-entry, agent entitlement
  and PR/evidence links. Closing an issue or merging a PR early never changes
  APS completion without qualifying evidence. Pure Git+CLI remains usable.
- **Confidence:** high
- **Dependencies:** SURF-001
- **Status:** Draft

### SURF-011: Demonstrate Jira Cloud enterprise portability

- **Intent:** Test APS against issue structures and approval models that
  differ significantly from both GitHub and Linear.
- **Expected Outcome:** A Jira Cloud baseline uses Atlassian Rovo MCP or
  authorised REST access to link a work item to APS, hand off an approved
  revision, and display objective delivery evidence without requiring
  Forge, A2A, mandatory custom fields or a particular workflow scheme.
- **Validation:** Two different Jira project/status configurations exercise
  transitions, issue types, access limits and absent fields. Missing API
  privileges degrade to read-only/unknown. Rename or move of a Jira item
  never changes its linked APS identity, scope or completion criteria.
  Record setup, licence and administrative friction.
- **Confidence:** medium
- **Dependencies:** SURF-001
- **Status:** Draft

### SURF-012: Recommend a preferred tracker using observed evidence

- **Intent:** Support adoption of the best practical work management
  experience without hard-coding a provider preference into APS.
- **Expected Outcome:** The GitHub/Linear/Jira comparison includes
  per-scenario measurements, weighted scores, availability and cost,
  independent-agent support, confidence and hard-gate failures. It records
  an owner-approved decision: adopt, trial longer, retain incumbent, or no
  winner. Slack and Teams are compared separately as optional overlays.
- **Validation:** Weights total 100, scores cite actual results using the
  design's 0-5 anchors; any untested core requirement is marked unknown.
  A provider failing authority, privacy, portability or completion gates
  cannot win. A score difference below five points is treated as a
  practical tie favouring an incumbent that meets all gates. Document
  switch costs; this work item performs no real migration.
- **Confidence:** medium
- **Dependencies:** SURF-004
- **Status:** Draft

### SURF-013: Verify cross-provider relinking and portability

- **Intent:** Let a team change or combine trackers without rewriting
  approved APS contracts, losing evidence or reauthorising execution.
- **Expected Outcome:** One reviewed APS work item is linked to GitHub,
  Linear and Jira identities and referenced in both Slack and Teams.
  Links can be deactivated, moved and restored independently of Git truth.
- **Validation:** Test GitHub → Linear → Jira → original-provider relinking;
  confirm APS item ID, approval SHA, dependencies, validation, history and
  evidence URLs remain intact. Ambiguous tenant IDs or private provider
  objects fail visibly. Neither a relink nor a chat notification launches
  an agent or creates a second APS item.
- **Confidence:** medium
- **Dependencies:** SURF-001, SURF-002, SURF-010, SURF-011
- **Status:** Draft

### SURF-014: Trial an approved work-surface choice with a real team

- **Intent:** Let an interested team adopt the best evidenced option in
  a bounded, reversible way without automatically migrating its backlog.
- **Expected Outcome:** An accountable owner explicitly authorises a small
  pilot cohort, selected tracker and optional chat overlay. The recorded
  protocol captures current provider, tenant permissions, issue mapping,
  existing URLs, agent availability, rollback steps and post-adoption effort.
  The public APS format and alternate integrations remain available.
- **Validation:** Approval is recorded *before* external writes. Dry-run
  migration enumerates duplicate risks and remapping; one real planning to
  review cycle succeeds; old issues, APS Git revision and PR/CI evidence
  remain retrievable; a rollback exercise restores the previous working
  surface without duplicating work. Expansion requires separate sign-off.
- **Confidence:** medium
- **Dependencies:** SURF-012
- **Status:** Draft
- **Approval Gate:** No live backlog, membership or agent changes without
  explicit owner, cohort and rollback approval

### SURF-015: Implement an optional GitHub provider adapter if needed

- **Intent:** Remove proven repetitive linking/status chores for Issues
  and optional Projects, without a mandatory GitHub project schema.
- **Expected Outcome:** If separately approved, a removable adapter handles
  issue/APS identity, idempotent mapping, attributable PR/CI evidence,
  permission-aware views, reconciliation and a dry-run. An Issues-only
  install remains supported when Projects is absent.
- **Validation:** Fixtures include Issues-only, optional Projects,
  cross-repo work, deleted links, differing issue/project statuses,
  duplicate events, permissions revocation and merged-not-shipped work.
  Retries do not duplicate remote writes or advance APS completion; the
  core CLI/Markdown behaves identically when the adapter is uninstalled.
- **Confidence:** low
- **Dependencies:** SURF-001, SURF-004
- **Status:** Draft
- **Approval Gate:** Recorded evidence of value and explicit separate approval

### SURF-016: Implement an optional Jira Cloud adapter if needed

- **Intent:** Reduce measured enterprise integration friction without
  importing Jira field/status semantics into the APS specification.
- **Expected Outcome:** If separately approved, a removable adapter supports
  tenant-scoped issue links, schema/field discovery, controlled status
  projections, idempotent retries and reconciliation. A native Forge/A2A
  remote-agent integration is an independent enhancement, not the baseline.
- **Validation:** Fixtures cover multiple Jira workflow schemes,
  inaccessible or missing fields, forbidden transitions, duplicate/out-of-
  order webhook events, revoked credentials and cross-tenant access.
  Plan approval and completion remain Git/evidence-governed. A non-Jira
  project needs no Jira packages or credentials.
- **Confidence:** low
- **Dependencies:** SURF-001, SURF-004
- **Status:** Draft
- **Approval Gate:** Recorded evidence of value and explicit separate approval

### SURF-017: Demonstrate Teams collaborative APS handoff

- **Intent:** Prove Microsoft's existing team chat can host an APS-backed
  multi-agent working conversation without a required custom APS app.
- **Expected Outcome:** A repeatable normal Teams channel/thread or group
  chat journey references an approved APS item and any GitHub/Linear/Jira
  issue. Participants can select existing *actually installed* agents,
  request a separate review where permitted and share relevant scope,
  blocker and Git/CI evidence links. It works as a link-only workflow in
  tenants blocking custom agents.
- **Validation:** Test one task with agent apps enabled and one with apps
  blocked or unavailable, documenting setup/tenant admin steps, group-chat
  versus channel behaviour and unsupported private/shared channel features.
  Verify that an @mention grants neither scope nor an exclusive claim,
  a changed requirement needs Git review, and no unconsented chat content
  or inaccessible private thread is imported into an agent context.
- **Confidence:** medium
- **Dependencies:** SURF-001
- **Status:** Draft

### SURF-018: Add an optional Microsoft Teams app if needed

- **Intent:** Provide richer conversation links, status and decisions only
  when the measured value outweighs tenant and deployment complexity.
- **Expected Outcome:** If separately approved, a removable Teams app
  built on supported Microsoft Teams/Agents SDK features can resolve APS
  references, respond to explicit mentions, optionally publish attributed
  progress or Adaptive Cards and link evidence. It must not require
  Copilot Studio, Teams Channel Agent or broader conversation access.
- **Validation:** Fixtures test bot installation/revocation, tenant isolation,
  explicit @mention routing, RSC denial, repeated activities, cross-channel
  leakage, inaccessible private/shared channels and unknown agent state.
  A standard-channel link-only journey and pure APS usage still work
  without the app. No message alone authorises execution or plan changes.
- **Confidence:** low
- **Dependencies:** SURF-001, SURF-004, SURF-017
- **Status:** Draft
- **Approval Gate:** Requires evidence-based separate implementation approval

## Execution and Adoption Strategy

**Phase A — contract and working native examples**

1. Ratify the provider-neutral authority/link contract (SURF-000/001).
2. Build **GitHub Issues/Projects** and **Linear** manual/MCP journeys in
   parallel (SURF-010/002), measuring technical reliability and usability.
3. Prove Jira Cloud against the same contract (SURF-011), and run independent
   **Slack** and **Teams** conversational pilots (SURF-003/017).
4. Prove changing/adding issue trackers preserves APS identity, approval and
   evidence (SURF-013).

**Phase B — evidence, selection and portable release**

1. Compare normal engineering tasks across GitHub, Linear, Jira and a pure
   Git+CLI control; compare Slack and Teams separately (SURF-004).
2. Publish a 100-point weighted tracker evaluation with hard safety gates,
   raw measurements and confidence, and record the team's explicit
   **adopt / keep incumbent / extended pilot / no winner** decision
   (SURF-012). GitHub is the technical reference, not the automatic winner.
3. Publish usable no-custom-adapter recipes for all tested providers and
   a provider-free path (SURF-005).
4. An interested team may *separately authorise* a small live adoption
   cohort with a dry-run, original-link retention and rollback (SURF-014).
   No live migration is authorised by this planning PR.

**Phase C — optional engineering only if shown to be worthwhile**

1. Add a richer read-only APS item/evidence projection only for proven
   gaps (SURF-006).
2. Independently approve any needed GitHub (SURF-015), Linear (SURF-007),
   Jira Cloud (SURF-016), Slack (SURF-008) or Teams (SURF-018) adapter.
3. Apply provider-neutral security, idempotency, authority and recovery
   conformance to any integration actually shipped (SURF-009).

Each optional adapter has its own approval gate. If the native/MCP journey is
good enough, record a no-build decision and ship documentation alone. The
adopting team's preferred tracker and chat app do not constrain the public
APS specification or require others to migrate.

## Decisions (proposed; ratify through SURF-000)

- **D-001:** Work management and chat are optional APS surfaces; Git/Markdown
  remains the approved source of planning intent and criteria.
- **D-002:** Public APS stays executor neutral; never bundle eddaskills,
  Anvil, a hosted runtime, a new agent supervisor or vendor-only credentials.
- **D-003:** GitHub Issues/Projects is the **technical reference**, Linear
  the **usability benchmark**, Jira Cloud the **enterprise interop test**.
  Those are evaluation roles, not a preselected winner.
- **D-004:** Slack and Microsoft Teams are **conversational overlays**
  usable with any tracker or a raw APS item, never required for core APS.
- **D-005:** Links to APS modules and work items are versioned and portable;
  no duplication of every APS action/checkpoint into external tickets.
- **D-006:** An agent mention, task assignment or Model/Reasoning hint is
  not a claim, approval, validation result or elevated authority.
- **D-007:** Use real Git/CI evidence and explicit unknown statuses; a
  closed issue or merged PR cannot silently mark APS Complete.
- **D-008:** TEAM owns atomic claims; PROG owns progress storage; consume
  ratified projections rather than implementing substitutes here.
- **D-009:** Native workflows and existing MCP precede optional provider
  adapters. Preview/partner-only APIs are never baseline requirements.
- **D-010:** Select an adopting team's preferred tracker only after common
  scenarios, weighted scoring, hard gates and human approval, with a
  reversible small-cohort pilot; the public APS contract stays agnostic.
- **D-011:** Compare Teams and Slack *separately* for conversation usability,
  tenant policy and installed agents; do not score them as issue trackers.

## Open Questions

- Where should external work references live so nested plans and multiple
  workspace IDs do not pollute minimal APS templates?
- How is the approval of an exact reviewed Git revision verified when an
  external issue or chat message initiates a run?
- Does the existing Markdown plus `aps export --json` suffice for all
  tested providers, or is a richer item-level read justified?
- How should GitHub Projects fields, Linear statuses and Jira transitions
  map to derived evidence without overriding APS states?
- Which agent/claim-warning is mandatory while TEAM is Draft?
- What native agent capabilities are **actually** enabled in test accounts?
- Does the user evidence warrant any installed integration at all?
- Which tracker and chat product does the adopting team already use? What
  are its account, licensing, admin and switching constraints?
- Can switching GitHub → Linear → Jira → GitHub preserve an APS work item's
  authorised revision and evidence as Slack/Teams threads reference it?
- Which Teams channel/group chat app permissions and RSC policies are
  granted, and what useful fallback remains when agent apps are blocked?

## Design and References

- [Portable work surfaces design](../designs/2026-10-22-work-surfaces.design.md)
- [Current APS JSON export](../../docs/integrations.md)
- [TEAM coordination module](./team-coordination.aps.md)
- [PROG progress module](./progress-journal.aps.md)
- [GitHub MCP](https://github.com/github/github-mcp-server)
- [GitHub agentic workflows](https://docs.github.com/en/copilot/concepts/agent-capabilities/agent-powered-workflows-and-operations/about-github-agentic-workflows)
- [Linear official MCP](https://linear.app/docs/mcp)
- [Atlassian Rovo MCP](https://developer.atlassian.com/cloud/rovo-mcp/guides/getting-started/)
- [Jira Cloud remote agents](https://developer.atlassian.com/platform/forge/remote-agents-in-jira/)
- [Slack Code](https://docs.slack.dev/ai/slack-code/)
- [Teams channel/group chat agent messages](https://learn.microsoft.com/en-us/microsoftteams/platform/bots/how-to/conversations/channel-and-group-conversations)
- [Teams SDK extension](https://learn.microsoft.com/en-us/microsoft-365/agents-sdk/teams/teams-extension)
- [Teams app/agent consent and administration](https://learn.microsoft.com/en-au/MicrosoftTeams/teams-app-setup-policies)

All SURF work items remain **Draft**. This PR requests review of planning,
not permission to implement provider adapters or migrate live issues.
