# aps-mcp

Optional MCP server exposing the [APS CLI](../bin/aps) command surface to
MCP-capable agents (ORCH-006, decision D-004).

The server wraps the CLI as a single codemode tool named `aps`. Agents send a
direct command or a natural-language request; the server routes it to an
allowlisted CLI invocation (`next`, `start`, `complete`, `graph`, `lint`) and
returns the result. There is no second source of truth — the markdown stays
authoritative, exactly as with the CLI.

## Setup

```bash
pnpm install   # Node >= 22.18 (runs TypeScript directly, no build step)
```

## Run

```bash
node src/index.ts
```

Environment:

- `APS_BIN` — path to the `aps` executable (default: sibling `../bin/aps`,
  then `$PATH`). The server is agnostic to which binary provides the command
  surface (see ORCH D-006).
- `APS_PLANS` — plan root directory passed to every command (default: the
  CLI's own default, `plans/` relative to its working directory).

## Example requests

| Request                                            | Routed to                              |
| -------------------------------------------------- | -------------------------------------- |
| `next auth`                                        | `aps next auth`                        |
| `what's the next ready work item in auth?`         | `aps next auth`                        |
| `start AUTH-003`                                   | `aps start AUTH-003`                   |
| `complete AUTH-003 with learning: "retry on 5xx"`  | `aps complete AUTH-003 --learning ...` |
| `show the dependency graph for auth`               | `aps graph auth`                       |

Unroutable requests return the command help as a tool error — the transport
stays up.

## Test

```bash
node --test          # routing unit tests + end-to-end MCP client tests
pnpm exec tsc -p .   # typecheck
```
