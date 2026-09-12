<#
.SYNOPSIS
    APS CLI - Anvil Plan Spec tooling (PowerShell)

.DESCRIPTION
    aps.ps1 init [dir]        Create APS structure in a new project
    aps.ps1 update [dir]      Update templates, skill, and commands
    aps.ps1 lint [file|dir]   Validate APS documents (default: plans/)
    aps.ps1 lint --json       Output as JSON
    aps.ps1 --help            Show this help

.EXAMPLE
    .\bin\aps.ps1 init
    .\bin\aps.ps1 update
    .\bin\aps.ps1 lint
    .\bin\aps.ps1 lint plans\index.aps.md
    .\bin\aps.ps1 lint . --json
#>

param(
    [Parameter(Position = 0)]
    [string]$Command,

    [Parameter(Position = 1, ValueFromRemainingArguments)]
    [string[]]$Arguments
)

$ErrorActionPreference = "Stop"

# Resolve script location (handles symlinks on PS 6+)
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$LibDir = Join-Path (Join-Path $ScriptDir "..") "lib"
$RulesDir = Join-Path $LibDir "rules"

# Import all modules globally (order matters: foundations first, then rules, then orchestrator)
Import-Module (Join-Path $LibDir "Output.psm1") -Force -Global
Import-Module (Join-Path $RulesDir "Common.psm1") -Force -Global
Import-Module (Join-Path $RulesDir "WorkItem.psm1") -Force -Global
Import-Module (Join-Path $RulesDir "Module.psm1") -Force -Global
Import-Module (Join-Path $RulesDir "Index.psm1") -Force -Global
Import-Module (Join-Path $RulesDir "Issues.psm1") -Force -Global
Import-Module (Join-Path $RulesDir "Design.psm1") -Force -Global
Import-Module (Join-Path $RulesDir "Release.psm1") -Force -Global
Import-Module (Join-Path $LibDir "Lint.psm1") -Force -Global
Import-Module (Join-Path $LibDir "Scaffold.psm1") -Force -Global

function Show-Help {
    Write-Host @"
aps - Anvil Plan Spec CLI (PowerShell)

Usage:
  aps.ps1 init [dir]        Create APS structure in a new project
  aps.ps1 update [dir]      Update templates, skill, and commands
  aps.ps1 lint [file|dir]   Validate APS documents
  aps.ps1 lint --json       Output results as JSON
  aps.ps1 --help            Show this help

Options:
  --json    Output results in JSON format
  --help    Show help for a command

Environment:
  APS_VERSION   Git ref to download from (default: main)

Examples:
  .\bin\aps.ps1 init                       # Init in current directory
  .\bin\aps.ps1 update                     # Update templates and skill
  .\bin\aps.ps1 lint                       # Lint plans\ directory
  .\bin\aps.ps1 lint plans\index.aps.md    # Lint specific file
  .\bin\aps.ps1 lint . --json              # Lint current dir, JSON output
"@
}

function Show-LintHelp {
    Write-Host @"
Usage: aps.ps1 lint [file|dir] [options]

Validate APS documents against expected structure.

Arguments:
  file|dir    File or directory to lint (default: plans\)

Options:
  --json      Output results in JSON format
  --help      Show this help

Exit codes:
  0    No errors (may include warnings)
  1    One or more errors found

Examples:
  aps.ps1 lint                        # Lint plans\ directory
  aps.ps1 lint plans\index.aps.md     # Lint specific file
  aps.ps1 lint plans\modules\         # Lint all modules
  aps.ps1 lint . --json               # JSON output
"@
}

function Invoke-InitCommand {
    param([string[]]$InitArgs)
    Invoke-ApsInit -Arguments $InitArgs
}

function Invoke-UpdateCommand {
    param([string[]]$UpdateArgs)
    Invoke-ApsUpdate -Arguments $UpdateArgs
}

function Invoke-LintCommand {
    param([string[]]$LintArgs)
    $target = "plans"
    $jsonOutput = $false

    if ($LintArgs) {
        foreach ($arg in $LintArgs) {
            switch ($arg) {
                "--json" { $jsonOutput = $true }
                "--help" { Show-LintHelp; return }
                "-h"     { Show-LintHelp; return }
                default {
                    if ($arg.StartsWith("-")) {
                        Write-ApsError "Unknown option: $arg"
                        exit 1
                    }
                    $target = $arg
                }
            }
        }
    }

    $success = Invoke-ApsLint -Target $target -JsonOutput:$jsonOutput
    if (-not $success) { exit 1 }
}

# Main dispatch
switch ($Command) {
    "init" {
        Invoke-InitCommand -InitArgs $Arguments
    }
    "update" {
        Invoke-UpdateCommand -UpdateArgs $Arguments
    }
    "lint" {
        Invoke-LintCommand -LintArgs $Arguments
    }
    "--help" { Show-Help }
    "-h"     { Show-Help }
    "help"   { Show-Help }
    ""       { Show-Help }
    default {
        Write-ApsError "Unknown command: $Command"
        Write-Host "Run 'aps.ps1 --help' for usage."
        exit 1
    }
}
