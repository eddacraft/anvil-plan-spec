# Solutions Library

Solutions are post-implementation writeups that capture **what we figured out**
during compound engineering — non-obvious approaches, dead ends avoided, and
patterns worth reusing. They live next to specs so future plans can start with
known answers rather than rediscovering them.

See [docs/workflow.md](workflow.md) for how the Learn phase feeds this folder.

## Categories

### Planning

Solutions related to APS planning, orchestration, and lifecycle management.

- [Context Package Regeneration](planning/context-package-regeneration.md) —
  How `aps start` regenerates `.aps/context/<ID>.md` on every invocation, and
  why ephemeral context beats long-lived snapshots.

## Adding a Solution

1. Pick the right category folder (create one if none fit).
2. Use [`templates/solution.template.md`](../../templates/solution.template.md)
   as the starting point.
3. Title the file with a kebab-case slug describing the problem
   (`retry-with-jitter.md`, not `auth-fix.md`).
4. Link it from the relevant section in this index.
5. Cross-link from the originating work item or design doc.

Solutions should be **short, specific, and reusable**. If a writeup is project-
specific or time-bound, it probably belongs in a design doc or learning line
on the work item instead.
