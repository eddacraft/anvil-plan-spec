#
# APS CLI Scaffold Module
# Port of lib/scaffold.sh — init and update workflows
#

# --- Configuration ---

$script:ApsVersion = if ($env:APS_VERSION) { $env:APS_VERSION } else { "main" }
$script:ApsBaseUrl = "https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/$script:ApsVersion"

# Semver of the PowerShell CLI release, stamped into managed skill markers.
# Mirrors APS_CLI_VERSION in lib/scaffold.sh; the native binary stamps its
# crate version. One semver across channels (D-036).
$script:ApsCliVersion = if ($env:APS_CLI_VERSION) { $env:APS_CLI_VERSION } else { "0.7.0" }

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

# --- v2 file lists (.aps/ layout, parity with lib/scaffold.sh) ---

# Managed skill payload for .claude/skills/ and .agents/skills/ (D-042)
$script:SkillFilesV2 = @(
    "scaffold/aps-planning/SKILL.md"
    "scaffold/aps-planning/reference.md"
    "scaffold/aps-planning/examples.md"
)

# Plan templates and rules for plans/
$script:PlanFilesV2 = @(
    "scaffold/plans/aps-rules-v2.md"
    "scaffold/plans/project-context.md"
    "scaffold/plans/issues.md"
    "scaffold/plans/modules/.module.template.md"
    "scaffold/plans/modules/.simple.template.md"
    "scaffold/plans/modules/.index-monorepo.template.md"
    "scaffold/plans/execution/.actions.template.md"
)

