#
# APS Update Script (PowerShell)
# Updates templates, rules, and skill in an existing APS project.
# Your specs (index.aps.md, modules/*.aps.md) are preserved.
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

# --- Download helpers ---

function Invoke-Download {
    <#
    .SYNOPSIS
        Download a scaffold file from GitHub (prefixed under scaffold/).
    #>
    param(
        [string]$Path,
        [string]$Destination
    )
    $url = "$BaseUrl/scaffold/$Path"
    $dir = Split-Path $Destination
    if ($dir) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
    try {
        Invoke-WebRequest -Uri $url -OutFile $Destination -UseBasicParsing -ErrorAction Stop
    } catch {
        Write-Err "Failed to download '$Path' from $url"
        Write-Host "       Please check your network connectivity and ensure APS_VERSION='$Version' is correct."
        exit 1
    }
}

function Invoke-DownloadRoot {
    <#
    .SYNOPSIS
        Download a file from the repo root (no scaffold/ prefix).
    #>
    param(
        [string]$Path,
        [string]$Destination
    )
    $url = "$BaseUrl/$Path"
    $dir = Split-Path $Destination
    if ($dir) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
    try {
        Invoke-WebRequest -Uri $url -OutFile $Destination -UseBasicParsing -ErrorAction Stop
    } catch {
        Write-Err "Failed to download '$Path' from $url"
        Write-Host "       Please check your network connectivity and ensure APS_VERSION='$Version' is correct."
        exit 1
    }
}

# --- Interactive prompt ---

function Request-YesNo {
    <#
    .SYNOPSIS
        Prompt user with a yes/no question. Returns $true for yes, $false for no.
        Non-interactive sessions default to the provided default.
    #>
    param(
        [string]$Prompt,
        [string]$Default = "n"
    )
    $isInteractive = [Environment]::UserInteractive -and -not [Console]::IsInputRedirected
    if ($isInteractive) {
        $ynHint = if ($Default -ceq "y") { "Y/n" } else { "y/N" }
        Write-Host "$Prompt [$ynHint] " -NoNewline
        $answer = Read-Host
        if ([string]::IsNullOrWhiteSpace($answer)) { $answer = $Default }
        return ($answer -cmatch '^[Yy]')
    } else {
        return ($Default -ceq "y")
    }
}

# --- Check if APS hooks are already configured ---

function Test-ApsHooks {
    <#
    .SYNOPSIS
        Returns $true if APS hooks are already configured in settings.
    #>
    $settings = Join-Path (Join-Path $Target ".claude") "settings.local.json"
    if (-not (Test-Path -LiteralPath $settings)) { return $false }
    $content = Get-Content -LiteralPath $settings -Raw -ErrorAction SilentlyContinue
    if (-not $content) { return $false }
    return ($content -cmatch 'aps-planning/scripts|aps-planning\\scripts|\[APS\]')
}

# --- Global update function ---

function Update-ApsGlobal {
    <#
    .SYNOPSIS
        Update a global APS CLI installation (bin/ + lib/ only).
    #>
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

    $cliAll = @(
        "bin/aps", "bin/aps.ps1",
        "lib/output.sh", "lib/Output.psm1",
        "lib/lint.sh", "lib/Lint.psm1",
        "lib/scaffold.sh", "lib/Scaffold.psm1",
        "lib/rules/common.sh", "lib/rules/Common.psm1",
        "lib/rules/module.sh", "lib/rules/Module.psm1",
        "lib/rules/index.sh", "lib/rules/Index.psm1",
        "lib/rules/workitem.sh", "lib/rules/WorkItem.psm1",
        "lib/rules/issues.sh", "lib/rules/Issues.psm1"
    )

    foreach ($f in $cliAll) {
        Invoke-DownloadRoot -Path $f -Destination (Join-Path $ApsHome $f)
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

# --- Header ---

Write-Host ""
Write-Host "Anvil Plan Spec (APS) Update" -ForegroundColor White
Write-Host ""

# --- Check for existing installation ---

if (-not (Test-Path -LiteralPath $PlansDir -PathType Container)) {
    Write-Err "No plans/ directory found at $Target"
    Write-Host ""
    Write-Host "To install APS in a new project:"
    Write-Host '  Invoke-Expression (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install.ps1" -UseBasicParsing).Content'
    Write-Host ""
    exit 1
}

# --- Ensure subdirectories exist (in case of older installations) ---

New-Item -ItemType Directory -Path (Join-Path $PlansDir "modules") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $PlansDir "execution") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $PlansDir "decisions") -Force | Out-Null

# --- Update CLI (bash + PowerShell) ---

Write-Step "Updating APS CLI"

$cliFilesBash = @(
    "bin/aps"
    "lib/output.sh"
    "lib/lint.sh"
    "lib/scaffold.sh"
    "lib/rules/common.sh"
    "lib/rules/module.sh"
    "lib/rules/index.sh"
    "lib/rules/workitem.sh"
    "lib/rules/issues.sh"
    "lib/rules/design.sh"
)

$cliFilesPowerShell = @(
    "bin/aps.ps1"
    "lib/Output.psm1"
    "lib/Lint.psm1"
    "lib/Scaffold.psm1"
    "lib/rules/Common.psm1"
    "lib/rules/Module.psm1"
    "lib/rules/Index.psm1"
    "lib/rules/WorkItem.psm1"
    "lib/rules/Issues.psm1"
    "lib/rules/Design.psm1"
)

