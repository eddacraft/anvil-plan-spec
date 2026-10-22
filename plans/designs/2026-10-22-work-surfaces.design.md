# Portable Work Surfaces: GitHub, Linear, Jira, Slack, Teams and BYO Agents

| Field   | Value |
| ------- | ----- |
| Date    | 2026-10-22 |
| Status  | Draft |
| Modules | [work-surfaces](../modules/work-surfaces.aps.md) |
| Scope   | SURF-000 — agree the external work-surface contract and staged adoption gates |

## Problem

APS provides portable, Git-reviewed intent and bounded work items, but teams
already organise work in issue trackers and discuss delivery in chat. Forcing
separate APS, ticket, agent and PR administration harms adoption; treating a
ticket or chat transcript as the only specification loses durable scope,
authorisation, dependencies and acceptance evidence.

**Structured work surfaces are interchangeable candidates**, not a
Linear-shaped core:

- **GitHub Issues + Projects:** the technical reference implementation,
  colocated with APS Git files, PRs and Actions. GitHub Projects is optional;
  plain Issues must work without a Project.
- **Linear:** the usability and agent-delegation benchmark, using its official
  MCP first and optional native agent capabilities when access permits.
- **Jira Cloud:** the enterprise interoperability test, using the Atlassian
  Rovo MCP/REST surfaces first and optional Forge remote-agent APIs later.
- **Slack and Microsoft Teams:** interchangeable conversational overlays, not
  replacements for issue tracking. Teams can collaborate in existing
  channels/group chats, and Slack in channels/threads. Either can reference
  approved APS items and any tracker without becoming canonical.

These are *interfaces to work*, not APS runtimes. APS must work with native
GitHub/Linear/Jira agents, custom coding harnesses, Claude Code, Codex, Cursor,
human delivery, and no provider. eddaskills remains an optional internal
eddacraft implementation, **never bundled or required by public APS**.

A technical reference is not a preselected winner. The team may opt to adopt
the best work surface after realistic, measured trials, while public APS keeps
support for all viable providers and a no-provider path.

## Desired Outcome

A person can capture work in the issue tracker they already use, associate
it with a reviewed APS item or module, delegate to their chosen executor and
understand the evidence and next decision without duplicating administration.
The same approved APS identity can be linked to GitHub, Linear, Jira and/or
Slack or Teams conversations, and survive a change of providers unchanged.

Select an **optional preferred work surface for an adopting team**, based on
real user experience, integration reliability, permissions, support burden and
switching cost. The decision must not make that provider canonical for APS or
require a migration by other users.

Success is lower total team friction and trustworthy outcomes, not integrations
built or a nominally winning score.

## Principles and Boundaries

- **Portable first:** Markdown in Git remains the source of planning intent;
  no GitHub Projects, Linear, Jira, Slack, Teams, Anvil, eddaskills, hosted
  APS account or model provider is
  necessary to read, edit, lint or execute against an APS work item.
- **A plan is not a ticket:** a GitHub, Linear or Jira issue may link to a bounded APS item or
  an APS module; many internal APS items need not produce many external issues.
- **Human-friendly first:** use the provider's normal workflows; do not require
  a dedicated APS bot, command vocabulary, issue template or another UI.
- **No agent execution in APS:** APS describes authorised outcomes and context.
  An external agent/harness decides how to implement, verify and publish within
  its own permissions and the project's policies.
- **No authority by mention:** a Slack/Teams @mention, GitHub/Linear/Jira assignment, ticket
  comment, webhook or model-selection hint cannot grant write, merge, release,
  deployment, financial or scope-expansion authority.
- **No automatic completion:** chat success messages, closed issues, agent
  session completion and merged PRs are not interchangeable with verified APS
  completion or release/ship evidence.
- **Provider parity, not UI sameness:** implement only capabilities each
  provider actually supports; preserve portable semantics and clearly mark
  any optional preview or organisation-dependent agent features.
- **Adopt the proven fit:** compare options under consistent tests and costs;
  explicitly approve a reversible pilot before changing a real team's backlog.
- **Optional, incremental adoption:** manual links plus existing MCP/CLI first;
  deterministic sync and vendor apps only if measured user friction warrants.
