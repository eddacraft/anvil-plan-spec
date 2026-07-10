#
# APS CLI Index Validation Rules
# Port of lib/rules/index.sh
#

# Dependencies (Output, Common) must be imported by the entry point.

# E004: Missing ## Modules section
function Test-E004Modules {
    param([string]$File)
    if (-not (Test-ApsSection -FilePath $File -SectionHeader "## Modules")) {
        Add-ApsResult -Path $File -Type "error" -Code "E004" -Message "Missing ## Modules section"
        return $false
    }
    return $true
}

# W019: Link in ## Modules points to a non-existent file
#
# Warning (not error) because the scaffold seed index intentionally links a
# placeholder module that the user creates later. `aps audit` reports the
# same condition as finding A004 with a non-zero exit for hard gating.
function Test-W019ModuleLinks {
    param([string]$File)
    $dir = Split-Path $File -Parent
    if (-not $dir) { $dir = "." }

    $lines = Get-Content -LiteralPath $File -ErrorAction SilentlyContinue
    $inModules = $false
    $lineNum = 0
    foreach ($line in $lines) {
        $lineNum++
        if ($line -match '^## Modules') { $inModules = $true; continue }
        if ($inModules -and $line -match '^## ') { $inModules = $false }
        if (-not $inModules) { continue }

        foreach ($m in [regex]::Matches($line, '\]\(([^)]+)\)')) {
            $target = $m.Groups[1].Value
            # Strip markdown link titles: ](path "title")
            $target = $target -replace '\s+["''].*$', ''
            # Skip pure anchors and any URI scheme (http, mailto, file, ...)
            if ($target -match '^(#|[A-Za-z][A-Za-z0-9+.-]*:)') { continue }
            $target = ($target -split '#')[0]
            if (-not $target) { continue }
            if (-not (Test-Path -LiteralPath (Join-Path $dir $target))) {
                Add-ApsResult -Path $File -Type "warning" -Code "W019" `
                    -Message "Module link target not found: $target" -Line "$lineNum"
            }
        }
    }
}

# W006: a module listed under a `### Conductor / Crosscutting` index subsection
# whose file exists but is not marked `Type: Conductor`. Keeps the index's
# conductor grouping honest — the inverse of COND-003's module check. Missing
# link targets are W019's job, so they are skipped. Mirrors the Rust
# check_w006_conductor_index / bash check_w006_conductor_index.
function Test-W006ConductorIndex {
    param([string]$File)
    $dir = Split-Path $File -Parent
    if (-not $dir) { $dir = "." }

    $lines = Get-Content -LiteralPath $File -ErrorAction SilentlyContinue
    $inConductor = $false
    $lineNum = 0
    foreach ($line in $lines) {
        $lineNum++
        if ($line -match '^### ') {
            $h = $line.ToLower()
            $inConductor = ($h -match 'conductor' -or $h -match 'crosscutting')
            continue
        }
        if ($line -match '^## ') { $inConductor = $false }
        if (-not $inConductor) { continue }

        foreach ($m in [regex]::Matches($line, '\]\(([^)]+)\)')) {
            $target = $m.Groups[1].Value
            $target = $target -replace '\s+["''].*$', ''   # strip link titles
            $target = ($target -split '#')[0]              # strip anchor
            if ($target -notmatch '\.aps\.md$') { continue }
            $path = Join-Path $dir $target
            if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { continue }
            if (-not (Test-ApsConductor -FilePath $path)) {
                Add-ApsResult -Path $File -Type "warning" -Code "W006" `
                    -Message "Module '$target' is listed under Conductor / Crosscutting but its file is not marked ``Type: Conductor``" -Line "$lineNum"
            }
        }
    }
}

# W004: Empty section check (index-specific sections)
function Test-W004EmptySectionsIndex {
    param([string]$File)
    $sections = @("## Overview", "## Problem & Success Criteria", "## Modules")
    foreach ($section in $sections) {
        if ((Test-ApsSection -FilePath $File -SectionHeader $section) -and
            -not (Test-ApsSectionHasContent -FilePath $File -SectionHeader $section)) {
            $line = Get-ApsLineNumber -FilePath $File -Pattern "^$([regex]::Escape($section))$"
            Add-ApsResult -Path $File -Type "warning" -Code "W004" -Message "Empty section: $section" -Line "$line"
        }
    }
}

# Run all index rules
function Invoke-ApsIndexLint {
    param([string]$File)
    $hasErrors = $false

    if (-not (Test-E004Modules -File $File)) { $hasErrors = $true }
    Test-W019ModuleLinks -File $File
    Test-W006ConductorIndex -File $File
    Test-W004EmptySectionsIndex -File $File

    return (-not $hasErrors)
}

Export-ModuleMember -Function 'Invoke-ApsIndexLint'
