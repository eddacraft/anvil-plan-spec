# Getting Started with APS

This guide walks you through adopting Anvil Plan Spec (APS) in your project.

## Which Template Should I Use?

Start here. Pick based on what you're trying to do:

| Situation                                          | Template                                                  | Time to Value |
| -------------------------------------------------- | --------------------------------------------------------- | ------------- |
| **Just trying APS**                                | [quickstart](../templates/quickstart.template.md)         | 5 minutes     |
| **Small feature** (1-3 work items, self-contained) | [simple](../templates/simple.template.md)                 | 15 minutes    |
| **Module with boundaries** (interfaces, deps)      | [module](../templates/module.template.md)                 | 30 minutes    |
| **Multi-module initiative**                        | [index](../templates/index.template.md)                   | 1 hour        |
| **Large initiative** (6+ modules)                  | [index-expanded](../templates/index-expanded.template.md) | 1-2 hours     |
| **Monorepo** (multiple packages/apps)              | [index-monorepo](../templates/index-monorepo.template.md) | 1-2 hours     |
| **Breaking a work item into actions**              | [actions](../templates/actions.template.md)               | 15 minutes    |
| **Technical/architectural design**                 | [design](../templates/design.template.md)                 | 30 minutes    |
| **Tracking dev-time discoveries**                  | [issues](../templates/issues.template.md)                 | 10 minutes    |

### Decision Tree

```mermaid
graph TD
    A[New work item] --> B{How big?}
    B -->|Large initiative| C[Create Index]
    B -->|Single module| D{Self-contained?}
    B -->|Single work item| E[Add to existing module]

    C --> F[Add modules]
    D -->|Yes| G[Use simple.template.md]
    D -->|No| H[Use module.template.md]

    F --> I{Ready to implement?}
    G --> I
    H --> I

    I -->|Yes| J[Add Work Items, set status=Ready]
    I -->|No| K[Leave as Draft, list blockers]

    J --> L{Complex work item?}
    L -->|Yes| M[Create Action Plan file]
    L -->|No| N[Execute directly]
```

## Quick Start

**Want to see APS in action first?** Check the [examples](../examples/):

- [User Authentication](../examples/user-auth/) — Adding auth to an existing app
- [OpenCode Companion](../examples/opencode-companion/) — Building a new tool

**Solo developer?** You don't need the full ceremony:

- Use `simple.template.md` for most features
- Skip formal modules — go straight to work items
- Only create an Index if you're planning weeks of work
- Action plans are optional — use when a work item feels complex

**Ready to scaffold?** Run this in your project:

```bash
# One-liner install (curl)
curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install | bash

# Pin to a specific version
curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install | VERSION=v0.3.0 bash

# Or from a cloned APS repo
./scaffold/install ./your-project
```

Windows PowerShell uses the native installer and the same `aps` commands:

```powershell
# Install and open the native onboarding TUI
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install.ps1)))

# Pin a release when required
$env:APS_VERSION = "v0.6.0"
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install.ps1)))
```

The no-argument installer opens `aps init` automatically. If you installed with
Scoop or `--cli`, run `aps init` yourself. The Ratatui-based wizard walks you
through agent ports, modules, and project context, then creates `plans/` with
templates and `aps-rules.md` for AI guidance.

## Prerequisites

- A project repository (any language/framework)
- Familiarity with markdown

## Setting Up Manually

If you prefer manual setup over the scaffold script:

### 1. Create folder structure

```text
your-project/
├── plans/
│   ├── index.aps.md           # Your main plan
│   ├── issues.md              # Development-time discoveries
│   ├── modules/               # Module specs
│   │   └── feature.aps.md
│   ├── execution/             # Action plan files
│   │   └── FEAT-001.actions.md
│   ├── designs/               # Technical designs (optional)
│   │   └── 2026-01-05-auth.design.md
│   └── decisions/             # ADRs (optional)
│       └── 001-use-jwt.md
└── .aps/                      # Tooling root (gitignored content lives here)
    └── context/               # Ephemeral context packages from `aps start`
        └── FEAT-001.md
```

### 2. Create your Index

Copy `index.template.md` to `plans/index.aps.md`. Fill in:

1. **Problem** — What are you solving?
2. **Success Criteria** — How do you know you're done?
3. **Modules** — List each bounded area of work

> **Tip:** The Index is non-executable. Focus on intent, not implementation.

### 3. Create Modules

For each module, create a file in `plans/modules/`:

- `module.template.md` — For modules with interfaces and dependencies
- `simple.template.md` — For small, self-contained features

Fill in Purpose, Scope, and leave Work Items empty until Ready.

### 4. Write a Design (Optional)