- **Do not duplicate TEAM or PROG:** transient claims/leases belong to
  [TEAM](../modules/team-coordination.aps.md); high-churn durable progress
  belongs to [PROG](../modules/progress-journal.aps.md). Both are Draft today.
  The work-surface module consumes their eventual projections and does not
  implement substitute locks or authoritative progress stores.

## Proposed Architecture

| Plane | Owns | Does not own |
| ----- | ---- | ------------ |
| GitHub/Linear/Jira work surface | Backlog, priority, issue lifecycle, team assignment and display | APS plan authority, execution locks, proof of success |
| Slack/Teams conversation (optional) | Human discussion, mentions, decisions, task channels/threads | Canonical backlog unless the team consciously chooses chat-only intake |
| APS intent in Git | Modules, work-item contract, dependencies, decisions, approved revision | Chat transcript, remote backlog priority, agent runtime |
| Execution provider | Native agent session, tool selection, implementation and local run state | Rewriting approved APS scope without review |
| Git / CI / delivery evidence | Candidate commits, PRs, checks, review, merge, release facts | Backlog ownership |
| Optional TEAM / PROG | Claims, handoffs, effective progress when ratified | Vendor-specific status transitions |

Conceptually:

```text
GitHub Issues + Projects   Linear issues   Jira work items
          \                  |               /
           \---- work-surface references -----/
                            |
                      APS identity
                  (Git-reviewed intent)
                            |
                any native agent / harness
                            |
                    Git / CI / evidence
                            |
                   read-only projections
                            |
            issue tracker(s) + optional Slack/Teams
```

**The provider-neutral surface contract**, initially a documented interface
rather than implemented methods, comprises:

| Operation | Contract | Availability |
| --------- | -------- | ------------ |
| Resolve | Map workspace + external object to APS module/item/revision | Required |
| Link | Associate external issue/conversation with APS identity idempotently | Required for optional adapters |
| Read | Show existing APS intent and source URLs, preserving permissions | Required |
| Propose | Create a *reviewable* plan change, never mutate approved scope silently | Optional |
| Project | Render attributed execution/evidence status, with unknown states | Optional |
| Delegate | Pass an authorised APS reference to a chosen existing executor | Provider-dependent; no APS runtime |
| Reconcile | Compare Git/CI truth with provider display and repair safe drift | Required before durable sync |

A manual/MCP recipe can fulfil Resolve, Link and Read without code in APS
core. Native delegation is not required to call an integration successful.
Communication is not execution. No provider-specific action can grant a
stronger plan permission than the existing project contract.

### Provider capability profiles (verify during pilot)

| Provider | First usable path | Optional deeper path | Specific limitation to test |
| -------- | ----------------- | -------------------- | --------------------------- |
| GitHub | Issues/Projects with official MCP and Git/PR/Actions links | Project GraphQL/API automation, native coding agents, agentic workflows | Projects and third-party agents may have separate permissions and preview eligibility |
| Linear | Official Linear MCP, issue/project links and comments | GraphQL/webhooks and Developer Preview assignable agent sessions | Agent/session APIs are preview; avoid treating agent plans as APS approval |
| Jira Cloud | Atlassian Rovo MCP and existing Jira issue workflows | REST/automation; Forge A2A remote-agent connector | Tenant workflows/custom fields and remote-agent installation increase complexity |
| Slack overlay | Existing channels, mentions and threads linked to APS work | Optional app and Slack Code integration | Code-channel APIs may be partner/beta-gated; no reliable shared execution lease by mention |
| Teams overlay | Normal channels and group chat with APS links | Optional Teams agent app, mentions, Adaptive Cards, notifications | Tenant policy and permissions; channel support varies; not Slack Code parity |

Provider APIs and access levels can change. Do not promise all optional
features in the baseline; record what was available in the tested tenant.

### Portable references and handoff (proposal; not yet a schema)

There are two separate identities:

- **APS identity:** repository and plan-root identity, optional federated child
  plan, stable work-item ID (for example `BILL-003`) and the exact reviewed Git
  revision used to authorise execution.
- **Surface identity:** provider, workspace/tenant, external object ID, URL and
  relationship (backlog issue, conversation, execution session, or evidence).

Neither identity is a permission grant. Relationships may be one-to-one,
one-to-many or module-level. Do not encode provider IDs in canonical APS work
item IDs. Do not make an external ID, credentials or model choice mandatory in
minimal APS templates. Exact storage and wire format require SURF-000 approval.

