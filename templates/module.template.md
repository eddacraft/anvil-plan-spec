<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!--
Work can begin when: status=Ready AND work items exist.
Module ID: Use 2-6 uppercase chars (AUTH, PAY, UI, CORE, etc.)
File naming: NN-name.aps.md by dependency order (01-core.aps.md, 02-auth.aps.md)
Packages: For monorepos, list affected packages (see docs/monorepo.md)
-->

# [Module Title]

| ID   | Owner     | Priority | Status | Packages          |
| ---- | --------- | -------- | ------ | ----------------- |
| AUTH | @username | medium   | Draft  | _(monorepo only)_ |

## Purpose

[Why this module exists and what problem it solves — one paragraph max]

## In Scope

- [What this module handles]

## Out of Scope _(optional)_

- [What belongs elsewhere — only if clarification needed]

## Interfaces _(optional)_

**Depends on:**

- [Module/Service] — [what we need]

**Exposes:**

- [API/function] — [what others use]

## Constraints _(optional)_

- [Architectural rules, e.g., "AUTH must not import from UI"]

## Ready Checklist

Change status to **Ready** when:

- [ ] Purpose and scope are clear
- [ ] Dependencies identified (or confirmed none)
- [ ] At least one work item defined

## Work Items

<!--
Required: Intent, Expected Outcome, Validation
Optional: Non-scope, Files, Dependencies, Confidence, Risks

Confidence levels:
- high: Clear requirements, familiar patterns
- medium: Some unknowns, moderate risk
- low: Exploratory, high uncertainty
-->

### AUTH-001: [Work item title]

- **Intent:** [What this achieves — one sentence]
- **Expected Outcome:** [Observable/testable result]
- **Validation:** `[test command]`
- **Confidence:** medium
- **Packages:** [Affected packages] _(monorepo only — inherits from module if omitted)_
- **Non-scope:** [What won't change] _(optional)_
- **Files:** [Likely files] _(optional — best effort)_
- **Dependencies:** AUTH-XXX _(optional)_

### AUTH-002: [Another work item]

- **Intent:** [What this achieves]
- **Expected Outcome:** [Testable result]
- **Validation:** `[test command]`
- **Confidence:** medium

## Execution _(optional)_

Action Plan: [./execution/AUTH.actions.md](./execution/AUTH.actions.md)

## Decisions _(optional)_

- **D-001:** [Decision] — [rationale]

## Notes _(optional)_

- [Additional context]
