# APS PreToolUse Hook
# Reminds the agent to re-read its current work item before code changes.
# Outputs JSON with additionalContext so the reminder reaches Claude.
#
# Only fires if an APS plans/ directory exists.

if ((Test-Path plans -PathType Container) -and
    ((Test-Path plans/index.aps.md -PathType Leaf) -or (Test-Path plans/modules -PathType Container))) {
    Write-Output '{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":"[APS] Re-read your current work item before making changes. Are you still on-plan?"}}'
}

exit 0
