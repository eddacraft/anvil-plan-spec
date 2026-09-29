#
# APS CLI Release Plan Validation Rules (REL-003)
#
# Release narratives live under a `releases/` directory as `v<version>.md`.
# Unlike design documents these rules are ERRORS, not warnings: a malformed
# release plan must fail CI rather than whisper. Hand-port of `lint_release`
# in cli/src/lint.rs and lib/rules/release.sh — same codes, order, severities
# and messages (D-039).
#

# Dependencies (Output, Common) must be imported by the entry point.

# Three numeric components, with optional ASCII prerelease/build identifiers.
# Version shape only; mirrors is_release_filename() in cli/src/lint.rs.
function Test-ApsReleaseFilename {
    param([string]$Basename)
    return ($Basename -cmatch '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?(\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?\.md$')
}

# R001: Release file must be named v<version>.md
function Test-R001ReleaseNaming {
    param([string]$File)
    $basename = Split-Path $File -Leaf
    if (-not (Test-ApsReleaseFilename -Basename $basename)) {
        Add-ApsResult -Path $File -Type "error" -Code "R001" `
            -Message "Release file must be named v<version>.md (e.g. v0.3.0.md)"
    }
}

# R002: Missing release header table with Target and Status fields
# Require Target and Status in one table body within the first 20 lines.
function Test-R002ReleaseHeaderTable {
    param([string]$File)
    $lines = @(Get-Content -LiteralPath $File -ErrorAction SilentlyContinue)
    $limit = [Math]::Min(20, $lines.Count)
    $previous = $table = $hasTarget = $hasStatus = $fence = $false
    for ($i = 0; $i -lt $limit; $i++) {
        $line = $lines[$i]
        if ($line -match '^(```|~~~)') { $fence = -not $fence }
        if ($fence -or -not $line.StartsWith('|')) {
            $previous = $table = $hasTarget = $hasStatus = $false
            continue
        }
        if ($previous -and $line -match '^\| *:?-{3,}:? *(\| *:?-{3,}:? *)+\|? *\r?$') {
            $table = $true
            $hasTarget = $hasStatus = $false
        } elseif ($table) {
            if ($line -cmatch '^\| *Target *\|') { $hasTarget = $true }
            if ($line -cmatch '^\| *Status *\|') { $hasStatus = $true }
            if ($hasTarget -and $hasStatus) { return }
        }
        $previous = $true
    }
    Add-ApsResult -Path $File -Type "error" -Code "R002" `
        -Message "Missing release header table with Target and Status fields"
}

# R003: Missing ## Release Theme section
function Test-R003ReleaseTheme {
    param([string]$File)
    if (-not (Test-ApsSection -FilePath $File -SectionHeader "## Release Theme")) {
        Add-ApsResult -Path $File -Type "error" -Code "R003" -Message "Missing ## Release Theme section"
    }
}

# R004: Missing ## What Ships section
function Test-R004WhatShips {
    param([string]$File)
    if (-not (Test-ApsSection -FilePath $File -SectionHeader "## What Ships")) {
        Add-ApsResult -Path $File -Type "error" -Code "R004" -Message "Missing ## What Ships section"
    }
}

# Run all release rules. Emission order is load-bearing for cross-CLI parity:
# R001, R002, R003, R004 — matching lint_release in cli/src/lint.rs.
function Invoke-ApsReleaseLint {
    param([string]$File)

    Test-R001ReleaseNaming -File $File
    Test-R002ReleaseHeaderTable -File $File
    Test-R003ReleaseTheme -File $File
    Test-R004WhatShips -File $File

    return $true
}

Export-ModuleMember -Function @(
    'Invoke-ApsReleaseLint'
    'Test-ApsReleaseFilename'
)
