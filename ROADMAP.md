# Roadmap

The APS roadmap lives in [`plans/index.aps.md`](plans/index.aps.md) — we use APS
to plan APS.

## Quick Overview

| Horizon               | Focus                                                             | Status  |
| --------------------- | ----------------------------------------------------------------- | ------- |
| **v0.2 Usability**    | Scaffold, templates, docs, validation                             | Done    |
| **v0.3 Distribution** | Install overhaul, multi-harness agents                            | Done    |
| **Near Term**         | Orchestration CLI (`next`/`start`/`complete`/`graph`), TUI wizard | Current |
| **Future**            | Conductor agent, MCP server, GitHub Action, formal spec           | Planned |

See [plans/index.aps.md](plans/index.aps.md) for the full breakdown with modules,
status, and work items.

## Non-Goals

These are explicitly out of scope:

- **Execution engines** — APS describes intent; it doesn't run code
- **Vendor plugins** — No Jira/Linear/Notion plugins (specs are portable markdown)
- **AI training** — Not a dataset for model fine-tuning
- **Hosted services** — No cloud component; everything runs locally

## Contributing

Have ideas for the roadmap? [Open an issue](https://github.com/EddaCraft/anvil-plan-spec/issues)
to discuss, or submit a PR updating [plans/index.aps.md](plans/index.aps.md).
