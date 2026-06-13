#
# APS Install Script (PowerShell)
# Creates APS structure in a new project with templates, skill, and commands.
#
# Usage:
#   Invoke-Expression (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install.ps1" -UseBasicParsing).Content
#   $env:APS_VERSION = "v0.2.0"; Invoke-Expression (Invoke-WebRequest -Uri "..." -UseBasicParsing).Content
#
# For updating existing projects, use the update script instead.
#

$ErrorActionPreference = "Stop"

$Version = if ($env:APS_VERSION) { $env:APS_VERSION } else { "main" }
$Target = "."
# Mode is chosen by a flag, or by the picker when none is given.
#   cli / init / agent / upgrade / setup (see usage)
$Mode = ""
$SetupTarget = ""

for ($i = 0; $i -lt $args.Count; $i++) {
    switch ($args[$i]) {
        { $_ -in "--cli", "--global", "-g" } { $Mode = "cli" }
        "--init"    { $Mode = "init" }
        "--agent"   { $Mode = "agent" }
        "--upgrade" { $Mode = "upgrade" }
        "--setup"   {
            $Mode = "setup"
            $SetupTarget = if ($i + 1 -lt $args.Count) { $args[$i + 1] } else { "" }
            if (-not $SetupTarget -or $SetupTarget -like "-*") {
                [Console]::Error.WriteLine("error: --setup requires a tool (e.g. --setup claude-code)")
                exit 1
            }
            $i++
        }
        default {
            if ($args[$i] -like "-*") {
                [Console]::Error.WriteLine("error: unknown option: $($args[$i])")
                exit 1
            }
            $Target = $args[$i]
        }
    }
}

# Validate TARGET (only for project-scoped modes; cli installs machine-wide)
if ($Mode -ne "cli") {
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

# --- Global install functions ---

function Set-ApsGlobalPath {
    <#
    .SYNOPSIS
        Add APS bin directory to user PATH (persistent via registry).
    #>
    param([string]$ApsHome)
    $binDir = Join-Path $ApsHome "bin"

    # Check if already on User PATH
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -and ($currentPath -split ';' | Where-Object { $_ -eq $binDir })) {
        Write-Info "$binDir is already on your PATH"
        return
    }

    if (Request-YesNo -Prompt "Add $binDir to your user PATH?" -Default "y") {
        if ($currentPath) {
            $newPath = "$binDir;$currentPath"
        } else {
            $newPath = $binDir
        }
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        $env:PATH = "$binDir;$env:PATH"

        Write-Info "Added to user PATH (persistent)"
        Write-Host "  Restart your terminal for the change to take effect in new sessions."
    } else {
        Write-Info "To add manually, run:"
        Write-Host "  [Environment]::SetEnvironmentVariable('PATH', '$binDir;' + [Environment]::GetEnvironmentVariable('PATH', 'User'), 'User')"
    }
}

function Install-ApsGlobal {
    <#
    .SYNOPSIS
        Install APS CLI globally (bin/ + lib/ only, no project scaffolding).
    #>
    $ApsHome = if ($env:APS_HOME) { $env:APS_HOME } else { Join-Path $HOME ".aps" }

    Write-Host ""
    Write-Host "Anvil Plan Spec (APS)" -ForegroundColor White
    Write-Host "Global CLI installation"
    Write-Host ""

    Write-Step "Installing APS CLI to $ApsHome"

    $cliAll = @(
        "bin/aps.ps1",
        "lib/Output.psm1",
        "lib/Lint.psm1",
        "lib/Scaffold.psm1",
        "lib/rules/Common.psm1",
        "lib/rules/Module.psm1",
        "lib/rules/Index.psm1",
        "lib/rules/WorkItem.psm1",
        "lib/rules/Issues.psm1",
        "lib/rules/Design.psm1"
    )

    foreach ($f in $cliAll) {
        Invoke-DownloadRoot -Path $f -Destination (Join-Path $ApsHome $f)
    }

    Write-Info "bin/aps.ps1 + lib/ installed to $ApsHome"

    Set-ApsGlobalPath -ApsHome $ApsHome

    Write-Host ""
    Write-Step "Global installation complete"
    Write-Host ""
    Write-Host "  $ApsHome\"
    Write-Host "  +-- bin\aps.ps1      <- CLI (PowerShell)"
    Write-Host "  +-- lib\             <- CLI libraries"
    Write-Host ""
    Write-Info "To create a new APS project:"
    Write-Host "  cd your-project; aps init"
    Write-Host ""
}

