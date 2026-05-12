#
# APS CLI Scaffold Module
# Port of lib/scaffold.sh — init and update workflows
#

# --- Configuration ---

$script:ApsVersion = if ($env:APS_VERSION) { $env:APS_VERSION } else { "main" }
$script:ApsBaseUrl = "https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/$script:ApsVersion"

# Files to download for plans/
$script:PlanFiles = @(
    "scaffold/plans/aps-rules.md"
    "scaffold/plans/modules/.module.template.md"
    "scaffold/plans/modules/.simple.template.md"
    "scaffold/plans/modules/.index-monorepo.template.md"
    "scaffold/plans/execution/.actions.template.md"
)

# Files to download for the planning skill
$script:SkillFiles = @(
    "scaffold/aps-planning/SKILL.md"
    "scaffold/aps-planning/reference.md"
    "scaffold/aps-planning/examples.md"
    "scaffold/aps-planning/hooks.md"
    "scaffold/aps-planning/scripts/install-hooks.sh"
    "scaffold/aps-planning/scripts/init-session.sh"
    "scaffold/aps-planning/scripts/check-complete.sh"
    "scaffold/aps-planning/scripts/pre-tool-check.sh"
    "scaffold/aps-planning/scripts/post-tool-nudge.sh"
    "scaffold/aps-planning/scripts/enforce-plan-update.sh"
    "scaffold/aps-planning/scripts/install-hooks.ps1"
    "scaffold/aps-planning/scripts/init-session.ps1"
    "scaffold/aps-planning/scripts/check-complete.ps1"
    "scaffold/aps-planning/scripts/pre-tool-check.ps1"
    "scaffold/aps-planning/scripts/post-tool-nudge.ps1"
    "scaffold/aps-planning/scripts/enforce-plan-update.ps1"
)

# Files to download for slash commands
$script:CommandFiles = @(
    "scaffold/commands/plan.md"
    "scaffold/commands/plan-status.md"
)

