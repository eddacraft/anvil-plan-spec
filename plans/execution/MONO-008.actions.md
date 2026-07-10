# Action Plan: MONO-008

| Field      | Value                                                     |
| ---------- | --------------------------------------------------------- |
| Source     | [../modules/monorepo.aps.md](../modules/monorepo.aps.md) |
| Work Item  | MONO-008 — Child-scope module status across trees         |
| Created by | @aneki / AI                                               |
| Status     | In Progress                                               |

## Goal

Resolve federated module statuses within their owning child plan and warn when
the same module ID is used by multiple child plans, preserving D-002's bare-ID
convention.

## Actions

### Action 1 — Specify colliding module behavior in the shared fixture

**Purpose**
Prove that Draft and Ready sibling modules with one ID are independently gated.

**Produces**

- A fixture scenario with one repeated module ID in two child trees.
- Bash and Rust assertions for child-scoped `aps next` behavior.

**Checkpoint**
Each child reports only work allowed by its own module status.

**Validate**
`./test/run.sh` and `cargo test --manifest-path cli/Cargo.toml`

### Action 2 — Scope orchestration module-status lookups by child

**Purpose**
Prevent one child module from overwriting a same-ID sibling's status.

**Produces**

- Child-aware Bash and Rust status lookup for work-item gating and module
  dependencies.

**Checkpoint**
Sibling module statuses cannot alter each other's ready queue.

**Validate**
`./bin/aps next --plans test/fixtures/monorepo/plans --child core`

**Depends on** 1

### Action 3 — Add deterministic module-ID collision linting

**Purpose**
Make permitted but operationally ambiguous module-ID reuse visible to authors.

**Produces**

- Matching Bash, PowerShell, and Rust warning output on the federation parent.
- Clean-fixture coverage proving no false positive.

**Checkpoint**
Lint identifies a repeated module ID and both child owners.

**Validate**
`./test/run.sh`, `pwsh -File test/ps-parity.ps1`, and `cargo test --manifest-path cli/Cargo.toml`

**Depends on** 1

### Action 4 — Verify and reconcile the delivered behavior

**Purpose**
Record fresh evidence and keep the APS plan true after implementation.

**Produces**

- Validation evidence for every supported implementation surface.
- Updated MONO-008 status and result summary.

**Checkpoint**
The plan records verified child-scoped module status behavior.

**Validate**

```sh
./bin/aps lint plans
./test/run.sh
cargo test --manifest-path cli/Cargo.toml
npx markdownlint-cli "**/*.md"
```

**Depends on** 2, 3

## Completion

- [ ] All checkpoints validated
- [ ] Bash and Rust resolve each child's module status independently
- [ ] Bash, PowerShell, and Rust lint parity covers module-ID collisions
- [ ] MONO-008 results record validation evidence