# --- Agent bootstrap: minimal planning layer + next steps ---

function Install-ApsAgent {
    Write-Host ""
    Write-Host "Anvil Plan Spec (APS)" -ForegroundColor White
    Write-Host "Agent bootstrap"
    Write-Host ""
    if (Test-Path -LiteralPath $PlansDir -PathType Container) {
        Write-Err "plans/ directory already exists at $Target"
        exit 1
    }
    Write-Step "Creating minimal planning layer"
    New-Item -ItemType Directory -Force -Path (Join-Path $PlansDir "modules") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $PlansDir "execution") | Out-Null
    Invoke-Download -Path "plans/index.aps.md" -Destination (Join-Path $PlansDir "index.aps.md")
    Invoke-Download -Path "plans/aps-rules.md" -Destination (Join-Path $PlansDir "aps-rules.md")
    Invoke-Download -Path "plans/project-context.md" -Destination (Join-Path $PlansDir "project-context.md")
    Write-Info "index.aps.md, aps-rules.md, project-context.md"
    Write-Step "Agent bootstrap complete"
    Write-Info "Read plans/aps-rules.md, then draft plans/index.aps.md. Run 'aps setup' to add tools."
}

# --- Upgrade: hand off to the update entrypoint (deep cleanup is INSTALL-013) ---

function Install-ApsUpgrade {
    Write-Host ""
    Write-Host "Anvil Plan Spec (APS)" -ForegroundColor White
    Write-Host "Upgrade existing project"
    Write-Host ""
    if (-not (Test-Path -LiteralPath $PlansDir -PathType Container)) {
        Write-Err "no plans/ directory at $Target — nothing to upgrade"
        exit 1
    }
    Write-Step "Refreshing templates and CLI via the update entrypoint"
    $u = "$BaseUrl/scaffold/update.ps1"
    Invoke-Expression ((Invoke-WebRequest -Uri $u -UseBasicParsing).Content)
}

# --- Setup: add one tool integration via aps setup ---

function Install-ApsSetup {
    param([string]$Tool)
    Write-Host ""
    Write-Host "Anvil Plan Spec (APS)" -ForegroundColor White
    Write-Host "Add integration: $Tool"
    Write-Host ""
    Write-Step "Run 'aps setup $Tool' once the CLI is installed (aps setup is the integration entrypoint)"
}

# --- Mode picker (no flag given) ---

function Select-ApsMode {
    Write-Host ""
    Write-Host "Anvil Plan Spec (APS)" -ForegroundColor White
    Write-Host "What would you like to do?"
    Write-Host ""
    Write-Host "  1) Install the APS CLI on this machine"
    Write-Host "  2) Initialize APS planning in this repository"
    Write-Host "  3) Initialize this repository for an AI agent"
    Write-Host "  4) Upgrade an existing APS project"
    Write-Host "  5) Add a tool integration"
    Write-Host ""
    $choice = Read-Host "Choose [1-5]"
    switch ($choice) {
        "1" { $script:Mode = "cli" }
        "2" { $script:Mode = "init" }
        "3" { $script:Mode = "agent" }
        "4" { $script:Mode = "upgrade" }
        "5" {
            $script:Mode = "setup"
            $script:SetupTarget = Read-Host "Tool (claude-code, copilot, codex, opencode, gemini)"
            if (-not $script:SetupTarget) { Write-Err "no tool given"; exit 1 }
        }
        default { Write-Err "invalid choice: $choice"; exit 1 }
    }
}