# CLI files — bash (bin/ and lib/)
$script:CliFilesBash = @(
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

# CLI files — PowerShell (bin/ and lib/)
$script:CliFilesPowerShell = @(
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

# --- Helpers ---

function Invoke-ApsDownload {
    <#
    .SYNOPSIS
        Download a scaffold file from GitHub (prefixed under scaffold/).
    #>
    param(
        [string]$Source,
        [string]$Destination
    )
    $url = "$script:ApsBaseUrl/$Source"
    $dir = Split-Path $Destination
    if ($dir) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
    try {
        Invoke-WebRequest -Uri $url -OutFile $Destination -UseBasicParsing -ErrorAction Stop
    } catch {
        Write-ApsError "Failed to download: $url"
        [Console]::Error.WriteLine("  Check your network and ensure APS_VERSION='$script:ApsVersion' is valid.")
        exit 1
    }
}

function Invoke-ApsDownloadRoot {
    <#
    .SYNOPSIS
        Download a file from the repo root (no scaffold/ prefix).
    #>
    param(
        [string]$Source,
        [string]$Destination
    )
    $url = "$script:ApsBaseUrl/$Source"
    $dir = Split-Path $Destination
    if ($dir) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
    try {
        Invoke-WebRequest -Uri $url -OutFile $Destination -UseBasicParsing -ErrorAction Stop
    } catch {
        Write-ApsError "Failed to download: $url"
        [Console]::Error.WriteLine("  Check your network and ensure APS_VERSION='$script:ApsVersion' is valid.")
        exit 1
    }
}

function Request-ApsYesNo {
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

function Test-ApsHooksConfigured {
    <#
    .SYNOPSIS
        Check if APS hooks are already configured in settings.local.json or settings.json.
    #>
    param(
        [string]$Target = "."
    )
    $claudeDir = Join-Path $Target ".claude"
    foreach ($name in @("settings.local.json", "settings.json")) {
        $settings = Join-Path $claudeDir $name
        if (-not (Test-Path -LiteralPath $settings)) { continue }
        $content = Get-Content -LiteralPath $settings -Raw -ErrorAction SilentlyContinue
        if (-not $content) { continue }
        if ($content -cmatch 'aps-planning/scripts' -or $content -cmatch '\.aps/scripts' -or $content -cmatch '\[APS\]') {
            return $true
        }
    }
    return $false
}

# --- Install functions ---

function Install-ApsPlans {
    <#
    .SYNOPSIS
        Download plan templates to the target directory.
    #>
    param([string]$Target)
    $plansDir = Join-Path $Target "plans"
    New-Item -ItemType Directory -Path (Join-Path $plansDir "modules") -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $plansDir "execution") -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $plansDir "decisions") -Force | Out-Null

    foreach ($f in $script:PlanFiles) {
        $rel = $f -creplace '^scaffold/plans/', ''
        Invoke-ApsDownload -Source $f -Destination (Join-Path $plansDir $rel)
    }
}

function Install-ApsIndex {
    <#
    .SYNOPSIS
        Download the index template (init only, not update).
    #>
    param([string]$Target)
    Invoke-ApsDownload -Source "scaffold/plans/index.aps.md" -Destination (Join-Path (Join-Path $Target "plans") "index.aps.md")
    $gitkeep = Join-Path (Join-Path (Join-Path $Target "plans") "decisions") ".gitkeep"
    if (-not (Test-Path -LiteralPath $gitkeep)) {
        New-Item -ItemType File -Path $gitkeep -Force | Out-Null
    }
}

function Install-ApsSkill {
    <#
    .SYNOPSIS
        Download skill files to the target directory.
    #>
    param([string]$Target)
    foreach ($f in $script:SkillFiles) {
        $rel = $f -creplace '^scaffold/', ''
        Invoke-ApsDownload -Source $f -Destination (Join-Path $Target $rel)
    }
}

function Install-ApsCommands {
    <#
    .SYNOPSIS
        Download slash commands to .claude/commands/.
    #>
    param([string]$Target)
    $commandsDir = Join-Path (Join-Path $Target ".claude") "commands"
    New-Item -ItemType Directory -Path $commandsDir -Force | Out-Null
    foreach ($f in $script:CommandFiles) {
        $rel = $f -creplace '^scaffold/commands/', ''
        Invoke-ApsDownload -Source $f -Destination (Join-Path $commandsDir $rel)
    }
}

function Install-ApsCli {
    <#
    .SYNOPSIS
        Download CLI files (both bash and PowerShell) to the target directory.
    #>
    param([string]$Target)
    foreach ($f in $script:CliFilesBash) {
        Invoke-ApsDownloadRoot -Source $f -Destination (Join-Path $Target $f)
    }
    foreach ($f in $script:CliFilesPowerShell) {
        Invoke-ApsDownloadRoot -Source $f -Destination (Join-Path $Target $f)
    }
}

function Install-ApsPath {
    <#
    .SYNOPSIS
        Set up PATH so `aps` works without ./bin/ prefix (direnv integration).
    #>
    param([string]$Target)
    Write-Host ""
    $hasDirenv = Get-Command direnv -ErrorAction SilentlyContinue
    if ($hasDirenv) {
        $envrc = Join-Path $Target ".envrc"
        if ((Test-Path -LiteralPath $envrc) -and ((Get-Content -LiteralPath $envrc -Raw -ErrorAction SilentlyContinue) -cmatch 'PATH_add bin')) {
            Write-ApsInfo "PATH already configured in .envrc"
        } elseif (Request-ApsYesNo -Prompt "Set up direnv so you can run 'aps' without ./bin/ prefix?" -Default "y") {
            if (Test-Path -LiteralPath $envrc) {
                Add-Content -LiteralPath $envrc -Value 'PATH_add bin'
            } else {
                Set-Content -LiteralPath $envrc -Value 'PATH_add bin'
            }
            Write-ApsInfo "Added 'PATH_add bin' to .envrc"
            Write-Host "  Run 'direnv allow' to activate"
        } else {
            Write-ApsInfo "To run aps without the path prefix, add to your .envrc:"
            Write-Host "  PATH_add bin"
        }
    } else {
        Write-ApsInfo "To run 'aps' without ./bin/ prefix, either:"
        Write-Host "  - Install direnv and add 'PATH_add bin' to .envrc"
        Write-Host "  - Or add 'export PATH=`"./bin:`$PATH`"' to your shell config"
    }
}

function Invoke-ApsHookPrompt {
    <#
    .SYNOPSIS
        Two-step hook installation prompt.
    #>
    param([string]$Target)
    Write-Host ""
    if (Request-ApsYesNo -Prompt "Install APS hooks into .claude/settings.local.json?" -Default "y") {
        Push-Location $Target
        try {
            & ./aps-planning/scripts/install-hooks.ps1
        } finally {
            Pop-Location
        }
    } else {
        if (Request-ApsYesNo -Prompt "Would you like me to copy them for you to install/review later?" -Default "y") {
            Write-ApsInfo "Hook scripts are at: aps-planning/scripts/"
            Write-Host "  Run .\aps-planning\scripts\install-hooks.ps1 when ready"
            Write-Host "  See aps-planning/hooks.md for what each hook does"
        } else {
            Write-ApsInfo "Skipping hooks. You can install them later:"
            Write-Host "  .\aps-planning\scripts\install-hooks.ps1"
        }
    }
}

# --- Subcommands ---

function Invoke-ApsInit {
    <#
    .SYNOPSIS
        Full init workflow — creates APS structure in a project.
    #>
    param([string[]]$Arguments)
    $target = "."
    if ($Arguments) {
        foreach ($arg in $Arguments) {
            switch ($arg) {
                "--help" { Show-ApsInitHelp; return }
                "-h"     { Show-ApsInitHelp; return }
                default  { $target = $arg }
            }
        }
    }

    $plansDir = Join-Path $target "plans"
    if (Test-Path -LiteralPath $plansDir -PathType Container) {
        Write-ApsError "plans/ directory already exists at $target"
        Write-Host ""
        Write-Host "To update an existing project:"
        Write-Host "  aps update"
        Write-Host ""
        Write-Host "To reinstall from scratch:"
        Write-Host "  rm -rf $plansDir && aps init"
        exit 1
    }

    Write-Host ""
    Write-ApsInfo "Initialising APS in $target"
    Write-Host ""

    # CLI (bin/aps + lib/)
    Install-ApsCli -Target $target
    Write-ApsInfo "bin/aps + lib/ (CLI)"

    # Templates and rules
    Install-ApsPlans -Target $target
    Install-ApsIndex -Target $target
    Write-ApsInfo "plans/ (templates, rules, index)"

    # Skill
    Install-ApsSkill -Target $target
    Write-ApsInfo "aps-planning/ (skill, reference, examples, hooks, scripts)"

    # Commands
    Install-ApsCommands -Target $target
    Write-ApsInfo ".claude/commands/ (plan, plan-status)"

    Write-Host ""
    Write-Host "  bin/"
    Write-Host "  +-- aps                              <- CLI (lint, init, update)"
    Write-Host ""
    Write-Host "  plans/"
    Write-Host "  +-- aps-rules.md                     <- Agent guidance (READ THIS)"
    Write-Host "  +-- index.aps.md                     <- Your main plan (edit this)"
    Write-Host "  +-- modules/"
    Write-Host "  |   +-- .module.template.md          <- Template for modules"
    Write-Host "  |   +-- .simple.template.md          <- Template for small features"
    Write-Host "  |   +-- .index-monorepo.template.md  <- Index for monorepos"
    Write-Host "  +-- execution/"
    Write-Host "  |   +-- .actions.template.md         <- Template for action plans"
    Write-Host "  +-- decisions/"
    Write-Host ""
    Write-Host "  aps-planning/"
    Write-Host "  +-- SKILL.md                         <- Planning skill (core rules)"
    Write-Host "  +-- reference.md                     <- APS format reference"
    Write-Host "  +-- examples.md                      <- Real-world examples"
    Write-Host "  +-- hooks.md                         <- Hook configuration guide"
    Write-Host "  +-- scripts/                         <- Hook install + session scripts"
    Write-Host ""
    Write-Host "  .claude/commands/"
    Write-Host "  +-- plan.md                          <- /plan command"
    Write-Host "  +-- plan-status.md                   <- /plan-status command"

    # Hooks
    Invoke-ApsHookPrompt -Target $target

    # PATH setup
    Install-ApsPath -Target $target

    Write-Host ""
    Write-ApsInfo "Next steps:"
    Write-Host "  1. Edit plans/index.aps.md to define your plan"
    Write-Host "  2. Copy templates to create modules (remove leading dot)"
    Write-Host "  3. Use /plan in Claude Code to start planning"
    Write-Host ""
}

function Show-ApsInitHelp {
    Write-Host @"
aps init - Create APS structure in a new project

Usage:
  aps init [target-dir]

Creates bin/aps CLI, plans/, aps-planning/ skill, .claude/commands/,
and optionally installs hooks and sets up PATH via direnv.

Refuses to run if plans/ already exists.

Options:
  --help    Show this help

Environment:
  APS_VERSION   Git ref to download from (default: main)

Examples:
  aps init              # Init in current directory
  aps init ./my-project # Init in a subdirectory
"@
}

function Invoke-ApsUpdate {
    <#
    .SYNOPSIS
        Full update workflow — updates APS templates, skill, CLI, and commands.
    #>
    param([string[]]$Arguments)
    $target = "."
    $globalUpdate = $false
    if ($Arguments) {
        foreach ($arg in $Arguments) {
            switch ($arg) {
                "--help"  { Show-ApsUpdateHelp; return }
                "-h"      { Show-ApsUpdateHelp; return }
                "--global" { $globalUpdate = $true }
                "-g"       { $globalUpdate = $true }
                default    { $target = $arg }
            }
        }
    }

    if ($globalUpdate) {
        Update-ApsGlobal
        return
    }

    $plansDir = Join-Path $target "plans"
    if (-not (Test-Path -LiteralPath $plansDir -PathType Container)) {
        Write-ApsError "No plans/ directory found at $target"
        Write-Host ""
        Write-Host "To create a new APS project:"
        Write-Host "  aps init"
        exit 1
    }

    Write-Host ""
    Write-ApsInfo "Updating APS in $target"
    Write-Host ""

    # CLI (always update -- this is how users get new features)
    Install-ApsCli -Target $target
    Write-ApsInfo "bin/aps + lib/ (CLI)"

    # Templates and rules (preserves user specs)
    Install-ApsPlans -Target $target
    Write-ApsInfo "plans/ (templates, rules)"

    # Skill
    Install-ApsSkill -Target $target
    Write-ApsInfo "aps-planning/ (skill, reference, examples, hooks, scripts)"

    # Commands
    Install-ApsCommands -Target $target
    Write-ApsInfo ".claude/commands/ (plan, plan-status)"

    # Hooks: prompt only if not already configured
    if (-not (Test-ApsHooksConfigured -Target $target)) {
        Invoke-ApsHookPrompt -Target $target
    } else {
        Write-Host ""
        Write-ApsInfo "Hooks already configured (not modified)."
        Write-Host "  To update: ./aps-planning/scripts/install-hooks.ps1"
    }

    Write-Host ""
    Write-ApsInfo "Your specs (index.aps.md, modules/*.aps.md) were NOT modified."
    Write-Host ""
}

function Update-ApsGlobal {
    <#
    .SYNOPSIS
        Update a global APS CLI installation (bin/ + lib/ only).
    #>
    $ApsHome = if ($env:APS_HOME) { $env:APS_HOME } else { Join-Path $HOME ".aps" }
    $binDir = Join-Path $ApsHome "bin"

    if (-not (Test-Path -LiteralPath $binDir -PathType Container)) {
        Write-ApsError "No global APS installation found at $ApsHome"
        Write-Host ""
        Write-Host "To install globally:"
        Write-Host '  irm https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install.ps1 | iex -- --global'
        Write-Host ""
        exit 1
    }

    Write-Host ""
    Write-ApsInfo "Updating global APS CLI at $ApsHome"
    Write-Host ""

    foreach ($f in $script:CliFilesBash) {
        Invoke-ApsDownloadRoot -Source $f -Destination (Join-Path $ApsHome $f)
    }
    foreach ($f in $script:CliFilesPowerShell) {
        Invoke-ApsDownloadRoot -Source $f -Destination (Join-Path $ApsHome $f)
    }

    Write-Host ""
    Write-ApsInfo "Global update complete"
    Write-ApsInfo "bin/aps + lib/ updated at $ApsHome"
    Write-Host ""
}

function Show-ApsUpdateHelp {
    Write-Host @"
aps update - Update APS templates, skill, CLI, and commands

Usage:
  aps update [target-dir]
  aps update --global

Updates the CLI, templates, rules, skill files, and commands without
touching your specs (index.aps.md, modules/*.aps.md, execution/*.actions.md).

If hooks are not yet configured, prompts to install them.

Options:
  --global  Update the global CLI installation (~/.aps/)
  --help    Show this help

Environment:
  APS_VERSION   Git ref to download from (default: main)
  APS_HOME      Custom global install location (default: ~/.aps)

Examples:
  aps update              # Update current directory
  aps update ./my-project # Update a subdirectory
  aps update --global     # Update global CLI
"@
}

Export-ModuleMember -Function @(
    'Invoke-ApsDownload'
    'Invoke-ApsDownloadRoot'
    'Install-ApsPlans'
    'Install-ApsIndex'
    'Install-ApsSkill'
    'Install-ApsCommands'
    'Install-ApsCli'
    'Update-ApsGlobal'
    'Invoke-ApsInit'
    'Invoke-ApsUpdate'
)
