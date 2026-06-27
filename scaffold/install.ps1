#
# APS Install Script (PowerShell)
# Multi-mode entrypoint: install the CLI, initialize a repo (minimal planning
# content by default), bootstrap a repo for an agent, upgrade, or add a tool.
#
# Usage:
#   & ([scriptblock]::Create((irm "https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install.ps1"))) --cli
#   $env:APS_VERSION = "v0.4.0"; & ([scriptblock]::Create((irm "..."))) --cli
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

$UseBinary = $false
# init footprint flags (INSTALL-011): minimal by default, opt in to extras.
$UseLocalCli = $false
$InstallHooks = $false

for ($i = 0; $i -lt $args.Count; $i++) {
    switch ($args[$i]) {
        { $_ -in "--cli", "--global", "-g" } { $Mode = "cli" }
        "--init"    { $Mode = "init" }
        "--agent"   { $Mode = "agent" }
        "--upgrade" { $Mode = "upgrade" }
        { $_ -in "--binary", "-b" } { $UseBinary = $true }
        { $_ -in "--local-cli", "--bash" } { $UseLocalCli = $true }
        "--hooks"   { $InstallHooks = $true }
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

# Reject an empty TARGET regardless of mode — "" makes PlansDir an absolute
# root path.
if (-not $Target) {
    [Console]::Error.WriteLine("error: TARGET must not be empty.")
    exit 1
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

function Get-ApsReleaseTarget {
    <#
    .SYNOPSIS
        Map this machine to a published release target triple, or $null when
        no Windows binary is available (caller falls back to the PowerShell CLI).
    #>
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64-pc-windows-gnu" }
        default { return $null }
    }
}

function Install-ApsBinary {
    <#
    .SYNOPSIS
        Download the prebuilt aps.exe from GitHub releases into $DestDir.
    .OUTPUTS
        $true on success, $false on any failure (so callers can fall back).
    #>
    param([string]$DestDir)

    $target = Get-ApsReleaseTarget
    if (-not $target) {
        Write-Warn "No release binary for $env:PROCESSOR_ARCHITECTURE; falling back to PowerShell CLI"
        return $false
    }

    if ($Version -eq "main") {
        $url = "https://github.com/EddaCraft/anvil-plan-spec/releases/latest/download/aps-$target.zip"
    } else {
        $v = $Version.TrimStart("v")
        $url = "https://github.com/EddaCraft/anvil-plan-spec/releases/download/v$v/aps-$target.zip"
    }

    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("aps-" + [System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Path $tmp -Force | Out-Null
    try {
        $zip = Join-Path $tmp "aps.zip"
        Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing
        Expand-Archive -Path $zip -DestinationPath $tmp -Force
        New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
        Move-Item -Path (Join-Path $tmp "aps.exe") -Destination (Join-Path $DestDir "aps.exe") -Force
        Write-Info "aps native binary ($target) installed to $DestDir\aps.exe"
        return $true
    } catch {
        Write-Warn "Failed to download release binary from $url; falling back to PowerShell CLI"
        return $false
    } finally {
        Remove-Item -Path $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

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

    # Binary-first (D-034 / INSTALL-015): install the native release binary by
    # default; fall back to the PowerShell CLI when no binary is available or
    # --bash is given.
    $kind = "powershell"
    $installedBinary = $false
    if (-not $UseLocalCli) {
        $installedBinary = Install-ApsBinary -DestDir (Join-Path $ApsHome "bin")
    }

    if ($installedBinary) {
        $kind = "binary"
    } elseif ($UseBinary -and -not $UseLocalCli) {
        # --binary requires the prebuilt binary: do not silently fall back.
        Write-Err "--binary requested but no release binary is available for $env:PROCESSOR_ARCHITECTURE"
        Write-Host "  Re-run without --binary to use the PowerShell CLI fallback."
        exit 1
    } else {
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
    }

    Set-ApsGlobalPath -ApsHome $ApsHome

    Write-Host ""
    Write-Step "Global installation complete"
    Write-Host ""
    if ($kind -eq "binary") {
        Write-Host "  $ApsHome\"
        Write-Host "  +-- bin\aps.exe      <- CLI (native release binary)"
    } else {
        Write-Host "  $ApsHome\"
        Write-Host "  +-- bin\aps.ps1      <- CLI (PowerShell)"
        Write-Host "  +-- lib\             <- CLI libraries"
    }
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

    Write-Step "Writing agent next steps"
    $nextSteps = @'
# APS Agent Bootstrap — Next Steps

This repository was just initialized with a minimal APS planning layer
for an AI agent. Before implementing anything:

1. Read `plans/aps-rules.md` for the planning conventions.
2. Ask the operator for the project intent — what is being built and why.
3. Populate `plans/project-context.md` with that durable background.
4. Draft `plans/index.aps.md` (problem, outcomes, scope, modules).
5. Wait for an approved work item before writing any implementation code.

No hooks, agents, or tool integrations were installed. Run `aps setup`
to add them when needed.
'@
    Set-Content -LiteralPath (Join-Path $PlansDir "agent-next-steps.md") -Value $nextSteps
    Write-Info "plans/agent-next-steps.md"
    Write-Step "Agent bootstrap complete"
    Get-Content -LiteralPath (Join-Path $PlansDir "agent-next-steps.md")
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
    # Save to a temp file so $Target can be forwarded as an argument — an
    # argument-less Invoke-Expression would always operate on the cwd.
    $u = "$BaseUrl/scaffold/update.ps1"
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) "aps-update-$PID.ps1"
    try {
        (Invoke-WebRequest -Uri $u -UseBasicParsing).Content | Set-Content -LiteralPath $tmp
        & $tmp $Target
    } finally {
        Remove-Item -LiteralPath $tmp -ErrorAction SilentlyContinue
    }
}

# --- Setup: add one tool integration via aps setup ---
#
# `aps setup` ships with the native binary (built out under INSTALL-012); the
# PowerShell CLI does not provide it yet, so gate on a setup-capable aps and
# write nothing if none is found — matching the bash installer.

function Install-ApsSetup {
    param([string]$Tool)
    Write-Host ""
    Write-Host "Anvil Plan Spec (APS)" -ForegroundColor White
    Write-Host "Add integration: $Tool"
    Write-Host ""

    $apsCmd = Get-Command aps -ErrorAction SilentlyContinue
    $hasSetup = $false
    if ($apsCmd) {
        try { & aps setup --help *>$null; $hasSetup = ($LASTEXITCODE -eq 0) } catch { $hasSetup = $false }
    }
    if (-not $hasSetup) {
        Write-Err "aps setup is not available from the installed CLI"
        Write-Host "  Install the native CLI first (it ships 'aps setup'), then run: aps setup $Tool"
        exit 1
    }
    Write-Step "Running aps setup $Tool"
    & aps setup $Tool
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

# --- Create directory structure ---

Write-Step "Creating directory structure"
New-Item -ItemType Directory -Path (Join-Path $PlansDir "modules") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $PlansDir "execution") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $PlansDir "decisions") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $PlansDir "designs") -Force | Out-Null

# --- Download templates (v2: rules + project context + trackers) ---

Write-Step "Downloading templates"

Invoke-Download -Path "plans/index.aps.md" -Destination (Join-Path $PlansDir "index.aps.md")
Invoke-Download -Path "plans/aps-rules-v2.md" -Destination (Join-Path $PlansDir "aps-rules.md")
Invoke-Download -Path "plans/project-context.md" -Destination (Join-Path $PlansDir "project-context.md")
Invoke-Download -Path "plans/issues.md" -Destination (Join-Path $PlansDir "issues.md")
Invoke-Download -Path "plans/modules/.module.template.md" -Destination (Join-Path $PlansDir (Join-Path "modules" ".module.template.md"))
Invoke-Download -Path "plans/modules/.simple.template.md" -Destination (Join-Path $PlansDir (Join-Path "modules" ".simple.template.md"))
Invoke-Download -Path "plans/modules/.index-monorepo.template.md" -Destination (Join-Path $PlansDir (Join-Path "modules" ".index-monorepo.template.md"))
Invoke-Download -Path "plans/execution/.actions.template.md" -Destination (Join-Path $PlansDir (Join-Path "execution" ".actions.template.md"))
New-Item -ItemType File -Path (Join-Path $PlansDir (Join-Path "decisions" ".gitkeep")) -Force | Out-Null
New-Item -ItemType File -Path (Join-Path $PlansDir (Join-Path "designs" ".gitkeep")) -Force | Out-Null
Write-Info "plans/ (rules, project-context, index, issues, templates)"

# --- Project contract ---

Write-Step "Writing project config"
$apsDir = Join-Path $Target ".aps"
New-Item -ItemType Directory -Path $apsDir -Force | Out-Null
$gitignore = Join-Path $apsDir ".gitignore"
if (-not (Test-Path -LiteralPath $gitignore) -or -not ((Get-Content -LiteralPath $gitignore -Raw -ErrorAction SilentlyContinue) -match 'context/')) {
    Add-Content -LiteralPath $gitignore -Value "context/"
}
$today = (Get-Date -Format "yyyy-MM-dd")
# $Version is a git ref ("main") by default; only pin it as cli_version when
# it is an explicit semver, else fall back to the release version.
if ($Version -match '^v?[0-9]') { $cliVersion = $Version -replace '^v', '' } else { $cliVersion = "0.4.0" }
$configBody = @"
# .aps/config.yml — written by installer, read by updater

# Project contract (INSTALL-014 / D-035): toolchain pin + runtime path
# defaults the global 'aps' binary discovers by walking up from cwd.
cli_version: "$cliVersion"
plans_dir: plans/
docs_dir: docs/
tooling_root: .aps/

aps:
  version: "0.4.0"
  config_schema: 1
  installed: "$today"
  updated: "$today"

project:
  type: simple
  monorepo_tool: ~
  profile: solo

tools:
  - name: generic
    # No tool integration — run 'aps setup' to add one
"@
Set-Content -LiteralPath (Join-Path $apsDir "config.yml") -Value $configBody
Write-Info ".aps/config.yml + .aps/.gitignore"

# --- Optional: vendored bash/PowerShell CLI runtime ---

if ($UseLocalCli) {
    Write-Step "Vendoring CLI into .aps/"
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
        Invoke-DownloadRoot -Path $f -Destination (Join-Path $apsDir $f)
    }
    Write-Info ".aps/bin/aps.ps1 + .aps/lib/ (vendored CLI)"
}

# --- Optional: hook scripts ---

if ($InstallHooks) {
    Write-Step "Installing hook scripts"
    $hookScripts = @(
        "install-hooks.ps1"
        "init-session.ps1"
        "check-complete.ps1"
        "pre-tool-check.ps1"
        "post-tool-nudge.ps1"
        "enforce-plan-update.ps1"
    )
    foreach ($s in $hookScripts) {
        Invoke-Download -Path "aps-planning/scripts/$s" -Destination (Join-Path $apsDir (Join-Path "scripts" $s))
    }
    Write-Info ".aps/scripts/ (hook scripts)"
}

# --- Success ---

Write-Host ""
Write-Step "Installation complete"
Write-Host ""
Write-Host "  plans/"
Write-Host "  +-- aps-rules.md              # Agent guidance (APS-managed)"
Write-Host "  +-- project-context.md        # Your project context (edit this)"
Write-Host "  +-- index.aps.md              # Your main plan (edit this)"
Write-Host "  +-- issues.md                 # Issue & question tracker"
Write-Host "  +-- modules/                  # Module templates"
Write-Host "  +-- execution/                # Action plan template"
Write-Host "  +-- decisions/                # ADRs"
Write-Host "  +-- designs/                  # Technical designs"
Write-Host ""
Write-Host "  .aps/config.yml               # Project contract (cli_version, paths)"
Write-Host ""

# --- Next steps ---

Write-Host ""
Write-Step "Next steps"
Write-Host ""
if (-not $UseLocalCli) {
    Write-Host "  1. Install the APS CLI: " -NoNewline
    Write-Host "& ([scriptblock]::Create((irm `"$BaseUrl/scaffold/install.ps1`"))) --cli" -ForegroundColor White
    Write-Host "  2. Edit " -NoNewline; Write-Host "plans\index.aps.md" -ForegroundColor White -NoNewline; Write-Host " to define your plan"
    Write-Host "  3. Add hooks/agents/tool skills with " -NoNewline; Write-Host "aps setup" -ForegroundColor White
} else {
    Write-Host "  1. Edit " -NoNewline; Write-Host "plans\index.aps.md" -ForegroundColor White -NoNewline; Write-Host " to define your plan"
    Write-Host "  2. Point your AI agent at plans\aps-rules.md, or run aps next"
}
Write-Host ""
Write-Host "Docs: https://github.com/EddaCraft/anvil-plan-spec"
Write-Host ""
}

# --- Resolve mode and dispatch ---

if (-not $Mode) {
    # Match Request-YesNo's interactivity check: redirected stdin must not
    # land in Read-Host and block.
    if ([Environment]::UserInteractive -and -not [Console]::IsInputRedirected) {
        Select-ApsMode
    } else {
        [Console]::Error.WriteLine("error: no mode given (use --cli/--init/--agent/--upgrade/--setup)")
        exit 1
    }
}

# Validate TARGET now that the mode is known (cli installs machine-wide).
if ($Mode -eq "cli") {
    if ($Target -ne ".") { Write-Warn "--cli installs machine-wide; ignoring TARGET '$Target'" }
} else {
    if ([System.IO.Path]::IsPathRooted($Target)) {
        [Console]::Error.WriteLine("error: Absolute paths are not allowed for TARGET; please use a relative path (e.g., .\my-project).")
        exit 1
    }
    if ($Target -cmatch '\.\.') {
        [Console]::Error.WriteLine("error: Parent directory references ('..') are not allowed in TARGET.")
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