For complex or multi-module work where the architecture isn't obvious, create a
design doc in `plans/designs/` before defining work items:

- Use [design.template.md](../templates/design.template.md) as a starting point
- Name it `YYYY-MM-DD-slug.design.md`
- Focus on the Problem, approach, and key Decisions
- Link to the modules it covers via the metadata table

Skip this for straightforward features, bug fixes, or work where the approach is
already established.

### 5. Add Work Items When Ready

Work Items are **execution authority**. Only add them when:

- The module scope is clear
- Dependencies are resolved
- You're ready to implement

Each work item needs:

- **Intent** — One sentence on what it achieves
- **Expected Outcome** — Testable result
- **Validation** — How to verify completion

### 6. Generate Action Plans (Optional)

For complex work items, create an actions file in `plans/execution/`.

Action plans translate "what to achieve" into "what actions to take":

- Each action has a **checkpoint** (observable state)
- Actions describe **what**, not **how**
- Actions can be grouped into **waves** for concurrent agents to execute in parallel

### 7. Track Issues & Questions

As you develop, you'll discover issues and questions that emerge during implementation — things that weren't apparent during planning. Log these in `plans/issues.md`:

- **Issues (ISS-NNN)** — Bugs, limitations, tech debt, edge cases
- **Questions (Q-NNN)** — Unknowns that need answers, deferred decisions

This keeps planning-level concerns visible without cluttering work items or your bug tracker.

## Driving Work with the CLI

Once you have a plan with at least one `Ready` work item, the orchestration
commands take over:

```bash
aps next                              # what's the next ready item across modules?
aps start AUTH-003                    # marks it In Progress, writes .aps/context/AUTH-003.md
# ...implement, run validation step...
aps complete AUTH-003 --learning "Retry on 5xx improved success rate by 18%"
aps graph auth                        # see work items + dependency arrows for a module
```

`aps start` enforces that every dependency is `Complete`. The context package
it writes (`.aps/context/<ID>.md`) is a focused brief — module scope,
decisions, upstream learnings, and related files — designed to be pasted into
any AI agent.

Full reference, error codes, and JSON output: [usage.md](./usage.md).

## Monorepo Setup

Working in a monorepo with multiple packages/apps? See [monorepo.md](./monorepo.md) for full guidance. Key differences:

1. **Use the monorepo index template** — [index-monorepo.template.md](../templates/index-monorepo.template.md)
2. **Add Packages to modules** — Each module declares which packages it affects
3. **"What's Next" view** — Prioritized queue across all packages
4. **"By Package" view** — Navigation grouped by package
5. **Session rituals** — Start/end rituals keep docs in sync

```text
monorepo/
├── plans/
│   ├── index.aps.md              # Uses index-monorepo format
│   └── modules/
│       ├── 01-auth.aps.md        # Packages: core, api
│       └── 02-cli.aps.md         # Packages: cli, shared
├── apps/
│   └── api/
└── packages/
    └── core/
```

## Index Template Formats

APS provides three index formats:

**Table format** (`index.template.md`) — compact, scannable, best for 2-6 modules:

```markdown
| Module                        | Purpose             | Status | Dependencies |
| ----------------------------- | ------------------- | ------ | ------------ |
| [auth](./modules/auth.aps.md) | User authentication | Ready  | —            |
```

**List format** (`index-expanded.template.md`) — more readable with many modules:

```markdown
### auth

- **Path:** ./modules/auth.aps.md
- **Status:** Ready
- **Priority:** high
- **Dependencies:** database, session
```

## Working with AI Assistants

APS includes prompts for AI tools:

| Task               | Prompt                                  |
| ------------------ | --------------------------------------- |
| Planning           | `docs/ai/prompting/index.prompt.md`     |
| Module design      | `docs/ai/prompting/module.prompt.md`    |
| Work item creation | `docs/ai/prompting/work-item.prompt.md` |
| Execution          | `docs/ai/prompting/actions.prompt.md`   |

OpenCode/Claude users: see `docs/ai/prompting/opencode/` for optimised variants.

APS ships first-class agent definitions for Claude Code, Codex, GitHub Copilot,
and OpenCode, plus Grok Build support via auto-discovered `.agents/skills/` —
`aps init` lets you select which tools to install agents for. See [docs/agents.md](./agents.md) for details on each port and the
APS-aware agents (planner, librarian, conductor).

When you scaffold APS, it includes `aps-rules.md` — point your AI agent at this
file and it will follow APS conventions automatically.

## Next Steps

- Review [workflow.md](./workflow.md) for day-to-day usage patterns
- Read [monorepo.md](./monorepo.md) if working in a multi-package repository
- Read [AGENTS.md](../AGENTS.md) for AI collaboration rules in this repo
