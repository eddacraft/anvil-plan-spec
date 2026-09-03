<!--
APS Design Template
====================
Use for: Technical/architectural designs that inform module and work item structure.
Location: designs/ (project root, peer to plans/)
Naming: YYYY-MM-DD-slug.design.md

A design doc captures architectural thinking BEFORE committing to work items.
It's the "why this approach" companion to the plan's "what to build".

FLEXIBLE: Not every field is required. The linter issues warnings, not errors.
If an agent creates a free-form design, that's fine — normalise later by adding
the minimum fields (Problem, Design, Status metadata).
-->

# [Design Title]

| Field   | Value                                     |
| ------- | ----------------------------------------- |
| Status  | Draft                                     |
| Created | YYYY-MM-DD                                |
| Modules | [module-id](plans/modules/NN-name.aps.md) |

## Problem

[What problem does this design address? Why can't the existing approach work?]

## Constraints _(optional)_

- [Constraint that shapes the design — performance, compatibility, security, etc.]

## Design

[Core of the document. Describe the approach, architecture, data flow.
Use diagrams (mermaid), tables, or pseudocode where they help.
Focus on WHAT the architecture looks like, not HOW to code it.]

## Alternatives Considered _(optional)_

| Alternative | Pros | Cons | Verdict             |
| ----------- | ---- | ---- | ------------------- |
| [Option A]  | ...  | ...  | Rejected — [reason] |

## Implementation Notes _(optional)_

[How should this design translate to modules and work items?
Which modules does it affect? What order should work proceed?]

## Decisions

- **D-001:** [Decision] — [rationale]

## Open Questions _(optional)_

- [ ] [Unresolved question that needs answering before work begins]
