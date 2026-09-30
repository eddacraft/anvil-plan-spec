# aps-mcp

Optional MCP server exposing the [APS CLI](../bin/aps) command surface to
MCP-capable agents (ORCH-006, decision D-004).

The server wraps the CLI as a single codemode tool named `aps`. Agents send a
direct command or a natural-language request; the server routes it to an
allowlisted CLI invocation (`next`, `start`, `complete`, `graph`, `lint`) and
returns the result. There is no second source of truth — the markdown stays
authoritative, exactly as with the CLI.

## Setup

### Windows PowerShell 5.1 or PowerShell 7

Install Node.js 24+ and pnpm, and build or install native aps.exe first. From
the repository root, explicitly select aps.exe: the server otherwise prefers
the sibling bash script, which is not a native Windows executable.

```powershell
$env:APS_BIN = (Resolve-Path -LiteralPath '.\cli\target\debug\aps.exe').Path
$env:APS_PLANS = (Resolve-Path -LiteralPath '.\examples\user-auth').Path
pnpm.cmd --dir mcp install --frozen-lockfile
if ($LASTEXITCODE -ne 0) { throw 'MCP dependency install failed' }
pnpm.cmd --dir mcp test
if ($LASTEXITCODE -ne 0) { throw 'MCP tests failed' }
pnpm.cmd --dir mcp exec tsc -p .
if ($LASTEXITCODE -ne 0) { throw 'MCP typecheck failed' }
node .\mcp\src\index.ts
```

The last command starts a stdio server, not an interactive prompt. Stop with
Ctrl+C when testing manually. In a harness, set command to the absolute node.exe
path, args to an array containing the absolute mcp/src/index.ts path, and env
to APS_BIN/APS_PLANS absolute paths. Do not embed shell quote characters inside
JSON path values; escape backslashes or use forward slashes. Keep a path with
spaces as one argument. Use a trusted plan tree; the example above is for
read-only requests unless you first copy it to a disposable directory.

The environment assignments affect this PowerShell process and its children,
not persistent user settings. Restore previous values or close this test shell.
For an installed CLI, replace APS_BIN with (Get-Command aps.exe).Source.

### macOS / Linux

```bash
cd mcp  # from the repository root
pnpm install --frozen-lockfile  # Node >= 22.18; TypeScript runs directly
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

Use an absolute `APS_PLANS` path when launching from `mcp/` or a harness with
a different working directory. `start` and `complete` modify that plan tree;
the server does not provide separate approval or sandboxing. Use only trusted
plans and grant the host only the filesystem access it needs.

## Example requests

| Request                                           | Routed to                              |
| ------------------------------------------------- | -------------------------------------- |
| `next auth`                                       | `aps next auth`                        |
| `what's the next ready work item in auth?`        | `aps next auth`                        |
| `start AUTH-003`                                  | `aps start AUTH-003`                   |
| `complete AUTH-003 with learning: "retry on 5xx"` | `aps complete AUTH-003 --learning ...` |
| `show the dependency graph for auth`              | `aps graph auth`                       |

Unroutable requests return the command help as a tool error — the transport
stays up.

## Test

```bash
pnpm test           # routing unit tests + end-to-end MCP client tests
pnpm exec tsc -p .   # typecheck
```
