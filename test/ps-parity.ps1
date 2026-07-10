#!/usr/bin/env pwsh
#
# PowerShell behavioural parity harness (CIP-001).
#
# The bash linter is canonical and test/run.sh covers it. This script runs the
# PowerShell entry point (bin/aps.ps1) over the fixture corpus so the lib/*.psm1
# port is verified *behaviourally*, not just by the string-parity greps in
# test/run.sh. Run locally with `pwsh test/ps-parity.ps1`; CI runs it on a runner
# with pwsh preinstalled (see the `powershell` job in .github/workflows/ci.yml).
#
# Coverage:
#   - nested-plans / federated lint (MONO-002): W003, W020, traversal
#   - cross-tree module-ID collision (MONO-008): W021
#   - conductor rules (COND-007): W002, W006
#   - status gating (COND-007 regression guard): W017 fires for an active
#     module and is emitted before W002 — this exercises Get-ApsStatus, whose
#     spaced-separator misparse once silently disabled W005/W017/W018 in
#     PowerShell and passed every string guard.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$ApsPs1 = Join-Path $ProjectRoot 'bin/aps.ps1'
$FixturesRoot = Join-Path $ScriptDir 'fixtures'
$Fixtures = Join-Path $FixturesRoot 'monorepo'

$script:failed = 0
function Pass($msg) { Write-Host "PASS $msg" -ForegroundColor Green }
function Fail($msg) { Write-Host "FAIL $msg" -ForegroundColor Red; $script:failed++ }

# Run `aps.ps1 lint <target>` and return combined stdout+stderr as one string.
function Invoke-Lint($target) {
    return (& pwsh -NoProfile -File $ApsPs1 lint $target 2>&1 | Out-String)
}

# Copy the monorepo fixture into a fresh temp dir and return its path.
function New-FixtureCopy {
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("aps-ps-parity-" + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmp | Out-Null
    Copy-Item -Path (Join-Path $Fixtures '*') -Destination $tmp -Recurse -Force
    return $tmp
}

Write-Host "Running PowerShell parity tests (nested-plans / MONO-002)...`n"

# Scenario 1: federated parent root follows ## Child Plans links
$out = Invoke-Lint (Join-Path $Fixtures 'plans')
if ($out -match '5 files checked' -and
    $out -match 'core/plans/index\.aps\.md' -and
    $out -match 'api/plans/modules/handlers\.aps\.md' -and
    $out -notmatch 'W003' -and $out -notmatch 'W020') {
    Pass 'federated parent root traverses child plans, cross-tree dep resolves'
} else {
    Fail "federated traversal (got: $out)"
}

# Scenario 2: bad cross-tree ref warns when the named child is in scope
$badref = New-FixtureCopy
try {
    $hfile = Join-Path $badref 'packages/api/plans/modules/handlers.aps.md'
    (Get-Content -LiteralPath $hfile -Raw).Replace('core:AUTH-001', 'core:AUTH-999') |
        Set-Content -LiteralPath $hfile -NoNewline
    $out = Invoke-Lint (Join-Path $badref 'plans')
    if ($out -match 'W003' -and $out -match 'core:AUTH-999') {
        Pass 'bad cross-tree ref flagged in federated lint'
    } else {
        Fail "bad cross-tree ref not flagged (got: $out)"
    }
} finally {
    Remove-Item -Recurse -Force $badref
}

# Scenario 3: isolated child with a cross-tree ref still lints clean
$out = Invoke-Lint (Join-Path $Fixtures 'packages/api/plans')
if ($out -notmatch 'W003') {
    Pass 'isolated child lints clean (external cross-tree ref stays silent)'
} else {
    Fail "isolated child flagged external ref (got: $out)"
}

# Scenario 4: W020 work-item ID collision across child trees
$col = New-FixtureCopy
try {
    $cfile = Join-Path $col 'packages/api/plans/modules/handlers.aps.md'
    (Get-Content -LiteralPath $cfile -Raw).Replace('HND-001', 'AUTH-001') |
        Set-Content -LiteralPath $cfile -NoNewline
    $out = Invoke-Lint (Join-Path $col 'plans')
    if ($out -match 'W020' -and $out -match 'AUTH-001') {
        Pass 'cross-tree ID collision detected (W020)'
    } else {
        Fail "W020 collision not detected (got: $out)"
    }
} finally {
    Remove-Item -Recurse -Force $col
}

# The clean fixture must NOT trip W020
$out = Invoke-Lint (Join-Path $Fixtures 'plans')
if ($out -notmatch 'W020') {
    Pass 'no W020 false positive on the clean fixture'
} else {
    Fail "W020 false positive on clean fixture (got: $out)"
}

# Scenario 5: W021 module ID collision across child trees
$moduleCol = New-FixtureCopy
try {
    $mfile = Join-Path $moduleCol 'packages/api/plans/modules/handlers.aps.md'
    $lines = Get-Content -LiteralPath $mfile
    $lines[4] = $lines[4].Replace('HND', 'AUTH')
    Set-Content -LiteralPath $mfile -Value $lines
    $out = Invoke-Lint (Join-Path $moduleCol 'plans')
    if ($out -match 'W021' -and $out -match "Module ID 'AUTH' defined in multiple child trees: api core") {
        Pass 'cross-tree module collision detected (W021)'
    } else {
        Fail "W021 module collision not detected (got: $out)"
    }
} finally {
    Remove-Item -Recurse -Force $moduleCol
}

Write-Host "`nConductor rules (COND-007)...`n"

# Scenario 6: conductor typo (W002) and mislisted index entry (W006)
$out = Invoke-Lint (Join-Path $FixturesRoot 'conductor/plans')
if ($out -match 'W002' -and $out -match 'AUTH-999' -and
    $out -match 'W006' -and $out -match 'auth\.aps\.md') {
    Pass 'conductor typo (W002) and mislisted index entry (W006) detected'
} else {
    Fail "conductor W002/W006 not detected (got: $out)"
}

# Scenario 7: clean conductor plan trips neither W002 nor W006
$out = Invoke-Lint (Join-Path $FixturesRoot 'conductor-clean/plans')
if ($out -notmatch 'W002' -and $out -notmatch 'W006') {
    Pass 'no W002/W006 false positive on the clean conductor fixture'
} else {
    Fail "W002/W006 false positive on clean conductor fixture (got: $out)"
}

# Scenario 8: an active (Ready) conductor emits W017 before W002. This is the
# COND-007 regression guard — if Get-ApsStatus mis-reads the spaced separator
# row, W017 silently vanishes and this fails (it would still "pass" a string
# guard, which is the whole point of running behaviourally).
$orderFile = Join-Path $FixturesRoot 'conductor-order/plans/modules/release-planning.aps.md'
$out = Invoke-Lint $orderFile
$p17 = $out.IndexOf('W017')
$p02 = $out.IndexOf('W002')
if ($p17 -ge 0 -and $p02 -ge 0 -and $p17 -lt $p02) {
    Pass 'active conductor emits W017 before W002 (status gating + order)'
} else {
    Fail "expected W017 before W002 on active conductor (got: $out)"
}

Write-Host ""
if ($script:failed -gt 0) {
    Write-Host "$($script:failed) PowerShell parity test(s) failed" -ForegroundColor Red
    exit 1
}
Write-Host "All PowerShell parity tests passed!" -ForegroundColor Green
exit 0
