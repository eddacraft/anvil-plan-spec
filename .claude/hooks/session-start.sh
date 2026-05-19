#!/usr/bin/env bash
# anvil-plan-spec session-start hook: record local session context and surface tool state.

set -euo pipefail

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(pwd)}"
LOG_DIR="$PROJECT_DIR/.claude/logs"
mkdir -p "$LOG_DIR"

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
if command -v jq >/dev/null 2>&1; then
  jq -n --arg ts "$TIMESTAMP" --arg project "$PROJECT_DIR" \
    '{timestamp: $ts, event: "session_start", project: $project}' >> "$LOG_DIR/session.log"
else
  printf '{"timestamp":"%s","event":"session_start","project":"%s"}\n' \
    "$TIMESTAMP" "$PROJECT_DIR" >> "$LOG_DIR/session.log"
fi

check_tool() {
  if command -v "$1" >/dev/null 2>&1; then
    printf '  [OK] %s\n' "$1"
  else
    printf '  [MISSING] %s\n' "$1"
  fi
}

printf '=== anvil-plan-spec Session Check ===\n'
printf 'Project: %s\n\n' "$PROJECT_DIR"

printf 'Required tools:\n'
check_tool node
check_tool pnpm
check_tool git
check_tool gh
check_tool jq

if [[ -d "$PROJECT_DIR/.git" ]]; then
  cd "$PROJECT_DIR"
  printf '\nGit status:\n'
  printf '  Branch: %s\n' "$(git branch --show-current 2>/dev/null || printf unknown)"
  printf '  Uncommitted changes: %s\n' "$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
fi

printf '=== Session Ready ===\n'

exit 0
