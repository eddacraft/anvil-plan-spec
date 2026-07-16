# APS Prompts — Grok Build (stub)

Grok Build follows the [generic APS prompts](../README.md) unchanged; this
stub exists so Grok users have a named entry point (D-006/D-040). Per the
[variant-vs-stub policy](../README.md#variant-vs-stub-policy), Grok gets no
full variant because its workflow does not diverge from the generic lifecycle.

Use the generic prompts directly:

- [index.prompt.md](../index.prompt.md) — index plans
- [module.prompt.md](../module.prompt.md) — module design
- [work-item.prompt.md](../work-item.prompt.md) — work items
- [actions.prompt.md](../actions.prompt.md) — action plans

Grok-specific wiring (all automatic — D-040):

- **Instructions:** Grok Build reads the [`AGENTS.md`](../../../../AGENTS.md)
  instruction-file family from the repo root down to the working directory.
  No `GROK.md` is needed.
- **Skills:** the planning skill at `.agents/skills/aps-planning/` (select
  `grok` in `aps init`, or run `aps setup grok`) is auto-discovered — no link
  or install command. Grok also picks up `.claude/` skills and agents when
  other tools installed them.
- **Subagents:** APS ships no Grok-specific agent files; if you want
  foreground `subAgents`, derive their instructions from
  `scaffold/agents/core/`.
