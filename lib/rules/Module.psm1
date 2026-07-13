#
# APS CLI Module/Simple Validation Rules
# Port of lib/rules/module.sh
#

# Dependencies (Output, Common, WorkItem) must be imported by the entry point.

# E001: Missing ## Purpose section
function Test-E001Purpose {
    param([string]$File)
    if (-not (Test-ApsSection -FilePath $File -SectionHeader "## Purpose")) {
        Add-ApsResult -Path $File -Type "error" -Code "E001" -Message "Missing ## Purpose section"
        return $false
    }
    return $true
}

# E002: Missing ## Work Items section
function Test-E002WorkItems {
    param([string]$File)
    if (-not (Test-ApsSection -FilePath $File -SectionHeader "## Work Items")) {
        Add-ApsResult -Path $File -Type "error" -Code "E002" -Message "Missing ## Work Items section"
        return $false
    }
    return $true
}

# E003: Missing ID/Status metadata table
function Test-E003Metadata {
    param([string]$File)
    if (-not (Test-ApsMetadataTable -FilePath $File)) {
        Add-ApsResult -Path $File -Type "error" -Code "E003" -Message "Missing ID/Status metadata table"
        return $false
    }
    return $true
}

# W004: Empty section check (module-specific sections)
function Test-W004EmptySectionsModule {
    param([string]$File)
    $sections = @("## Purpose", "## In Scope")
    foreach ($section in $sections) {
        if ((Test-ApsSection -FilePath $File -SectionHeader $section) -and
            -not (Test-ApsSectionHasContent -FilePath $File -SectionHeader $section)) {
            $line = Get-ApsLineNumber -FilePath $File -Pattern "^$([regex]::Escape($section))$"
            Add-ApsResult -Path $File -Type "warning" -Code "W004" -Message "Empty section: $section" -Line "$line"
        }
    }
}

# W017: Active module missing or stale Last reviewed: field
#
# Modules that are Ready or In Progress should carry
# `**Last reviewed:** YYYY-MM-DD` near the top so staleness is detectable.
# Threshold configurable via APS_STALE_DAYS (default 60).
function Test-W017LastReviewed {
    param([string]$File)
    $status = Get-ApsStatus -FilePath $File

    # Only active modules are required to be fresh
    if ($status -notmatch '(?i)^(ready|in progress)') { return }

    $lines = Get-Content -LiteralPath $File -ErrorAction SilentlyContinue
    $reviewed = $null
    foreach ($line in $lines) {
        if ($line -match '^\*\*Last reviewed:\*\* *(\d{4}-\d{2}-\d{2})') {
            $reviewed = $Matches[1]
            break
        }
    }

    if (-not $reviewed) {
        Add-ApsResult -Path $File -Type "warning" -Code "W017" `
            -Message "Active module has no **Last reviewed:** field"
        return
    }

    $staleDays = 60
    if ($env:APS_STALE_DAYS -match '^\d+$') { $staleDays = [int]$env:APS_STALE_DAYS }

    try {
        $reviewedDate = [datetime]::ParseExact($reviewed, 'yyyy-MM-dd', $null)
    } catch {
        return
    }
    # Floor, not round: bash truncates ((now-reviewed)/86400) and Rust counts
    # whole civil days, so `[int]` (which rounds to nearest) would report one
    # day more than the other two CLIs at some times of day (D-038/D-039).
    $ageDays = [int][math]::Floor(((Get-Date) - $reviewedDate).TotalDays)
    if ($ageDays -gt $staleDays) {
        $line = Get-ApsLineNumber -FilePath $File -Pattern '^\*\*Last reviewed:\*\*'
        Add-ApsResult -Path $File -Type "warning" -Code "W017" `
            -Message "Last reviewed $reviewed is $ageDays days old (threshold: $staleDays)" -Line "$line"
    }
}

