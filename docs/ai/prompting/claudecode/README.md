# Claude Code Prompts

Prompt templates for using APS with Claude Code's Tasks feature.

## Overview

Claude Code Tasks provide runtime coordination for executing APS work items:

- Dependencies via `blockedBy`
- Multi-session collaboration
- Wave-based parallel execution
- Real-time status broadcasting

APS remains the **planning layer** (durable, git-versioned), while Tasks handle **execution** (ephemeral, runtime).

## Prompts

| Prompt                                                       | Use When                                 |
| ------------------------------------------------------------ | ---------------------------------------- |
| [tasks-from-module.prompt.md](./tasks-from-module.prompt.md) | Starting work on an APS module           |
| [wave-planning.prompt.md](./wave-planning.prompt.md)         | Planning parallel execution              |
| [agent-assignment.prompt.md](./agent-assignment.prompt.md)   | Distributing work across multiple agents |
| [sync-status.prompt.md](./sync-status.prompt.md)             | Session end - updating APS files         |

## Quick Start

1. **Start session:**

   ```
   Read plans/modules/02-auth.aps.md and create Tasks from Ready work items.
   Show me the wave breakdown.
   ```

2. **Execute waves:**

   ```
   Start Wave 1. Work on AUTH-001 first.
   ```

3. **End session:**

   ```
   Session complete. AUTH-001 done, AUTH-002 blocked on API key.
   Update APS files and show the diff.
   ```

## Shared Task Lists

For multi-session collaboration:

```bash
# All terminals use same task list
CLAUDE_CODE_TASK_LIST_ID=myproject-auth claude
```

## See Also

- [aps-rules.md](../../../../plans/aps-rules.md) — Agent guidance with Tasks section
- [plans/modules/tasks.aps.md](../../../../plans/modules/tasks.aps.md) — Tasks integration roadmap
