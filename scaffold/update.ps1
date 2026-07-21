#
# APS Update Script (PowerShell)
# Brings an existing APS project to the current (v2) layout — D-043.
# Your specs (index.aps.md, modules/*.aps.md) are preserved.
#
# v2 projects (.aps/config.yml) are refreshed in place; v1 projects
# (root aps-planning/, .claude/commands/, root bin/ + lib/) are migrated:
# legacy trees are backed up to .aps/backup/<timestamp>/ and removed,
# .aps/config.yml is created, and the packaged scaffold/aps-planning/
# payload lands in the managed skill trees with `.aps-managed.json`
# markers (D-042).
#
# The heavy lifting is delegated to a capable `aps` CLI (native binary on
# PATH, or the project's vendored .aps/bin/aps.ps1). When neither is
# present, the pinned PowerShell CLI is fetched into a temp dir and used —
# the update never silently skips managed markers.
#
# Usage:
#   Invoke-Expression (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/update.ps1" -UseBasicParsing).Content
#   $env:APS_VERSION = "v0.2.0"; Invoke-Expression (Invoke-WebRequest -Uri "..." -UseBasicParsing).Content
#
# For new projects, use the install script instead.
#

$ErrorActionPreference = "Stop"

$Version = if ($env:APS_VERSION) { $env:APS_VERSION } else { "main" }
$GlobalInstall = $false
$Target = "."

foreach ($a in $args) {
    if ($a -eq "--global" -or $a -eq "-g") {
        $GlobalInstall = $true
    } else {
        $Target = $a
    }
}

# Validate TARGET (only for project-scoped updates)
if (-not $GlobalInstall) {
    if ([System.IO.Path]::IsPathRooted($Target)) {
        [Console]::Error.WriteLine("error: Absolute paths are not allowed for TARGET; please use a relative path (e.g., .\my-project).")
        exit 1
    }

    if ($Target -cmatch '\.\.') {
        [Console]::Error.WriteLine("error: Parent directory references ('..') are not allowed in TARGET.")
        exit 1
    }
}

$PlansDir = Join-Path $Target "plans"
$BaseUrl  = "https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/$Version"

# --- Output helpers ---

function Write-Info  { param([string]$Msg) Write-Host "> " -ForegroundColor Green -NoNewline; Write-Host $Msg }
function Write-Warn  { param([string]$Msg) Write-Host "> " -ForegroundColor Yellow -NoNewline; Write-Host $Msg }
function Write-Err   { param([string]$Msg) Write-Host "error: " -ForegroundColor Red -NoNewline; Write-Host $Msg }
function Write-Step  { param([string]$Msg) Write-Host "==> " -ForegroundColor Blue -NoNewline; Write-Host $Msg -ForegroundColor White }

# CLI runtime file lists (mirrors V2_CLI_FILES in lib/scaffold.sh plus the
# PowerShell twins the global channel also carries).
$CliFilesBash = @(
    "bin/aps"
    "lib/output.sh", "lib/lint.sh", "lib/orchestrate.sh", "lib/audit.sh", "lib/export.sh"
    "lib/scaffold.sh"
    "lib/rules/common.sh", "lib/rules/module.sh", "lib/rules/index.sh"
    "lib/rules/workitem.sh", "lib/rules/issues.sh", "lib/rules/design.sh"
)
$CliFilesPowerShell = @(
    "bin/aps.ps1"
    "lib/Output.psm1", "lib/Lint.psm1", "lib/Scaffold.psm1"
    "lib/rules/Common.psm1", "lib/rules/Module.psm1", "lib/rules/Index.psm1"
    "lib/rules/WorkItem.psm1", "lib/rules/Issues.psm1", "lib/rules/Design.psm1"
)

# Hook script basenames (both shells) as shipped into .aps/scripts/.
$HookScriptNames = @(
    "install-hooks", "init-session", "check-complete"
    "pre-tool-check", "post-tool-nudge", "enforce-plan-update"
)

# --- Download helper ---