The initial read path is the existing `aps export --json` (`aps-export/v1`),
together with the canonical Markdown and its Git revision. That export currently
provides IDs, status and dependencies but not a full executable handoff with
every intent, expected-outcome and validation field. First test whether the
existing CLI/MCP and Markdown are sufficient. Only if the cross-provider pilot demonstrates
friction should SURF-006 define an additive, versioned item-level read or
outcome projection. Do not commit to a new `aps describe` command or schema
before the pilot.

A useful eventual handoff would expose:

| Field | Requirement |
| ----- | ----------- |
| Stable identity | Repo, plan-root/child, work-item ID, reviewed commit |
| Contract | Intent, scope/non-scope, expected outcome, validation, decisions |
| Readiness | Effective approved state, dependency references, known blockers |
| Source links | Canonical plan location plus related external work references |
| Evidence references | Candidate commit, PR, CI/check results and review links |
| Provenance | Which underlying source asserted each fact, when and against which revision |
| Unknowns | Explicit `unknown` or `unavailable`, never reconstructed claims |

This is an *information contract*, not a request to start or complete execution.

### Lifecycle and source-of-truth rules

| Interaction | Rule |
| ----------- | ---- |
| Ticket/message creates a request | Intake only; proposed scope needs APS review before Ready |
| User approves a plan | Approval must bind to the correct repo, authorised actor and exact reviewed revision |
| Agent begins work | Must check APS readiness and dependencies; TEAM claim when supported; otherwise document that exclusive claim is unavailable |
| Agent reports progress | Surface may show an attributed activity, not rewrite durable APS truth |
| PR opens / CI runs | Project real evidence and review-readiness, not `Complete` |
| PR merges | Display integrated/merged evidence; do not equate with shipped or completed |
| Release/ship criteria pass | Only then may authorised reconciliation mark APS Complete where that project's policy requires it |
| GitHub/Linear/Jira issue changes | Backlog state stays provider-owned; APS scope/approval requires a reviewed Git revision |
| Slack messages alter requirements | Treat as requested change; reconcile through APS; do not silently broaden execution |

APS's canonical work-item states stay Draft, Ready, In Progress, Complete and
Blocked. More descriptive statuses such as `claimed`, `in review`, `merged`,
`awaiting release`, `stale` and `needs decision` are **derived views**, not
new core state tokens. The mappings must be workspace-configurable and fail
closed when evidence is missing. Existing GitHub/Linear/Jira auto-close rules must
not mark an APS-controlled deliverable finished at PR creation or merge.

## Human Journeys

### A. GitHub Issues + Projects: technical reference

1. A developer or product manager creates a GitHub issue; it may be in a
   GitHub Project, but a Project is not required.
2. They link the issue to an existing APS item/module or propose a plan PR.
3. A human reviews APS scope in Git and authorises the exact revision.
4. The user assigns the issue to an eligible native coding agent or passes the
   approved item to an independent harness. No special agent required.
5. PRs, CI and review evidence remain in GitHub. The issue/project shows
   appropriate links, blockers and an **evidence-backed** state.
6. An issue closed on PR merge before actual APS completion is detected,
   rather than silently marking the contract Complete.

Use the official GitHub MCP server, issues, project fields (where present)
and normal Git operations for the first published recipe. GitHub Agentic
Workflows and third-party agent assignment are optional preview capabilities.

### B. Linear: usability and delegation benchmark

1. A product manager creates/selects an issue using the normal Linear UI.
2. A native agent or developer links it to an approved APS work item, or
   proposes a Git-reviewed plan change.
3. A human approves scope and the executor uses the same APS revision.
4. A Linear-native coding agent or another tool implements and reports
   attributed work; PR/CI evidence remains independently checkable.
5. Linear exposes relevant progress and decisions without creating a parallel
   acceptance-criteria database or false Completed status.

One Linear issue may represent a module; internal APS items need not become
individual tickets. Linear's official MCP suffices for the baseline. Its
assignable agent/session APIs are Developer Preview, not a requirement.

### C. Jira Cloud: enterprise interoperability

1. An enterprise user finds or creates a Jira work item through normal Jira
   workflows or Atlassian Rovo MCP.
