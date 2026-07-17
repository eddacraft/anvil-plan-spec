# Native Windows/PowerShell APS user journey (CIB-002/CIB-003/CIB-004).

param(
    [Parameter(Mandatory = $true)]
    [string]$ReleaseArchive
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$global:LASTEXITCODE = 0

$RepoRoot = Split-Path -Parent $PSScriptRoot
$ReleaseArchive = (Resolve-Path -LiteralPath $ReleaseArchive).Path
$Installer = Join-Path $RepoRoot "scaffold\install.ps1"
$Work = Join-Path ([System.IO.Path]::GetTempPath()) ("aps-windows-journey-" + [guid]::NewGuid())
$ApsHome = Join-Path $Work "home\.aps"
$BinDir = Join-Path $ApsHome "bin"
$InstalledAps = Join-Path $BinDir "aps.exe"
$OriginalUserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$OriginalApsHome = $env:APS_HOME
$global:ApsRequestedUris = @()

# Exercise the public release-download/extraction path without depending on an
# already-published PR artifact. The archive is built with the same GNU target
# and zip layout as release.yml; only transport is replaced.
function Invoke-WebRequest {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$Uri,
        [string]$OutFile,
        [switch]$UseBasicParsing
    )
    $global:ApsRequestedUris += $Uri
    if (-not $OutFile) { throw "unexpected content-only request: $Uri" }
    Copy-Item -LiteralPath $ReleaseArchive -Destination $OutFile -Force
}