function Invoke-Download {
    <#
    .SYNOPSIS
        Download a repo-root-relative file from GitHub, or copy it from a
        local checkout when APS_LOCAL is set (parity with bash download()
        and lib/Scaffold.psm1 Invoke-ApsDownload).
    #>
    param(
        [string]$Path,
        [string]$Destination
    )
    $dir = Split-Path $Destination
    if ($dir) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
    if ($env:APS_LOCAL) {
        $localPath = Join-Path $env:APS_LOCAL $Path
        if (Test-Path -LiteralPath $localPath) {
            Copy-Item -LiteralPath $localPath -Destination $Destination -Force
            return
        }
        Write-Err "Local file not found: $localPath"
        exit 1
    }
    $url = "$BaseUrl/$Path"
    try {
        Invoke-WebRequest -Uri $url -OutFile $Destination -UseBasicParsing -ErrorAction Stop
    } catch {
        Write-Err "Failed to download '$Path' from $url"
        [Console]::Error.WriteLine("       Please check your network connectivity and ensure APS_VERSION='$Version' is correct.")
        exit 1
    }
}

# --- Check if APS hooks are configured (either Claude settings file) ---

function Test-ApsHooks {
    $claudeDir = Join-Path $Target ".claude"
    foreach ($name in @("settings.local.json", "settings.json")) {
        $settings = Join-Path $claudeDir $name
        if (-not (Test-Path -LiteralPath $settings)) { continue }
        $content = Get-Content -LiteralPath $settings -Raw -ErrorAction SilentlyContinue
        if (-not $content) { continue }
        if ($content -cmatch 'aps-planning/scripts' -or $content -cmatch 'aps-planning\\scripts' -or
            $content -cmatch '\.aps/scripts' -or $content -cmatch '\[APS\]') {
            return $true
        }
    }
    return $false
}

# --- Layout detection (mirrors is_v1_layout / is_v2_layout in lib/scaffold.sh) ---

function Test-V2Layout {
    return (Test-Path -LiteralPath (Join-Path (Join-Path $Target ".aps") "config.yml") -PathType Leaf)
}

function Test-V1Layout {
    # Same five marker classes as bash is_v1_layout; bash/pwsh CLI file
    # variants count once per class.
    $markers = 0
    if ((Test-Path -LiteralPath (Join-Path $Target "bin/aps") -PathType Leaf) -or
        (Test-Path -LiteralPath (Join-Path $Target "bin/aps.ps1") -PathType Leaf))             { $markers++ }
    if (Test-Path -LiteralPath (Join-Path $Target "aps-planning") -PathType Container)         { $markers++ }
    if (Test-Path -LiteralPath (Join-Path $Target ".claude/commands/plan.md") -PathType Leaf)  { $markers++ }
    if ((Test-Path -LiteralPath (Join-Path $Target "lib/output.sh") -PathType Leaf) -or
        (Test-Path -LiteralPath (Join-Path $Target "lib/Output.psm1") -PathType Leaf))         { $markers++ }
    if (Test-Path -LiteralPath (Join-Path $Target "plans/aps-rules.md") -PathType Leaf)        { $markers++ }
    return ($markers -ge 2)
}

# --- Global update: CLI only ---

function Update-ApsGlobal {
    $ApsHome = if ($env:APS_HOME) { $env:APS_HOME } else { Join-Path $HOME ".aps" }
    $binDir = Join-Path $ApsHome "bin"

    if (-not (Test-Path -LiteralPath $binDir -PathType Container)) {
        Write-Err "No global APS installation found at $ApsHome"
        Write-Host ""
        Write-Host "To install globally:"
        Write-Host '  irm https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install.ps1 | iex -- --global'
        Write-Host ""
        exit 1
    }

    Write-Host ""
    Write-Host "Anvil Plan Spec (APS) Global Update" -ForegroundColor White
    Write-Host ""

    Write-Step "Updating APS CLI at $ApsHome"

    foreach ($f in ($CliFilesBash + $CliFilesPowerShell)) {
        Invoke-Download -Path $f -Destination (Join-Path $ApsHome $f)
    }

    Write-Host ""
    Write-Step "Global update complete"
    Write-Info "bin/aps + bin/aps.ps1 + lib/ updated at $ApsHome"
    Write-Host ""
}