2. The work item links to an APS module or item without relying on a specific
   issue key syntax, mandatory custom field, status scheme or Forge app.
3. The approved APS revision reaches a compatible existing coding executor;
   Jira's work item stays the human priority/assignment interface.
4. Evidence URLs, blockers and status discrepancies are displayed with
   permissions honoured, not translated into unconditional Jira transitions.
5. If justified later, a separate Forge remote-agent adapter may use its A2A
   connector for assign/mention/progress; do not host APS solely for this.

Test differing project schemes, access levels and issue types. The initial
scope is **Jira Cloud**; Jira Data Center is not implicitly certified.

### D. Slack: collaborative overlay for any tracker

1. A person asks in a shared `#engineering` channel about an APS-backed task.
2. They link a GitHub, Linear or Jira issue (or no issue) and the same APS item.
3. The team selects existing agents by mention; optional Model/Reasoning hints
   do not constitute an APS model router or grant a claim.
4. Use a thread for short work or a task-specific Slack Code channel where
   actually supported. Post important decisions, blockers and Git/CI links
   only, not every implementation step.
5. Changed requirements result in a proposed plan revision; another
   mentioned agent does not acquire an exclusive execution lease.

The first recipe must work with ordinary Slack threads and existing agents;
beta partner-only Slack Code API access is not a prerequisite.

### E. Microsoft Teams: enterprise conversational overlay

1. A user discusses a GitHub/Linear/Jira issue (or APS-only work) in a
   Teams standard channel thread or group chat, linking the approved APS item.
2. The team @mentions a coding agent **already installed and allowed** in
   that conversation. An @mention is an interaction, not a claim, permission
   grant or guarantee the agent can implement code.
3. The chosen existing agent obtains the approved Git revision and executes
   in its own authorised environment. Post relevant blockers, decisions
   and independently verifiable Git/CI evidence, not every agent step.
4. Optional Adaptive Cards and proactive messages make decisions and status
   clearer, but cannot replace a reviewed plan or Git-based approval.
5. A second compatible agent can review, within its actual permissions.
   There is no required Copilot Studio, Channel Agent or APS bot.

**Teams SDK capabilities:** group-chat/channel agents, explicit mentions,
threaded replies, Adaptive Cards and proactive messages where installed and
permitted. These do not automatically deliver Slack Code's shared code/diff
experience. The separate built-in Teams **Channel Agent** currently permits
only one Channel Agent per channel; this is *not* a limit on every custom
Teams SDK agent or group-chat integration.

By default, installed Teams agents only see messages directly @mentioning
them; broader channel/chat access requires resource-specific consent.
App deployment may require tenant admin permission. Shared/private channel
support and the ability to post messages or cards vary by feature and tenant.
A standard-channel/thread link-only recipe must work even with all agent
apps blocked; advanced APIs are optional and are not prerequisites.

### E. Provider-free and cross-provider portability

A team can run `aps next`, read Markdown, and give a Ready work item to a
human or native harness without any issue tracker, Slack or extra service.

The exact **same Git-reviewed APS item** must be linkable to GitHub, Linear
and Jira simultaneously, referenceable in Slack and Teams, or exported and
relinked when a team changes its chosen tracker. Preserve identity, review history, validation and evidence
URLs. Do not assert that copying an issue between trackers copies authorisation
or gives the new provider permission to launch agents.

## Integration Strategy and Release Gates

| Stage | Result | Gate |
| ----- | ------ | ---- |
| 0 — Contract | Ratify provider-neutral identity, authority, status, trust and link semantics | SURF-000/001 |
| 1 — Two complementary baselines | GitHub Issues/Projects as technical reference and Linear as UX benchmark, both via existing MCP/CLI | SURF-010 and SURF-002 |
| 2 — Portability | Jira Cloud MCP pilot, Slack and Teams threads, provider-switch/relink test | SURF-011, SURF-003, SURF-017, SURF-013 |
| 3 — Comparative pilot | Same tasks and users across GitHub, Linear, Jira and no-provider control; Slack as optional overlay | SURF-004 |
| 4 — Selection | Publish measured scores, hard-gate results, switching costs and recommendation; human records winner or no-winner | SURF-012 |
| 5 — Public pattern | Publish all usable native recipes, preserving provider choice regardless of selected internal tool | SURF-005 |
| 6 — Opt-in team adoption | Only on explicit owner approval, trial the preferred provider on bounded live work with rollback | SURF-014 |
| 7 — Optional hardening | Richer APS read and separately approved tracker/chat adapters if justified | SURF-006/007/008/015/016/018 |
| 8 — Conformance | Security, identity, idempotency, degradation and interop suite for any shipped adapters | SURF-009 |