# --- Default project scaffold (init mode) ---

function Install-ApsInit {

# --- Header ---

Write-Host ""
Write-Host "Anvil Plan Spec (APS)" -ForegroundColor White
Write-Host "Lightweight specs for AI-assisted development"
Write-Host ""

# --- Check for existing installation ---

if (Test-Path -LiteralPath $PlansDir -PathType Container) {
    Write-Err "plans/ directory already exists at $Target"
    Write-Host ""
    Write-Host "To update templates in an existing project:"
    Write-Host '  Invoke-Expression (Invoke-WebRequest -Uri "https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/update.ps1" -UseBasicParsing).Content'
    Write-Host ""
    Write-Host "To reinstall from scratch:"
    Write-Host "  Remove-Item -Recurse -Force $PlansDir; <then re-run this script>"
    Write-Host ""
    exit 1
}

# --- Install CLI (PowerShell) ---

Write-Step "Installing APS CLI"

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

foreach ($f in $cliFilesPowerShell) {
    Invoke-DownloadRoot -Path $f -Destination (Join-Path $Target $f)
}

Write-Info "bin/aps.ps1 + lib/ (CLI)"

# --- Create directory structure ---

Write-Step "Creating directory structure"
New-Item -ItemType Directory -Path (Join-Path $PlansDir "modules") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $PlansDir "execution") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $PlansDir "decisions") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $Target "designs") -Force | Out-Null

# --- Download templates ---

Write-Step "Downloading templates"

Invoke-Download -Path "plans/aps-rules.md" -Destination (Join-Path $PlansDir "aps-rules.md")
Write-Info "aps-rules.md"

Invoke-Download -Path "plans/index.aps.md" -Destination (Join-Path $PlansDir "index.aps.md")
Write-Info "index.aps.md"

Invoke-Download -Path "plans/modules/.module.template.md" -Destination (Join-Path $PlansDir (Join-Path "modules" ".module.template.md"))
Write-Info "modules/.module.template.md"

Invoke-Download -Path "plans/modules/.simple.template.md" -Destination (Join-Path $PlansDir (Join-Path "modules" ".simple.template.md"))
Write-Info "modules/.simple.template.md"

Invoke-Download -Path "plans/modules/.index-monorepo.template.md" -Destination (Join-Path $PlansDir (Join-Path "modules" ".index-monorepo.template.md"))
Write-Info "modules/.index-monorepo.template.md"

Invoke-Download -Path "plans/execution/.actions.template.md" -Destination (Join-Path $PlansDir (Join-Path "execution" ".actions.template.md"))
Write-Info "execution/.actions.template.md"

Invoke-Download -Path "designs/.design.template.md" -Destination (Join-Path $Target (Join-Path "designs" ".design.template.md"))
Write-Info "designs/.design.template.md"

$gitkeep = Join-Path $PlansDir (Join-Path "decisions" ".gitkeep")
New-Item -ItemType File -Path $gitkeep -Force | Out-Null

# --- Install skill ---

Write-Step "Installing APS planning skill"

$SkillDir    = Join-Path $Target "aps-planning"
$CommandsDir = Join-Path (Join-Path $Target ".claude") "commands"