# Hook scripts for .aps/scripts/ (PowerShell variants)
$script:ScriptFilesV2 = @(
    "aps-planning/scripts/install-hooks.ps1"
    "aps-planning/scripts/init-session.ps1"
    "aps-planning/scripts/check-complete.ps1"
    "aps-planning/scripts/pre-tool-check.ps1"
    "aps-planning/scripts/post-tool-nudge.ps1"
    "aps-planning/scripts/enforce-plan-update.ps1"
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
    $dir = Split-Path $Destination
    if ($dir) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
    # Local mode: copy from a source repo instead of downloading (parity with
    # bash download()'s APS_LOCAL).
    if ($env:APS_LOCAL) {
        $localPath = Join-Path $env:APS_LOCAL $Source
        if (Test-Path -LiteralPath $localPath) {
            Copy-Item -LiteralPath $localPath -Destination $Destination -Force
            return
        }
        Write-ApsError "Local file not found: $localPath"
        exit 1
    }
    $url = "$script:ApsBaseUrl/$Source"
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
    Invoke-ApsDownload -Source $Source -Destination $Destination
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

# --- Managed skill markers (D-042 / INSTALL-020) ---
#
# Sidecar `.aps-managed.json` written next to each managed skill tree so
# installs and updates can distinguish APS-owned content from user edits.
# The JSON shape, per-file SHA-256 hashes, and bundle digest are
# byte-identical with the Rust implementation (cli/src/managed.rs) and the
# bash port (lib/scaffold.sh): any CLI can verify a tree written by any
# other. Phase 1 covers the planning skill (SKILL.md, reference.md,
# examples.md); agent inventory is Phase 3.

$script:ManagedMarkerName = ".aps-managed.json"
$script:SkillPayloadDir = $null

function Get-ApsFileSha256 {
    param([string]$Path)
    return (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Get-ApsStringSha256 {
    param([string]$Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = $sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($Text))
    } finally {
        $sha.Dispose()
    }
    return (-join ($bytes | ForEach-Object { $_.ToString('x2') }))
}

function Get-ApsSkillPayload {
    <#
    .SYNOPSIS
        Canonical skill payload, downloaded once per session to a temp dir.
    #>
    if ($script:SkillPayloadDir -and (Test-Path -LiteralPath $script:SkillPayloadDir)) {
        return $script:SkillPayloadDir
    }
    $dir = Join-Path ([System.IO.Path]::GetTempPath()) ("aps-skill-payload-" + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
    foreach ($f in $script:SkillFilesV2) {
        $rel = $f -creplace '^scaffold/aps-planning/', ''
        Invoke-ApsDownload -Source $f -Destination (Join-Path $dir $rel)
    }
    $script:SkillPayloadDir = $dir
    return $dir
}

function Get-ApsManagedManifestJson {
    <#
    .SYNOPSIS
        Canonical marker JSON for a payload dir. Matches Rust
        SkillManifest::to_json byte-for-byte: camelCase keys, two-space
        indent, files sorted bytewise, LF line endings, trailing newline.
        bundleDigest is SHA-256 over sorted "name=hash\n" lines.
    #>
    param([string]$PayloadDir)
    $names = @($script:SkillFilesV2 | ForEach-Object { $_ -creplace '^scaffold/aps-planning/', '' })
    [Array]::Sort($names, [System.StringComparer]::Ordinal)
    $material = ""
    $fileLines = @()
    foreach ($n in $names) {
        $h = Get-ApsFileSha256 -Path (Join-Path $PayloadDir $n)
        $material += "$n=$h`n"
        $fileLines += "    `"$n`": `"$h`""
    }
    $digest = Get-ApsStringSha256 -Text $material
    $json = "{`n"
    $json += "  `"schemaVersion`": 1,`n"
    $json += "  `"kind`": `"skill`",`n"
    $json += "  `"name`": `"aps-planning`",`n"
    $json += "  `"cliVersion`": `"$script:ApsCliVersion`",`n"
    $json += "  `"bundleDigest`": `"$digest`",`n"
    $json += "  `"files`": {`n"
    $json += ($fileLines -join ",`n") + "`n"
    $json += "  }`n}`n"
    return $json
}

function Write-ApsSkillMarker {
    param([string]$SkillDir, [string]$PayloadDir)
    $json = Get-ApsManagedManifestJson -PayloadDir $PayloadDir
    $markerPath = Join-Path $SkillDir $script:ManagedMarkerName
    # LF + no BOM so the marker is byte-identical across platforms and CLIs.
    [System.IO.File]::WriteAllText($markerPath, $json, [System.Text.UTF8Encoding]::new($false))
}

function Get-ApsMarkerEntries {
    <#
    .SYNOPSIS
        Parse a marker's files map into a name->hash hashtable. Returns $null
        when the marker is not a readable skill manifest (schemaVersion 1,
        kind "skill", files map of plain string pairs).
    #>
    param([string]$MarkerPath)
    $text = Get-Content -LiteralPath $MarkerPath -Raw -ErrorAction SilentlyContinue
    if (-not $text) { return $null }
    if ($text -notmatch '"schemaVersion"\s*:\s*1') { return $null }
    if ($text -notmatch '"kind"\s*:\s*"skill"') { return $null }
    $m = [regex]::Match($text, '"files"\s*:\s*\{([^}]*)\}')
    if (-not $m.Success) { return $null }
    $entries = @{}
    foreach ($line in ($m.Groups[1].Value -split "`n")) {
        if ($line -match '^\s*$') { continue }
        if ($line -match '^\s*"([^"]+)"\s*:\s*"([^"]*)",?\s*$') {
            $entries[$Matches[1]] = $Matches[2]
        } else {
            return $null
        }
    }
    return $entries
}

function Get-ApsSkillDirState {
    <#
    .SYNOPSIS
        Classify a skill dir against the canonical payload:
        absent | fresh | stale | dirty | unmanaged | broken
    #>
    param([string]$SkillDir, [string]$PayloadDir)
    if (-not (Test-Path -LiteralPath $SkillDir -PathType Container)) { return "absent" }
    $markerPath = Join-Path $SkillDir $script:ManagedMarkerName
    if (-not (Test-Path -LiteralPath $markerPath -PathType Leaf)) {
        foreach ($f in $script:SkillFilesV2) {
            $rel = $f -creplace '^scaffold/aps-planning/', ''
            if (Test-Path -LiteralPath (Join-Path $SkillDir $rel) -PathType Leaf) { return "unmanaged" }
        }
        # Empty dir (or only non-skill files) — safe to install.
        return "absent"
    }
    $entries = Get-ApsMarkerEntries -MarkerPath $markerPath
    if ($null -eq $entries) { return "broken" }
    # Dirty when any tracked file is missing or no longer matches the marker.
    foreach ($name in $entries.Keys) {
        $path = Join-Path $SkillDir $name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { return "dirty" }
        if ((Get-ApsFileSha256 -Path $path) -cne $entries[$name]) { return "dirty" }
    }
    # Canonical serialisation makes equivalence a byte comparison (all three
    # CLIs write the same shape). A semantically-equivalent but non-canonical
    # marker classifies as stale and converges to canonical on update.
    $markerText = Get-Content -LiteralPath $markerPath -Raw
    if ($markerText -ceq (Get-ApsManagedManifestJson -PayloadDir $PayloadDir)) { return "fresh" }
    return "stale"
}

function Test-ApsSkillFilesMatch {
    param([string]$SkillDir, [string]$PayloadDir)
    foreach ($f in $script:SkillFilesV2) {
        $rel = $f -creplace '^scaffold/aps-planning/', ''
        $installed = Join-Path $SkillDir $rel
        if (-not (Test-Path -LiteralPath $installed -PathType Leaf)) { return $false }
        if ((Get-ApsFileSha256 -Path $installed) -cne (Get-ApsFileSha256 -Path (Join-Path $PayloadDir $rel))) { return $false }
    }
    return $true
}

function Copy-ApsSkillFiles {
    param([string]$SkillDir, [string]$PayloadDir)
    New-Item -ItemType Directory -Path $SkillDir -Force | Out-Null
    foreach ($f in $script:SkillFilesV2) {
        $rel = $f -creplace '^scaffold/aps-planning/', ''
        Copy-Item -LiteralPath (Join-Path $PayloadDir $rel) -Destination (Join-Path $SkillDir $rel) -Force
    }
}

function Invoke-ApsSkillReconcile {
    <#
    .SYNOPSIS
        Reconcile one skill tree with managed-marker safety. Returns the
        outcome: added | updated | unchanged | adopted | dirty-skipped |
        unmanaged-skipped | broken-skipped (mirrors Rust
        reconcile_managed_skill).
    #>
    param([string]$SkillDir)
    $payload = Get-ApsSkillPayload
    $state = Get-ApsSkillDirState -SkillDir $SkillDir -PayloadDir $payload
    switch ($state) {
        "absent" {
            Copy-ApsSkillFiles -SkillDir $SkillDir -PayloadDir $payload
            Write-ApsSkillMarker -SkillDir $SkillDir -PayloadDir $payload
            return "added"
        }
        "fresh" { return "unchanged" }
        "stale" {
            # Content may already match the payload (only the marker drifted)
            # — avoid needless rewrites/mtime churn.
            if (-not (Test-ApsSkillFilesMatch -SkillDir $SkillDir -PayloadDir $payload)) {
                Copy-ApsSkillFiles -SkillDir $SkillDir -PayloadDir $payload
            }
            Write-ApsSkillMarker -SkillDir $SkillDir -PayloadDir $payload
            return "updated"
        }
        "dirty" { return "dirty-skipped" }
        "unmanaged" {
            if (Test-ApsSkillFilesMatch -SkillDir $SkillDir -PayloadDir $payload) {
                Write-ApsSkillMarker -SkillDir $SkillDir -PayloadDir $payload
                return "adopted"
            }
            return "unmanaged-skipped"
        }
        "broken" { return "broken-skipped" }
    }
}

function Install-ApsManagedSkill {
    <#
    .SYNOPSIS
        Reconcile + user-facing messaging for one skill tree.
    #>
    param([string]$SkillDir, [string]$Label)
    $result = Invoke-ApsSkillReconcile -SkillDir $SkillDir
    switch ($result) {
        "dirty-skipped" {
            Write-ApsWarning "$Label has local edits — left untouched."
            [Console]::Error.WriteLine("  Restore the files (or remove the directory) and re-run to refresh.")
        }
        "unmanaged-skipped" {
            Write-ApsWarning "$Label exists but was not installed by APS — left untouched."
            [Console]::Error.WriteLine("  Remove the directory to let APS manage it.")
        }
        "broken-skipped" {
            Write-ApsWarning "$Label has an unreadable $script:ManagedMarkerName — left untouched."
        }
    }
    return $result
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
        Download CLI files (PowerShell runtime) to the target directory.
    #>
    param([string]$Target)
    foreach ($f in $script:CliFilesPowerShell) {
        Invoke-ApsDownloadRoot -Source $f -Destination (Join-Path $Target $f)
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

# --- v2 install/update functions (.aps/ layout) ---

function Test-ApsV2Layout {
    param([string]$Target = ".")
    return (Test-Path -LiteralPath (Join-Path (Join-Path $Target ".aps") "config.yml") -PathType Leaf)
}

function Install-ApsPlansV2 {
    <#
    .SYNOPSIS
        Refresh v2 plan templates and rules (preserves user specs and the
        user-owned project-context.md / issues.md).
    #>
    param([string]$Target)
    $plansDir = Join-Path $Target "plans"
    foreach ($sub in @("modules", "execution", "decisions", "designs")) {
        New-Item -ItemType Directory -Path (Join-Path $plansDir $sub) -Force | Out-Null
    }
    foreach ($f in $script:PlanFilesV2) {
        $rel = $f -creplace '^scaffold/plans/', ''
        $dest = Join-Path $plansDir $rel
        if ($rel -ceq "aps-rules-v2.md") {
            $dest = Join-Path $plansDir "aps-rules.md"
        }
        if (($rel -ceq "project-context.md" -or $rel -ceq "issues.md") -and (Test-Path -LiteralPath $dest)) {
            continue
        }
        Invoke-ApsDownload -Source $f -Destination $dest
    }
}

function Install-ApsIndexV2 {
    <#
    .SYNOPSIS
        Seed plans/index.aps.md (init only, never update) and keep the empty
        decisions/ and designs/ dirs in git. Mirrors v2_install_index in
        lib/scaffold.sh.
    #>
    param([string]$Target)
    $plansDir = Join-Path $Target "plans"
    Invoke-ApsDownload -Source "scaffold/plans/index.aps.md" -Destination (Join-Path $plansDir "index.aps.md")
    foreach ($sub in @("decisions", "designs")) {
        $gitkeep = Join-Path (Join-Path $plansDir $sub) ".gitkeep"
        if (-not (Test-Path -LiteralPath $gitkeep)) {
            New-Item -ItemType File -Path $gitkeep -Force | Out-Null
        }
    }
}

function Write-ApsConfigV2 {
    <#
    .SYNOPSIS
        Write .aps/config.yml + .aps/.gitignore (the per-project contract).
        Mirrors write_config in lib/scaffold.sh: same key order and per-tool
        subkeys, so any CLI's updater can read what this one installed.
        LF + no BOM keeps the file byte-comparable across the three CLIs.
    #>
    param(
        [string]$Target,
        [string]$Profile = "solo",
        [string[]]$Tools = @()
    )
    $apsDir = Join-Path $Target ".aps"
    New-Item -ItemType Directory -Path $apsDir -Force | Out-Null

    # Ignore ephemeral CLI-generated context regardless of whether a local CLI
    # was vendored (the global binary writes here too).
    $gitignore = Join-Path $apsDir ".gitignore"
    $existing = if (Test-Path -LiteralPath $gitignore) { @(Get-Content -LiteralPath $gitignore -ErrorAction SilentlyContinue) } else { @() }
    if ($existing -cnotcontains "context/") {
        Add-Content -LiteralPath $gitignore -Value "context/"
    }

    $today = Get-Date -Format "yyyy-MM-dd"
    $lines = @(
        "# .aps/config.yml — written by installer, read by updater"
        ""
        "# Project contract (INSTALL-014 / D-035): toolchain pin + runtime path"
        "# defaults the global 'aps' binary discovers by walking up from cwd."
        "cli_version: `"$script:ApsCliVersion`""
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
        "  profile: $Profile"
        ""
        "tools:"
    )
    foreach ($tool in $Tools) {
        $lines += "  - name: $tool"
        switch ($tool) {
            "claude-code" {
                $lines += "    skill: .claude/skills/aps-planning"
                $lines += "    hooks: full"
                $lines += "    agents:"
                $lines += "      - aps-planner"
                $lines += "      - aps-librarian"
                $lines += "      - aps-conductor"
            }
            "copilot" {
                $lines += "    skill: .claude/skills/aps-planning"
                $lines += "    instruction_file: AGENTS.md"
            }
            "codex" {
                $lines += "    skill: .agents/skills/aps-planning"
                $lines += "    instruction_file: AGENTS.md"
            }
            "opencode" {
                $lines += "    skill: .claude/skills/aps-planning"
            }
            "grok" {
                $lines += "    skill: .agents/skills/aps-planning"
                $lines += "    instruction_file: AGENTS.md"
            }
            "antigravity" {
                $lines += "    skill: .agents/skills/aps-planning"
                $lines += "    instruction_file: AGENTS.md"
            }
            "generic" {
                $lines += "    # No tool integration"
            }
        }
    }
    [System.IO.File]::WriteAllText((Join-Path $apsDir "config.yml"), (($lines -join "`n") + "`n"), [System.Text.UTF8Encoding]::new($false))
}

function Install-ApsScriptsV2 {
    param([string]$Target)
    $scriptsDir = Join-Path (Join-Path $Target ".aps") "scripts"
    New-Item -ItemType Directory -Path $scriptsDir -Force | Out-Null
    foreach ($f in $script:ScriptFilesV2) {
        $rel = $f -creplace '^aps-planning/scripts/', ''
        Invoke-ApsDownload -Source $f -Destination (Join-Path $scriptsDir $rel)
    }
}

function Install-ApsSkillV2 {
    param([string]$Target)
    $skillDir = Join-Path $Target (Join-Path ".claude" (Join-Path "skills" "aps-planning"))
    Install-ApsManagedSkill -SkillDir $skillDir -Label ".claude/skills/aps-planning" | Out-Null
}

function Install-ApsAgentsSkillV2 {
    param([string]$Target)
    $skillDir = Join-Path $Target (Join-Path ".agents" (Join-Path "skills" "aps-planning"))
    Install-ApsManagedSkill -SkillDir $skillDir -Label ".agents/skills/aps-planning" | Out-Null
}

function Install-ApsToolAgents {
    <#
    .SYNOPSIS
        Refresh one tool's agent files (plain downloads, Phase 3 will bring
        these under managed markers).
    #>
    param([string]$Target, [string]$Tool)
    $roles = @("aps-planner", "aps-librarian", "aps-conductor")
    switch ($Tool) {
        "claude-code" {
            foreach ($r in $roles) {
                Invoke-ApsDownload -Source "scaffold/agents/claude-code/$r.md" -Destination (Join-Path $Target (Join-Path ".claude" (Join-Path "agents" "$r.md")))
            }
        }
        "copilot" {
            foreach ($r in $roles) {
                Invoke-ApsDownload -Source "scaffold/agents/copilot/$r.md" -Destination (Join-Path $Target (Join-Path ".github" (Join-Path "agents" "$r.md")))
            }
        }
        "opencode" {
            foreach ($r in $roles) {
                Invoke-ApsDownload -Source "scaffold/agents/opencode/$r.md" -Destination (Join-Path $Target (Join-Path ".opencode" (Join-Path "agents" "$r.md")))
            }
        }
        "codex" {
            foreach ($r in $roles) {
                Invoke-ApsDownload -Source "scaffold/agents/codex/$r.toml" -Destination (Join-Path $Target (Join-Path ".codex" (Join-Path "agents" "$r.toml")))
            }
            $snippet = Join-Path $Target (Join-Path ".codex" (Join-Path "agents" "codex-config-snippet.toml"))
            if (Test-Path -LiteralPath $snippet) { Remove-Item -LiteralPath $snippet -Force }
        }
    }
}

function Update-ApsV2 {
    <#
    .SYNOPSIS
        Update a v2-layout project: plan templates, hook scripts, vendored
        pwsh CLI (when present), and managed skill trees per config.yml.
    #>
    param([string]$Target)

    Write-Host ""
    Write-ApsInfo "Updating APS v2 in $Target"
    Write-Host ""

    $apsDir = Join-Path $Target ".aps"

    # Vendored PowerShell CLI: refresh only when this project vendored one.
    if (Test-Path -LiteralPath (Join-Path (Join-Path $apsDir "bin") "aps.ps1") -PathType Leaf) {
        foreach ($f in $script:CliFilesPowerShell) {
            Invoke-ApsDownloadRoot -Source $f -Destination (Join-Path $apsDir $f)
        }
        Write-ApsInfo ".aps/bin/aps.ps1 + .aps/lib/ (CLI)"
    }

    # Plans (preserves user specs)
    Install-ApsPlansV2 -Target $Target
    Write-ApsInfo "plans/ (templates, rules)"

    # D-044: retire the legacy plans/.aps-version stamp on update.
    $apsVersionFile = Join-Path (Join-Path $Target "plans") ".aps-version"
    if (Test-Path -LiteralPath $apsVersionFile -PathType Leaf) {
        Remove-Item -LiteralPath $apsVersionFile -Force
        Write-ApsInfo "Removed legacy plans/.aps-version (superseded by .aps/config.yml)"
    }

    # Hook scripts: refresh only when installed.
    if (Test-Path -LiteralPath (Join-Path $apsDir "scripts") -PathType Container) {
        Install-ApsScriptsV2 -Target $Target
        Write-ApsInfo ".aps/scripts/ (hook scripts)"
    }

    # Read config.yml to determine which tool files to refresh.
    $config = Join-Path $apsDir "config.yml"
    $tools = @(Select-String -LiteralPath $config -Pattern '^\s*- name:\s*(\S+)' -AllMatches |
        ForEach-Object { $_.Matches } | ForEach-Object { $_.Groups[1].Value })
    foreach ($tool in $tools) {
        switch ($tool) {
            "claude-code" { Install-ApsSkillV2 -Target $Target; Install-ApsToolAgents -Target $Target -Tool "claude-code" }
            "copilot"     { Install-ApsSkillV2 -Target $Target; Install-ApsToolAgents -Target $Target -Tool "copilot" }
            "opencode"    { Install-ApsSkillV2 -Target $Target; Install-ApsToolAgents -Target $Target -Tool "opencode" }
            "codex"       { Install-ApsToolAgents -Target $Target -Tool "codex"; Install-ApsAgentsSkillV2 -Target $Target }
            "grok"        { Install-ApsAgentsSkillV2 -Target $Target }
            "antigravity" { Install-ApsAgentsSkillV2 -Target $Target }
        }
    }
    if ($tools.Count -gt 0) {
        Write-ApsInfo "Tool-specific files refreshed per config.yml"
    }

    # Update the updated timestamp.
    $today = Get-Date -Format "yyyy-MM-dd"
    $configText = Get-Content -LiteralPath $config -Raw
    $configText = $configText -replace 'updated:.*', "updated: `"$today`""
    Set-Content -LiteralPath $config -Value $configText -NoNewline

    Write-Host ""
    Write-ApsInfo "Your specs (index.aps.md, modules/*.aps.md) were NOT modified."
    Write-Host ""
}

# --- Subcommands ---

function Invoke-ApsInit {
    <#
    .SYNOPSIS
        Init workflow — scaffolds the v2 minimal layout (INSTALL-011 /
        INSTALL-023): plans templates + seed index + .aps/config.yml only.
        Hook scripts and tool skills are opt-in (--hooks / --tools); skills
        land under managed markers (D-042) so `aps update` can refresh them.
    #>
    param([string[]]$Arguments)
    $target = "."
    $optTools = $null
    $installHooks = $false
    # --non-interactive is accepted for bash-CLI flag parity; this port never
    # prompts, so non-interactive defaults (solo/small/generic) always apply.
    if ($Arguments) {
        for ($i = 0; $i -lt $Arguments.Count; $i++) {
            switch ($Arguments[$i]) {
                "--help"            { Show-ApsInitHelp; return }
                "-h"                { Show-ApsInitHelp; return }
                "--non-interactive" { }
                "--hooks"           { $installHooks = $true }
                "--tools" {
                    if ($i + 1 -ge $Arguments.Count) {
                        Write-ApsError "--tools requires a value (e.g. --tools claude-code)"
                        exit 1
                    }
                    $i++
                    $optTools = $Arguments[$i]
                }
                default { $target = $Arguments[$i] }
            }
        }
    }

    $toolNames = @("claude-code", "copilot", "codex", "opencode", "grok", "antigravity", "generic")
    $selectedTools = @("generic")
    if ($optTools) {
        $selectedTools = @($optTools -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ })
        foreach ($t in $selectedTools) {
            if ($t -ceq "gemini") {
                Write-ApsError "'gemini' was retired in v0.7 (D-040); supported tools: $($toolNames -join ' ')"
                exit 1
            }
            if ($toolNames -cnotcontains $t) {
                Write-ApsError "unknown tool '$t'; supported tools: $($toolNames -join ' ')"
                exit 1
            }
        }
    }

    $plansDir = Join-Path $target "plans"
    if (Test-Path -LiteralPath $plansDir -PathType Container) {
        Write-ApsError "plans/ directory already exists at $target"
        Write-Host ""
        Write-Host "To update an existing project:"
        Write-Host "  aps update"
        exit 1
    }

    Write-Host ""
    Write-ApsInfo "Initialising APS v2 in $target"
    Write-Host ""

    # Plans (always — the irreducible core of an APS project)
    Install-ApsPlansV2 -Target $target
    Install-ApsIndexV2 -Target $target
    Write-ApsInfo "plans/ (templates, rules, project-context, designs)"

    # Hook scripts — opt-in only.
    if ($installHooks) {
        Install-ApsScriptsV2 -Target $target
        Write-ApsInfo ".aps/scripts/ (hook scripts)"
    }

    # Tool-specific files (skills/agents) — installed only for selected tools
    foreach ($tool in $selectedTools) {
        switch ($tool) {
            "claude-code" {
                Install-ApsSkillV2 -Target $target
                Install-ApsToolAgents -Target $target -Tool "claude-code"
                Write-ApsInfo ".claude/skills/aps-planning/ (skill)"
                Write-ApsInfo ".claude/agents/ (planner, librarian, conductor)"
            }
            "copilot" {
                Install-ApsSkillV2 -Target $target
                Install-ApsToolAgents -Target $target -Tool "copilot"
                Write-ApsInfo ".claude/skills/aps-planning/ (skill — Copilot auto-discovers)"
                Write-ApsInfo ".github/agents/ (planner, librarian, conductor)"
            }
            "opencode" {
                Install-ApsSkillV2 -Target $target
                Install-ApsToolAgents -Target $target -Tool "opencode"
                Write-ApsInfo ".claude/skills/aps-planning/ (skill — OpenCode auto-discovers)"
                Write-ApsInfo ".opencode/agents/ (planner, librarian, conductor)"
            }
            "codex" {
                Install-ApsToolAgents -Target $target -Tool "codex"
                Install-ApsAgentsSkillV2 -Target $target
                Write-ApsInfo ".codex/agents/ (planner, librarian, conductor TOML configs)"
                Write-ApsInfo ".agents/skills/aps-planning/ (skill)"
            }
            "grok" {
                Install-ApsAgentsSkillV2 -Target $target
                Write-ApsInfo ".agents/skills/aps-planning/ (skill — Grok Build auto-discovers)"
            }
            "antigravity" {
                Install-ApsAgentsSkillV2 -Target $target
                Write-ApsInfo ".agents/skills/aps-planning/ (skill — Antigravity auto-discovers)"
            }
        }
    }

    # Config (always — the per-project contract read by `aps update`)
    Write-ApsConfigV2 -Target $target -Tools $selectedTools
    Write-ApsInfo ".aps/config.yml (install configuration)"

    # Print layout
    Write-Host ""
    Write-Host "  .aps/"
    Write-Host "  +-- config.yml                       <- Project contract (cli_version, paths)"
    if ($installHooks) {
        Write-Host "  +-- scripts/                         <- Hook scripts"
    }
    Write-Host ""
    Write-Host "  plans/"
    Write-Host "  +-- aps-rules.md                     <- Agent guidance (APS-managed)"
    Write-Host "  +-- project-context.md               <- Your project context (edit this)"
    Write-Host "  +-- index.aps.md                     <- Your main plan (edit this)"
    Write-Host "  +-- issues.md                        <- Issue & question tracker"
    Write-Host "  +-- modules/                         <- Module specs"
    Write-Host "  +-- execution/                       <- Action plans"
    Write-Host "  +-- decisions/                       <- ADRs"
    Write-Host "  +-- designs/                         <- Technical designs"

    Write-Host ""
    if ($selectedTools -ccontains "claude-code") {
        Write-ApsInfo "Next: point Claude Code at plans/aps-rules.md and edit plans/project-context.md"
    } else {
        Write-ApsInfo "Next: edit plans/project-context.md with your project details"
    }
    Write-Host ""
    Write-ApsInfo "This repo uses the global 'aps' binary. Add hooks, agents, or a"
    Write-Host "  vendored CLI later with: aps setup"
    Write-Host ""
}

function Show-ApsInitHelp {
    Write-Host @"
aps init - Create APS structure in a new project (v2 layout)

Usage:
  aps.ps1 init [target-dir] [options]

Creates minimal planning content (plans/ + .aps/config.yml). By default it
does NOT install hook scripts or tool skills — the global 'aps' binary on
PATH drives the repo. Opt in with --hooks / --tools, or add them later
with 'aps setup'.

Refuses to run if plans/ already exists.

Options:
  --tools TOOLS       Comma-separated: claude-code,copilot,codex,opencode,grok,antigravity,generic
                      (default: generic — no tool integration)
  --hooks             Also install hook scripts into .aps/scripts
  --non-interactive   Accepted for bash-CLI parity (this port never prompts)
  --help              Show this help

Environment:
  APS_VERSION   Git ref to download from (default: main)

Examples:
  .\bin\aps.ps1 init                             # Minimal layout in current directory
  .\bin\aps.ps1 init .\my-project                # Init in a subdirectory
  .\bin\aps.ps1 init --tools claude-code         # Also install the Claude Code skill + agents
  .\bin\aps.ps1 init --non-interactive --hooks   # Minimal layout + hook scripts
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

    # v2 layout (.aps/config.yml) gets the v2 update path; the legacy flow
    # below only ever runs for pre-.aps v1 installs.
    if (Test-ApsV2Layout -Target $target) {
        Update-ApsV2 -Target $target
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
    Write-ApsInfo "Updating APS v1 in $target"
    Write-ApsWarning "Consider migrating to the v2 layout: aps migrate (native binary)"
    Write-Host ""

    # CLI (always update -- this is how users get new features)
    Install-ApsCli -Target $target
    Write-ApsInfo "bin/aps.ps1 + lib/ (CLI)"

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

    foreach ($f in $script:CliFilesPowerShell) {
        Invoke-ApsDownloadRoot -Source $f -Destination (Join-Path $ApsHome $f)
    }

    Write-Host ""
    Write-ApsInfo "Global update complete"
    Write-ApsInfo "bin/aps.ps1 + lib/ updated at $ApsHome"
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
    'Install-ApsSkill'
    'Install-ApsCommands'
    'Install-ApsCli'
    'Update-ApsGlobal'
    'Invoke-ApsInit'
    'Invoke-ApsUpdate'
    'Test-ApsV2Layout'
    'Update-ApsV2'
    'Get-ApsManagedManifestJson'
    'Get-ApsSkillPayload'
    'Get-ApsSkillDirState'
    'Invoke-ApsSkillReconcile'
    'Install-ApsManagedSkill'
)
