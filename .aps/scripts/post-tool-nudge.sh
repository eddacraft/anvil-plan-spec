#!/usr/bin/env bash
# APS PostToolUse Hook
# Nudges the agent to update specs after code changes.
# Outputs JSON with additionalContext so the nudge reaches Claude.
#
# Only fires if an APS plans/ directory exists.

set -euo pipefail

if [ -d plans ]; then
  cat <<'EOF'
{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"[APS] If you completed a work item or discovered new scope, update the APS spec now."}}
EOF
fi

exit 0
