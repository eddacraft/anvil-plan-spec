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

# True when a basename is `v<digit>...md` — `v0.3.0.md`, `v1.2.0-beta.md`.
# Mirrors is_release_filename() in cli/src/lint.rs: a literal lowercase `v`
# followed by an ASCII digit, and a `.md` extension. `-cmatch` keeps the check
# case-sensitive like the Rust and bash versions (PowerShell's `-match` is
# case-insensitive by default, which would wrongly accept `V0.3.0.md`).
function Test-ApsReleaseFilename {
    param([string]$Basename)
    return ($Basename -cmatch '^v[0-9].*\.md$')
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
# The header table drives the release — Target (which version) and Status
# (where it is in the lifecycle). Both rows must appear in the first 20 lines.
function Test-R002ReleaseHeaderTable {
    param([string]$File)
    $lines = @(Get-Content -LiteralPath $File -ErrorAction SilentlyContinue)
    $limit = [Math]::Min(20, $lines.Count)
    $hasTarget = $false
    $hasStatus = $false
    for ($i = 0; $i -lt $limit; $i++) {
        # -cmatch: the Rust and bash checks are case-sensitive, so `| target |`
        # must not satisfy this rule. PowerShell's bare -match would.
        if ($lines[$i] -cmatch '^\| *Target *\|') { $hasTarget = $true }
        if ($lines[$i] -cmatch '^\| *Status *\|') { $hasStatus = $true }
    }
    if (-not ($hasTarget -and $hasStatus)) {
        Add-ApsResult -Path $File -Type "error" -Code "R002" `
            -Message "Missing release header table with Target and Status fields"
    }
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
