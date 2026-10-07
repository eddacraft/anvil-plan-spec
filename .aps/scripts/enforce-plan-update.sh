#!/usr/bin/env bash
# APS Plan Update Enforcer
# Blocks session end if code was changed but no plan files were updated.
# Pairs with check-complete.sh: that checks statuses, this checks file changes.
#
# Uses a session baseline marker (written by init-session.sh) to detect all
# changes — committed and uncommitted — during the session. Falls back to
# uncommitted-only checks if no baseline exists.
#
# Usage: ./aps-planning/scripts/enforce-plan-update.sh [plans-dir]
#
# Exit codes:
#   0 — Plans were updated (or no code changes detected)
#   2 — Code changed but plans were not updated (blocks Claude from stopping)

set -euo pipefail

PLANS_DIR="${1:-plans}"
BASELINE_FILE=".claude/.aps-session-baseline"

# If no plans directory, nothing to enforce
if [ ! -d "$PLANS_DIR" ]; then
  exit 0
fi

# Not a git repo — can't diff, skip enforcement
if ! git rev-parse --is-inside-work-tree &>/dev/null; then
  exit 0
fi

# Collect all changed files during this session
CHANGED_FILES=""

if [ -f "$BASELINE_FILE" ]; then
  BASELINE=$(cat "$BASELINE_FILE")

  # Validate the baseline is a real commit
  if git rev-parse --verify "$BASELINE" &>/dev/null; then
    # Changes in commits since session start
    COMMITTED=$(git diff --name-only "$BASELINE" HEAD 2>/dev/null || true)
    # Uncommitted changes (staged + unstaged)
    UNCOMMITTED=$(git diff --name-only HEAD 2>/dev/null || true)
    STAGED=$(git diff --name-only --cached 2>/dev/null || true)

    # Untracked files (new files not yet staged)
    UNTRACKED=$(git ls-files --others --exclude-standard 2>/dev/null || true)

    CHANGED_FILES=$(printf '%s\n%s\n%s\n%s' "$COMMITTED" "$UNCOMMITTED" "$STAGED" "$UNTRACKED" | sort -u)
  fi
fi

# Fallback: no baseline or invalid baseline — check uncommitted only
if [ -z "$CHANGED_FILES" ]; then
  UNCOMMITTED=$(git diff --name-only 2>/dev/null || true)
  STAGED=$(git diff --name-only --cached 2>/dev/null || true)
  UNTRACKED=$(git ls-files --others --exclude-standard 2>/dev/null || true)
  CHANGED_FILES=$(printf '%s\n%s\n%s' "$UNCOMMITTED" "$STAGED" "$UNTRACKED" | sort -u)
fi

# Filter out empty lines and ephemeral .claude/ files (e.g. session baseline)
CHANGED_FILES=$(echo "$CHANGED_FILES" | grep -v '^$' | grep -v '^\.claude/' || true)

# Nothing changed at all — nothing to enforce
if [ -z "$CHANGED_FILES" ]; then
  exit 0
fi

# Split into plan files and non-plan files (anchor match to path start)
PLAN_CHANGES=$(echo "$CHANGED_FILES" | grep -E "^${PLANS_DIR}/" || true)
CODE_CHANGES=$(echo "$CHANGED_FILES" | grep -Ev "^${PLANS_DIR}/" || true)

# If no code changes (plan-only session), nothing to enforce
if [ -z "$CODE_CHANGES" ]; then
  exit 0
fi

# Code was changed — were plans updated too?
if [ -z "$PLAN_CHANGES" ]; then
  # Block: code changed but plans weren't touched
  {
    echo "Code was modified but no plan files were touched."
    echo ""
    echo "Changed files outside plans/:"
    echo "$CODE_CHANGES" | head -10 | sed 's/^/  /'
    TOTAL=$(echo "$CODE_CHANGES" | wc -l | tr -d ' ')
    if [ "$TOTAL" -gt 10 ]; then
      echo "  ... and $((TOTAL - 10)) more"
    fi
    echo ""
    echo "Before ending this session, update the relevant APS spec:"
    echo "  1. Mark completed work items as Complete"
    echo "  2. Update In Progress items with current state"
    echo "  3. Add any newly discovered work as Draft items"
    echo "  4. Update module status if all work items are done"
  } >&2
  exit 2
fi

# Both code and plans changed — good
exit 0
