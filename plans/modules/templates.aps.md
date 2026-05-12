# Templates Module

| ID  | Owner  | Priority | Status   |
| --- | ------ | -------- | -------- |
| TPL | @aneki | high     | Complete |

## Purpose

Keep APS templates minimal, consistent, and easy for agents and humans to fill
without over-prescribing implementation detail.

## In Scope

- Index, module, monorepo index, simple feature, quickstart, issues, actions,
  design, and solution templates
- Optional-field guidance
- Terminology consistency across templates

## Out of Scope

- Project-specific templates
- Vendor-specific task formats

## Interfaces

**Exposes:**

- `templates/*.template.md`
- `scaffold/plans/modules/.*.template.md`
- `scaffold/plans/execution/.actions.template.md`

## Work Items

### TPL-001: Mark optional fields and simplify templates — Complete

- **Intent:** Reduce adoption friction without weakening APS structure
- **Expected Outcome:** Templates distinguish required structure from optional
  planning detail and use consistent field names.
- **Validation:** Templates lint and generated fixtures remain valid
- **Files:** templates/, scaffold/plans/
- **Confidence:** high