$skillFilesPowerShell = @(
    "aps-planning/SKILL.md"
    "aps-planning/reference.md"
    "aps-planning/examples.md"
    "aps-planning/hooks.md"
    "aps-planning/scripts/install-hooks.ps1"
    "aps-planning/scripts/init-session.ps1"
    "aps-planning/scripts/check-complete.ps1"
    "aps-planning/scripts/pre-tool-check.ps1"
    "aps-planning/scripts/post-tool-nudge.ps1"
    "aps-planning/scripts/enforce-plan-update.ps1"
)

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
Write-Step "Installation complete"
Write-Host ""
Write-Host "  bin/"
Write-Host "  +-- aps.ps1                          <- CLI (PowerShell)"
Write-Host ""
Write-Host "  designs/"
Write-Host "  +-- .design.template.md              <- Template for technical designs"
Write-Host ""
Write-Host "  plans/"
Write-Host "  +-- aps-rules.md              # Agent guidance"
Write-Host "  +-- index.aps.md              # Your main plan"
Write-Host "  +-- modules/"
Write-Host "  |   +-- .module.template.md           # Module template"
Write-Host "  |   +-- .simple.template.md           # Simple feature template"
Write-Host "  |   +-- .index-monorepo.template.md   # Index for monorepos"
Write-Host "  +-- execution/"
Write-Host "  |   +-- .actions.template.md  # Action plan template"
Write-Host "  +-- decisions/"
Write-Host ""
Write-Host "  aps-planning/"
Write-Host "  +-- SKILL.md                  # Planning skill (core rules)"
Write-Host "  +-- reference.md              # APS format reference"
Write-Host "  +-- examples.md               # Real-world examples"
Write-Host "  +-- hooks.md                  # Hook configuration guide"
Write-Host "  +-- scripts/                  # Hook install + session scripts"
Write-Host ""
Write-Host "  .claude/commands/"
Write-Host "  +-- plan.md                   # legacy Claude command"
Write-Host "  +-- plan-status.md            # legacy Claude command"
Write-Host ""

# --- Interactive hook prompt ---

if (Request-YesNo -Prompt "Install APS hooks into .claude/settings.local.json?" -Default "y") {
    $hookScript = Join-Path (Join-Path (Join-Path $Target "aps-planning") "scripts") "install-hooks.ps1"
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

# --- PATH setup ---

Write-Host ""
$hasDirenv = Get-Command direnv -ErrorAction SilentlyContinue
if ($hasDirenv) {
    $envrc = Join-Path $Target ".envrc"
    if ((Test-Path -LiteralPath $envrc) -and ((Get-Content -LiteralPath $envrc -Raw -ErrorAction SilentlyContinue) -cmatch 'PATH_add bin')) {
        Write-Info "PATH already configured in .envrc"
    } elseif (Request-YesNo -Prompt "Set up direnv so you can run 'aps' without .\bin\ prefix?" -Default "y") {
        if (Test-Path -LiteralPath $envrc) {
            Add-Content -LiteralPath $envrc -Value 'PATH_add bin'
        } else {
            Set-Content -LiteralPath $envrc -Value 'PATH_add bin'
        }
        Write-Info "Added 'PATH_add bin' to .envrc"
        Write-Host "  Run 'direnv allow' to activate"
    } else {
        Write-Info "To run aps without the path prefix, add to your .envrc:"
        Write-Host "  PATH_add bin"
    }
} else {
    Write-Info "To run 'aps' without .\bin\ prefix, either:"
    Write-Host "  - Install direnv and add 'PATH_add bin' to .envrc"
    Write-Host "  - Or run: .\bin\aps.ps1"
}

# --- Next steps ---

Write-Host ""
Write-Step "Next steps"
Write-Host ""
Write-Host "  1. Edit " -NoNewline; Write-Host "plans\index.aps.md" -ForegroundColor White -NoNewline; Write-Host " to define your plan"
Write-Host "  2. Copy templates to create modules (remove the leading dot)"
Write-Host "  3. Point your AI agent at plans\aps-rules.md, or run aps next"
Write-Host ""
Write-Host "Docs: https://github.com/EddaCraft/anvil-plan-spec"
Write-Host ""
}

# --- Resolve mode and dispatch ---

if (-not $Mode) {
    if ([Environment]::UserInteractive) {
        Select-ApsMode
    } else {
        [Console]::Error.WriteLine("error: no mode given (use --cli/--init/--agent/--upgrade/--setup)")
        exit 1
    }
}

switch ($Mode) {
    "cli"     { Install-ApsGlobal }
    "init"    { Install-ApsInit }
    "agent"   { Install-ApsAgent }
    "upgrade" { Install-ApsUpgrade }
    "setup"   { Install-ApsSetup -Tool $SetupTarget }
    default   { Write-Err "unknown mode: $Mode"; exit 1 }
}
