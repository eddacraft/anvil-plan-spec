# Integrations

Low-lock-in ways to expose APS plans to external workflows. Markdown in
`plans/` stays the source of truth; everything here is read-only over it.

## GitHub Action: APS Lint

The repo root ships a composite action (`action.yml`) so a team gets central
lint enforcement with one line — no install scripts to copy or keep updated:

```yaml
name: APS
on:
  pull_request:
permissions:
  contents: read
jobs:
  aps:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: eddacraft/anvil-plan-spec@v0.7.0
        with:
          plans-dir: plans
```

The action runs its own vendored CLI, so the lint version is pinned by the
action ref — bump the tag to bump the linter, in lockstep for every teammate
and CI run.

### Inputs

| Input            | Default          | Effect                                                                 |
| ---------------- | ---------------- | ---------------------------------------------------------------------- |
| `plans-dir`      | `plans`          | Directory linted (federated trees are traversed from its index)        |
| `strict`         | `false`          | Also fail on a `.aps/config.yml` `cli_version` pin mismatch            |
| `rollup-comment` | `false`          | Maintain one sticky PR comment with plan status                        |
| `github-token`   | `${{ github.token }}` | Token for the comment                                              |

### Plan-status PR comment

With `rollup-comment: true` (and `permissions: pull-requests: write`), the
action posts a single comment — updated in place on every push — showing the
lint summary plus `aps rollup` for federated trees or the
`aps next --by-package` ready queue otherwise. That gives reviewers and leads
plan visibility without running the CLI:

```yaml
    permissions:
      contents: read
      pull-requests: write
    steps:
      - uses: actions/checkout@v5
      - uses: eddacraft/anvil-plan-spec@v0.7.0
        with:
          rollup-comment: "true"
```

Prefer wiring the workflow yourself (JSON artefacts, path filters)? Copy
[`ci-lint-example.yml`](./ci-lint-example.yml) instead — same checks, no
action dependency.

## JSON export: `aps export --json`

`aps export --json` emits a machine-readable snapshot of a plan tree on
stdout — the substrate for dashboards, sync experiments, and anything else
that should read plan state without parsing markdown:

```bash
aps export --json            # nearest plans/ dir
aps export --json --plans docs/plans
```

Implemented in the Rust binary and the bash CLI with byte-identical output
(D-039). Ordering is deterministic: modules in file order, work items in
document order — running it twice on the same tree byte-matches.

### Shape (schema `aps-export/v1`)

```json
{
  "schema": "aps-export/v1",
  "generated_by": "aps 0.7.0",
  "plans_dir": "plans",
  "modules": [
    {
      "id": "AUTH",
      "file": "plans/modules/auth.aps.md",
      "status": "In Progress",
      "type": null,
      "packages": null,
      "work_items": [
        {
          "id": "AUTH-001",
          "title": "Implement login endpoint",
          "status": "Ready",
          "line": 21,
          "dependencies": ["AUTH-002", "core:SSO-001"],
          "packages": null
        }
      ]
    }
  ]
}
```

Field notes:

- `schema` — versioned; consumers should check it. The shape is **v1 and may
  still move** while the export is young; pin your `aps` version if you build
  on it.
- `status` — the canonical vocabulary (`Draft / Ready / In Progress /
  Complete / Blocked`), normalised the same way `aps next` normalises it.
- `dependencies` — tokens as written, including cross-tree `child:ID` refs.
- `packages` — the effective `Packages:` tags (item field, else module
  column), `null` when untagged.
- Absent values are `null`, never omitted — consumers can rely on the keys.

Validation commands and richer tree metadata land with future iterations;
GitHub issue/project sync experiments build on this export rather than on
private parsers.
