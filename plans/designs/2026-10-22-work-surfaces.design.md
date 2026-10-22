# Portable Work Surfaces: Linear, Slack and Bring-Your-Own Agents

| Field   | Value |
| ------- | ----- |
| Date    | 2026-10-22 |
| Status  | Draft |
| Modules | [work-surfaces](../modules/work-surfaces.aps.md) |
| Scope   | SURF-000 — agree the external work-surface contract and staged adoption gates |

## Problem

APS provides portable, Git-reviewed intent and bounded work items, but most
teams organise work in an issue tracker or discuss it in a team conversation.
Asking people to maintain an issue, an APS module, a coding-agent task and a PR
by hand adds administration. Conversely, replacing APS with an issue description
or chat history loses durable intent, dependency, authority and validation
boundaries.

Two useful entry points deserve a public, vendor-optional pattern:

1. **Linear** as a structured backlog, prioritisation and delegation surface.
2. **Slack** as a collaborative team room: people can mention different coding
   agents, discuss one work item in a thread or task-specific code channel, and
   make decisions together.

These are *interfaces to work*, not new APS runtimes. APS should work with
Linear-native agents, Slack-connected coding agents, Claude Code, Codex,
Cursor, another team's custom harness, manual development, or no external
provider at all. eddaskills is an optional internal eddacraft consumer, **never
a bundled runtime, dependency or preferred execution path**.

## Desired Outcome

A person can create or discuss work in the tool they already use, link it to
an existing APS item or propose a new bounded contract, authorise that contract
through the project's normal review process, and give it to their chosen
executor. They can see meaningful progress and evidence without manually
re-keying status in several places. A different executor or issue tracker can
take over without loss of durable plan intent.

Success is *less work for the team*, not the number of integrations built.

## Principles and Boundaries

- **Portable first:** Markdown in Git remains the source of planning intent;
  no Linear, Slack, Anvil, eddaskills, hosted APS account or model provider is
  necessary to read, edit, lint or execute against an APS work item.
- **A plan is not a ticket:** a Linear issue may link to a bounded APS item or
  an APS module; many internal APS items need not produce many external issues.
- **Human-friendly first:** use the provider's normal workflows; do not require
  a dedicated APS bot, command vocabulary, issue template or another UI.
- **No agent execution in APS:** APS describes authorised outcomes and context.
  An external agent/harness decides how to implement, verify and publish within
  its own permissions and the project's policies.
- **No authority by mention:** a Slack @mention, Linear assignment, ticket
  comment, webhook or model-selection hint cannot grant write, merge, release,
  deployment, financial or scope-expansion authority.
- **No automatic completion:** chat success messages, closed issues, agent
  session completion and merged PRs are not interchangeable with verified APS
  completion or release/ship evidence.
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
| Work surfaces (Linear, Slack, other) | Human intake, discussion, assignment, priorities, display | APS scope/approval, execution safety, proof of delivery |
| APS intent in Git | Modules, work-item contract, dependencies, decisions, approved revision | Chat transcript, remote backlog priority, agent runtime |
| Execution provider | Agent session, tool selection, implementation and local run state | Rewriting approved APS scope without review |
| Git / CI / provider evidence | Candidate commits, PRs, test runs, review, merge and release facts | Business backlog intent |
| Optional TEAM / PROG | Claims, handoffs, effective progress when ratified | Vendor-specific ticket state |

Conceptually:

```text
Linear issues/projects          Slack team channel/thread
        |                               |
        +---------- optional links -----+
                        |
                  APS work reference
                        |
            Git-reviewed APS contract
                        |
           any native agent or harness
                        |
                Git / CI / evidence
                        |
                read-only projections
                        |
                 Linear and/or Slack
```

Communication is not execution. An agent may be invoked directly within Linear
or Slack, or externally with an APS reference. This integration does not create
a second agent supervisor, model router or mandatory service.

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
existing CLI/MCP and Markdown are sufficient. Only if the pilot demonstrates
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
| Linear status changes | Backlog state remains Linear-owned, but cannot change APS scope/approval without a reviewed plan revision |
| Slack messages alter requirements | Treat as requested change; reconcile through APS; do not silently broaden execution |

APS's canonical work-item states stay Draft, Ready, In Progress, Complete and
Blocked. More descriptive statuses such as `claimed`, `in review`, `merged`,
`awaiting release`, `stale` and `needs decision` are **derived views**, not
new core state tokens. The mappings must be workspace-configurable and fail
closed when evidence is missing. Existing GitHub/Linear auto-close rules must
not mark an APS-controlled deliverable finished at PR creation or merge.

## Human Journeys

### A. Linear: issue to approved APS work

