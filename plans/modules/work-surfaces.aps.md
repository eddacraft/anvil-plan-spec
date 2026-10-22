# Work Surfaces Module — Linear, Slack and BYO Agents

| ID   | Owner  | Priority | Status |
| ---- | ------ | -------- | ------ |
| SURF | @aneki | medium   | Draft  |

**Last reviewed:** 2026-10-22

## Purpose

Make APS easy to use from existing issue trackers, team conversations and
native coding agents without turning APS into a project manager, hosted agent
runtime or mandatory integration service. Publish a provider-neutral external
work reference and low-ceremony adoption pattern, prove it with Linear and
Slack, then consider optional adapters only when they add measurable value.

## Background

APS already has Git-native plans, a multi-harness planning interface, the
optional APS MCP server and `aps export --json`. That is enough to explore
work-surface integrations without re-implementing execution.

The team wants a public pattern where:

- Linear can organise and delegate work while APS keeps the approved intent
  and validation contract in Git.
- A Slack team conversation can mention different coding agents to plan,
  execute or review the *same* APS-backed work, with a thread or task-specific
  code channel providing collaborative visibility.
- A team can bring its own agent, custom harness or direct human workflow.
- eddaskills remains an optional **internal eddacraft** harness, not bundled,
  installed or required by public APS. Anvil is similarly optional.
- The result reduces duplicate administration rather than making people
  maintain parallel backlogs and statuses.

Existing ownership constrains this module:

- [INTEGRATIONS](./integrations.aps.md) owns the present versioned
  `aps export --json` and GitHub Action; consume and extend only through
  its documented interface.
- [ORCH](./orchestrate.aps.md) owns readiness, dependency-aware item selection
  and the APS status transition/CLI.
- [TEAM](./team-coordination.aps.md) (Draft) owns actor/run identity,
  atomic claims, leases and cross-process handoffs; no new claims here.
- [PROG](./progress-journal.aps.md) (Draft) owns progress journals and effective
  progress for standing backlogs; no new progress database here.
- [AGENT](./agents.aps.md) owns existing APS agent/skill scaffolding;
  no additional bundled execution agent is required.

The proposed approach and concrete failure/UX cases are in
[the work-surface design](../designs/2026-10-22-work-surfaces.design.md).

## Desired Outcomes

- Existing Linear users can find/link authorised APS work from their normal
  issue flow without copying all acceptance criteria into two systems.
- Slack users can coordinate distinct coding agents from a shared discussion
  without treating a mention as an execution permission or exclusive claim.
- Native Linear/Slack agents and arbitrary third-party harnesses consume APS
  without installing eddaskills, Anvil or a dedicated APS execution service.
- Plan scope, authority and outcome validation stay reviewable in Git; chat and
  backlog statuses are attributed projections with evidence links.
- A no-provider local-first project continues to behave identically.
- Evidence from a real usability pilot determines whether an API-based adapter
  is worth building; documentation-only is a legitimate successful outcome.

## In Scope

- An optional, provider-neutral external work reference and handoff boundary
- Human-first Linear MCP and Slack conversation/code-channel examples
- A clear mapping of issue/module/item granularity and evidence/status projection
- Gradual adoption with no required extra account, hosted service or bot
- A comparative user-experience pilot with explicit adoption/safety criteria
- Conditional, separately approved richer APS read/projection and provider apps
- Idempotent, permission-safe provider adapter contract and conformance tests
- Docs for native-agent, custom-agent and no-agent use

## Out of Scope

- Bundling eddaskills, Anvil or another runtime into public APS
- Building or choosing coding models, native agents, orchestration loops,
  independent verification engines or a new multi-agent supervisor
- Replacing Linear, Slack, GitHub, CI or their native execution workflows
- Requiring a hosted APS tenant, server, database or subscription
- Making Slack/Linear fields mandatory in minimal APS templates
- Treating tickets/chat/agent sessions as plan approval, verified completion
  or authorisation for extra work
- Universal two-way sync, automatic mass ticket creation or mirroring each
  agent checkpoint as a chat message
- Building TEAM claims, PROG journals or a general-purpose messaging bus
- Depending on Slack Code beta APIs or Linear agent preview APIs for the
  initial public integration pattern

## Interfaces and Constraints

**Depends on:**

- INTEGRATIONS-002 — current JSON export (already Complete)
- ORCH — work-item status/dependency contract (already Complete)
- SPEC — core APS vocabulary and optional model/reasoning hints (existing)
- TEAM and PROG — future optional projections only; neither is a prerequisite
  for the initial manual/read-only examples

**Exposes (subject to SURF-000):**

- Provider-neutral external work references and authority/evidence mapping
- A Git/APS-only workflow and a manual provider-assisted workflow
- Linear MCP and Slack collaboration examples
- Optional future adapter conformance cases rather than core vendor APIs
- Any additive item-level read/outcome projection justified by pilot evidence

**Constraints:**

- Markdown/Git stays canonical and fully usable offline.
- Default workflows cannot require new network calls or extra software.
- No new core status token solely to satisfy Linear/Slack.
- Published examples must work without eddaskills or Anvil.
- Permission, tenant and plan-revision boundaries must be checked before
  enabling provider-originated execution or state-changing synchronisation.
- CLI or linter behaviour changes obey D-039 three-way parity requirements.
- Draft designs and provider previews cannot be described as shipped APS APIs.

## Ready Checklist

- [x] Problem, users and incremental adoption path documented
- [x] INTEGRATIONS, ORCH, TEAM, PROG and AGENT ownership identified
- [x] Existing provider capabilities and availability caveats documented
- [x] Testable work items and explicit validation outcomes drafted
- [ ] SURF-000 authority/link/status decisions reviewed and ratified
- [ ] Open questions about reference persistence and approved-revision
      binding resolved, with versioned defaults where necessary
- [ ] Human owner authorises promotion from Draft before any execution

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
