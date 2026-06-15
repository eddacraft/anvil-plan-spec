# APS Upstream Brief: Production Governance Patterns from anvil-001

**Date:** 2026-06-15  
**Type:** Consumer feedback / patterns brief for the canonical APS project  
**Status:** Draft — for review and potential seeding into `eddacraft/anvil-plan-spec` (ROADMAP, docs, or new reference guidance)  
**Author:** Synthesised from anvil-001 operational experience (NBI, agent harness, reconciliation discipline)  
**Audience:** Maintainers and contributors to [anvil-plan-spec](https://github.com/eddacraft/anvil-plan-spec)

| Upstream (APS repo)                          | Downstream (anvil-001 heavy use)                          |
| -------------------------------------------- | --------------------------------------------------------- |
| Portable format, `aps-rules.md`, CLI (`next`/`start`/`complete`/`graph`/`lint`), templates, agent guidance, `project-context.md` scaffold | Full governance OS layered on top: NBI selector, mandatory dev-workflow routing, planning-council, atomic discipline, reconciliation/audit machinery, continuous-improvement loop |

## Purpose

This brief captures the battle-tested extensions, process innovations, and agent-surface patterns that the most advanced consumer of APS (anvil-001) has built on the portable foundation. It is intended to:

- Help the APS repo evolve its guidance, templates, CLI surface, and reference material so that *other* adopters can more easily achieve the same "tight, living plans" outcome without reinventing the wheel.
- Identify high-value patterns that may deserve portable treatment (or at least strong recommended-practice documentation).
- Surface concrete, low-friction ways the APS project can help its users (seeds, reference modules, CLI affordances, docs updates).
- Maintain the clean separation: core format + orchestration stays portable; heavy governance and release integration can remain project-specific but benefit from better hooks and examples.

It is **not** a request to pull proprietary Anvil code. It follows the spirit of ADR-055 (narrow carve-outs for read-only format consumers) and the one-way seed model used for the APS dashboard starter.

## Context — anvil-001 as the Primary Production Consumer

anvil-001 is a large monorepo (Rust core + TS surfaces) that has used APS for all multi-step work since late 2025. It currently manages:

- `plans/index.aps.md` as the single source of truth (active modules + archived).
- ~50 active modules under `plans/modules/` (plus 130+ archived).
- Extensive `plans/execution/`, `plans/decisions/`, `plans/specs/`, `plans/reviews/`, `plans/audits/`.
- A standing `continuous-improvement-backlog.aps.md` (CIB).
- Full release records under `plans/releases/`.

It also ships the TypeScript implementation of the format itself (`packages/aps`: parser, loader, validator with rich adversarial fixtures, state/locking, templates) and has its own `packages/adapters` for speckit/bmad interop.

The key differentiator vs lighter APS users is the **closed-loop operating system** built around the format: every non-trivial task (code, docs, planning, release, agent work) is forced through APS truth gates.

## Key Innovations Proven at Scale

### 1. NBI — Next Best Item (Living Ranked Selector)

The single most visible process addition.

- A dedicated "Next Best Items" section at the top of the root index (see `plans/index.aps.md:89` onward).
- Ranked table: `Rank | NBI | Mode | Source | Why now | Next action`.
- Explicit dated "NBI review notes" that record every re-ranking, readiness promotion pass, release closeout, and rationale (multiple passes in June 2026 alone, including post-v0.8.0/v0.8.1 archive cascades and v0.9 scoping tied to ADR-075).
- Selection rules that prefer unblocked Ready work advancing the current release claim, adoption, trust, or recurring friction.
- Surfaced and acted on via `/plan-status` (which triggers reconciliation) and then routed through `dev-workflow`.
- Modules record the triggering NBI pass in their "Last reviewed:" lines (e.g. `graph-v2-foundation.aps.md`).

**Why it matters for other projects:** `aps next` gives a single next item. NBI gives a short, prioritised, *debated* shortlist with "why now" context that survives release windows and personnel change. It turns the index into an active decision record rather than a static backlog.

**Portable value:** An optional but recommended `## Next Best Items` section + guidance on how to maintain it (review notes, rank rules). Could be supported by a future `aps nbi` or enhanced `aps next --ranked` affordance.

### 2. Agent / Workflow Surface (Planning as First-Class)

anvil-001 maintains a sophisticated, mandatory routing layer (much of it vendored/specialised from broader patterns developed by @joshuaboys, with anvil-specific tuning):

- `dev-workflow` (MANDATORY for any task touching the repo): APS Truth Gate → Ready/In Progress → Worktrunk branch from main → TDD → tiered Council (`/council quick|mini|full`) → addressing-pr-reviews → post-merge verification plan (tracked in `plans/reviews/post-merge/`) → continuous-improvement note → cleanup offer.
- `planning-workflow`: Intent → truth discovery (via `aps-planning`) → match existing item or create new → design gate (`brainstorming` or `planning-council`) → synthesis → readiness validation → clean handoff block.
- `aps-planning`: Session context loading, truth validation (drift, deps, `Blocks on:`, scope vs reality), reconciliation reports with explicit decisions (`valid | needs-plan-update | blocked`).
- `planning-council`: Multi-role judgement with dedicated playbooks (`plan-create`, `direction-validate`, `pre-execution-validate`, `plan-amend`). Uses stable roles mapped to specialist agents.
- Dedicated `anvil-plan-spec` agent for non-trivial module/task authoring, status sync, wave planning, and reconciliation.
- Commands: `/plan`, `/plan-status` (NBI-aware), `/council`.
- Fable-5 tuned variants (`f5-planning-workflow`, `f5-aps-loop`, `f5-dev-workflow`).

These treat APS not just as a document format but as the **authoritative execution-authorisation substrate** for the entire development lifecycle (including the development of the agent surface itself).

**Help for the APS repo:** Reference "governance harness" patterns or a recommended `agents/` + skills inventory. The upstream already has strong agent definitions and an `aps-planning/` dir — anvil-001's specialisation shows what a heavy user adds on top (mandatory routing, council for planning decisions, continuous-improvement substrate).

### 3. The "Keep Plans Alive" Discipline (the Tight Process)

This is the cultural/operational layer that other projects rarely match:

- **Single source of truth only:** `plans/index.aps.md` is canonical. Explicit rules against shadow indexes/summaries (repeated in `AGENTS.md`, `.claude/rules/aps-index.md`, `aps-planning` skill, `planning-workflow`).
- **Atomic updates:** Mark `In Progress` *before* substantive work. After completing a work item, update status + bump header count + index row in the *same* change. Archive completed modules with `git mv` to `plans/archive/modules/` + index path update in the same commit.
- **Status extensions + lifecycle narrative** (in `plans/project-context.md`): Canonical `Proposed/Ready/In Progress/Done/Blocked` + Anvil prose `Merged → Released/Shipped → Complete`. `DONE_PATTERNS` in tooling are case-sensitive.
- **Reconciliation machinery:** `scripts/aps/drift-check.mjs` (progress + release-record alignment, `shipped-aps-without-release-record` advisory), `active-lint.mjs`, `index-counts.mjs`. `.claude/workflows/aps-reconciliation-sweep.js` (adversarial semantic drift). Frequent dated audits under `plans/audits/`.
- **Continuous improvement substrate:** Every non-trivial agent session appends a compact entry to `plans/reviews/continuous-improvement-log.md`. Recurring items promoted to the standing CIB module. "Improvement: none" is accepted when there is no signal.
- **Documentation governance closeout:** Any change to `docs/**`, `plans/**`, AGENTS.md, READMEs etc. requires classification, cross-link/APS/index/ADR updates, validation runs, and an explicit "Docs Closeout" note.
- **Post-merge verification plans:** Extracted to tracked `plans/reviews/post-merge/<slug>.md` (gitignore exception) rather than living only in PR descriptions. Cleanup agent advances states using release records + these plans.
- **Anchor rescoring** and other cross-cutting process gates have their own documented processes with snapshot templates (`docs/guides/anchor-rescoring-process.md`).

**Portable value:** The core of `aps-rules.md` already encourages `project-context.md`. The APS repo could ship stronger recommended sections or a "governance reference" (example `project-context.md` fragments for NBI maintenance, reconciliation expectations, CI log, docs closeout checklist). The scripts/aps tooling was already the subject of an OSS carve-out discussion (ADR-055).

### 4. Other Proven Extensions

- Wave-based parallel execution (with gates) inside action plans.
- Explicit cross-cutting module conventions (`Coordinates with:`, `Blocks on:`, `Supersedes:`) with sweep/closeout rules.
- Release metadata blocks on work items (changeType, releaseIntent, releaseScope, releaseNote) — used by release orchestration.
- Full traceability from APS ID → branch naming convention → commit trailers → PR → release record → cleanup → archive.
- `plans/completed-index.aps.md` + frozen archive rows for historical releases.

## What Is Portable vs. Anvil-Specific

**Strongly portable / high value to surface in APS guidance:**

- NBI table + maintenance notes (as an advanced pattern on top of `aps next`).
- Atomic update + archive discipline (rules that "agents keep forgetting").
- Continuous-improvement log + standing CIB intake module.
- Docs-governance closeout checklist (tying prose changes to plan truth).
- Post-merge verification plan extraction.
- Cross-cutting module callout conventions.
- Recommended `project-context.md` skeleton for release integration and status dialect.

**Project-specific (keep in `project-context.md`):**

- Exact lifecycle prose labels and extension mapping.
- Worktrunk/`wt` branching policy and Council tiering.
- Release-record format and cleanup agent implementation.
- The full vendored agent skill/agent surface (anvil-001's `dev-workflow` etc. are heavily tuned to its monorepo + release model).

**Already partially aligned:** The upstream now scaffolds `project-context.md` and has its own `aps-planning/` surface + orchestration CLI. The divergence in anvil-001 is mostly specialisation and enforcement.

## Proposed Contributions / Concrete Help the APS Repo Could Offer

1. **NBI guidance + optional section.** Add a short "Advanced Patterns" or "Keeping Plans Alive" page (or section in `workflow.md` / `ai-agent-guide.md`) describing the NBI table shape, review-note convention, and rank-selection heuristics. Optionally extend templates to include a commented NBI starter in the index template.

2. **Stronger "project-context.md" reference content.** Ship example fragments or a richer default for the update discipline, reconciliation expectations, CI log, and docs closeout. This is the natural place for "how to not let your plans rot."

3. **CLI affordances that support the discipline.** `aps next --with-reasons` or a lightweight `aps nbi` (or just better JSON output from existing commands) that surfaces ranked/why-now data. Make it easy for heavy users to script their own NBI selector.

4. **Reference "governance surface" examples.** A small set of reference files (minimal `continuous-improvement-log.md` shape, post-merge verification plan template, simple reconciliation script) that adopters can copy rather than invent.

5. **Agent surface inventory guidance.** The upstream already has good agent definitions. Document a minimal "recommended surface" for teams that want dev-workflow-style mandatory routing (even if implemented in their own agent config or custom MCP/tools).

6. **Seed more of the mechanical tooling** (where licensing/IP allows per the ADR-055 carve-out model). The drift-check / index-counts / active-lint logic (scrubbed of Anvil dialect) would be high-value for any project that wants mechanical enforcement of the "keep current" rules.

7. **Explicit status dialect section.** Clarify in the spec how projects may (and should document) extensions like `Merged`/`Released/Shipped` while keeping canonical values as the portable contract. The upstream validator already normalises some aliases; making the extension pattern first-class would help.

## Recommendations & Suggested Next Steps

- Treat this brief as input to the APS repo's own planning (it has plans/ and uses APS on itself).
- File a lightweight issue or ROADMAP item in `eddacraft/anvil-plan-spec` titled "Capture production governance patterns (NBI, update discipline, agent routing) from heavy consumers".
- Consider a small "governance patterns" reference directory or page in the APS docs rather than bloating the core spec.
- If the APS repo wants live feedback, invite the anvil-001 team (or a representative) to a short planning-council-style review of any proposed additions.
- In anvil-001, any concrete follow-ups (e.g. "seed NBI example back", "contribute scrubbed reconciliation helper") can be tracked as CIB items or a small dedicated APS module.

## References (in anvil-001)

- NBI section + review notes: `plans/index.aps.md:89-154` (and ongoing updates)
- Core discipline rules: `plans/aps-rules.md`, `plans/project-context.md` (lifecycle, status extensions, keeping current, docs governance), `.claude/rules/aps-index.md`
- Agent surface: `.claude/skills/{aps-planning,planning-workflow,planning-council,dev-workflow,f5-*-*}/SKILL.md`; `.claude/agents/anvil-plan-spec.md`; `.claude/commands/{plan,plan-status,council}.md`
- Mechanical support: `scripts/aps/{drift-check,active-lint,index-counts,advance-released}.mjs` + `lib/modules.mjs`; `.claude/workflows/aps-reconciliation-sweep.js`
- Continuous improvement: `plans/reviews/continuous-improvement-log.md`; standing CIB module
- Related decisions: ADR-055 (OSS carve-out for APS consumers), ADR-018 (product IP boundaries), various in `plans/decisions/`
- Product implementation: `packages/aps/` (parser/validator/loader/state/templates) + adapters
- Example of NBI-driven work in practice: `plans/modules/graph-v2-foundation.aps.md` (Last reviewed notes), recent NBI review notes in the index, `plans/reviews/continuous-improvement-log.md` entries mentioning NBI

This pattern set has made APS the genuine backbone of delivery rather than a side artefact. Sharing the successful abstractions (while respecting the portable core) would materially help the APS project achieve its adoption goals.

---

**Docs closeout note (per project rules):** This is a new brainstorm/brief in `plans/brainstorms/`. It references existing APS, agent-surface, and governance docs but does not alter canonical truth files (`index.aps.md`, modules, AGENTS.md, or core guides). No index or decision-log update required at this stage; if promoted to an APS work item or contribution PR against the APS repo, the appropriate closeout steps will be followed. Validation: structure follows existing brief and triage patterns in the directory.
