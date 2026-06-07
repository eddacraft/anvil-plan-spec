# Issues & Questions Tracker

> Development-time discoveries. See `plans/aps-rules.md` → "Issues & Questions
> Tracker" for conventions.

---

## Issues

### ISS-001: Markdown parser is not fence-aware outside build_id_index

| Field      | Value       |
| ---------- | ----------- |
| Status     | Open        |
| Severity   | Medium      |
| Discovered | DOGFOOD-002 |
| Module     | VAL         |

**Context:** Council review (session council-b459ae20) showed `### FAKE-999:`
inside a fenced code block is treated as a real work item by
`get_work_items`, triggering false E005 errors and polluting `aps next` /
`aps graph` item lists. `build_id_index` (cross-file W003) was made
fence-aware during DOGFOOD-002; the shared parser helpers were not.

**Impact:** Code blocks containing example work-item headers produce
false lint errors and phantom orchestration items.

---

### ISS-002: Link checks follow paths outside the plan root

| Field      | Value       |
| ---------- | ----------- |
| Status     | Open        |
| Severity   | Low         |
| Discovered | DOGFOOD-002 |
| Module     | VAL         |

**Context:** W019 / A004 existence checks resolve `../`-style link targets
without confining them to the plan root, so an out-of-tree file that exists
passes as a valid module link, and lint output echoes attacker-chosen paths
(a filesystem existence oracle when logs are shared). Flagged by the council
security reviewer; deferred — requires `realpath` canonicalisation across
bash + PowerShell engines.

**Impact:** Semantically invalid index links pass; minor information
disclosure in shared CI logs.

---

### ISS-003: TUI wizard lacks panic-time terminal restore

| Field      | Value   |
| ---------- | ------- |
| Status     | Open    |
| Severity   | Low     |
| Discovered | TUI-003 |
| Module     | TUI     |

**Context:** `wizard::run()` restores raw mode / alternate screen / bracketed
paste on normal exit (best-effort since TUI-003), but a panic inside
`run_loop` leaves the terminal in raw mode. Council review (session
council-b2bd78ac) found no panic paths in the current code; the exposure is
future regressions. Fix is a panic hook or Drop guard — natural to land with
TUI-004, which raises the stakes by writing to disk.

**Impact:** A future panic would leave the user's terminal unusable until
`reset`.

---

## Questions

_None yet._
