# APS PostToolUse Hook
# Nudges the agent to update specs after code changes.
# Outputs JSON with additionalContext so the nudge reaches Claude.
#
# Only fires if an APS plans/ directory exists.

if (Test-Path plans -PathType Container) {
    Write-Output '{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"[APS] If you completed a work item or discovered new scope, update the APS spec now."}}'
}

exit 0
