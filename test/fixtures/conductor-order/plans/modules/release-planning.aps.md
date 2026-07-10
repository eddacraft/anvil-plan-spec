# Release Planning

| ID  | Type      | Owner  | Priority | Status |
| --- | --------- | ------ | -------- | ------ |
| REL | Conductor | @aneki | medium   | Ready  |

<!-- Ready + no **Last reviewed:** field exercises W017; the bad cross-ref
     below exercises W002. Their relative order must match Rust lint_module. -->

## Purpose

Coordinate a release; this fixture locks the W017-before-W002 emission order.

## Coordinated Modules

| Module | Role | Status |
| --- | --- | --- |
| [auth](./auth.aps.md) | ships the auth slice | Complete |

## Cross-Module Work Items

- [AUTH-999](./auth.aps.md) — typo, resolves nowhere (expect W002)

## Work Items

### REL-001: Coordinate the cut

- **Intent:** Coordinate the release cut across the coordinated modules.
- **Expected Outcome:** Every coordinated module is shipped in the cut.
- **Validation:** Release notes list each coordinated module.
