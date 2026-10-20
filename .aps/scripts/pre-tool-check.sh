#!/usr/bin/env bash
# APS PreToolUse Hook
# Reminds the agent to re-read its current work item before code changes.
# Outputs JSON with additionalContext so the reminder reaches Claude.
#
# Only fires if an APS plans/ directory exists.

set -euo pipefail

if [ -d plans ] && [ -f plans/index.aps.md -o -d plans/modules ]; then
  cat <<'EOF'
{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":"[APS] Re-read your current work item before making changes. Are you still on-plan?"}}
EOF
fi

exit 0
