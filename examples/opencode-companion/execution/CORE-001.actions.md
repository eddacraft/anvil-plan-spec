<!-- APS Execution: See docs/ai/prompting/actions.prompt.md -->

# Action Plan: CORE-001

| Field      | Value                                             |
| ---------- | ------------------------------------------------- |
| Source     | [./modules/core.aps.md](../modules/core.aps.md)   |
| Work Item  | CORE-001 — Implement config discovery and parsing |
| Created by | AI                                                |
| Status     | Ready                                             |

## Prerequisites

- [ ] Tauri project initialised
- [ ] Zod package installed
- [ ] Sample OpenCode config available for testing

## Actions

### Action 1 — Research OpenCode config location

**Purpose**
Determine standard config file location for OpenCode.

**Produces**
Documented path(s) for OpenCode configuration.

**Checkpoint**
Document confirms `~/.config/opencode/` or `~/.opencode/` path

**Validate**
Manual inspection of local OpenCode installation

### Action 2 — Define config TypeScript types

**Purpose**
Establish type definitions for OpenCode configuration.

**Produces**
TypeScript interface for OpenCode config.

**Checkpoint**
`src/core/types.ts` exports `OpenCodeConfig` type

**Validate**
`npm run build` succeeds

### Action 3 — Create Zod schema for config

**Purpose**
Enable runtime validation of configuration data.

**Produces**
Zod schema matching OpenCode config structure.

**Checkpoint**
Schema validates known config fields, allows unknown

**Validate**
Unit test with sample config passes

### Action 4 — Implement getConfigPath function

**Purpose**
Provide cross-platform path resolution for config file.

**Produces**
Function returning config file path.

**Checkpoint**
Function returns correct path on macOS and Linux

**Validate**
`npm test -- config.test.ts` — path tests pass

### Action 5 — Implement getConfig function

**Purpose**
Read and parse OpenCode configuration.

**Produces**
Function reading config file and returning typed object.

**Checkpoint**
Function reads and parses config, returns typed object

**Validate**
`npm test -- config.test.ts` — all tests pass

### Action 6 — Handle missing/malformed config

**Purpose**
Gracefully handle configuration errors.

**Produces**
Error handling and default config logic.

**Checkpoint**
Function returns sensible defaults, logs warning

**Validate**
Test with missing file and invalid JSON passes

## Completion

- [ ] All checkpoints validated
- [ ] Work item marked complete in core.aps.md

**Completed by:** (pending)
