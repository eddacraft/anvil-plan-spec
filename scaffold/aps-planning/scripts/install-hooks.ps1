#
# APS Hooks Installer
# Merges APS hook configuration into .claude/settings.local.json
# Preserves existing settings and permissions.
#
# Usage:
#   ./.aps/scripts/install-hooks.ps1            # Install all hooks
#   ./.aps/scripts/install-hooks.ps1 --minimal   # PreToolUse + Stop + SessionStart only
#   ./.aps/scripts/install-hooks.ps1 --remove    # Remove APS hooks
#
# No external dependencies (uses native PowerShell JSON support)

param(
    [Alias("m")][switch]$Minimal,
    [Alias("r")][switch]$Remove,
    [Alias("h")][switch]$Help
)

# Also handle --long-form args passed as bare strings (for cross-platform consistency)
$extraArgs = $args
foreach ($a in $extraArgs) {
    switch ($a) {
        { $_ -ceq '--minimal' -or $_ -ceq '-m' } { $Minimal = $true }
        { $_ -ceq '--remove'  -or $_ -ceq '-r' } { $Remove  = $true }
        { $_ -ceq '--help'    -or $_ -ceq '-h' } { $Help    = $true }
        default { Write-Host "Unknown option: $a"; exit 1 }
    }
}

if ($Help) {
    Write-Host "Usage: ./.aps/scripts/install-hooks.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  --minimal, -m  Install PreToolUse + Stop + SessionStart hooks"
    Write-Host "  --remove, -r   Remove all APS hooks"
    Write-Host "  --help, -h     Show this help"
    exit 0
}

# Determine mode
if ($Remove) {
    $Mode = "remove"
} elseif ($Minimal) {
    $Mode = "minimal"
} else {
    $Mode = "full"
}

$SettingsDir  = ".claude"
$SettingsFile = Join-Path $SettingsDir "settings.local.json"

# --- Helper functions ---

function Write-Info  { param([string]$Msg) Write-Host "[aps] " -ForegroundColor Green -NoNewline; Write-Host $Msg }
function Write-Warn  { param([string]$Msg) Write-Host "[aps] " -ForegroundColor Yellow -NoNewline; Write-Host $Msg }
function Write-Err   { param([string]$Msg) Write-Host "[aps] " -ForegroundColor Red -NoNewline; Write-Host $Msg }

function Test-ApsHook {
    <#
    .SYNOPSIS
        Returns $true if the given hook entry is APS-related.
    #>
    param([hashtable]$Entry)

    # Old format: { "hook": "...aps-planning/scripts..." }
    $hookStr = if ($Entry.ContainsKey("hook")) { $Entry["hook"] } else { "" }
    if ($hookStr -cmatch 'aps-planning/scripts' -or $hookStr -cmatch '\.aps/scripts' -or $hookStr -cmatch '\[APS\]') {
        return $true
    }

    # New format: { "hooks": [{ "command": "...aps-planning/scripts..." }] }
    if ($Entry.ContainsKey("hooks")) {
        foreach ($h in $Entry["hooks"]) {
            $cmd = ""
            if ($h -is [hashtable] -and $h.ContainsKey("command")) {
                $cmd = $h["command"]
            } elseif ($h.PSObject -and $h.PSObject.Properties["command"]) {
                $cmd = $h.command
            }
            if ($cmd -cmatch 'aps-planning/scripts' -or $cmd -cmatch '\.aps/scripts') {
                return $true
            }
        }
    }

    return $false
}

function ConvertTo-Hashtable {
    <#
    .SYNOPSIS
        Deep-converts a PSCustomObject (from ConvertFrom-Json) to nested hashtables.
        PS5's ConvertFrom-Json returns PSCustomObject, not hashtable.
    #>
    param([Parameter(ValueFromPipeline)]$InputObject)

    if ($null -eq $InputObject) { return $null }

    if ($InputObject -is [System.Collections.IEnumerable] -and $InputObject -isnot [string]) {
        $list = [System.Collections.ArrayList]::new()
        foreach ($item in $InputObject) {
            $null = $list.Add((ConvertTo-Hashtable $item))
        }
        return @(, $list.ToArray())
    }

    if ($InputObject -is [psobject] -and $InputObject -isnot [string] -and $InputObject -isnot [System.ValueType]) {
        $ht = [ordered]@{}
        foreach ($prop in $InputObject.PSObject.Properties) {
            $ht[$prop.Name] = ConvertTo-Hashtable $prop.Value
        }
        return $ht
    }

    return $InputObject
}

# --- Ensure .claude directory and settings file exist ---

if (-not (Test-Path $SettingsDir -PathType Container)) {
    New-Item -ItemType Directory -Path $SettingsDir -Force | Out-Null
}

if (-not (Test-Path $SettingsFile -PathType Leaf)) {
    Set-Content -Path $SettingsFile -Value '{}'
    Write-Info "Created $SettingsFile"
}

# --- Load settings as hashtable ---

$raw = Get-Content -Path $SettingsFile -Raw
$settings = ConvertTo-Hashtable (ConvertFrom-Json $raw)
if ($null -eq $settings) { $settings = [ordered]@{} }