# --- Branch: global update exits early ---

if ($GlobalInstall) {
    Update-ApsGlobal
    exit 0
}

# --- Delegate resolution ---
#
# The updater does not embed the v2 refresh or the managed-marker logic; it
# runs `aps update`, which owns both (Update-ApsV2 in lib/Scaffold.psm1).

$script:BootstrapDir = $null
$script:ApsCmd = $null       # command name (native binary) or path to aps.ps1
$script:ApsCmdIsScript = $false
$script:ApsCmdDesc = ""
$script:PsExe = (Get-Process -Id $PID).Path

function Invoke-CliBootstrap {
    <#
    .SYNOPSIS
        Fetch the pinned PowerShell CLI into a temp dir and delegate to it.
    #>
    $script:BootstrapDir = Join-Path ([System.IO.Path]::GetTempPath()) ("aps-update-cli-" + [System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Path $script:BootstrapDir -Force | Out-Null
    foreach ($f in $CliFilesPowerShell) {
        Invoke-Download -Path $f -Destination (Join-Path $script:BootstrapDir $f)
    }
    $script:ApsCmd = Join-Path (Join-Path $script:BootstrapDir "bin") "aps.ps1"
    $script:ApsCmdIsScript = $true
    $script:ApsCmdDesc = "PowerShell CLI fetched at ref '$Version'"
}

function Invoke-ApsDelegate {
    <#
    .SYNOPSIS
        Run the resolved aps CLI with the given arguments; returns the exit
        code. Scripts run in a child shell so their `exit` cannot end this
        updater.
    #>
    param([string[]]$DelegateArgs)
    # Out-Host keeps the delegate's output on screen instead of leaking it
    # into this function's return value.
    if ($script:ApsCmdIsScript) {
        & $script:PsExe -NoProfile -ExecutionPolicy Bypass -File $script:ApsCmd @DelegateArgs | Out-Host
    } else {
        & $script:ApsCmd @DelegateArgs | Out-Host
    }
    return $LASTEXITCODE
}

function Resolve-ApsCli {
    $native = Get-Command aps -ErrorAction SilentlyContinue
    if ($native -and $native.CommandType -eq 'Application') {
        $script:ApsCmd = $native.Source
        $script:ApsCmdIsScript = $false
        try {
            & $script:ApsCmd update --help *> $null
            if ($LASTEXITCODE -eq 0) {
                $script:ApsCmdDesc = "installed aps CLI ($($native.Source))"
                return
            }
        } catch { }
        $script:ApsCmd = $null
    }
    $vendored = Join-Path $Target (Join-Path ".aps" (Join-Path "bin" "aps.ps1"))
    if (Test-Path -LiteralPath $vendored -PathType Leaf) {
        try {
            & $script:PsExe -NoProfile -ExecutionPolicy Bypass -File $vendored update --help *> $null
            if ($LASTEXITCODE -eq 0) {
                $script:ApsCmd = $vendored
                $script:ApsCmdIsScript = $true
                $script:ApsCmdDesc = "vendored CLI (.aps/bin/aps.ps1)"
                return
            }
        } catch { }
    }
    Invoke-CliBootstrap
}

# --- Minimal project config (mirrors write_min_config in scaffold/install) ---

function Write-MinConfig {
    param([string]$Tool)
    $today = Get-Date -Format "yyyy-MM-dd"
    # $Version is a git ref ("main") by default; only use it as the contract
    # pin when it is an explicit semver. Otherwise fall back to the release.
    $cliVersion = if ($Version -cmatch '^v?[0-9]') { $Version -creplace '^v', '' } else { "0.7.0" }
    $apsDir = Join-Path $Target ".aps"
    New-Item -ItemType Directory -Path $apsDir -Force | Out-Null
    $gitignore = Join-Path $apsDir ".gitignore"
    if (-not (Test-Path -LiteralPath $gitignore)) {
        Set-Content -LiteralPath $gitignore -Value ""
    }
    $ignoreLines = @(Get-Content -LiteralPath $gitignore -ErrorAction SilentlyContinue)
    if ($ignoreLines -cnotcontains 'context/') {
        Add-Content -LiteralPath $gitignore -Value 'context/'
    }
    $lines = @(
        "# .aps/config.yml — written by updater (v1 migration), read by updater"
        ""
        "# Project contract (INSTALL-014 / D-035): toolchain pin + runtime path"
        "# defaults the global 'aps' binary discovers by walking up from cwd."
        "cli_version: `"$cliVersion`""
        "plans_dir: plans/"
        "docs_dir: docs/"
        "tooling_root: .aps/"
        ""
        "aps:"
        "  version: `"0.7.0`""
        "  config_schema: 1"
        "  installed: `"$today`""
        "  updated: `"$today`""
        ""
        "project:"
        "  type: simple"
        "  monorepo_tool: ~"
        "  profile: solo"
        ""
        "tools:"
        "  - name: $Tool"
    )
    if ($Tool -ceq "claude-code") {
        $lines += @(
            "    skill: .claude/skills/aps-planning"
            "    hooks: full"
            "    agents:"
            "      - aps-planner"
            "      - aps-librarian"
            "      - aps-conductor"
        )
    } else {
        $lines += "    # No tool integration — run 'aps setup' to add one"
    }
    Set-Content -LiteralPath (Join-Path $apsDir "config.yml") -Value ($lines -join "`n")
}

# --- Small filesystem helpers ---

function Remove-DirIfEmpty {
    param([string]$Path)
    if ((Test-Path -LiteralPath $Path -PathType Container) -and
        -not (Get-ChildItem -LiteralPath $Path -Force | Select-Object -First 1)) {
        Remove-Item -LiteralPath $Path -Force
        return $true
    }
    return (-not (Test-Path -LiteralPath $Path))
}

# --- v1 -> v2 migration (minimum footprint; deep cleanup is `aps upgrade`) ---
#
# D-033 safety: generated legacy trees are backed up to .aps/backup/<ts>/
# before removal; anything ambiguous is left in place with a warning.

$script:MigrationBackup = ""

function Invoke-V1Migration {
    $ts = Get-Date -Format "yyyyMMdd-HHmmss"
    $backup = Join-Path (Join-Path (Join-Path $Target ".aps") "backup") $ts
    $script:MigrationBackup = ".aps/backup/$ts"

    Write-Step "Migrating v1 layout to the current layout"
    New-Item -ItemType Directory -Path $backup -Force | Out-Null

    # Legacy skill dir: remove only when it is recognisably APS-generated.
    $legacySkill = Join-Path $Target "aps-planning"
    if (Test-Path -LiteralPath $legacySkill -PathType Container) {
        $skillMd = Join-Path $legacySkill "SKILL.md"
        $hooksMd = Join-Path $legacySkill "hooks.md"
        if ((Test-Path -LiteralPath $skillMd) -or (Test-Path -LiteralPath $hooksMd)) {
            Copy-Item -LiteralPath $legacySkill -Destination (Join-Path $backup "aps-planning") -Recurse -Force
            Remove-Item -LiteralPath $legacySkill -Recurse -Force
            Write-Info "Backed up + removed legacy aps-planning/ (fresh skill goes to the managed trees)"
        } else {
            Write-Warn "aps-planning/ has unrecognised contents — left untouched, review manually"
        }
    }

    # Legacy Claude commands (D-023: no shipped commands).
    $commandsDirLegacy = Join-Path (Join-Path $Target ".claude") "commands"
    $removedCmd = $false
    foreach ($cmdFile in @("plan.md", "plan-status.md")) {
        $src = Join-Path $commandsDirLegacy $cmdFile
        if (Test-Path -LiteralPath $src -PathType Leaf) {
            New-Item -ItemType Directory -Path (Join-Path $backup "commands") -Force | Out-Null
            Copy-Item -LiteralPath $src -Destination (Join-Path (Join-Path $backup "commands") $cmdFile) -Force
            Remove-Item -LiteralPath $src -Force
            $removedCmd = $true
        }
    }
    Remove-DirIfEmpty -Path $commandsDirLegacy | Out-Null
    if ($removedCmd) {
        Write-Info "Backed up + removed legacy Claude commands"
    }

    # Root vendored CLI (generated content; superseded by the global binary
    # or the vendored .aps/bin runtime).
    $rootBin = Join-Path $Target "bin"
    $rootBinAps = Join-Path $rootBin "aps"
    $rootBinApsPs1 = Join-Path $rootBin "aps.ps1"
    if ((Test-Path -LiteralPath $rootBinAps) -or (Test-Path -LiteralPath $rootBinApsPs1)) {
        Remove-Item -LiteralPath $rootBinAps -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $rootBinApsPs1 -Force -ErrorAction SilentlyContinue
        if (Remove-DirIfEmpty -Path $rootBin) {
            Write-Info "Removed legacy bin/"
        } else {
            Write-Warn "bin/ contains non-APS files — removed only bin/aps"
        }
    }
    $rootLib = Join-Path $Target "lib"
    $libIsAps = (Test-Path -LiteralPath (Join-Path $rootLib "output.sh")) -or
                (Test-Path -LiteralPath (Join-Path $rootLib "Output.psm1"))
    if ((Test-Path -LiteralPath $rootLib -PathType Container) -and $libIsAps) {
        foreach ($f in ($CliFilesBash + $CliFilesPowerShell)) {
            if ($f -clike "lib/*") {
                Remove-Item -LiteralPath (Join-Path $Target $f) -Force -ErrorAction SilentlyContinue
            }
        }
        Remove-DirIfEmpty -Path (Join-Path $rootLib "rules") | Out-Null
        if (Remove-DirIfEmpty -Path $rootLib) {
            Write-Info "Removed legacy lib/"
        } else {
            Write-Warn "lib/ contains non-APS files — removed only APS files"
        }
    }

    # aps-rules.md is APS-managed and will be replaced with the v2 rules.
    $rules = Join-Path $PlansDir "aps-rules.md"
    if (Test-Path -LiteralPath $rules -PathType Leaf) {
        Copy-Item -LiteralPath $rules -Destination (Join-Path $backup "aps-rules.md") -Force
        Write-Info "Backed up plans/aps-rules.md (replaced with the v2 rules)"
    }

    # Hooks: keep them working only when the project had them wired up.
    if ($script:HooksConfigured) {
        $settings = Join-Path (Join-Path $Target ".claude") "settings.local.json"
        if (Test-Path -LiteralPath $settings -PathType Leaf) {
            $content = Get-Content -LiteralPath $settings -Raw
            if ($content -cmatch 'aps-planning/scripts' -or $content -cmatch 'aps-planning\\scripts') {
                $content = $content.Replace('aps-planning/scripts', '.aps/scripts')
                $content = $content.Replace('aps-planning\scripts', '.aps\scripts')
                Set-Content -LiteralPath $settings -Value $content -NoNewline
                Write-Info "Rewrote hook paths (aps-planning/scripts -> .aps/scripts) in settings.local.json"
            }
        }
        New-Item -ItemType Directory -Path (Join-Path (Join-Path $Target ".aps") "scripts") -Force | Out-Null
    }

    # Project contract. v1 installs were Claude Code integrations (skill +
    # commands), so the migrated config keeps that tool enabled.
    Write-MinConfig -Tool "claude-code"
    Write-Info "Created .aps/config.yml (claude-code)"
}

# --- Post-delegate pruning ---
#
# `aps update` may refresh the vendored CLI and hook scripts a project never
# had (the bash CLI does so unconditionally); this project only keeps what
# it already had (or what the v1 migration carried over).

function Clear-UnrequestedScripts {
    $d = Join-Path (Join-Path $Target ".aps") "scripts"
    if (-not (Test-Path -LiteralPath $d -PathType Container)) { return }
    foreach ($name in $HookScriptNames) {
        Remove-Item -LiteralPath (Join-Path $d "$name.sh") -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath (Join-Path $d "$name.ps1") -Force -ErrorAction SilentlyContinue
    }
    if (-not (Remove-DirIfEmpty -Path $d)) {
        Write-Warn ".aps/scripts contains non-APS files — left in place"
    }
}

function Clear-UnrequestedVendoredCli {
    $apsDir = Join-Path $Target ".aps"
    $binDir = Join-Path $apsDir "bin"
    Remove-Item -LiteralPath (Join-Path $binDir "aps") -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath (Join-Path $binDir "aps.ps1") -Force -ErrorAction SilentlyContinue
    Remove-DirIfEmpty -Path $binDir | Out-Null
    foreach ($f in ($CliFilesBash + $CliFilesPowerShell)) {
        if ($f -clike "lib/*") {
            Remove-Item -LiteralPath (Join-Path $apsDir $f) -Force -ErrorAction SilentlyContinue
        }
    }
    Remove-DirIfEmpty -Path (Join-Path (Join-Path $apsDir "lib") "rules") | Out-Null
    Remove-DirIfEmpty -Path (Join-Path $apsDir "lib") | Out-Null
}

function Invoke-ProjectUpdate {
    Write-Step "Refreshing project via $script:ApsCmdDesc"
    $code = Invoke-ApsDelegate -DelegateArgs @("update", $Target)
    if ($code -ne 0) {
        Write-Err "aps update failed"
        exit 1
    }
    if (-not $script:KeepScripts)     { Clear-UnrequestedScripts }
    if (-not $script:KeepVendoredCli) { Clear-UnrequestedVendoredCli }
}

function Test-MarkersMissing {
    <#
    .SYNOPSIS
        True when a skill tree this run created is missing its managed
        marker — the sign of a pre-D-042 delegate CLI. Pre-existing trees
        are exempt (the reconcile may legitimately leave user-owned trees
        unmarked).
    #>
    foreach ($tree in @(".claude/skills/aps-planning", ".agents/skills/aps-planning")) {
        if ($script:PreExistingTrees -ccontains $tree) { continue }
        $skill = Join-Path $Target (Join-Path $tree "SKILL.md")
        $marker = Join-Path $Target (Join-Path $tree ".aps-managed.json")
        if ((Test-Path -LiteralPath $skill -PathType Leaf) -and
            -not (Test-Path -LiteralPath $marker -PathType Leaf)) {
            return $true
        }
    }
    return $false
}

# --- Project update ---

Write-Host ""
Write-Host "Anvil Plan Spec (APS) Update" -ForegroundColor White
Write-Host ""

# Check for existing installation
if (-not (Test-Path -LiteralPath $PlansDir -PathType Container)) {
    Write-Err "No plans/ directory found at $Target"
    Write-Host ""
    Write-Host "To install APS in a new project:"
    Write-Host '  Invoke-Expression (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install.ps1" -UseBasicParsing).Content'
    Write-Host ""
    exit 1
}

# Ensure subdirectories exist (in case of older installations)
New-Item -ItemType Directory -Path (Join-Path $PlansDir "modules") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $PlansDir "execution") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $PlansDir "decisions") -Force | Out-Null

