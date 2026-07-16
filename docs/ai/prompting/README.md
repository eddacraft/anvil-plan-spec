# APS Prompting Entry Points

The generic prompts in this directory are the source of truth for driving the
APS lifecycle from any AI tool:

| Prompt                                       | Use for                              |
| -------------------------------------------- | ------------------------------------ |
| [index.prompt.md](./index.prompt.md)         | Creating or revising an index plan   |
| [module.prompt.md](./module.prompt.md)       | Designing a module                   |
| [work-item.prompt.md](./work-item.prompt.md) | Specifying work items                |
| [actions.prompt.md](./actions.prompt.md)     | Writing checkpoint-based action plans |

They teach the canonical status vocabulary
(`Draft / Ready / In Progress / Complete / Blocked`, D-037) and defer to
[`AGENTS.md`](../../../AGENTS.md) and `plans/aps-rules.md` for lifecycle rules.

## Variant-vs-stub policy

_(PROMPTS-002; resolves module D-001.)_ A harness gets a **full prompt
variant** only when its workflow diverges from the generic APS lifecycle —
different flow control, a native execution layer, or orchestration features
the generic prompts can't express. Otherwise it gets a **one-screen stub**
that points at the generic prompts and the harness's instruction file.

When adding a new harness, default to a stub. Promote it to a variant only
once a real workflow difference is documented — never pre-emptively.

## Coverage

| Harness     | Entry point                        | Shape                |
| ----------- | ---------------------------------- | -------------------- |
| generic     | this directory                     | Full set             |
| OpenCode    | [opencode/](./opencode/)           | Full variant (flow control differs) |
| Claude Code | [claudecode/](./claudecode/)       | Orchestration deltas (Tasks integration) |
| Copilot     | [copilot/](./copilot/README.md)    | Stub                 |
| Codex       | [codex/](./codex/README.md)        | Stub                 |
| Grok        | [grok/](./grok/README.md)          | Stub                 |

The five harnesses APS targets are Claude Code, Copilot, Codex, OpenCode, and
Grok (D-013/D-019 as amended by D-040).
