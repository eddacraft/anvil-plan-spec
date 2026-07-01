#!/usr/bin/env pwsh
#
# PowerShell parity harness for the nested-plans lint behaviour (MONO-002).
#
# The bash linter is canonical and test/run.sh covers it (tests 42-46). This
# script runs the same four scenarios through the PowerShell entry point
# (bin/aps.ps1) so the lib/*.psm1 port is verified *behaviourally*, not just by
# the string-parity grep in test/run.sh. Run locally with `pwsh test/ps-parity.ps1`;
# CI runs it on a runner with pwsh preinstalled.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$ApsPs1 = Join-Path $ProjectRoot 'bin/aps.ps1'
$Fixtures = Join-Path $ScriptDir 'fixtures/monorepo'

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

Write-Host ""
if ($script:failed -gt 0) {
    Write-Host "$($script:failed) PowerShell parity test(s) failed" -ForegroundColor Red
    exit 1
}
Write-Host "All PowerShell parity tests passed!" -ForegroundColor Green
exit 0
