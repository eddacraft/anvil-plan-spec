# APS Quick Reference

Compact reference for the Anvil Plan Spec format. Read SKILL.md first.

> **Canonical sources:** This is a context-window-friendly summary. Full details
> live in the upstream Anvil Plan Spec repo:
> <https://github.com/EddaCraft/anvil-plan-spec> (`docs/getting-started.md`,
> `templates/`, and `scaffold/plans/aps-rules.md`). The local
> `plans/aps-rules.md` in your project is the operative agent guidance. If this
> file and the canonical sources diverge, the canonical sources win.

## Hierarchy

```
Index (non-executable overview)
  └─ Module (bounded work area)
       └─ Work Item (execution authority)
            └─ Action Plan (execution breakdown, optional)
```

## Template Picker

| You need... | Use | File pattern |
|-------------|-----|--------------|
| Technical design for complex work | Design | `designs/YYYY-MM-DD-slug.design.md` |
| Quick feature (1-3 items) | Simple | `feature.aps.md` |
| Bounded area with interfaces | Module | `NN-name.aps.md` |
| Multi-module initiative | Index + Modules | `index.aps.md` + `modules/` |
| Break down complex work item | Action Plan | `execution/ID.actions.md` |

## Required Work Item Fields

Every work item **must** have:

```markdown
### PREFIX-001: [Title]

- **Intent:** [What outcome — one sentence]
- **Expected Outcome:** [Observable/testable result]
- **Validation:** `[command to verify]`
```

## Optional Work Item Fields

```markdown
- **Confidence:** low | medium | high
- **Dependencies:** OTHER-001, OTHER-002
- **Files:** src/auth.ts, tests/auth.test.ts
- **Non-scope:** [What won't change]
- **Status:** Draft | Ready | In Progress | Blocked | Complete
```

## Module Metadata Table

```markdown
| ID | Owner | Priority | Status | Packages |
|----|-------|----------|--------|----------|
| AUTH | @user | medium | Draft | *(monorepo only)* |
```

## Status Values

| Status | Meaning |
|--------|---------|
| Draft | Defining scope, not ready for work |
| Ready | Approved, work items defined, can begin |
| In Progress | Actively being worked on |
| Blocked | Waiting on external dependency |
| Complete | All work items validated and done |

## Action Plan Checkpoint Format

```markdown
### Action N — [Verb] [target]

**Purpose:** [Why this action exists]
**Produces:** [Concrete artefacts]
**Checkpoint:** [Observable state — max 12 words]
**Validate:** `[command]` *(optional)*
```

## File Layout

```
designs/                          # Technical designs (optional, project root)
└── YYYY-MM-DD-slug.design.md

plans/
├── aps-rules.md              # Agent guidance
├── index.aps.md              # Root plan
├── modules/
│   └── NN-name.aps.md        # Module specs
├── execution/
│   └── ID.actions.md         # Action plans
└── decisions/
    └── NNN-title.md          # ADRs
```

## Design Document Format

Minimum fields for a design doc (warnings only — free-form is accepted):

```markdown
# [Design Title]

| Field | Value |
|-------|-------|
| Status | Draft / Approved / Superseded |
| Created | YYYY-MM-DD |
| Modules | [module-id](plans/modules/NN-name.aps.md) |

## Problem
[What problem does this design address?]

## Design
[Architecture, approach, data flow]
```

## Naming Rules

- Design docs: `YYYY-MM-DD-slug.design.md` (in `designs/`)
- Module files: zero-padded prefix by dependency order (`01-`, `02-`)
- Work item IDs: module prefix + three digits (`AUTH-001`)
- Action plans: `WORK-ITEM-ID.actions.md` or `MODULE.actions.md`

## Validation

```bash
./bin/aps lint                          # Lint all plans/
./bin/aps lint plans/modules/auth.aps.md  # Lint specific file
./bin/aps lint --json                   # JSON output
```

## Error Codes

| Code | Rule |
|------|------|
| E001 | Missing `## Purpose` section |
| E002 | Missing `## Work Items` section |
| E003 | Missing ID/Status metadata table |
| E004 | Missing `## Modules` section (index) |
| E005 | Work item missing Intent, Expected Outcome, or Validation |
| W001 | Work item ID format should be `PREFIX-NNN` |
| W003 | Dependency references unknown work item ID |
| W004 | Empty required section |
| W005 | Status=Ready but no work items defined |
| W014 | Design doc missing `## Problem` section |
| W015 | Design doc missing `## Design` section |
| W016 | Design doc missing metadata table with Status |
