# APS Session Initializer
# Checks for APS planning files and reports status.
# Use as a hook or run manually at session start.
#
# Usage: ./aps-planning/scripts/init-session.ps1 [plans-dir]

param(
    [string]$PlansDir = "plans"
)

Write-Host "APS Planning Session" -ForegroundColor White
Write-Host ("$([char]0x2500)" * 21)

# Check if plans/ exists
if (-not (Test-Path $PlansDir -PathType Container)) {
    Write-Host "No plans/ directory found." -ForegroundColor Yellow
    Write-Host "Run /plan to start APS planning, or create plans/ manually."
    exit 0
}

# Check for index
$indexPath = Join-Path $PlansDir "index.aps.md"
if (Test-Path $indexPath -PathType Leaf) {
    $title = (Get-Content $indexPath -TotalCount 5) |
        Where-Object { $_ -cmatch '^# ' } |
        Select-Object -First 1
    if ($title) {
        $title = $title -creplace '^# ', ''
    }
    if (-not $title) { $title = "[untitled]" }
    Write-Host "Plan: " -ForegroundColor Green -NoNewline
    Write-Host $title
} else {
    Write-Host "No index.aps.md found." -ForegroundColor Yellow
}

# Check for aps-rules.md
$rulesPath = Join-Path $PlansDir "aps-rules.md"
if (Test-Path $rulesPath -PathType Leaf) {
    Write-Host "Agent rules: " -ForegroundColor Green -NoNewline
    Write-Host "plans/aps-rules.md"
}

# Count modules
$moduleCount   = 0
$readyCount    = 0
$progressCount = 0
$completeCount = 0

$modulesDir = Join-Path $PlansDir "modules"
if (Test-Path $modulesDir -PathType Container) {
    foreach ($f in Get-ChildItem (Join-Path $modulesDir "*.aps.md") -File -ErrorAction SilentlyContinue) {
        # Skip hidden template files
        if ($f.Name -cmatch '^\.') { continue }
        $moduleCount++

        # Check status from metadata table
        $content = Get-Content $f.FullName -ErrorAction SilentlyContinue
        if ($content -imatch '\| *Ready *\|') {
            $readyCount++
        } elseif ($content -imatch '\| *In Progress *\|') {
            $progressCount++
        } elseif ($content -imatch '\| *Complete *\|') {
            $completeCount++
        }
    }
}

# Also check for simple specs (*.aps.md at plans/ root, not index)
foreach ($f in Get-ChildItem (Join-Path $PlansDir "*.aps.md") -File -ErrorAction SilentlyContinue) {
    if ($f.Name -cmatch '^index') { continue }
    $moduleCount++
}

if ($moduleCount -gt 0) {
    Write-Host "Modules: " -ForegroundColor Green -NoNewline
    Write-Host "$moduleCount total"
    if ($readyCount -gt 0)    { Write-Host "  Ready: $readyCount" }
    if ($progressCount -gt 0) { Write-Host "  In Progress: $progressCount" }
    if ($completeCount -gt 0) { Write-Host "  Complete: $completeCount" }
} else {
    Write-Host "No modules found." -ForegroundColor Yellow
}

# Find work items from non-Complete modules only
Write-Host ""
Write-Host "Work items to act on:" -ForegroundColor White

$foundItems = 0
$searchPaths = @()
$modulesGlob = Join-Path $modulesDir "*.aps.md"
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

    # Skip modules with Complete status
    $content = Get-Content $f.FullName -ErrorAction SilentlyContinue
    if ($content -imatch '\| *Complete *\|') { continue }

    # Look for work item headers (### PREFIX-NNN: Title)
    foreach ($line in $content) {
        if ($line -cmatch '^### [A-Z]+-[0-9]+:') {
            $itemId    = $line -creplace '^### ([A-Z]+-[0-9]+):.*', '$1'
            $itemTitle = $line -creplace '^### [A-Z]+-[0-9]+: *', ''
            $fileName  = Split-Path $f.FullName -Leaf
            Write-Host "  - ${itemId}: ${itemTitle}  ($fileName)"
            $foundItems++
        }
    }
}

if ($foundItems -eq 0) {
    Write-Host "  (none - all modules complete or no work items defined)"
}

Write-Host ""
Write-Host "Tip: Read the relevant module spec before starting work."

# Save session baseline for enforce-plan-update.ps1
try {
    $null = git rev-parse --is-inside-work-tree 2>$null
    if ($LASTEXITCODE -eq 0) {
        if (-not (Test-Path .claude -PathType Container)) {
            New-Item -ItemType Directory -Path .claude -Force | Out-Null
        }
        $hash = (git rev-parse HEAD 2>$null)
        if ($hash) { [IO.File]::WriteAllText(".claude/.aps-session-baseline", $hash.Trim()) }
    }
} catch {
    # Silently ignore git errors (not in a repo, etc.)
}
