# APS Prompts — GitHub Copilot (stub)

Copilot follows the [generic APS prompts](../README.md) unchanged; this stub
exists so Copilot users have a named entry point (D-006). Per the
[variant-vs-stub policy](../README.md#variant-vs-stub-policy), Copilot gets no
full variant because its workflow does not diverge from the generic lifecycle.

Use the generic prompts directly:

- [index.prompt.md](../index.prompt.md) — index plans
- [module.prompt.md](../module.prompt.md) — module design
- [work-item.prompt.md](../work-item.prompt.md) — work items
- [actions.prompt.md](../actions.prompt.md) — action plans

Copilot-specific wiring:

- **Instructions:** Copilot reads [`AGENTS.md`](../../../../AGENTS.md) at the
  repo root — no separate instruction file needed.
- **Agents:** APS planner/librarian/conductor agents install to
  `.github/agents/` (select `copilot` in `aps init`, or run
  `aps setup copilot`). Commit them so Copilot picks them up.
- **Skills:** Copilot auto-discovers the planning skill at
  `.claude/skills/aps-planning/`.
