# Action Plan: ORCH-006

| Field      | Value                                                         |
| ---------- | ------------------------------------------------------------- |
| Source     | [./modules/orchestrate.aps.md](../modules/orchestrate.aps.md) |
| Work Item  | ORCH-006 — Create MCP server                                  |
| Created by | @aneki / AI                                                   |
| Status     | Complete                                                      |

## Prerequisites

- [x] ORCH-001, ORCH-002 Complete (CLI surface to wrap exists)
- [x] D-004 resolved (TypeScript, MCP SDK)
- [x] D-007 resolved (implement now; server wraps `aps` command surface)

## Actions

### 1. Scaffold MCP server package

- **Checkpoint:** `mcp/` package exists with SDK dependency, runs on Node
- **Validate:** `cd mcp && pnpm install`

### 2. Implement request routing

- **Checkpoint:** Direct commands and natural-language requests map to safe
  CLI invocations; unroutable input yields help text
- **Validate:** `cd mcp && node --test test/route.test.ts`

### 3. Implement server and tool execution

- **Checkpoint:** Single `aps` tool discoverable; calls execute CLI and
  return results; malformed input handled gracefully
- **Validate:** `cd mcp && node --test test/server.test.ts`

### 4. Wire into repo test suite and docs

- **Checkpoint:** `test/run.sh` covers MCP server; docs mention how to
  configure it
- **Validate:** `./test/run.sh`

## Completion

- [x] All checkpoints validated
- [x] Work item marked complete in `orchestrate.aps.md`

**Completed by:** @aneki / AI — 2026-06-08
