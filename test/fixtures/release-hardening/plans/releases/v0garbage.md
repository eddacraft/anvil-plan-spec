# Release Plan: v0.3.0

| Field            | Value               |
| ---------------- | ------------------- |
| Target           | v0.3.0              |
| Cut from         | `main`              |
| Previous release | v0.2.0 (2026-05-01) |
| Status           | Shipped             |
| Date             | 2026-06-01          |

## Release Theme

**Release-plan linting** — the fixture release narrative every CLI must accept
unchanged. It carries the full shape R001–R004 look for: a `v<version>.md`
name, a header table with Target and Status rows, and both narrative sections.

## What Ships

| Area | Detail |
| ---- | ------ |
| Lint | R001–R004 in all three CLIs |

## Verification

- `aps lint plans` reports this file valid in bash, Rust and PowerShell.
