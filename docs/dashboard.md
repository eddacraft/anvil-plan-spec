# APS Dashboard

**Status:** Draft
**Date:** 2026-03-10

## Purpose

Render the current state of APS plan files as a human-readable dashboard. Single source of truth: the APS files themselves. No external API, no sync, no database.

## Output Formats

### 1. Terminal (`aps dash`)

Default output for CLI usage. Coloured, compact, designed for quick scanning.

```
Anvil — APS Dashboard (2026-03-10 14:50 AWST)

🔨 Checked Out (3)
  RENG-003  Boundary checks              @morgan       2h ago
  DEFER-001 GH Issue filing              codex-abc    14h ago  ⚠️ stale
  DASH-002  Core views                   @josh         1h ago

📋 Ready (7)
  RENG-005  Command safety               high    blocked-by: RENG-003
  EMBER-002 Event pipeline               high
  DASH-003  Architecture views           medium
  ...

🔍 In Review (1)
  RENG-001  Secret scan port             checked-in 3h ago, CI pending

✅ Recently Completed (last 7d)
  RENG-002  Anti-pattern detection        completed 2d ago
  OPA-001   Policy engine base            completed 5d ago

⚠️ Stale Checkouts (1)
  DEFER-001 GH Issue filing              codex-abc    14h ago  (no commits since checkout)

📊 Summary: 3 active · 7 ready · 1 review · 12 complete · 1 stale
```

### 2. Markdown (`STATUS.md`)

Written to repo root (or configurable path). Same data, GitHub-renderable.

```markdown
# APS Dashboard
_Last updated: 2026-03-10 14:50 AWST_

## 🔨 Checked Out (3)
| Item | Description | Owner | Duration | Notes |
|---|---|---|---|---|
| RENG-003 | Boundary checks | @morgan | 2h | |
| DEFER-001 | GH Issue filing | codex-abc | 14h | ⚠️ stale |
| DASH-002 | Core views | @josh | 1h | |

## 📋 Ready (7)
| Item | Description | Priority | Blocked by |
|---|---|---|---|
| RENG-005 | Command safety | high | RENG-003 |
| EMBER-002 | Event pipeline | high | — |
| DASH-003 | Architecture views | medium | — |

## 🔍 In Review (1)
| Item | Description | Checked in | CI |
|---|---|---|---|
| RENG-001 | Secret scan port | 3h ago | pending |

## ✅ Recently Completed (7d)
| Item | Description | Completed |
|---|---|---|
| RENG-002 | Anti-pattern detection | 2d ago |
| OPA-001 | Policy engine base | 5d ago |

## ⚠️ Alerts
- **DEFER-001** checked out 14h ago with no commits — may be stale

---
_3 active · 7 ready · 1 review · 12 complete · 1 stale_
```

## CLI Interface

```
aps dash [options]

Options:
  -f, --refresh <seconds>   Watch mode — refresh every N seconds
  -o, --output <path>       Write STATUS.md to path (default: stdout for terminal)
  --format <term|md|json>   Output format (default: term)
  --module <id>             Filter to a single module
  --stale <hours>           Stale threshold override (default: 24)
  --completed-days <n>      Show completed items from last N days (default: 7)
  --no-color                Disable terminal colours
```

### Examples

```bash
# Quick look
aps dash

# Watch mode, refresh every 5 seconds
aps dash -f5

# Write markdown dashboard to repo
aps dash --format md -o STATUS.md

# JSON for piping to other tools
aps dash --format json

# Just one module
aps dash --module reng
```

## Data Sources

The dashboard reads exclusively from APS plan files:

| Data | Source |
|---|---|
| Work items + status | `plans/index.aps.md` + `plans/modules/*.aps.md` |
| Checkout state | Inline metadata in work items (owner, timestamp) |
| Dependencies | `blocked-by` fields in work items |
| Completion evidence | `branch`, `commits` fields from check-in |
| Staleness | Checkout timestamp vs current time (+ optional `git log --grep` for commit activity) |

No API calls. No network. Pure file reads + optional git queries.

## Refresh Strategies

| Strategy | Trigger | Use case |
|---|---|---|
| Manual | `aps dash` | Developer wants a quick look |
| Watch | `aps dash -f5` | Active work session, keep a terminal open |
| Cron/systemd timer | Every 10–15 min | Auto-update `STATUS.md` in repo |
| Post-commit hook | On push to main | Always-current `STATUS.md` on default branch |
| CI step | On PR merge | Guaranteed fresh after state changes |

### Recommended setup

```bash
# Cron: refresh STATUS.md every 15 minutes
*/15 * * * * cd /path/to/repo && aps dash --format md -o STATUS.md && git add STATUS.md && git diff --cached --quiet || git commit -m "chore: refresh APS dashboard" && git push
```

Or a systemd timer for more control over logging and failure handling.

## Sections

### Checked Out
Items currently being worked on. Sorted by checkout duration (longest first — surfaces stale checkouts). Shows owner and elapsed time.

### Ready
Items available for pickup. Sorted by priority, then by dependency chain depth (items that unblock others float up). Shows blocking dependencies.

### In Review
Items that have been checked in but not yet verified complete. Shows CI status if available.

### Recently Completed
Items verified complete in the last N days (default 7). Provides a sense of velocity and recent progress.

### Alerts
Actionable issues:
- Stale checkouts (no commits past threshold)
- Failed verification (CI red on checked-in items)
- Orphaned commits (commits referencing item codes not in the plan)
- Drift (items marked complete with no commit evidence)

### Summary Line
One-line counts: active · ready · review · complete · stale

## JSON Output

For tooling and automation:

```json
{
  "generated": "2026-03-10T14:50:00+0800",
  "summary": {
    "checkedOut": 3,
    "ready": 7,
    "review": 1,
    "complete": 12,
    "stale": 1
  },
  "items": [
    {
      "id": "RENG-003",
      "module": "reng",
      "description": "Boundary checks",
      "status": "Checked Out",
      "owner": "@morgan",
      "checkedOut": "2026-03-10T12:50:00+0800",
      "priority": "high",
      "blockedBy": [],
      "blocks": ["RENG-005"]
    }
  ],
  "alerts": [
    {
      "type": "stale_checkout",
      "item": "DEFER-001",
      "owner": "codex-abc",
      "hours": 14
    }
  ]
}
```

## Future: Notification Layer

When notification channels are available, the dashboard diff can trigger alerts:

```
# Pseudocode
previous = load("STATUS.md.prev")
current = generate_dashboard()
diff = compare(previous, current)

for change in diff:
  if change.type == "newly_stale":
    notify(owner, "Your checkout on {item} is stale")
  if change.type == "checked_in":
    notify(channel, "{item} checked in by {owner}, awaiting verification")
  if change.type == "completed":
    notify(channel, "✅ {item} verified complete")
```

This is additive — doesn't change the dashboard itself, just watches its output.
