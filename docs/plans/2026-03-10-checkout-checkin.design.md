# Work Item Check-out / Check-in Protocol

**Status:** Superseded by v0.3.0 orchestration CLI (`aps start` / `aps complete`).
**Date:** 2026-03-10
**Note:** This design proposed an explicit `checkout`/`checkin` state model. The
shipped implementation in v0.3.0 uses a leaner `Draft → Ready → In Progress →
Complete` state machine driven by `aps start` and `aps complete`. See
[docs/usage.md](../usage.md) for the actual CLI surface. Preserved here as a
design artefact.

## Problem

APS plan files frequently drift from reality:

- Items marked `Complete` that were never verified
- Items still showing `Ready` or `Draft` despite having been implemented
- Multiple agents or humans working the same item without knowing
- No audit trail of who worked on what, or when

This erodes trust in the plan, which defeats the purpose of having one. Both humans and agents end up ignoring or second-guessing the plan state.

## Solution: Check-out / Check-in

A lightweight state protocol for work items, inspired by version control locks and library book check-outs.

### Work Item Lifecycle

```
Ready → Checked Out → Checked In → Complete
                ↓            ↓
             Blocked      Review
                ↓
            Released
```

### States

| State         | Meaning                                                                  |
| ------------- | ------------------------------------------------------------------------ |
| `Ready`       | Available for pickup — dependencies met, no one working on it            |
| `Checked Out` | Someone (human or agent) is actively working on it                       |
| `Blocked`     | Cannot proceed — dependency, question, or external blocker               |
| `Released`    | Was checked out, but released without completion (abandoned or deferred) |
| `Checked In`  | Work submitted, pending verification                                     |
| `Review`      | Check-in verification failed — needs attention                           |
| `Complete`    | Verified done — all acceptance criteria met                              |

### Check-out

Claiming a work item for active work. Prevents duplicate effort.

**Required fields** (added to the work item in the APS file):

```markdown
- **Status:** Checked Out
- **Owner:** @mbrighthand | codex-session-abc123 | josh
- **Checked out:** 2026-03-10T14:45:00+0800
```

**Rules:**

1. Cannot check out an item that is already `Checked Out` (unless the checkout has expired — see Stale Checkouts below)
2. Owner must be identifiable — agent session ID, GitHub username, or human name
3. Checking out an item SHOULD be atomic (to avoid race conditions in multi-agent setups)

### Check-in

Submitting completed work for verification. This is NOT the same as marking `Complete`.

**Required fields:**

```markdown
- **Status:** Checked In
- **Checked in:** 2026-03-10T16:30:00+0800
- **Branch:** feat/reng-003-boundary-check
- **Commits:** abc1234, def5678
```

**Rules:**

1. Must include at least one commit reference
2. Commits MUST contain the APS item code in the message (e.g., `feat(engine): add boundary check (RENG-003)`)
3. Check-in does NOT mean complete — verification happens next

### Verification (Check-in → Complete)

After check-in, verification determines whether the work is actually done.

**Verification checks (in order of increasing rigour):**

| Level | Check                                                | Automated?               |
| ----- | ---------------------------------------------------- | ------------------------ |
| L0    | Commits with item code exist on a pushed branch      | ✅ `git log --grep`      |
| L1    | CI passes on the branch                              | ✅ `gh run list`         |
| L2    | Changed files are plausible for the work item scope  | ⚠️ Heuristic             |
| L3    | Acceptance criteria from the work item are satisfied | ⚠️ Agent review or human |

**Minimum bar:** L0 + L1 for automated check-in. L2/L3 for high-priority or complex items.

**On verification failure:** Status moves to `Review`, not back to `Ready`. The work exists but needs attention.

### Commit Message Convention

All commits associated with a work item MUST include the item code:

```
<type>(<scope>): <description> (<ITEM-CODE>)
```

Examples:

```
feat(engine): add boundary check (RENG-003)
fix(checks): handle empty dependency list (RENG-003)
test(engine): boundary check edge cases (RENG-003)
```

This is the **primary traceability mechanism**. It works regardless of who made the commit — human, Claude Code, Codex, or any other agent.

**Enforcement options:**

- **Git hook** (`commit-msg`): Warn or reject commits without a valid item code
- **Agent instructions**: Skill/AGENTS.md instructs agents to always include the code
- **CI check**: Lint PR commits for item code presence

### Stale Checkout Detection (Sweep)

Checkouts can go stale — an agent crashed, a human forgot, work was abandoned.

**Rules:**

1. A checkout older than **24 hours** without new commits is flagged as `Stale`
2. A checkout older than **72 hours** without activity is auto-released (status → `Released`)
3. Sweep can be run manually or on a schedule (heartbeat, cron, CI)

**Sweep logic:**

```
For each Checked Out item:
  1. Check git log for commits with item code since checkout time
  2. If no commits found and checkout age > 24h → flag Stale
  3. If no commits found and checkout age > 72h → set Released
  4. If commits found → update last-activity timestamp
```

### Reconciliation (Drift Detection)

Periodic scan to find mismatches between plan state and git reality:

| Finding                                                            | Action                                                 |
| ------------------------------------------------------------------ | ------------------------------------------------------ |
| Item is `Ready` but commits with its code exist on a merged branch | Flag for check-in (likely completed but never updated) |
| Item is `Complete` but no commits with its code exist              | Flag for review (false completion)                     |
| Item is `Checked Out` but owner session no longer exists           | Release checkout                                       |
| Commits reference an item code that doesn't exist in the plan      | Warn (orphaned work)                                   |

### Multi-Agent Coordination

When multiple agents operate on the same repo:

1. **Check-out is the lock.** One agent per item.
2. **File-level atomicity:** Use `O_EXCL` or equivalent when writing checkout state to prevent races (see existing APS lock file approach in anvil-001).
3. **Agent identity:** Each agent session must have a unique, stable identifier written into the checkout.
4. **Visibility:** All agents read the same APS files — a checked-out item is visible to everyone.

## Integration Points

### Skills / AGENTS.md

Agent-facing instructions should include:

- Always check out before starting work on an APS item
- Always include the item code in commit messages
- Check in when work is complete (don't just mark as Done)
- Run verification before claiming completion

### Git Hooks

Optional `commit-msg` hook:

```bash
#!/bin/sh
# Warn if commit message doesn't contain an APS item code
if ! grep -qE '\([A-Z]+-[0-9]+\)' "$1"; then
  echo "⚠️  No APS item code found in commit message."
  echo "   Format: <type>(<scope>): <description> (<ITEM-CODE>)"
  echo "   Proceeding anyway..."
fi
```

### CI

Optional PR check:

- Verify all commits in the PR contain a valid APS item code
- Cross-reference with plan files to confirm the item exists and is `Checked Out`

## Open Questions

1. **Granularity:** Should sub-tasks within a work item have their own checkout, or is item-level enough?
2. **Conflict resolution:** If two agents race to check out the same item, how is the loser notified? (Currently: file lock prevents it, but the UX of "try again" is poor)
3. **Partial completion:** What if an agent completes 3 of 5 acceptance criteria? Check in to `Review`? Or keep checked out?
4. **Cross-repo items:** Some work items span multiple repos. Checkout is per-file — how do we handle this?