1. A product manager creates or selects a Linear issue as usual.
2. An agent or developer finds a relevant APS module/item and links it, or
   proposes new APS work through a reviewable plan PR.
3. The responsible human reviews the *APS intent* in Git; priority and business
   conversation stay in Linear.
4. Once the item is genuinely Ready, the human delegates it to a Linear-native
   coding agent, a native CLI agent, or another executor.
5. The executor reads the same approved APS revision, works in its own native
   environment and links code/CI/review evidence.
6. Linear presents concise progress, the next required decision and links to
   evidence; status is not falsely advanced by optimistic agent prose.

A single issue may represent an APS module, while its internal APS work items
remain invisible in Linear unless the team asks to track them individually.

**Manual first:** Linear's official MCP can find/create/update issues. No
dedicated APS agent installation or webhook service is required for the first
published recipe.

### B. Slack: collaborate in one team room

1. A person asks in `#engineering` about an APS-backed change.
2. They reference an APS work item or ask their chosen agent to locate/propose
   one using the normal repository instructions. An optional `@APS` helper is
   *not* a prerequisite.
3. The team selects an executor by its existing agent mention (for example
   `@Codex`) and can ask a different agent to review or advise. The specific
   model is a provider capability, not a new APS model router. Existing
   optional `Model:` and `Reasoning:` APS hints remain hints.
4. Short discussions remain in a Slack thread. Longer collaborative work
   can use one dedicated **Slack Code channel per task** where available, or
   a normal channel/thread fallback. The parent team channel stays navigable.
5. Keep plan/PR/evidence URLs visible and post only meaningful decisions,
   blocked questions and milestones. On completion, summarise in the source
   thread; archive a task channel where supported.
6. Requirements altered in chat trigger a proposed APS plan change, not a
   direct silent rewrite. Two mentioned agents must not treat the same work
   as exclusively theirs without an actual coordination claim.

**Compatibility:** Slack's normal `app_mention` events and threaded messages
can underpin optional apps. Slack Code's richer code-channel management APIs
are beta/partner-gated as of this design date; their availability must be
rechecked during SURF-008. The published baseline must work without them.

### C. Bring your own / no work surface

A team can run `aps next`, inspect Markdown, and hand an approved work-item
reference to its native harness, custom executor or human. It can record
outcomes through normal Git review. No Slack, Linear, hosted service, eddaskills
or Anvil account is needed.

### D. Linear plus Slack

A Linear issue links to the same APS identity as a Slack thread or code
channel. Linear is the durable backlog; Slack is the discussion space. Neither
copies or overwrites APS intent. A linked PR/release evidence record may appear
in both without duplicating the work itself.

## Integration Strategy and Release Gates

| Stage | Result | Gate |
| ----- | ------ | ---- |
| 0 — Contract | Ratified authority, link, status and permission rules | SURF-000 |
| 1 — No custom adapter | Linear MCP and Slack/native-agent worked recipes; Git/APS authoritative | SURF-001 through SURF-003 |
| 2 — Experience pilot | Measure administration saved, ambiguity, reliability, tool independence and no-provider fallback | SURF-004 |
| 3 — Public pattern | Publish useful, runnable examples without creating a hosted dependency | SURF-005 |
| 4 — Optional hardening | Add richer APS read/projection only for proven gaps | SURF-006, separately approved |
| 5 — Optional provider adapters | Deterministic Linear sync or Slack app/Code experience only if justified | SURF-007/008, each separately approved |
| 6 — Conformance | Adversarial sync/permission/failure matrix for any production adapter | SURF-009 |

Linear Agent APIs and plans are Developer Preview / technology-preview
surfaces. Pilot with stable MCP first; a native assignable agent is optional
and must be gated on API stability and demonstrated value. For unattended
sync, prefer GraphQL plus webhooks and idempotent reconciliation over an
interactive MCP session masquerading as a durable controller.

The Slack Code APIs are beta and available only to select partners.
The normal-channel/thread recipe is independently shippable. A custom Slack
app is optional, not part of the APS public core.

If Stage 2 shows little additional value from automated synchronisation,
**stop at the documented integration pattern**. That is a successful outcome.

## Ease of Use and Value Criteria

Measure real users and tasks; do not claim time savings without evidence.

- A new user already using APS and a selected surface can follow the published
  manual integration journey in **15 minutes or less** without creating an APS
  account or installing a second agent/harness (target to test, not a claim).
- At most **one** authoritative APS work item/contract is maintained; links
  and projections do not require copy/pasting an independent set of acceptance
  criteria into the provider. Track extra manual updates per work item.
- A non-specialist can find the goal, current decision owner, blockers and
  PR/verification links in the familiar surface without opening agent logs.
- A team can use one issue to track a module and can work without issue
  explosion when many internal APS items exist.
