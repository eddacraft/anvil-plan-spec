# Scaffold Module

| ID       | Owner  | Priority | Status   |
| -------- | ------ | -------- | -------- |
| SCAFFOLD | @aneki | high     | Complete |

## Purpose

Provide the initial APS project scaffold so users can create a working
`plans/` directory with templates, rules, and starter documentation.

## In Scope

- Starter `plans/index.aps.md`
- Module, simple feature, monorepo, issue, action, design, and solution templates
- Initial shell installer entry point
- Portable markdown-first layout

## Out of Scope

- Interactive customization beyond the initial shell prompts
- Multi-tool agent packaging
- Native binary distribution

## Interfaces

**Exposes:**

- `scaffold/plans/index.aps.md`
- `scaffold/plans/modules/`
- `templates/*.template.md`
- `scaffold/init.sh`

## Work Items

### SCAFFOLD-001: Create starter plan scaffold — Complete

- **Intent:** Let new projects start with a valid APS structure
- **Expected Outcome:** Scaffold contains starter index, module templates, and
  supporting planning files.
- **Validation:** Fresh scaffold passes `./bin/aps lint scaffold/plans`
- **Files:** scaffold/plans/, templates/
- **Confidence:** high
