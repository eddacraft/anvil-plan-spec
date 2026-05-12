<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- Executable only if work items exist and status is Ready. -->

# Core Module

| Scope | Owner | Priority | Status |
|-------|-------|----------|--------|
| CORE | @josh | high | Ready |

## Purpose

Provide data access layer for OpenCode configuration and session history. Handles file system operations, config parsing, and session discovery.

## In Scope

- Locate OpenCode config directory
- Parse configuration files
- List and read session history
- Watch for config/session changes

## Out of Scope

- UI rendering (see UI module)
- Modifying OpenCode processes
- Network operations

## Interfaces

**Depends on:**

- File system — `~/.opencode/` directory

**Exposes:**

- `getConfig()` → OpenCodeConfig
- `listSessions()` → Session[]
- `getSession(id)` → SessionDetail
- `watchChanges(callback)` → unsubscribe

## Boundary Rules

- CORE must not import UI components
- CORE must not spawn or signal OpenCode processes

## Acceptance Criteria

- [ ] Correctly locates config on macOS and Linux
- [ ] Handles missing/malformed config gracefully
- [ ] File watching doesn't block main thread
- [ ] All functions have TypeScript types

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Config format unknown | Start with documented fields only |
| Large session files | Stream/paginate, don't load all into memory |

## Work Items

### CORE-001: Implement config discovery and parsing

- **Intent:** Locate and parse OpenCode configuration
- **Expected Outcome:** `getConfig()` returns typed config object
- **Scope:** New `core/config.ts` module
- **Non-scope:** Config modification, UI
- **Files:** `src/core/config.ts`, `src/core/types.ts`
- **Dependencies:** None
- **Validation:** `npm test -- config.test.ts`
- **Confidence:** medium (config format not fully documented)
- **Risks:** Format may vary by version

### CORE-002: Implement session listing

- **Intent:** Enumerate past OpenCode sessions
- **Expected Outcome:** `listSessions()` returns array of session metadata
- **Scope:** New `core/sessions.ts` module
- **Non-scope:** Full session content, real-time updates
- **Files:** `src/core/sessions.ts`
- **Dependencies:** CORE-001 (needs config path)
- **Validation:** `npm test -- sessions.test.ts`
- **Confidence:** medium
- **Risks:** Session storage format undocumented

## Execution

Action Plan: [../execution/CORE-001.actions.md](../execution/CORE-001.actions.md)

## Decisions

- **D-001:** Use Zod for config parsing — runtime validation with TypeScript inference

## Notes

- Need to research OpenCode config/session file locations and formats