- Switching from one agent to another leaves the accepted APS intent intact.
- A no-Linear/no-Slack Git+CLI journey remains valid with the adapter absent.
- **Zero** silent plan rewrites, false Complete claims, duplicate execution
  starts from retried events, or leaked secrets in tested failure cases.

Pilot matrix: one small bug, one multi-item feature and one merged-but-not-
released change; exercise (a) Linear, (b) Slack threads, (c) both linked to
the same APS item and (d) a provider-free control. Include a planner/product
participant, a developer and a reviewer. Record elapsed setup/work-handoff
time, manual re-entry count, status mismatch count, interventions and perceived
clarity. Each stage's promotion should cite this evidence.

## Failure Modes, Trust and Recovery

| Scenario | Expected behaviour |
| -------- | ------------------ |
| Linear issue closed before CI/release | APS not marked Complete; human-visible mismatch |
| Edited issue contradicts approved Git plan | Proposed revision, never silent APS overwrite |
| Slack instruction attempts to expand scope | Escalation/plan review; no extra authority granted |
| Two agents claim the same item | TEAM claim when available; otherwise surface concurrency uncertainty and avoid implying exclusivity |
| Webhook delivered twice or out of order | Idempotent relation keys/event handling; no duplicate issues, runs or updates |
| Provider unavailable, rate-limited or revoked | Git truth remains available; queue/retry only safe operations; show `unknown` |
| PR merged while sync process is down | Reconcile from authoritative Git/CI evidence; no invented events |
| Repo/branch/plan revision changes after approval | Detect revision drift; require revalidation/re-authorisation when material |
| Private issue or thread links shared elsewhere | Enforce workspace/project visibility; never cross-post hidden content |
| Prompt injection or malicious ticket/comment | Treat external content as untrusted input, never a tool instruction or permission |
| Model or agent switching | Contract remains portable; model hint never grants tools/permissions |
| Release/merge automation conflicts with Linear rules | Disable/configure contradictory auto-close mappings; fail safe |
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
| Move APS source of truth into Linear | Reject: vendor lock-in, breaks Git review and offline/manual use |
| Build a required APS chat bot and orchestration service | Reject: unnecessary second runtime and account requirement |
| Bundle eddaskills or Anvil with public integration | Reject: internal/product-specific capability; independent opt-in consumers |
| Full bidirectional issue/plan sync from day one | Reject: conflicting ownership and high failure/maintenance cost |
| Mirror every APS action as a ticket or chat message | Reject: noisy for humans, duplicates the backlog |
| Start with Slack Code APIs | Defer: partner-gated beta; normal channels and threads are sufficient for proof |
| Use only docs and never build adapters | Valid if the usability pilot does not justify additional infrastructure |

## Proposed Decisions (require SURF-000 ratification)

- **D-001:** Work surfaces are optional interfaces, not APS planning authority.
- **D-002:** The public contract is provider- and executor-neutral; no bundled
  eddaskills, Anvil, agent engine, credentials or hosted service.
- **D-003:** Link external work to stable APS identity and reviewed Git
  revision; keep acceptance criteria and execution authorisation in Git.
- **D-004:** Try Linear MCP and native agents before developing an API-backed
  adapter or Linear agent installation.
- **D-005:** Try Slack mentions/threads and native coding agents before
  developing a Slack app or relying on beta Slack Code APIs.
- **D-006:** External statuses are attributed, evidence-backed projections;
  they cannot silently expand scope or mark APS Complete.
- **D-007:** TEAM owns atomic claims and PROG owns progress storage; this
  module must neither reimplement nor block on unshipped TEAM/PROG features
  for read-only/manual examples.
- **D-008:** Adapter development follows measured ease-of-use evidence,
  not an assumed need for full bidirectional sync.

## Open Questions

1. What stable external reference format and storage location give usable
   links across clones, nested plan trees and multiple provider workspaces,
   without polluting minimal APS work-item fields?
2. How should a reviewed plan revision be bound to human approval when the
   initial command comes from Linear/Slack?
3. Is the existing APS Markdown plus `aps export --json` adequate for the
   first pilots, or is an item-level structured read/projection needed?
4. How should a team configure Linear/Slack status labels without changing the
   core APS state vocabulary or conflicting with GitHub auto-close?
5. Which execution/claim warning must be shown while TEAM is Draft?
6. Which SDK/API features are actually available to non-partners when the
   optional advanced providers are implemented?
7. Does the pilot demonstrate enough value to warrant maintaining an
   optional API-backed adapter at all?

## References and Existing Ownership

- [APS integration export](../../docs/integrations.md)
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

This is a planning proposal. It does not claim that any Linear/Slack adapter,
APS Slack bot, distributed claim service or unattended native-agent bridge
already exists.
