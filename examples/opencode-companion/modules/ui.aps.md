<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- Executable only if work items exist and status is Ready. -->

# UI Module

| ID  | Owner | Priority | Status |
| --- | ----- | -------- | ------ |
| UI  | @josh | high     | Draft  |

## Purpose

Provide the visual interface for the companion app. Displays session history, configuration, and real-time updates.

## In Scope

- Session list view
- Session detail view
- Configuration display
- Real-time update indicators

## Out of Scope

- Data fetching logic (uses CORE module)
- Desktop window management (Tauri handles this)

## Interfaces

**Depends on:**

- CORE — all data access functions

**Exposes:**

- React components for each view
- Route definitions

## Boundary Rules

- UI must not access file system directly
- UI must not import Node.js modules (web context only)

## Acceptance Criteria

- [ ] Session list loads within 500ms
- [ ] UI remains responsive during file watching
- [ ] Graceful display when no sessions exist

## Risks & Mitigations

| Risk                           | Mitigation                        |
| ------------------------------ | --------------------------------- |
| Large session lists            | Virtual scrolling                 |
| Tauri/React integration issues | Use official Tauri React template |

## Work Items

> **Status: Draft** — Blocked on CORE module

No work items authorised. Blockers:

- [ ] CORE-001 and CORE-002 must be complete
- [ ] Design mockups needed

## Decisions

(none yet)

## Notes

- Consider using Tailwind for styling (fast iteration)