**No preselected winner.** GitHub is the technical reference because Git/PR/CI
are colocated. Linear is the user-experience benchmark. Jira is a deliberate
enterprise stress test. That allocation is a *test design*, not an adoption
decision. Slack and Teams are compared as conversational overlays on each tracker,
not scored as replacement backlogs. Tenant familiarity is useful but is not
proof of APS-specific coding-agent support.

### Comparable evaluation and a decision rule

Evaluate all three structured providers on the same three work scenarios:
(a) small bug, (b) multi-item feature linked to one issue, and (c) merged but
not yet released change. Include one user in each role: product/lead,
implementer and reviewer; retain a pure Git+APS no-provider control. Repeat
a handoff to a different agent and a provider outage/revocation.

**Score only observed results** using an anchored 0-5 rating and weighted
categories (total 100). Record raw timings, counts, provider configuration,
permissions and version/access tier alongside each score.

| Criterion | Weight | Evidence |
| --------- | -----: | -------- |
| Daily usability and clarity | 25 | Task completion, findability, decision visibility, participant rating |
| Setup and ongoing administration | 15 | Setup minutes, re-entry count, required issue/status edits |
| Agent integration and BYO freedom | 15 | Native delegation availability; independent-agent handoff |
| APS fidelity and portability | 15 | Correct approval/revision mapping, module mapping and relink |
| Reliability and support burden | 15 | Duplicate events, recovery, sync discrepancy and maintenance |
| Permissions and enterprise fit | 10 | Tenant separation, custom workflows, permission limitations |
| Total cost and switching friction | 5 | Licence/seat/tool costs, developer upkeep, migration effort |
| **Total** | **100** | **Observed, not assumed** |

Scoring anchors: 0 = cannot complete the scenario; 1 = substantial manual
workarounds; 2 = repeatable but high friction; 3 = works with tolerable
administration; 4 = smooth with small limitations; 5 = native and minimal
extra work. Unsupported preview features do not score as delivered.
Separate measured data from evaluator judgement. Report untested criteria as
unknown, not zero or five; do not declare a winner while core tests are missing.

**Non-compensable gates:** no silent APS scope rewrite; no unauthorised
execution or cross-tenant disclosure; no false APS Complete; no duplicate
external actions from retries; no dependence on eddaskills, Anvil, vendor
agent choice or a hosted APS account; no provider required for basic APS use.
A provider that fails an essential gate cannot win regardless of points.

Recommendation rules:

1. Publish evidence, weighted scores, known access restrictions, confidence
   and which features depended on optional preview availability.
2. Choose the highest eligible provider *only* if it materially reduces work
   compared with the incumbent, accounting for migration and ongoing costs.
3. If scores differ by fewer than five points out of 100, treat the outcome
   as a practical tie. Prefer the team's current tracker if it meets all
   gates; no forced move for cosmetic improvements.
4. The accountable team owner records an explicit **adopt / pilot longer /
   keep incumbent / no winner** decision. APS itself remains vendor-neutral.
5. Do not automatically create, close, move or delete live work as a result
   of a scorecard. A migration is a separate approved operation.

### Conversational overlay comparison (separate from tracker ranking)

Compare Slack and Teams as optional overlays with the **same** approved APS
item, tracker link, changed-requirement scenario and two-agent discussion.
Do **not** include chat providers in the weighted GitHub/Linear/Jira
tracker selection.

| Factor | Slack | Microsoft Teams |
| ------ | ----- | --------------- |
| Baseline | Existing channel/thread and agent mention | Existing standard channel thread/group chat, APS links and installed agent mention |
| Agent UX | Native agents, optionally Slack Code | Tenant-enabled agent apps; built-in Channel Agent is not a coding workspace |
| Rich interaction | Threads, links, optional Slack Code | Threads, links, optional Adaptive Cards and proactive messages |
| Permission risk | Workspace app permissions, Slack Code beta | Tenant installation, RSC, shared/private channel limitations |
| Portability | Identical APS ID and evidence links | Identical APS ID and evidence links |

