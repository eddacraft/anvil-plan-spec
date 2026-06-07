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
            if ($target -match '^(https?://|mailto:|#)') { continue }
            $target = ($target -split '#')[0]
            if (-not $target) { continue }
            if (-not (Test-Path -LiteralPath (Join-Path $dir $target))) {
                Add-ApsResult -Path $File -Type "warning" -Code "W019" `
                    -Message "Module link target not found: $target" -Line "$lineNum"
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
    Test-W004EmptySectionsIndex -File $File

    return (-not $hasErrors)
}

Export-ModuleMember -Function 'Invoke-ApsIndexLint'