foreach ($f in $cliFilesBash) {
    Invoke-DownloadRoot -Path $f -Destination (Join-Path $Target $f)
}
foreach ($f in $cliFilesPowerShell) {
    Invoke-DownloadRoot -Path $f -Destination (Join-Path $Target $f)
}

Write-Info "bin/aps + bin/aps.ps1 + lib/ (CLI)"

# --- Update templates and rules (NOT index.aps.md — user's plan is preserved) ---

Write-Step "Updating templates and rules"

Invoke-Download -Path "plans/aps-rules.md" -Destination (Join-Path $PlansDir "aps-rules.md")
Write-Info "aps-rules.md"

Invoke-Download -Path "plans/modules/.module.template.md" -Destination (Join-Path $PlansDir (Join-Path "modules" ".module.template.md"))
Write-Info "modules/.module.template.md"

Invoke-Download -Path "plans/modules/.simple.template.md" -Destination (Join-Path $PlansDir (Join-Path "modules" ".simple.template.md"))
Write-Info "modules/.simple.template.md"

Invoke-Download -Path "plans/modules/.index-monorepo.template.md" -Destination (Join-Path $PlansDir (Join-Path "modules" ".index-monorepo.template.md"))
Write-Info "modules/.index-monorepo.template.md"

Invoke-Download -Path "plans/execution/.actions.template.md" -Destination (Join-Path $PlansDir (Join-Path "execution" ".actions.template.md"))
Write-Info "execution/.actions.template.md"

# --- Update skill ---

Write-Step "Updating APS planning skill"

$SkillDir    = Join-Path $Target "aps-planning"
$CommandsDir = Join-Path (Join-Path $Target ".claude") "commands"

$skillFilesBash = @(
    "aps-planning/SKILL.md"
    "aps-planning/reference.md"
    "aps-planning/examples.md"
    "aps-planning/hooks.md"
    "aps-planning/scripts/install-hooks.sh"
    "aps-planning/scripts/init-session.sh"
    "aps-planning/scripts/check-complete.sh"
    "aps-planning/scripts/pre-tool-check.sh"
    "aps-planning/scripts/post-tool-nudge.sh"
    "aps-planning/scripts/enforce-plan-update.sh"
)

$skillFilesPowerShell = @(
    "aps-planning/scripts/install-hooks.ps1"
    "aps-planning/scripts/init-session.ps1"
    "aps-planning/scripts/check-complete.ps1"
    "aps-planning/scripts/pre-tool-check.ps1"
    "aps-planning/scripts/post-tool-nudge.ps1"
    "aps-planning/scripts/enforce-plan-update.ps1"
)

foreach ($f in $skillFilesBash) {
    Invoke-Download -Path $f -Destination (Join-Path $Target $f)
}
foreach ($f in $skillFilesPowerShell) {
    Invoke-Download -Path $f -Destination (Join-Path $Target $f)
}

Write-Info "aps-planning/ (skill, reference, examples, hooks, scripts)"

New-Item -ItemType Directory -Path $CommandsDir -Force | Out-Null
Invoke-Download -Path "commands/plan.md" -Destination (Join-Path $CommandsDir "plan.md")
Invoke-Download -Path "commands/plan-status.md" -Destination (Join-Path $CommandsDir "plan-status.md")

Write-Info ".claude/commands/ (plan, plan-status)"

# --- Success ---

Write-Host ""
Write-Step "Update complete"
Write-Host ""
Write-Host "  Updated:"
Write-Host "    - bin/aps + bin/aps.ps1 + lib/ (CLI)"
Write-Host "    - aps-rules.md (agent guidance)"
Write-Host "    - modules/.module.template.md"
Write-Host "    - modules/.simple.template.md"
Write-Host "    - modules/.index-monorepo.template.md"
Write-Host "    - execution/.actions.template.md"
Write-Host "    - aps-planning/ (skill + scripts)"
Write-Host "    - .claude/commands/ (plan, plan-status)"
Write-Host ""
Write-Warn "Your specs were preserved:"
Write-Host "    - index.aps.md"
Write-Host "    - modules/*.aps.md"
Write-Host "    - execution/*.actions.md"
Write-Host ""

# --- Interactive hook prompt (only if hooks not already configured) ---

if (-not (Test-ApsHooks)) {
    Write-Host ""
    if (Request-YesNo -Prompt "Install APS hooks into .claude/settings.local.json?" -Default "y") {
        Push-Location $Target
        try {
            & (Join-Path "aps-planning" (Join-Path "scripts" "install-hooks.ps1"))
        } finally {
            Pop-Location
        }
    } else {
        if (Request-YesNo -Prompt "Copy hook scripts for you to install/review later?" -Default "y") {
            Write-Info "Hook scripts are at: aps-planning/scripts/"
            Write-Host "  Run .\aps-planning\scripts\install-hooks.ps1 when ready"
            Write-Host "  See aps-planning\hooks.md for what each hook does"
        } else {
            Write-Info "Skipping hooks. You can install them later:"
            Write-Host "  .\aps-planning\scripts\install-hooks.ps1"
        }
    }
} else {
    Write-Warn "Hook configuration was NOT modified."
    Write-Host "    To update hooks: .\aps-planning\scripts\install-hooks.ps1"
}

Write-Host ""
