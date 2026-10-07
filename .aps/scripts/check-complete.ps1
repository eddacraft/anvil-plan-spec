# APS Completion Checker
# Verifies that all In Progress work items and action plans have been completed.
# Use as a Stop hook or run before ending a session.
#
# Usage: ./aps-planning/scripts/check-complete.ps1 [plans-dir]
#
# Exit codes:
#   0 — All work items resolved (or JSON decision returned)
#   2 — Work items still in progress (blocks Claude from stopping)

param(
    [string]$PlansDir = "plans"
)

# If no plans directory, nothing to check
if (-not (Test-Path $PlansDir -PathType Container)) {
    exit 0
}

$incomplete = 0
$complete   = 0

# Check all APS files for work item status
$searchPaths = @()
$modulesGlob = Join-Path $PlansDir "modules" "*.aps.md"
$rootGlob    = Join-Path $PlansDir "*.aps.md"

foreach ($glob in @($modulesGlob, $rootGlob)) {
    foreach ($f in Get-ChildItem $glob -File -ErrorAction SilentlyContinue) {
        $searchPaths += $f
    }
}

foreach ($f in $searchPaths) {
    # Skip hidden template files
    if ($f.Name -cmatch '^\.') { continue }
    # Skip index
    if ($f.Name -cmatch '^index') { continue }

    # Parse work items and their statuses
    $currentItem = ""
    $content = Get-Content $f.FullName -ErrorAction SilentlyContinue
    foreach ($line in $content) {
        if ($line -cmatch '^### [A-Z]+-[0-9]+:') {
            $currentItem = $line -creplace '^### ', '' -creplace ' *$', ''
        }

        if ($currentItem) {
            # Check for In Progress status (matches both **Status:** and Status: formats)
            if ($line -imatch '\*\*Status:\*\* *In Progress|Status: *In Progress') {
                $fileName = Split-Path $f.FullName -Leaf
                Write-Host "Still in progress: " -ForegroundColor Yellow -NoNewline
                Write-Host "$currentItem ($fileName)"
                $incomplete++
                $currentItem = ""
            }
            # Check for Complete status (matches both **Status:** and Status: formats)
            elseif ($line -imatch '\*\*Status:\*\* *Complete|Status: *Complete') {
                $complete++
                $currentItem = ""
            }
        }
    }
}

# Check action plans for incomplete checkpoints (only In Progress ones)
$executionDir = Join-Path $PlansDir "execution"
if (Test-Path $executionDir -PathType Container) {
    foreach ($f in Get-ChildItem (Join-Path $executionDir "*.actions.md") -File -ErrorAction SilentlyContinue) {
        $content = Get-Content $f.FullName -ErrorAction SilentlyContinue
        # Check if this action plan is In Progress
        $isInProgress = $content | Where-Object { $_ -imatch '^\| *Status *\|.*(In Progress|In-Progress)' }
        if ($isInProgress) {
            $unchecked = ($content | Where-Object { $_ -match '^ *- \[ \]' }).Count
            if ($unchecked -gt 0) {
                $fileName = Split-Path $f.FullName -Leaf
                Write-Host "Unchecked items: " -ForegroundColor Yellow -NoNewline
                Write-Host "$unchecked in $fileName"
                $incomplete++
            }
        }
    }
}

if ($incomplete -gt 0) {
    # Exit 2 blocks Claude from stopping. stderr is fed back to Claude.
    [Console]::Error.WriteLine("Session incomplete. $incomplete item(s) still need attention.")
    if ($complete -gt 0) {
        [Console]::Error.WriteLine("Session status: $complete complete, $incomplete incomplete")
    }
    [Console]::Error.WriteLine("")
    [Console]::Error.WriteLine("Before ending this session:")
    [Console]::Error.WriteLine("  1. Complete or explicitly mark items as Blocked")
    [Console]::Error.WriteLine("  2. Update work item statuses in the module spec")
    [Console]::Error.WriteLine("  3. Add any discovered work as Draft items")
    [Console]::Error.WriteLine("  4. Commit APS changes to git")
    exit 2
} else {
    exit 0
}
