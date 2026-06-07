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

## Questions

_None yet._
