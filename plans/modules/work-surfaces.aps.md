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

### SURF-000: Ratify the work-surface contract

- **Intent:** Approve the smallest portable integration boundary that adds
  user value without granting work surfaces or agents planning authority.
- **Expected Outcome:** The linked design is accepted or explicitly amended;
  the ownership matrix, external reference rules, approved-revision binding,
  provider status mapping, failure semantics and adoption gates are unambiguous.
  Open questions have written v1 defaults. No bundled harness or hosted
  dependency is introduced.
- **Validation:** Design review resolves each open question and checks the
  failure matrix; `aps lint plans` has no errors; an agent without Linear,
  Slack or eddaskills can identify an existing Ready item using APS alone.
- **Confidence:** high
- **Dependencies:** None
- **Status:** Draft

### SURF-001: Specify portable external work references and handoffs

- **Intent:** Make an external ticket, conversation or executor refer to the
  correct APS work and reviewed revision without duplicating its authority.
- **Expected Outcome:** A small versioned reference/profile contract handles
  repository, plan root, federated child, item/module ID, reviewed Git
  revision, provider/workspace/object identity and relationship cardinality.
  Optional link storage, ownership of each field, conflict/relink behaviour,
  and unavailable-source treatment are documented. Minimal APS templates
  remain unchanged.
- **Validation:** Fixtures cover two Linear issues linked to one module,
  one Linear issue plus one Slack thread linked to one item, renamed external
  objects, duplicate external IDs in different workspaces, missing providers,
  and an unapproved revision. All links resolve or return an explicit
  ambiguity/error; neither rewrites APS intent nor needs provider secrets.
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

### SURF-004: Measure real usability and cross-surface reliability

- **Intent:** Decide from evidence whether the public pattern genuinely
  improves adoption and reduces work instead of increasing administration.
- **Expected Outcome:** A documented pilot tests a small bug, multi-item
  feature and merged-but-not-shipped change across Linear, Slack, a combined
  linked journey and provider-free Git/CLI control. It includes product,
  engineering and reviewer perspectives, an executor switch, recovery from
  missing provider context and a duplicate-execution risk disclosure.
  Results compare manual entry, setup/handoff time, interventions, status
  mismatch and ability to find evidence.
- **Validation:** Pilot records the actual measurements against the design's
  targets; no-proprietary-harness and no-provider paths both succeed; zero
  silent scope rewrites and zero false Complete claims in the exercised
  cases. A written go/no-go identifies whether any optional adapter or
  enhanced APS export is justified, including a valid decision to stop.
- **Confidence:** medium
- **Dependencies:** SURF-002, SURF-003
- **Status:** Draft

### SURF-005: Publish a tool-neutral adoption pattern

- **Intent:** Make APS plus existing team tools approachable without requiring
  readers to learn a new agent orchestration process.
- **Expected Outcome:** Public docs give concise setup, authority and
  troubleshooting guidance for Linear MCP, Slack threads, agent-selection
  hints, a Git/CLI-only workflow and linked Linear+Slack use. Each example
  separates supported behaviour from proposed adapter capabilities and
  labels Slack Code beta/Linear agent preview limits.
- **Validation:** A person unfamiliar with this repository can run the
  three baseline journeys from the docs, link the relevant APS contract,
  understand who owns each decision, and complete setup in no more than
  15 minutes after normal provider access exists (measured target). All
  local links resolve, APS lint passes and no guide instructs readers to
  install eddaskills, Anvil or a hosted APS service.
- **Confidence:** high
- **Dependencies:** SURF-004
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

### SURF-009: Conformance-test any approved provider adapter

- **Intent:** Make optional integrations safe, predictable and replaceable
  without making APS responsible for executing agents.
- **Expected Outcome:** A reusable contract test matrix checks every approved
  provider adapter against authority, identity, visibility, idempotency,
  failure recovery, security, no-provider fallback and vendor portability.
  It explicitly distinguishes observed CI/PR facts from self-reported agent
  activity and from authorised APS completion.
- **Validation:** For each shipped adapter, adversarial fixtures prove
  repeated/stale events cannot start duplicate work or overwrite approved
  scope; secrets stay out of logs; cross-workspace access is denied; degraded
  source status is `unknown` rather than fabricated; unauthorised users
  cannot approve a different revision. All existing APS CLI/MCP workflows
  remain green without the provider. If no adapter is approved, this item
  remains Draft rather than manufacturing infrastructure.
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
