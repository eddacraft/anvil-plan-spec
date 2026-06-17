# Release Planning

A **release plan** (or release narrative) is a single markdown file per
version that tells the story of a release: its theme, what ships, the
criteria for calling it a success, the risks, and a map back to the APS
modules that delivered it.

This guide is the deep reference. For where release plans sit among the
other planning archives, see the
[Release Narrative](workflow.md#release-narrative) section of the workflow
guide — this page picks up from there with the full lifecycle: when to
start a plan, how to enumerate the work, how to handle scope changes, the
status flow, and the hand-off to release tooling.

> A release plan is richer than a `CHANGELOG.md` entry. The CHANGELOG lists
> _what_ changed, terse and for users. The release plan explains _why_ this
> release is being cut. Keep CHANGELOG entries short and have them link to
> the release plan for context.

## Where release plans live

Release plans live in `plans/releases/`, one file per version, named after
the target version:

```text
plans/
└── releases/
    ├── README.md                # what this directory is for
    ├── .release.template.md      # the template to copy
    ├── v0.3.0.md
    └── v0.4.0.md
```

`aps init` scaffolds `plans/releases/` with the `README.md` and the
`.release.template.md` (on by default for the team and agent-operator
profiles; opt-in for solo — see the init wizard's Components step). To start
a plan, copy the template:

```sh
cp plans/releases/.release.template.md plans/releases/v0.4.0.md
```

The filename **must** match `v<version>.md` (e.g. `v0.3.0.md`,
`v1.2.0-beta.md`) — `aps lint` flags anything else under `releases/` with
`R001`. `README.md` and the dotfile template are not linted.

## When to start a release plan

Start a release plan when **any** of these holds:

- **You are bundling work across multiple modules** into one versioned
  artifact. A release cuts _across_ modules; a release plan is the place
  that bundle gets a name and a narrative.
- **The release marks a strategic shift** you want to find again six months
  later — a framework swap, a distribution change, a breaking-change
  boundary.
- **You want a shared agreement** on success criteria and risks before the
  cut, so the team is aligned on what "done" means for the release.

Anchor the plan to a **cut-date target** early, even a rough one. The plan
is a planning artifact first and draft release notes second — starting it
while scope is still forming is the point.

**Skip it for tiny patch releases.** A terse `CHANGELOG.md` entry carries a
bugfix release on its own. Reach for a release plan when the story is worth
telling.

## Anatomy of a release plan

The [`release.template.md`](../templates/release.template.md) is the
canonical shape. The required spine — enforced by `aps lint` — is:

| Section              | Purpose                                            | Lint |
| -------------------- | -------------------------------------------------- | ---- |
| Header table         | `Target` version and `Status` (lifecycle state)    | R002 |
| `## Release Theme`   | One paragraph: the strategic narrative             | R003 |
| `## What Ships`      | Grouped capability tables with APS module IDs      | R004 |

The header table also carries `Cut from`, `Previous release`, and `Date`
rows — useful context, but not lint-enforced (only `Target` and `Status`
are). The template likewise carries **Success Criteria**, **Risks**, **Out
of Scope**, **Rollout**, **Related**, and a **Retrospective** (filled in
after ship). None of these are lint-enforced, but a useful release plan has
them.

See the [lint codes reference](usage.md#error-codes) for `R001`–`R004`.

## Enumerating the work

The **What Ships** section is the heart of the plan. Build it from work
that has actually landed since the previous release:

1. **Collect the completed items.** Pull `Complete` work items from
   `plans/completed.aps.md` (the task-table roll-up) and the module specs
   under `plans/modules/`. Everything completed since the previous
   release's tag is a candidate.
2. **Group by area, not by module file.** Group capabilities by the story
   that reads best for _this_ release — usually a domain or user-facing
   surface (e.g. "Orchestration CLI", "Distribution"), which may span
   several modules.
3. **Cite the module IDs.** End each area with the APS modules and the work
   items it covers, e.g. `**APS module:** [ORCH](../modules/orchestrate.aps.md)
   (ORCH-001 through ORCH-004 Complete)`. This is the map back from
   narrative to spec.

Keep each capability row to one line — a release plan is a map, not a
manual. Link out to the module specs and decision records for depth.

## Handling scope changes

Scope shifts between starting a plan and cutting the release. Manage it in
the plan itself rather than letting it drift:

- **Use the `## Out of Scope` section** to record what could have been in
  the release but was intentionally deferred, and _why_. This is the
  primary guard against scope creep mid-cut. Name the target the work moved
  to (`deferred to v0.5.0`) or the condition for revisiting it.
- **Move items, don't delete them.** When work slips, move it from What
  Ships to Out of Scope with a one-line reason. The trail is the value.
- **Update the plan during the cut.** The plan is live until `Shipped`.
  Re-grouping What Ships, tightening Success Criteria, and adding risks as
  they surface are all expected.

## Status flow

The `Status` row in the header table tracks where the release is in its
lifecycle:

```text
Planning  →  Cutting  →  Shipped  →  Archived
```

| Status   | Meaning                                                          |
| -------- | --------------------------------------------------------------- |
| Planning | Scope is still forming; work items are being pulled in.         |
| Cutting  | The branch is cut; remaining work is being landed and verified. |
| Shipped  | Released to users. Fill in the ship date and the retrospective. |
| Archived | Superseded by a later release; kept for history.                |

Move the status forward as the release progresses; the transitions are
manual edits to the header row. `Shipped` is the signal to write the
**Retrospective** once the release has been live long enough to learn from.

## Hand-off to release tooling

APS records the _intent_ of a release; it does not cut the release for you.
When the plan reaches `Cutting` → `Shipped`, hand off to your project's
release tooling:

- **Tag and publish** with whatever the project uses — `cargo-dist`,
  `semantic-release`, GoReleaser, a GitHub release, or a manual tag.
- **Write the `CHANGELOG.md` entry** and link it back to the release plan.
  The CHANGELOG is the terse user-facing record; the plan is the context
  behind it.
- **Record the rollout** in the plan's `## Rollout` section: distribution
  channels, migration steps, and how the release is communicated.

Release _automation_ (changelog generation, tagging, publishing) is
deliberately out of scope for APS — it belongs in those dedicated tools.
The release plan is the human-readable bridge between the planning surface
and the release machinery.

## Validating release plans

Run the linter to catch malformed release plans before they pollute the
planning surface:

```sh
aps lint plans/releases/v0.4.0.md
# or lint the whole tree
aps lint plans
```

`aps lint` checks every `plans/releases/v<version>.md` for the required
spine (`R001`–`R004`). A malformed file exits non-zero, so it slots into CI
the same way the rest of the plan tree does.

## Worked example: v0.3.0

This repository dogfoods the pattern. [`plans/releases/v0.3.0.md`](../plans/releases/v0.3.0.md)
is a complete release plan and the local equivalent of the anvil-001 trial
that seeded this module. It shows every section in practice:

- **Header table** — `Target: v0.3.0`, `Status: Shipped`, cut from `main`,
  previous release `v0.2.0`.
- **Release Theme** — "Orchestration & Multi-Agent Reach", the one-paragraph
  arc of the release.
- **What Ships** — grouped by area (Orchestration CLI, TUI Onboarding,
  Layout v2 & Distribution, Process & Branching), each ending with the APS
  module IDs it covers.
- **Success Criteria** — observable, checked-off signals
  (`aps next/start/complete` works end-to-end, migration is reversible).
- **Risks** — migration breakage, state-machine confusion, test coverage —
  each with a mitigation.
- **Out of Scope** — conductor promotion, the `aps release` subcommand, and
  the archive helper, each deferred with a reason.
- **Rollout** and **Related** — distribution, migration, and links back to
  the CHANGELOG, the completed roll-up, and the module specs.

Read it top to bottom before writing your first plan; copying its shape is
faster than starting from the bare template.

## Related

- [Workflow guide → Release Narrative](workflow.md#release-narrative) —
  where release plans sit among the planning archives.
- [`templates/release.template.md`](../templates/release.template.md) — the
  template to copy.
- [CLI reference → lint codes](usage.md#error-codes) — `R001`–`R004`.
- [`plans/releases/v0.3.0.md`](../plans/releases/v0.3.0.md) — the worked
  example.