# W002: a conductor module's coordination sections reference a work-item ID
# that resolves nowhere in the plan tree — most likely a typo. Conductor
# modules legitimately reference IDs owned by other modules (that is the point),
# so only unresolved refs are flagged, and only for `Type: Conductor` modules.
# Mirrors the Rust check_w002_conductor_refs / bash check_w002_conductor_refs.
function Test-W002ConductorRefs {
    param([string]$File, [string[]]$TreeIds = @())
    if (-not (Test-ApsConductor -FilePath $File)) { return }

    $treeSet = [System.Collections.Generic.HashSet[string]]::new()
    foreach ($id in $TreeIds) { $null = $treeSet.Add($id) }

    $lines = Get-Content -LiteralPath $File -ErrorAction SilentlyContinue
    if (-not $lines) { return }

    $sections = @("## Coordinated Modules", "## Cross-Module Work Items")
    foreach ($section in $sections) {
        $inSection = $false
        $inComment = $false
        for ($i = 0; $i -lt $lines.Count; $i++) {
            $line = $lines[$i]
            if (-not $inSection) {
                if ($line -ceq $section) { $inSection = $true }
                continue
            }
            if ($line -match '^## ') { break }
            $trimmed = $line.TrimStart()
            if ($inComment) {
                if ($trimmed -match '-->') { $inComment = $false }
                continue
            }
            if ($trimmed -match '^<!--') {
                if ($trimmed -notmatch '-->') { $inComment = $true }
                continue
            }
            foreach ($m in [regex]::Matches($line, '[A-Z]+-[0-9]{3}')) {
                $id = $m.Value
                if (-not $treeSet.Contains($id)) {
                    Add-ApsResult -Path $File -Type "warning" -Code "W002" `
                        -Message "Cross-module reference '$id' not found in plan tree" -Line "$($i + 1)"
                }
            }
        }
    }
}

# W022: a `Packages:` scope tag (metadata-table column or work-item field)
# that resolves to no directory in the workspace — most likely a typo
# (PKG-002, the tagged-monorepo analogue of W002/W006). Resolution tries the
# entry as given plus the conventional packages/<entry> and apps/<entry>
# roots. Silent when the workspace has no packages/ or apps/ directory, so
# single-package projects never pay for the check. Mirrors the Rust/bash
# check_w022_packages.
function Test-W022Packages {
    param([string]$File)

    # Workspace root: the path prefix before the last plans component.
    $norm = $File -replace '\\', '/'
    $idx = $norm.LastIndexOf('/plans/')
    if ($idx -gt 0) {
        $root = $norm.Substring(0, $idx)
    } elseif ($norm -match '^plans/') {
        $root = '.'
    } else {
        return
    }
    if (-not (Test-Path -LiteralPath (Join-Path $root 'packages') -PathType Container) -and
        -not (Test-Path -LiteralPath (Join-Path $root 'apps') -PathType Container)) {
        return
    }

    $lines = Get-Content -LiteralPath $File -ErrorAction SilentlyContinue
    if (-not $lines) { return }

    # Collect (line, value) pairs: the metadata table's first data row in the
    # Packages column (mirrors Get-ApsModuleType semantics), plus every
    # `- **Packages:**` work-item field.
    $found = [System.Collections.ArrayList]::new()
    $pkgCol = -1
    $headerSeen = $false
    $tableDone = $false
    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        if (-not $tableDone) {
            $isIdHeader = $line -match '^\| *ID *\|'
            if (-not $headerSeen) {
                if ($isIdHeader) {
                    $cols = ($line -split '\|') | ForEach-Object { $_.Trim() }
                    for ($j = 0; $j -lt $cols.Count; $j++) {
                        if ($cols[$j] -ceq 'Packages') { $pkgCol = $j }
                    }
                    $headerSeen = $true
                    continue
                }
            } elseif ($line -match '^\|') {
                if ($line -match '^[|: -]+$' -or $isIdHeader) { continue }
                if ($pkgCol -ge 0) {
                    $vals = ($line -split '\|') | ForEach-Object { $_.Trim() }
                    if ($pkgCol -lt $vals.Count -and $vals[$pkgCol] -ne '') {
                        $null = $found.Add(@(($i + 1), $vals[$pkgCol]))
                    }
                }
                $tableDone = $true
            }
        }
        if ($line -match '^\s*- \*\*Packages:\*\*\s*(.*)$' -and $Matches[1] -ne '') {
            $null = $found.Add(@(($i + 1), $Matches[1]))
        }
    }

    foreach ($pair in $found) {
        $lineNum = $pair[0]
        foreach ($raw in ($pair[1] -split ',')) {
            # Trim whitespace and backticks; skip prose placeholders like
            # _(monorepo only)_ (anything outside [A-Za-z0-9@._/-]).
            $entry = $raw.Trim().Trim('`').Trim()
            if ($entry -eq '') { continue }
            if ($entry -notmatch '^[A-Za-z0-9@._/-]+$') { continue }
            $resolves = (Test-Path -LiteralPath (Join-Path $root $entry) -PathType Container) -or
                (Test-Path -LiteralPath (Join-Path (Join-Path $root 'packages') $entry) -PathType Container) -or
                (Test-Path -LiteralPath (Join-Path (Join-Path $root 'apps') $entry) -PathType Container)
            if (-not $resolves) {
                Add-ApsResult -Path $File -Type "warning" -Code "W022" `
                    -Message "Packages entry '$entry' does not resolve to a workspace directory" -Line "$lineNum"
            }
        }
    }
}

# W005: Status=Ready but no work items
function Test-W005ReadyNoItems {
    param([string]$File)
    $status = Get-ApsStatus -FilePath $File
    if ($status -ceq "Ready") {
        $items = Get-ApsWorkItems -FilePath $File
        if ($items.Count -eq 0) {
            Add-ApsResult -Path $File -Type "warning" -Code "W005" -Message "Status is Ready but no work items defined"
        }
    }
}

# Run all module/simple rules
function Invoke-ApsModuleLint {
    param([string]$File, [string[]]$TreeIds = @(), [hashtable]$ChildIds = @{})
    $hasErrors = $false

    if (-not (Test-E001Purpose -File $File)) { $hasErrors = $true }
    if (-not (Test-E002WorkItems -File $File)) { $hasErrors = $true }
    if (-not (Test-E003Metadata -File $File)) { $hasErrors = $true }

    Test-W004EmptySectionsModule -File $File
    Test-W005ReadyNoItems -File $File
    # W017 then W002 then W022 — mirror the Rust lint_module call order so
    # byte-level diffs of lint output stay identical across all three CLIs
    # (D-038/D-039).
    Test-W017LastReviewed -File $File
    Test-W002ConductorRefs -File $File -TreeIds $TreeIds
    Test-W022Packages -File $File

    if (Test-ApsSection -FilePath $File -SectionHeader "## Work Items") {
        if (-not (Invoke-ApsWorkItemLint -File $File -TreeIds $TreeIds -ChildIds $ChildIds)) { $hasErrors = $true }
    }

    return (-not $hasErrors)
}

Export-ModuleMember -Function 'Invoke-ApsModuleLint'