# Pre-state, recorded before migration can create any of it.
$script:HooksConfigured = Test-ApsHooks
$apsScriptsDir = Join-Path (Join-Path $Target ".aps") "scripts"
$HadScripts = Test-Path -LiteralPath $apsScriptsDir -PathType Container
$HadVendoredCli = (Test-Path -LiteralPath (Join-Path $Target ".aps/bin/aps") -PathType Leaf) -or
                  (Test-Path -LiteralPath (Join-Path $Target ".aps/bin/aps.ps1") -PathType Leaf)
$script:PreExistingTrees = @()
foreach ($tree in @(".claude/skills/aps-planning", ".agents/skills/aps-planning")) {
    if (Test-Path -LiteralPath (Join-Path $Target (Join-Path $tree "SKILL.md")) -PathType Leaf) {
        $script:PreExistingTrees += $tree
    }
}

$Mode = "v2"
if (Test-V2Layout) {
    $Mode = "v2"
} elseif (Test-V1Layout) {
    $Mode = "v1"
} else {
    $Mode = "plans-only"
}

$script:KeepScripts = ($HadScripts -or $script:HooksConfigured)
$script:KeepVendoredCli = $HadVendoredCli
if ($Mode -ceq "v1" -and
    ((Test-Path -LiteralPath (Join-Path $Target "bin/aps") -PathType Leaf) -or
     (Test-Path -LiteralPath (Join-Path $Target "bin/aps.ps1") -PathType Leaf))) {
    # A v1 project vendored its CLI at the root; keep it vendored at .aps/bin.
    $script:KeepVendoredCli = $true
}