function Invoke-Checked {
    param(
        [scriptblock]$Command,
        [string]$Label
    )
    $global:LASTEXITCODE = 0
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

function Invoke-CapturedNative {
    param(
        [scriptblock]$Command
    )
    $PreviousErrorActionPreference = $ErrorActionPreference
    try {
        # Windows PowerShell 5.1 promotes redirected native stderr to an error
        # record. Keep expected non-zero CLI results inspectable by the harness.
        $ErrorActionPreference = "Continue"
        $global:LASTEXITCODE = 0
        $Output = (& $Command 2>&1 | Out-String)
        $ExitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $PreviousErrorActionPreference
    }
    [pscustomobject]@{
        Output = $Output
        ExitCode = $ExitCode
    }
}

function Assert-Contains {
    param(
        [string]$Path,
        [string]$Text
    )
    if (-not (Select-String -LiteralPath $Path -SimpleMatch $Text -Quiet)) {
        throw "$Path does not contain expected text: $Text"
    }
}

try {
    New-Item -ItemType Directory -Path $Work -Force | Out-Null
    $env:APS_HOME = $ApsHome
    # Avoid changing the real user PATH or prompting in the installer.
    $TestUserPath = if ($OriginalUserPath) { "$BinDir;$OriginalUserPath" } else { $BinDir }
    [Environment]::SetEnvironmentVariable("PATH", $TestUserPath, "User")
    $env:PATH = "$BinDir;$env:PATH"

    Invoke-Checked -Label "PowerShell release installer" -Command {
        & $Installer --cli --binary
    }
    if (-not ($global:ApsRequestedUris -match "aps-x86_64-pc-windows-gnu.zip")) {
        throw "installer did not request the shipped Windows GNU archive"
    }
    if (-not (Test-Path -LiteralPath $InstalledAps -PathType Leaf)) {
        throw "PowerShell installer did not extract aps.exe"
    }
    Invoke-Checked -Label "aps --version" -Command { & $InstalledAps --version }

    # Default onboarding covers the single-project route. CI has redirected
    # input, so the native binary uses its documented non-interactive defaults.
    $Single = Join-Path $Work "single"
    New-Item -ItemType Directory -Path $Single -Force | Out-Null
    Push-Location $Single
    try {
        Invoke-Checked -Label "installer onboarding handoff" -Command {
            & $Installer --onboard --binary
        }
        Assert-Contains -Path ".aps\config.yml" -Text "shape: single"
        Assert-Contains -Path ".aps\config.yml" -Text "  - index"
        if (Select-String -LiteralPath "plans\index.aps.md" -SimpleMatch "## Modules by Package" -Quiet) {
            throw "single-project onboarding wrote the monorepo root"
        }
    } finally {
        Pop-Location
    }

    $Project = Join-Path $Work "monorepo"
    New-Item -ItemType Directory -Path $Project -Force | Out-Null
    Push-Location $Project
    try {
        Invoke-Checked -Label "monorepo init" -Command {
            & $InstalledAps init --non-interactive --profile team --shape monorepo --tools generic
        }
        Assert-Contains -Path "plans\index.aps.md" -Text "## Modules by Package"
        Assert-Contains -Path ".aps\config.yml" -Text "shape: monorepo"
        Assert-Contains -Path ".aps\config.yml" -Text "  - monorepo-index"

        Invoke-Checked -Label "setup hooks" -Command { & $InstalledAps setup hooks --yes }
        if (-not (Test-Path -LiteralPath ".aps\scripts\install-hooks.ps1" -PathType Leaf)) {
            throw "PowerShell setup command did not install install-hooks.ps1"
        }
        Invoke-Checked -Label "PowerShell hook installer" -Command {
            & .\.aps\scripts\install-hooks.ps1 --minimal
        }
        Invoke-Checked -Label "lint" -Command { & $InstalledAps lint plans }
        $NextResult = Invoke-CapturedNative { & $InstalledAps next }
        if ($NextResult.ExitCode -ne 1 -or $NextResult.Output -notmatch "No ready work item found") {
            throw "next/status did not report the empty ready queue"
        }
        Invoke-Checked -Label "update" -Command { & $InstalledAps update . }
        Invoke-Checked -Label "migrate dry run" -Command { & $InstalledAps migrate }
        Invoke-Checked -Label "doctor" -Command { & $InstalledAps doctor }
    } finally {
        Pop-Location
    }

    $Nested = Join-Path $Work "nested"
    New-Item -ItemType Directory -Path $Nested -Force | Out-Null
    Push-Location $Nested
    try {
        Invoke-Checked -Label "nested init" -Command {
            & $InstalledAps init --non-interactive --profile team --templates index-nested --tools generic
        }
        Assert-Contains -Path "plans\index.aps.md" -Text "## Child Plans"
        Assert-Contains -Path ".aps\config.yml" -Text "  - index-nested"
        if (-not (Test-Path -LiteralPath "packages\core\plans\index.aps.md" -PathType Leaf)) {
            throw "nested init did not create the core child plan"
        }
        Invoke-Checked -Label "rollup" -Command { & $InstalledAps rollup }
    } finally {
        Pop-Location
    }

    $Lifecycle = Join-Path $Work "lifecycle"
    Copy-Item -LiteralPath (Join-Path $RepoRoot "test\fixtures\orchestrate") -Destination $Lifecycle -Recurse
    Push-Location $Lifecycle
    try {
        $NextReady = (& $InstalledAps next auth 2>&1 | Out-String)
        if ($LASTEXITCODE -ne 0 -or $NextReady -notmatch "AUTH-003") {
            throw "next did not select AUTH-003"
        }
        Invoke-Checked -Label "start" -Command { & $InstalledAps start AUTH-003 }
        Invoke-Checked -Label "graph" -Command { & $InstalledAps graph auth }
        $AuditResult = Invoke-CapturedNative { & $InstalledAps audit auth --no-run }
        if ($AuditResult.ExitCode -notin 0, 1) {
            throw "audit failed with exit code $($AuditResult.ExitCode)"
        }
        $Export = (& $InstalledAps export --json 2>&1 | Out-String)
        if ($LASTEXITCODE -ne 0 -or $Export -notmatch '"work_items"') {
            throw "export did not return work-item JSON"
        }
        Invoke-Checked -Label "complete" -Command {
            & $InstalledAps complete AUTH-003 --learning "native Windows PowerShell journey"
        }
        $NextAfterComplete = (& $InstalledAps next auth 2>&1 | Out-String)
        if ($LASTEXITCODE -ne 0 -or $NextAfterComplete -notmatch "AUTH-004") {
            throw "complete did not advance the ready queue"
        }
    } finally {
        Pop-Location
    }

    Write-Host "native Windows PowerShell user journey passed"
} finally {
    [Environment]::SetEnvironmentVariable("PATH", $OriginalUserPath, "User")
    $env:APS_HOME = $OriginalApsHome
    Remove-Item -LiteralPath $Work -Recurse -Force -ErrorAction SilentlyContinue
}