# --- Remove mode ---

if ($Mode -ceq "remove") {
    if ($settings.Contains("hooks")) {
        $hookKeys = @($settings["hooks"].Keys)
        foreach ($event in $hookKeys) {
            $hooks = $settings["hooks"][$event]
            $filtered = @($hooks | Where-Object { -not (Test-ApsHook $_) })
            if ($filtered.Count -eq 0) {
                $settings["hooks"].Remove($event)
            } else {
                $settings["hooks"][$event] = $filtered
            }
        }
        if ($settings["hooks"].Count -eq 0) {
            $settings.Remove("hooks")
        }
    }

    $json = ConvertTo-Json $settings -Depth 10
    Set-Content -Path $SettingsFile -Value $json -NoNewline
    # Append trailing newline
    Add-Content -Path $SettingsFile -Value ""

    Write-Info "Removed APS hooks from $SettingsFile"
    exit 0
}

# --- Define APS hooks (pointing to .ps1 scripts) ---

$pretool = [ordered]@{
    matcher = "Write|Edit|Bash"
    hooks   = @(
        [ordered]@{ type = "command"; command = "./.aps/scripts/pre-tool-check.ps1" }
    )
}

$posttool = [ordered]@{
    matcher = "Write|Edit"
    hooks   = @(
        [ordered]@{ type = "command"; command = "./.aps/scripts/post-tool-nudge.ps1" }
    )
}

$stop = [ordered]@{
    hooks = @(
        [ordered]@{ type = "command"; command = "./.aps/scripts/check-complete.ps1" }
    )
}

$enforcePlan = [ordered]@{
    hooks = @(
        [ordered]@{ type = "command"; command = "./.aps/scripts/enforce-plan-update.ps1" }
    )
}

$sessionStart = [ordered]@{
    hooks = @(
        [ordered]@{ type = "command"; command = "./.aps/scripts/init-session.ps1" }
    )
}

# --- Build hooks based on mode ---

if ($Mode -ceq "minimal") {
    $newHooks = [ordered]@{
        PreToolUse   = @($pretool)
        Stop         = @($stop, $enforcePlan)
        SessionStart = @($sessionStart)
    }
} else {
    $newHooks = [ordered]@{
        PreToolUse   = @($pretool)
        PostToolUse  = @($posttool)
        Stop         = @($stop, $enforcePlan)
        SessionStart = @($sessionStart)
    }
}

# --- Idempotent merge: remove existing APS hooks, then add new ones ---

if (-not $settings.Contains("hooks")) {
    $settings["hooks"] = [ordered]@{}
}

foreach ($event in $newHooks.Keys) {
    if (-not $settings["hooks"].Contains($event)) {
        $settings["hooks"][$event] = @()
    }

    # Filter out existing APS hooks (idempotent, handles old+new format)
    $existing = @($settings["hooks"][$event] | Where-Object { -not (Test-ApsHook $_) })

    # Append new APS hooks
    $settings["hooks"][$event] = @($existing) + @($newHooks[$event])
}

# --- Write settings back ---

$json = ConvertTo-Json $settings -Depth 10
Set-Content -Path $SettingsFile -Value $json -NoNewline
# Append trailing newline
Add-Content -Path $SettingsFile -Value ""

# --- Ensure session baseline is gitignored ---

$BaselineEntry = ".claude/.aps-session-baseline"

if (Test-Path .gitignore -PathType Leaf) {
    $gitignoreContent = Get-Content .gitignore -Raw -ErrorAction SilentlyContinue
    if ($gitignoreContent -notmatch [regex]::Escape($BaselineEntry)) {
        Add-Content -Path .gitignore -Value ""
        Add-Content -Path .gitignore -Value "# APS session baseline (ephemeral)"
        Add-Content -Path .gitignore -Value $BaselineEntry
        Write-Info "Added $BaselineEntry to .gitignore"
    }
} else {
    Set-Content -Path .gitignore -Value "# APS session baseline (ephemeral)"
    Add-Content -Path .gitignore -Value $BaselineEntry
    Write-Info "Created .gitignore with $BaselineEntry"
}

# --- Summary ---

Write-Info "Installed APS hooks ($Mode mode) into $SettingsFile"
Write-Host ""
Write-Host "  Hooks added:"

if ($Mode -ceq "full") {
    Write-Host "    PreToolUse          - Reminds agent to check plan before code changes"
    Write-Host "    PostToolUse         - Nudges agent to update specs after changes"
    Write-Host "    Stop (Completion)   - Blocks session end if work items unresolved"
    Write-Host "    Stop (Plan Update)  - Blocks session end if code changed but plans untouched"
    Write-Host "    SessionStart        - Shows planning status at session start"
} else {
    Write-Host "    PreToolUse          - Reminds agent to check plan before code changes"
    Write-Host "    Stop (Completion)   - Blocks session end if work items unresolved"
    Write-Host "    Stop (Plan Update)  - Blocks session end if code changed but plans untouched"
    Write-Host "    SessionStart        - Writes session baseline for change detection"
}

Write-Host ""
Write-Info "See aps-planning/hooks.md for details on each hook."