switch ($Mode) {
    "v1" {
        Invoke-V1Migration
    }
    "plans-only" {
        Write-Step "Adopting the current layout (plans/ found, no .aps/config.yml)"
        Write-MinConfig -Tool "generic"
        Write-Info "Created .aps/config.yml (generic — run 'aps setup' to add a tool)"
    }
}

# Forward the version pin to the delegate CLI.
$prevApsVersion = $env:APS_VERSION
$env:APS_VERSION = $Version
try {
    Resolve-ApsCli
    Invoke-ProjectUpdate

    # A delegate CLI that predates managed markers (D-042) would have
    # installed skill trees without them. Never leave that silent: redo the
    # refresh with the pinned CLI, which is marker-capable.
    if (Test-MarkersMissing) {
        if (-not $script:BootstrapDir) {
            Write-Warn "$script:ApsCmdDesc did not write managed skill markers — retrying with the pinned CLI"
            Invoke-CliBootstrap
            Invoke-ProjectUpdate
        }
        if (Test-MarkersMissing) {
            Write-Err "managed skill markers are still missing after the update"
            [Console]::Error.WriteLine("       Update your installed aps CLI and re-run:")
            [Console]::Error.WriteLine("       irm $BaseUrl/scaffold/install.ps1 | iex -- --cli")
            exit 1
        }
    }
} finally {
    if ($null -eq $prevApsVersion) {
        Remove-Item Env:APS_VERSION -ErrorAction SilentlyContinue
    } else {
        $env:APS_VERSION = $prevApsVersion
    }
    if ($script:BootstrapDir -and (Test-Path -LiteralPath $script:BootstrapDir)) {
        Remove-Item -LiteralPath $script:BootstrapDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# --- Success ---

Write-Host ""
Write-Step "Update complete"
Write-Host ""
Write-Host "  Updated:"
Write-Host "    - plans/ templates and aps-rules.md (your specs untouched)"
Write-Host "    - managed skill trees per .aps/config.yml (with .aps-managed.json markers)"
if ($script:KeepScripts) {
    Write-Host "    - .aps/scripts/ (hook scripts)"
}
if ($script:KeepVendoredCli) {
    Write-Host "    - .aps/bin + .aps/lib (vendored CLI)"
}
if ($Mode -ceq "v1") {
    Write-Host ""
    Write-Warn "v1 layout migrated — legacy files were backed up to $script:MigrationBackup/"
    Write-Host "    Review .aps/config.yml and adjust tools/profile if needed."
}
Write-Host ""
Write-Warn "Your specs were preserved:"
Write-Host "    - index.aps.md"
Write-Host "    - modules/*.aps.md"
Write-Host "    - execution/*.actions.md"
Write-Host ""
if (-not $script:KeepScripts) {
    Write-Info "Hooks are not installed. Add them any time with: aps setup hooks"
    Write-Host ""
}
