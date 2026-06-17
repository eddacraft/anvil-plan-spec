# Release Plans

This directory holds **release narratives** — one markdown file per version
that tells the story of a release: its theme, what ships, success criteria,
risks, and a map back to the APS modules that delivered it.

A release narrative is richer than a CHANGELOG entry. CHANGELOG lists *what*
changed; a release plan explains *why* the release is being cut. Keep
CHANGELOG entries terse and link them here for context.

## Creating a release plan

Copy the template in this directory and rename it for the version you are
cutting:

```sh
cp plans/releases/.release.template.md plans/releases/v0.3.0.md
```

Then fill in the sections. The template (`.release.template.md`) is the
canonical structure; see `docs/workflow.md` → "Release Narrative" for
guidance on when and how to write one.

## Naming

One file per release, named after the target version:

- `v0.3.0.md`
- `v1.2.0-beta.md`

## Status flow

Each release moves through a small lifecycle, tracked in the `Status` row of
the file's header table:

```text
Planning → Cutting → Shipped → Archived
```

- **Planning** — scope is still forming; work items are being pulled in.
- **Cutting** — branch is cut; remaining work is being landed and verified.
- **Shipped** — released to users; fill in the date and retrospective.
- **Archived** — superseded by a later release; kept for history.

## When to skip

Tiny patch releases do not need a narrative — let CHANGELOG carry those.
Write a release plan when the release bundles work across multiple modules
or marks a strategic shift worth finding again later.
