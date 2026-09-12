# Roadmap

The APS roadmap lives in [`plans/index.aps.md`](plans/index.aps.md) — we use APS
to plan APS.

## Quick Overview

| Horizon                   | Focus                                                                  | Status  |
| ------------------------- | ---------------------------------------------------------------------- | ------- |
| **v0.2 Usability**        | Scaffold, templates, docs, validation                                  | Done    |
| **v0.3 Orchestration**    | Orchestration CLI (`next`/`start`/`complete`/`graph`), multi-agent reach | Done    |
| **v0.4 Distribution**     | Crosscutting conductor modules, binary-first install, TUI wizard        | Done    |
| **v0.5–v0.6 Monorepo**    | Federated nested plans, the tagged package tier, full-lifecycle tooling | Done    |
| **v0.7 Team Foundations** | Managed skill freshness, JSON export, GitHub Action, MCP server         | Done    |
| **v0.8 Harnesses**        | Twelve supported harnesses from one shared planning asset               | Done    |
| **Near Term**             | Release tooling (`aps release`), plan-doctor accuracy, lint parity      | Current |
| **Future**                | Team coordination plane, progress journals, profile-aware CLI           | Planned |

## Non-Goals

These are explicitly out of scope:

- **Execution engines** — APS describes intent; it doesn't run code
- **Vendor plugins** — No Jira/Linear/Notion plugins (specs are portable markdown)
- **AI training** — Not a dataset for model fine-tuning
- **Hosted services** — No cloud component; everything runs locally

## Contributing

Have ideas for the roadmap? [Open an issue](https://github.com/EddaCraft/anvil-plan-spec/issues)
to discuss, or submit a PR updating [plans/index.aps.md](plans/index.aps.md).
