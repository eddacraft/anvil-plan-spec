# Issues & Questions Tracker

> Development-time discoveries that emerge while building. Not a bug tracker replacement — a lightweight log for planning-level concerns that need visibility.

## When to Use This

Add entries here when you discover:

- **Issues** — Bugs, limitations, tech debt, edge cases noticed but not yet handled
- **Questions** — Unknowns that need answers, design decisions deferred, clarifications needed

**Don't add:** Routine bugs (use your bug tracker), implementation todos (use work items), solved problems (use solutions/).

---

## Issues

<!--
Issues are problems discovered during development.
ID format: ISS-NNN (e.g., ISS-001)
Status: Open | Resolved | Deferred | Won't Fix
Severity: Critical | High | Medium | Low
-->

### ISS-001: [Brief description]

| Field      | Value                                                            |
| ---------- | ---------------------------------------------------------------- |
| Status     | Open                                                             |
| Severity   | Medium                                                           |
| Discovered | [WORK-ITEM-ID or activity, e.g., "AUTH-002" or "manual testing"] |
| Module     | [MODULE-ID, e.g., "AUTH"]                                        |

**Context:** [What you observed — be specific]

**Impact:** [How this affects the system or users]

<!-- When resolved, add:
**Resolution:** [How it was fixed]
**Resolved:** YYYY-MM-DD
**Related:** [PR #NNN, WORK-ITEM-ID]
-->

---

## Questions

<!--
Questions are unknowns that emerged during development.
ID format: Q-NNN (e.g., Q-001)
Status: Open | Answered | Deferred
Priority: High | Medium | Low
-->

### Q-001: [The question]

| Field      | Value                       |
| ---------- | --------------------------- |
| Status     | Open                        |
| Priority   | Medium                      |
| Discovered | [WORK-ITEM-ID or activity]  |
| Assigned   | [@username or "unassigned"] |

**Context:** [Why this question came up]

**Options considered:** _(optional)_

1. [Option A] — [tradeoffs]
2. [Option B] — [tradeoffs]

<!-- When answered, add:
**Answer:** [The decision/answer]
**Rationale:** [Why this answer]
**Answered:** YYYY-MM-DD
**Related:** [Decision D-NNN, PR #NNN]
-->

---

## Resolved

<!--
Move resolved issues and answered questions here to keep the active sections clean.
Keep entries for 1-2 sprints as reference, then archive or delete.
-->

### ISS-000: [Example resolved issue]

| Field      | Value    |
| ---------- | -------- |
| Status     | Resolved |
| Severity   | Medium   |
| Discovered | AUTH-001 |
| Module     | AUTH     |

**Context:** Rate limiting triggered unexpectedly during load testing.

**Resolution:** Increased rate limit threshold from 100 to 500 req/min for authenticated users.

**Resolved:** 2024-01-15

**Related:** PR #42, AUTH-003

---

## Quick Reference

### Issue Severities

| Severity | Meaning                                     |
| -------- | ------------------------------------------- |
| Critical | Blocks work, data loss risk, security issue |
| High     | Significant impact, needs attention soon    |
| Medium   | Moderate impact, can be scheduled           |
| Low      | Minor inconvenience, fix when convenient    |

### Referencing from Other Documents

From work items or notes, reference as:

- `See ISS-001` or `Related: ISS-001, Q-002`
- In commit messages: `Addresses ISS-001`

### ID Allocation

- Issues: `ISS-001`, `ISS-002`, etc.
- Questions: `Q-001`, `Q-002`, etc.

Sequential numbering within each category. Don't reuse IDs.
