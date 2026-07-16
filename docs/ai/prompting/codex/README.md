# APS Prompts — Codex (stub)

Codex follows the [generic APS prompts](../README.md) unchanged; this stub
exists so Codex users have a named entry point (D-006). Per the
[variant-vs-stub policy](../README.md#variant-vs-stub-policy), Codex gets no
full variant — its agent mechanism differs (TOML roles), but the planning
workflow itself matches the generic lifecycle.

Use the generic prompts directly:

- [index.prompt.md](../index.prompt.md) — index plans
- [module.prompt.md](../module.prompt.md) — module design
- [work-item.prompt.md](../work-item.prompt.md) — work items
- [actions.prompt.md](../actions.prompt.md) — action plans

Codex-specific wiring:

- **Instructions:** Codex reads [`AGENTS.md`](../../../../AGENTS.md) at the
  repo root.
- **Agents:** roles install to `.codex/agents/*.toml` (select `codex` in
  `aps init`, or run `aps setup codex`); merge the generated
  `codex-config-snippet.toml` into `.codex/config.toml`, then spawn with
  `/agent spawn aps-planner`.
- **Skills:** the planning skill installs to `.agents/skills/aps-planning/`;
  register it with `codex skills install .agents/skills/aps-planning`.