Record setup time, admin interventions, clarity, ability to use different
agents, privacy limits and preferences of actual participants. If the team's
incumbent conversational platform meets the bar, keep it rather than force
migration merely to use APS. A team may use neither.

### Opt-in adoption protocol

If the team wants to use the winner, migrate **only a bounded pilot cohort**
of active work items. Preserve original issue URLs, discussion/history links,
APS Git identity/revisions and evidence. Preview field/priority/status
mappings, check access for humans and agents, avoid duplicate notifications,
and provide reversible rollback to the incumbent. Do not treat a new issue
ID as a new APS work item. Run at least one real planning-to-review cycle,
then measure coordination cost and delivery clarity again. Promotion to the
team's default tracker requires human sign-off; publishing APS adapters does
not require that team adoption.

### Provider maturity boundaries

- GitHub: official MCP, Issues/Projects, PRs and Actions are a useful baseline.
  Copilot issue assignment exists; third-party agents and GitHub Agentic
  Workflows have preview/eligibility constraints and must be separately tested.
- Linear: official MCP is sufficient for basic issue operations; native agent
  session/plan APIs are Developer Preview / technology preview. Durable sync
  would use GraphQL and webhook-driven reconciliation, not a live MCP chat
  masquerading as a controller.
- Jira Cloud: Atlassian Rovo MCP works for external clients with appropriate
  access. A native remote-agent integration needs a Forge app and A2A-based
  service, and is **not** the baseline proof.
- Slack: ordinary channels/threads are baseline. Advanced Slack Code APIs
  may require partner/beta access; check availability when the optional app
  is considered.
- Teams: normal standard-channel/group chat and APS links are baseline.
  Teams SDK apps can add @mentions, cards and proactive messages where
  installed and permitted; broader conversation reads need resource-specific
  consent. The separate built-in Channel Agent has a one-per-channel limit.
  Validate feature support in each actual tenant; no special Teams app is
  required to follow the Markdown plan.

If the measured manual integration is already good enough, **stop at the
documented recipes**. That remains a successful outcome.

## Ease of Use and Value Criteria

Measure real users and tasks; do not assert time savings or score a preferred
provider before completing the pilots.

- A user with existing APS/provider access can follow each native recipe
  within **15 minutes** without an APS account, eddaskills or new agent
  runtime (proposed target, to measure).
- At most one Git-reviewed APS contract is authoritative; link projections
  instead of maintaining duplicated acceptance criteria.
- A non-specialist can see the task, decision owner, blocker, proof and next
  action in their chosen issue tracker without parsing agent logs.
- GitHub Projects is optional; no issue explosion for a multi-item module.
  Jira schemes/custom fields and Linear issue structures retain their normal
  semantics instead of mirroring APS action checkpoints.
- A change of agent and a change of tracker both leave APS identity/review
  history intact; reconnecting links is tested explicitly.
- Pure Git/CLI and no-provider operations remain fully functional.
- Tested failure cases contain **zero** silent plan rewrites, false Complete,
  unintended duplicate external actions or cross-workspace data leaks.
- Compare actual provider costs, availability and administrator requirements
  for the adopting team rather than assuming every preview/API is enabled.
- Compare Slack and Teams on the same work conversations without treating
  either as a replacement for GitHub, Linear or Jira.
- Track setup time, repeated updates per issue, participant clarity, handling
  of blockers, permissions and recovery across the same three scenarios.

Selection and adoption evidence should be written as a versioned report linked
from SURF-004 and SURF-012; this plan is not itself a finding of which product
wins.

## Failure Modes, Trust and Recovery

