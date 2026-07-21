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

# Scenario 7: clean conductor plan lints fully clean — no W002/W006, and the
# success summary is present so a run that failed for an unrelated reason (bad
# path, runtime error) can't pass merely by not emitting those codes.
$out = Invoke-Lint (Join-Path $FixturesRoot 'conductor-clean/plans')
if ($out -match 'no issues' -and $out -notmatch 'W002' -and $out -notmatch 'W006') {
    Pass 'no W002/W006 false positive on the clean conductor fixture'
} else {
    Fail "clean conductor fixture did not lint cleanly (got: $out)"
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

# Scenario 9: managed skill markers (INSTALL-020 / D-042). `aps.ps1 update` on
# a v2 project reconciles skill trees by marker state: absent -> added with a
# canonical marker, fresh -> untouched, dirty -> refused with user content
# preserved. Runs offline via APS_LOCAL against this repo's scaffold payload.
$env:APS_LOCAL = $ProjectRoot
$mng = Join-Path ([System.IO.Path]::GetTempPath()) ("aps-ps-managed-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path (Join-Path $mng "plans") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $mng ".aps") -Force | Out-Null
@"
cli_version: "0.6.0"
plans_dir: plans/
tooling_root: .aps/

aps:
  updated: "2026-01-01"

tools:
  - name: claude-code
    skill: .claude/skills/aps-planning
"@ | Set-Content -LiteralPath (Join-Path $mng ".aps/config.yml")

$skillDir = Join-Path $mng ".claude/skills/aps-planning"
$marker = Join-Path $skillDir ".aps-managed.json"

# Plant a legacy version stamp: D-044 retires it, so update must remove it.
Set-Content -LiteralPath (Join-Path $mng "plans/.aps-version") -Value "0.5.0"

& pwsh -NoProfile -File $ApsPs1 update $mng *> $null
$markerText = if (Test-Path -LiteralPath $marker) { Get-Content -LiteralPath $marker -Raw } else { "" }
if ($markerText -match '"schemaVersion": 1' -and
    $markerText -match '"kind": "skill"' -and
    $markerText -match '"SKILL.md": "[0-9a-f]{64}"') {
    Pass 'v2 update installs the skill with a canonical managed marker'
} else {
    Fail "managed marker missing or malformed after v2 update (got: $markerText)"
}
if (-not (Test-Path -LiteralPath (Join-Path $mng "plans/.aps-version"))) {
    Pass 'v2 update removes the retired plans/.aps-version stamp (D-044)'
} else {
    Fail 'plans/.aps-version survived the v2 update'
}

# Fresh: a second update must not rewrite the marker or files.
$before = Get-Content -LiteralPath $marker -Raw
& pwsh -NoProfile -File $ApsPs1 update $mng *> $null
if ((Get-Content -LiteralPath $marker -Raw) -ceq $before) {
    Pass 'fresh skill tree survives update byte-for-byte'
} else {
    Fail 'fresh update rewrote the managed marker'
}

# Dirty: a user edit is refused and preserved.
Add-Content -LiteralPath (Join-Path $skillDir "SKILL.md") -Value "user edit"
$out = (& pwsh -NoProfile -File $ApsPs1 update $mng 2>&1 | Out-String)
$skillText = Get-Content -LiteralPath (Join-Path $skillDir "SKILL.md") -Raw
if ($out -match 'local edits' -and $skillText -match 'user edit') {
    Pass 'dirty skill tree is refused and user content preserved'
} else {
    Fail "dirty tree not protected (output: $out)"
}

Remove-Item -LiteralPath $mng -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item Env:APS_LOCAL -ErrorAction SilentlyContinue

Write-Host "`nInit v2 minimal layout (INSTALL-023)...`n"

# Scenario 10: `aps.ps1 init` scaffolds the v2 minimal layout (INSTALL-011 /
# INSTALL-023): plans templates + seed index + .aps/config.yml only — no root
# aps-planning/, no .claude/commands/, no root bin/. Tool skills are opt-in
# via --tools and land under managed markers; `aps.ps1 update` then accepts
# the result as a v2 layout without touching the fresh skill tree.
$env:APS_LOCAL = $ProjectRoot

# Minimal default: no --tools means no tool footprint at all.
$initMin = Join-Path ([System.IO.Path]::GetTempPath()) ("aps-ps-init-min-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $initMin | Out-Null
& pwsh -NoProfile -File $ApsPs1 init $initMin --non-interactive *> $null
if ((Test-Path (Join-Path $initMin ".aps/config.yml")) -and
    (Test-Path (Join-Path $initMin "plans/index.aps.md")) -and
    (Test-Path (Join-Path $initMin "plans/aps-rules.md")) -and
    -not (Test-Path (Join-Path $initMin ".claude"))) {
    Pass 'init minimal default: plans + config only, no .claude/ at all'
} else {
    $tree = (Get-ChildItem -Force $initMin | ForEach-Object Name) -join ' '
    Fail "init minimal default footprint wrong (top level: $tree)"
}
Remove-Item -LiteralPath $initMin -Recurse -Force -ErrorAction SilentlyContinue

# With --tools claude-code: managed skill installed, still no v1 footprint.
$initCc = Join-Path ([System.IO.Path]::GetTempPath()) ("aps-ps-init-cc-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $initCc | Out-Null
& pwsh -NoProfile -File $ApsPs1 init $initCc --non-interactive --tools claude-code *> $null
$ccMarker = Join-Path $initCc ".claude/skills/aps-planning/.aps-managed.json"
if ((Test-Path (Join-Path $initCc ".aps/config.yml")) -and
    (Test-Path (Join-Path $initCc "plans/index.aps.md")) -and
    (Test-Path (Join-Path $initCc "plans/aps-rules.md")) -and
    (Test-Path -LiteralPath $ccMarker) -and
    -not (Test-Path (Join-Path $initCc "aps-planning")) -and
    -not (Test-Path (Join-Path $initCc ".claude/commands")) -and
    -not (Test-Path (Join-Path $initCc "bin"))) {
    Pass 'init --tools claude-code: managed skill installed, no v1 footprint'
} else {
    $tree = (Get-ChildItem -Force $initCc | ForEach-Object Name) -join ' '
    Fail "init --tools claude-code footprint wrong (top level: $tree)"
}

# The init result is a valid v2 layout: `update` runs the v2 path cleanly and
# leaves the fresh skill marker byte-identical.
$ccBefore = Get-Content -LiteralPath $ccMarker -Raw
& pwsh -NoProfile -File $ApsPs1 update $initCc *> $null
$updExit = $LASTEXITCODE
if ($updExit -eq 0 -and (Get-Content -LiteralPath $ccMarker -Raw) -ceq $ccBefore) {
    Pass 'update accepts the init result as v2, fresh marker byte-identical'
} else {
    Fail "update after init not clean (exit $updExit) or marker rewritten"
}
Remove-Item -LiteralPath $initCc -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item Env:APS_LOCAL -ErrorAction SilentlyContinue

Write-Host ""
if ($script:failed -gt 0) {
    Write-Host "$($script:failed) PowerShell parity test(s) failed" -ForegroundColor Red
    exit 1
}
Write-Host "All PowerShell parity tests passed!" -ForegroundColor Green
exit 0
