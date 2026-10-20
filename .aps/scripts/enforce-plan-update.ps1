# APS Plan Update Enforcer
# Blocks session end if code was changed but no plan files were updated.
# Pairs with check-complete.ps1: that checks statuses, this checks file changes.
#
# Uses a session baseline marker (written by init-session.ps1) to detect all
# changes -- committed and uncommitted -- during the session. Falls back to
# uncommitted-only checks if no baseline exists.
#
# Usage: ./aps-planning/scripts/enforce-plan-update.ps1 [plans-dir]
#
# Exit codes:
#   0 -- Plans were updated (or no code changes detected)
#   2 -- Code changed but plans were not updated (blocks Claude from stopping)

param(
    [string]$PlansDir = "plans"
)

$BaselineFile = ".claude/.aps-session-baseline"

# If no plans directory, nothing to enforce
if (-not (Test-Path $PlansDir -PathType Container)) {
    exit 0
}

# Not a git repo -- can't diff, skip enforcement
$null = git rev-parse --is-inside-work-tree 2>$null
if ($LASTEXITCODE -ne 0) {
    exit 0
}

# Collect all changed files during this session
$ChangedFiles = @()

if (Test-Path $BaselineFile -PathType Leaf) {
    $Baseline = (Get-Content $BaselineFile -Raw).Trim()

    # Validate the baseline is a real commit
    $null = git rev-parse --verify $Baseline 2>$null
    if ($LASTEXITCODE -eq 0) {
        # Changes in commits since session start
        $Committed   = @(git diff --name-only $Baseline HEAD 2>$null) | Where-Object { $_ -cne '' }
        # Uncommitted changes (staged + unstaged)
        $Uncommitted = @(git diff --name-only HEAD 2>$null) | Where-Object { $_ -cne '' }
        $Staged      = @(git diff --name-only --cached 2>$null) | Where-Object { $_ -cne '' }

        # Untracked files (new files not yet staged)
        $Untracked   = @(git ls-files --others --exclude-standard 2>$null) | Where-Object { $_ -cne '' }

        $ChangedFiles = @($Committed) + @($Uncommitted) + @($Staged) + @($Untracked) |
            Sort-Object -Unique
    }
}

# Fallback: no baseline or invalid baseline -- check uncommitted only
if ($ChangedFiles.Count -eq 0) {
    $Uncommitted = @(git diff --name-only 2>$null) | Where-Object { $_ -cne '' }
    $Staged      = @(git diff --name-only --cached 2>$null) | Where-Object { $_ -cne '' }
    $Untracked   = @(git ls-files --others --exclude-standard 2>$null) | Where-Object { $_ -cne '' }
    $ChangedFiles = @($Uncommitted) + @($Staged) + @($Untracked) |
        Sort-Object -Unique
}

# Filter out empty lines and ephemeral .claude/ files (e.g. session baseline)
$ChangedFiles = @($ChangedFiles) | Where-Object { $_ -cne '' } | Where-Object { $_ -cnotmatch '^\.claude/' }

# Nothing changed at all -- nothing to enforce
if ($ChangedFiles.Count -eq 0) {
    exit 0
}

# Split into plan files and non-plan files (anchor match to path start)
$PlanChanges = @($ChangedFiles) | Where-Object { $_ -cmatch "^$PlansDir/" }
$CodeChanges = @($ChangedFiles) | Where-Object { $_ -cnotmatch "^$PlansDir/" }

# If no code changes (plan-only session), nothing to enforce
if ($CodeChanges.Count -eq 0) {
    exit 0
}

# Code was changed -- were plans updated too?
if ($PlanChanges.Count -eq 0) {
    # Block: code changed but plans weren't touched
    [Console]::Error.WriteLine("Code was modified but no plan files were touched.")
    [Console]::Error.WriteLine("")
    [Console]::Error.WriteLine("Changed files outside plans/:")
    $Shown = @($CodeChanges) | Select-Object -First 10
    foreach ($file in $Shown) {
        [Console]::Error.WriteLine("  $file")
    }
    $Total = @($CodeChanges).Count
    if ($Total -gt 10) {
        $Remaining = $Total - 10
        [Console]::Error.WriteLine("  ... and $Remaining more")
    }
    [Console]::Error.WriteLine("")
    [Console]::Error.WriteLine("Before ending this session, update the relevant APS spec:")
    [Console]::Error.WriteLine("  1. Mark completed work items as Complete")
    [Console]::Error.WriteLine("  2. Update In Progress items with current state")
    [Console]::Error.WriteLine("  3. Add any newly discovered work as Draft items")
    [Console]::Error.WriteLine("  4. Update module status if all work items are done")
    exit 2
}

# Both code and plans changed -- good
exit 0
