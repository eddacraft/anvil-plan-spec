<!-- APS: See docs/workflow.md → "Release Narrative" for guidance -->
<!--
Filename: plans/releases/v[version].md  (e.g. v0.3.0.md, v1.2.0-beta.md)

A release narrative is richer than a CHANGELOG entry. CHANGELOG lists what
changed; the release doc tells the story of *why* this release is being
cut — its theme, success criteria, risks, and a map back to the APS
modules that shipped it.

When to write one:
- You are cutting a release that bundles work across multiple modules.
- A release marks a strategic shift you want future you to find easily.
- Skip for tiny patch releases — let CHANGELOG carry those.

Keep CHANGELOG entries terse and have them link here for context.
-->

# Release Plan: v[version]

| Field            | Value                                       |
| ---------------- | ------------------------------------------- |
| Target           | v[version]                                  |
| Cut from         | `main` (or your integration branch)         |
| Previous release | v[previous-version]                         |
| Status           | Planning / Cutting / Shipped                |
| Date             | YYYY-MM-DD (planning), YYYY-MM-DD (shipped) |

## Release Theme

**[Theme in 3-7 words]** — one short paragraph capturing the narrative arc
of the release. What changed strategically? What new capability does this
unlock for users? Write the version of this you would want to see six
months from now when you have forgotten the details.

## What Ships

Group by area (module, surface, or user-facing capability — whichever reads
best for this release). One table per area.

### [Area name] ([MODULE-ID(s) covered])

Short framing sentence about the area.

| Area         | Detail                                 |
| ------------ | -------------------------------------- |
| [Subsection] | One-line description of the capability |
| [Subsection] | Another capability                     |

**APS modules:** [MODULE-ID] (X/Y work items done), [MODULE-ID] (…)

### [Another area]

| Area | Detail |
| ---- | ------ |
| …    | …      |

## Success Criteria

How will we know this release is successful? Concrete, observable signals.

- [ ] [Criterion — usually a user-visible outcome, sometimes a metric]
- [ ] [Criterion]
- [ ] [Criterion]

## Risks

Surface the things most likely to go wrong, with mitigations.

| Risk               | Impact | Mitigation                   |
| ------------------ | ------ | ---------------------------- |
| [Risk description] | High   | [What we are doing about it] |
| [Risk description] | Medium | [What we are doing about it] |

## Out of Scope

Explicitly call out work that could be in this release but is intentionally
deferred — protects against scope creep mid-cut.

- [Capability] — deferred to v[next-version] because [reason]
- [Capability] — moved to backlog; revisit when [condition]

## Rollout

How does this release reach users?

- Distribution channels: [npm / cargo / homebrew / GitHub release / etc.]
- Migration steps required: [none / link to migration guide]
- Communication: [CHANGELOG entry, blog post, release announcement]

## Related

- **CHANGELOG entry:** [CHANGELOG.md#v[version]](../../CHANGELOG.md)
- **Completed roll-up:** [plans/completed.aps.md](../completed.aps.md)
- **Module specs:** links to the `plans/modules/*.aps.md` files covered
- **Decision records:** any ADRs in `plans/decisions/` that shaped the cut

## Retrospective _(after ship)_

Filled in once the release has been out long enough to learn from. Keep it
short — bullet points, not essays.

**What went well:**

- [Observation]

**What we would do differently:**

- [Observation]

**Surprises:**

- [Observation]