| Scenario | Expected behaviour |
| -------- | ------------------ |
| GitHub/Linear/Jira issue closed before CI/release | APS not marked Complete; human-visible mismatch |
| Edited issue contradicts approved Git plan | Proposed revision, never silent APS overwrite |
| Slack/Teams instruction attempts to expand scope | Escalation/plan review; no extra authority granted |
| Two agents claim the same item | TEAM claim when available; otherwise surface concurrency uncertainty and avoid implying exclusivity |
| Webhook delivered twice or out of order | Idempotent relation keys/event handling; no duplicate issues, runs or updates |
| Provider unavailable, rate-limited or revoked | Git truth remains available; queue/retry only safe operations; show `unknown` |
| PR merged while sync process is down | Reconcile from authoritative Git/CI evidence; no invented events |
| Repo/branch/plan revision changes after approval | Detect revision drift; require revalidation/re-authorisation when material |
| Private issue or Slack/Teams thread links shared elsewhere | Enforce workspace/project visibility; never cross-post hidden content |
| Teams agent lacks @mention or RSC access | Work from only authorised visible context; never imply access to entire thread |
| Teams private/shared channel API cannot post | Link-only/manual fallback, explicit unsupported outcome |
| Teams admin blocks agent installation | Ordinary conversation links plus APS still work |
| Prompt injection or malicious ticket/comment | Treat external content as untrusted input, never a tool instruction or permission |
| Model or agent switching | Contract remains portable; model hint never grants tools/permissions |
| Release/merge automation conflicts with tracker rules | Disable/configure contradictory auto-close mappings; fail safe |
| Project board field disagrees with issue status | Explain which field is provider-owned; never infer APS completion |
| Jira issue transitions/custom field not present | Read-only fallback, explicit unsupported/unknown rather than data loss |
| Team switches trackers or re-links an issue | Preserve APS identity/evidence; detect duplicate links and verify authorised access |
| Selection rubric has gaps or biased preview assumptions | Publish unknowns; no winner until comparative baseline is reliable |
| Pilot migration fails or duplicates work | Reconcile with incumbent, undo writes where safe, preserve history URLs |
| Offline local-only APS project | Existing CLI/Markdown works with all integrations disabled |

A production adapter additionally needs authenticated tenant isolation,
least-privilege OAuth scopes, verified inbound webhook signatures, permission
revocation, secret-safe logging, replay protection, rate limits, explicit
ownership of user-initiated external writes, and reconcilable recovery.
Provider messages should contain summaries/links rather than sensitive
source-code context by default.

## Alternatives Considered

| Alternative | Assessment |
| ----------- | ---------- |
| Move APS source of truth into any tracker | Reject: vendor lock-in, breaks Git review and offline/manual use |
| Build a required APS chat bot and orchestration service | Reject: unnecessary second runtime and account requirement |
| Bundle eddaskills or Anvil with public integration | Reject: internal/product-specific capability; independent opt-in consumers |
| Full bidirectional issue/plan sync from day one | Reject: conflicting ownership and high failure/maintenance cost |
| Mirror every APS action as a ticket or chat message | Reject: noisy for humans, duplicates the backlog |
| Treat GitHub technical reference as automatic team winner | Reject: implementation convenience is not measured user value |
| Start with Slack Code APIs | Defer: partner-gated beta; normal channels and threads are sufficient for proof |
| Require Copilot Studio or Teams Channel Agent | Reject: vendor-specific agent runtime and tenant access, not necessary for APS |
| Use only docs and never build adapters | Valid if the usability pilot does not justify additional infrastructure |

## Proposed Decisions (require SURF-000 ratification)

- **D-001:** Work surfaces are optional; APS/Git owns approved intent.
- **D-002:** Public contract is provider- and executor-neutral; eddaskills,
  Anvil, a hosted runtime and vendor plugins remain opt-in only.
- **D-003:** Bind external references to stable APS identity and reviewed
  Git revision; issue and chat content cannot silently change scope.
- **D-004:** GitHub Issues/Projects is the *technical reference*, Linear
  the *usability benchmark*, Jira Cloud the *enterprise portability test*.
  No adoption winner is decided in advance.
- **D-005:** Slack and Teams are interchangeable conversational overlays,
  independently usable with GitHub, Linear, Jira or no tracker. Mentions do
  not confer authority and native agent/tenant restrictions are not APS
  contract requirements.
- **D-006:** External states are attributed, evidence-backed projections,
  never independent APS completion or permission grants.
- **D-007:** TEAM owns claims, PROG owns progress storage; SURF only consumes
  ratified projections. Baseline examples cannot depend on unfinished work.
- **D-008:** Pilot existing MCP/native capabilities before building optional
  GraphQL/REST/webhook/Forge/Slack Code adapters.
