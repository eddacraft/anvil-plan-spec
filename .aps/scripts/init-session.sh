#!/usr/bin/env bash
# APS Session Initializer
# Checks for APS planning files and reports status.
# Use as a hook or run manually at session start.
#
# Usage: ./aps-planning/scripts/init-session.sh [plans-dir]

set -euo pipefail

PLANS_DIR="${1:-plans}"

# Colors (if terminal supports them)
if [ -t 1 ]; then
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  RED='\033[0;31m'
  BOLD='\033[1m'
  NC='\033[0m'
else
  GREEN='' YELLOW='' RED='' BOLD='' NC=''
fi

echo -e "${BOLD}APS Planning Session${NC}"
echo "─────────────────────"

# Check if plans/ exists
if [ ! -d "$PLANS_DIR" ]; then
  echo -e "${YELLOW}No plans/ directory found.${NC}"
  echo "Run /plan to start APS planning, or create plans/ manually."
  exit 0
fi

# Check for index
if [ -f "$PLANS_DIR/index.aps.md" ]; then
  TITLE=$(head -5 "$PLANS_DIR/index.aps.md" | grep '^# ' | head -1 | sed 's/^# //')
  echo -e "${GREEN}Plan:${NC} ${TITLE:-[untitled]}"
else
  echo -e "${YELLOW}No index.aps.md found.${NC}"
fi

# Check for aps-rules.md
if [ -f "$PLANS_DIR/aps-rules.md" ]; then
  echo -e "${GREEN}Agent rules:${NC} plans/aps-rules.md"
fi

# Count modules
MODULE_COUNT=0
READY_COUNT=0
PROGRESS_COUNT=0
COMPLETE_COUNT=0

if [ -d "$PLANS_DIR/modules" ]; then
  for f in "$PLANS_DIR/modules/"*.aps.md; do
    [ -f "$f" ] || continue
    # Skip hidden template files
    basename "$f" | grep -q '^\.' && continue
    MODULE_COUNT=$((MODULE_COUNT + 1))

    # Check status from metadata table
    if grep -qi '| *Ready *|' "$f" 2>/dev/null; then
      READY_COUNT=$((READY_COUNT + 1))
    elif grep -qi '| *In Progress *|' "$f" 2>/dev/null; then
      PROGRESS_COUNT=$((PROGRESS_COUNT + 1))
    elif grep -qi '| *Complete *|' "$f" 2>/dev/null; then
      COMPLETE_COUNT=$((COMPLETE_COUNT + 1))
    fi
  done
fi

# Also check for simple specs (*.aps.md at plans/ root, not index)
for f in "$PLANS_DIR/"*.aps.md; do
  [ -f "$f" ] || continue
  basename "$f" | grep -q '^index' && continue
  MODULE_COUNT=$((MODULE_COUNT + 1))
done

if [ "$MODULE_COUNT" -gt 0 ]; then
  echo -e "${GREEN}Modules:${NC} $MODULE_COUNT total"
  [ "$READY_COUNT" -gt 0 ] && echo "  Ready: $READY_COUNT"
  [ "$PROGRESS_COUNT" -gt 0 ] && echo "  In Progress: $PROGRESS_COUNT"
  [ "$COMPLETE_COUNT" -gt 0 ] && echo "  Complete: $COMPLETE_COUNT"
else
  echo -e "${YELLOW}No modules found.${NC}"
fi

# Find work items from non-Complete modules only
echo ""
echo -e "${BOLD}Work items to act on:${NC}"

FOUND_ITEMS=0
for f in "$PLANS_DIR/modules/"*.aps.md "$PLANS_DIR/"*.aps.md; do
  [ -f "$f" ] || continue
  basename "$f" | grep -q '^\.' && continue
  basename "$f" | grep -q '^index' && continue

  # Skip modules with Complete status
  if grep -qi '| *Complete *|' "$f" 2>/dev/null; then
    continue
  fi

  # Look for work item headers (### PREFIX-NNN: Title)
  while IFS= read -r line; do
    if echo "$line" | grep -Eq '^### [A-Z][A-Z]*-[0-9][0-9]*:'; then
      ITEM_ID=$(echo "$line" | sed 's/^### \([A-Z]*-[0-9]*\):.*/\1/')
      ITEM_TITLE=$(echo "$line" | sed 's/^### [A-Z]*-[0-9]*: *//')
      echo "  - $ITEM_ID: $ITEM_TITLE  ($(basename "$f"))"
      FOUND_ITEMS=$((FOUND_ITEMS + 1))
    fi
  done < "$f"
done

if [ "$FOUND_ITEMS" -eq 0 ]; then
  echo "  (none — all modules complete or no work items defined)"
fi

echo ""
echo "Tip: Read the relevant module spec before starting work."

# Save session baseline for enforce-plan-update.sh
if git rev-parse --is-inside-work-tree &>/dev/null; then
  mkdir -p .claude
  git rev-parse HEAD > .claude/.aps-session-baseline 2>/dev/null || true
fi