- **D-009:** Evaluate providers against common scenarios, hard safety gates
  and a predeclared scorecard. A team may adopt the winner only by explicit
  approval and a reversible pilot.
- **D-010:** Publishing multi-provider APS patterns and an internal team's
  preferred tracker are separate decisions.

## Open Questions

1. Which portable external-reference shape and storage location work across
   nested plan trees, multiple providers/workspaces and a changing tracker?
2. How is reviewed-plan approval proved for actions initiated by each native
   agent or external issue/comment?
3. Does the existing APS export plus Markdown supply the context each provider
   needs, or does the pilot justify an additive item-level read projection?
4. What provider-specific status mapping avoids GitHub Projects vs issue
   status disagreements, Jira workflow differences and Linear auto-close?
5. Which claim/concurrency warning is necessary while TEAM remains Draft?
6. Which native agent/MCP/preview capabilities are accessible in the actual
   test accounts, and what is the operational cost?
7. Does measured reduced administration justify any production adapter?
8. What is the adopting team's incumbent tracker **and chat platform**,
   current issue volume, licence/access constraints and pilot cohort? Capture during
   SURF-012 rather than guessing now.
9. Can an APS item be relinked from GitHub to Linear to Jira, then restored,
   while Slack and Teams conversations reference its unchanged identity?
10. Which Teams agent apps can be installed in the actual tenant? Does it
    permit agent @mentions, proactive messages and richer contextual access?
11. Is Slack or Teams already the team's preferred chat platform, and does
    either add sufficient value to justify new installation or permissions?

## References and Existing Ownership

- [APS integration export](../../docs/integrations.md)
- [GitHub official MCP](https://github.com/github/github-mcp-server)
- [GitHub Issues agent assignment](https://docs.github.com/en/copilot/how-tos/copilot-on-github/use-copilot-agents/overview)
- [GitHub agentic workflows](https://docs.github.com/en/copilot/concepts/agent-capabilities/agent-powered-workflows-and-operations/about-github-agentic-workflows) — public preview
- [Atlassian Rovo MCP](https://developer.atlassian.com/cloud/rovo-mcp/guides/getting-started/)
- [Jira remote agents](https://developer.atlassian.com/platform/forge/remote-agents-in-jira/) — requires Forge and A2A
- [APS team coordination](../modules/team-coordination.aps.md)
- [APS progress journal](../modules/progress-journal.aps.md)
- [APS orchestration](../modules/orchestrate.aps.md)
- [APS agent packaging](../modules/agents.aps.md)
- [Linear MCP](https://linear.app/docs/mcp)
- [Linear developer APIs](https://linear.app/developers)
- [Linear agent integration](https://linear.app/developers/agents) — Developer Preview
- [Linear agent session interaction](https://linear.app/developers/agent-interaction)
- [Slack app mentions](https://docs.slack.dev/app-management/quickstart-app-settings/)
- [Slack Code guide](https://docs.slack.dev/ai/slack-code/)
- [Slack Code developer availability](https://slack.dev/slack-code-a-dedicated-place-for-agent-work/)
- [Slack agent design guidance](https://docs.slack.dev/concepts/agent-design/)
- [Teams agents in channels and group chats](https://learn.microsoft.com/en-us/microsoftteams/platform/bots/how-to/conversations/channel-and-group-conversations)
- [Teams agent app administration](https://learn.microsoft.com/en-au/MicrosoftTeams/teams-app-setup-policies)
- [Teams proactive messages](https://learn.microsoft.com/en-us/microsoftteams/platform/bots/how-to/conversations/send-proactive-messages)
- [Teams SDK extension](https://learn.microsoft.com/en-us/microsoft-365/agents-sdk/teams/teams-extension)
- [Teams Channel Agent limitations](https://learn.microsoft.com/microsoftteams/set-up-channel-agent-teams)
- [Teams full-message RSC consent](https://learn.microsoft.com/en-us/microsoftteams/platform/bots/how-to/conversations/channel-messages-for-bots-and-agents)

This is a planning proposal. It does not claim that any GitHub/Linear/Jira/
Slack/Teams adapter, APS chat bot, distributed claim service, provider-wide native
agent integration or unattended agent bridge already exists. No comparison
scores or internal adoption decision have been recorded yet.
