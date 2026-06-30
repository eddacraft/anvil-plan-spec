#
# APS CLI Core Linting Logic
# Port of lib/lint.sh — file discovery, type detection, dispatch
#

# Dependencies (Output, Common, rules/*) must be imported by the entry point
# before this module is loaded. See bin/aps.ps1.

function Get-ApsFileType {
    param([string]$FilePath)
    $name = Split-Path $FilePath -Leaf
    $dir = Split-Path $FilePath -Parent

    # Skip template files (dotfiles)
    if ($name.StartsWith('.')) { return "template" }

    # Index files
    if ($name -eq "index.aps.md") { return "index" }

    # Completed-work archive (parallel to index.aps.md)
    if ($name -eq "completed.aps.md") { return "archive" }

    # Issues tracker
    if ($name -eq "issues.md") { return "issues" }

    # Design files (in designs/ directory)
    if ($FilePath -match '(^|[/\\])designs([/\\])' -and $name -match '\.design\.md$') { return "design" }

    # Actions files
    if ($FilePath -match '[/\\]execution[/\\]' -and $name -match '\.actions\.md$') { return "actions" }

    # Module files
    if ($dir -match '[/\\]modules($|[/\\])') { return "module" }

    # Simple for other .aps.md
    if ($name -match '\.aps\.md$') { return "simple" }

    return "unknown"
}

function Find-ApsFiles {
    param([string]$Directory)
    Get-ChildItem -Path $Directory -Recurse -File -ErrorAction SilentlyContinue |
        Where-Object {
            -not $_.Name.StartsWith('.') -and
            ($_.Name -match '\.aps\.md$' -or $_.Name -match '\.actions\.md$' -or $_.Name -match '\.design\.md$' -or $_.Name -eq 'issues.md')
        } |
        Sort-Object FullName |
        ForEach-Object { $_.FullName }
}

# Cross-file ID index: work item and decision IDs from the whole plan tree.
# W003 resolves dependencies against this when the in-file check misses.
function Build-ApsIdIndex {
    param([string[]]$Files)
    $ids = [System.Collections.Generic.HashSet[string]]::new()
    foreach ($f in $Files) {
        $lines = Get-Content -LiteralPath $f -ErrorAction SilentlyContinue
        # Fence-aware: IDs inside code blocks are examples, not definitions
        $fence = $false
        foreach ($line in $lines) {
            if ($line -match '^(```|~~~)') { $fence = -not $fence; continue }
            if ($fence) { continue }
            # Work item headers: ### AUTH-001: title
            if ($line -match '^### ([A-Za-z]+-[0-9]+):') { $null = $ids.Add($Matches[1]) }
            # Decision entries: - **D-026:** text
            if ($line -match '^- \*\*(D-[0-9]+):') { $null = $ids.Add($Matches[1]) }
        }
    }
    return @($ids)
}

# Lexically normalise a path: collapse '.' and '..' without touching the
# filesystem, so child-plan links (joined onto a parent dir) dedupe cleanly
# against recursively-found paths. .NET's GetFullPath does the collapse and
# yields an absolute path, matching the absolute paths Find-ApsFiles returns.
# (MONO-002 — port of normalize_path in lib/lint.sh)
function Resolve-ApsPath {
    param([string]$Path)
    return [System.IO.Path]::GetFullPath($Path)
}

# Emit child-plan index paths declared in a parent index's "## Child Plans"
# section. Each list item links a child index.aps.md relative to the parent;
# paths are resolved against the parent dir and normalised. (MONO-002 — port
# of resolve_child_plan_links)
function Resolve-ApsChildPlanLinks {
    param([string]$IndexFile)
    $lines = Get-Content -LiteralPath $IndexFile -ErrorAction SilentlyContinue
    if (-not $lines) { return @() }
    $dir = Split-Path -Parent $IndexFile
    $inSection = $false
    $results = [System.Collections.ArrayList]::new()
    foreach ($line in $lines) {
        if ($line -match '^## ') {
            $inSection = ($line -match '^## Child Plans[ \t]*$')
            continue
        }
        if ($inSection -and $line -match '\]\(([^)]+)\)') {
            $link = $Matches[1]
            if ([string]::IsNullOrEmpty($link)) { continue }
            $resolved = Resolve-ApsPath (Join-Path $dir $link)
            if (Test-Path -LiteralPath $resolved -PathType Leaf) {
                $null = $results.Add($resolved)
            }
        }
    }
    return @($results)
}

# Expand a file list by following ## Child Plans links transitively, deduped on
# normalised paths. A federated parent root thus validates its child plan trees
# as one plan, even though children live outside the parent dir. (MONO-002 —
# port of expand_child_plans)
function Expand-ApsChildPlans {
    param([string[]]$Files)
    $seen = [System.Collections.Generic.HashSet[string]]::new()
    $result = [System.Collections.ArrayList]::new()
    foreach ($f in $Files) {
        if ($seen.Add((Resolve-ApsPath $f))) { $null = $result.Add($f) }
    }

    $queue = [System.Collections.Generic.Queue[string]]::new()
    foreach ($r in $result) { $queue.Enqueue($r) }
    while ($queue.Count -gt 0) {
        $current = $queue.Dequeue()
        if ((Split-Path -Leaf $current) -ne 'index.aps.md') { continue }
        foreach ($childIndex in (Resolve-ApsChildPlanLinks -IndexFile $current)) {
            if ([string]::IsNullOrEmpty($childIndex)) { continue }
            foreach ($cf in (Find-ApsFiles -Directory (Split-Path -Parent $childIndex))) {
                if ($seen.Add((Resolve-ApsPath $cf))) {
                    $null = $result.Add($cf)
                    $queue.Enqueue($cf)
                }
            }
        }
    }
    return @($result)
}

# Per-child work-item ID registry for cross-tree (`<name>:<ID>`) resolution.
# Keyed by path-derived child name (the segment above a child's plans/ dir).
# W003 reads this to validate prefixed deps; empty when no child trees are in
# scope (a child linted alone), which is how isolated cross-tree refs stay
# silent. Returns a hashtable name -> string[] of IDs. (MONO-002 — port of
# build_child_registry + the APS_CHILD_IDS global)
function Build-ApsChildRegistry {
    param([string[]]$Files)
    $registry = @{}
    foreach ($f in $Files) {
        if ((Split-Path -Leaf $f) -ne 'index.aps.md') { continue }
        $root = Split-Path -Parent $f                       # .../<name>/plans
        $name = Split-Path -Leaf (Split-Path -Parent $root) # <name>
        if ([string]::IsNullOrEmpty($name) -or $name -eq '.' -or $name -eq '/') { continue }

        # This root's own files (index + modules), tolerating an absent modules/
        $rootFiles = Get-ChildItem -Path $root -Recurse -Depth 1 -File -Filter '*.aps.md' -ErrorAction SilentlyContinue
        if (-not $rootFiles) { continue }

        if (-not $registry.ContainsKey($name)) {
            $registry[$name] = [System.Collections.Generic.HashSet[string]]::new()
        }
        foreach ($rf in $rootFiles) {
            $lines = Get-Content -LiteralPath $rf.FullName -ErrorAction SilentlyContinue
            $fence = $false
            foreach ($line in $lines) {
                if ($line -match '^(```|~~~)') { $fence = -not $fence; continue }
                if ($fence) { continue }
                if ($line -match '^### ([A-Za-z]+-[0-9]+):') { $null = $registry[$name].Add($Matches[1]) }
            }
        }
    }
    $out = @{}
    foreach ($k in $registry.Keys) { $out[$k] = @($registry[$k]) }
    return $out
}

# W020: the same work-item ID is defined in more than one child tree. Per D-002
# IDs are bare per tree and may legitimately collide across trees, so this is a
# warning (each tree stays independently valid) — but it makes cross-tree refs
# ambiguous, so surface it. Only fires when a federation parent (a
# `## Child Plans` index) is in scope. (MONO-002 — port of
# check_cross_tree_collisions)
function Test-ApsCrossTreeCollisions {
    param([string[]]$Files, [hashtable]$ChildIds)
    if ($ChildIds.Count -eq 0) { return }

    # Attach warnings to the federation parent (the index declaring children).
    $parentFile = $null
    foreach ($f in $Files) {
        if ((Split-Path -Leaf $f) -ne 'index.aps.md') { continue }
        $lines = Get-Content -LiteralPath $f -ErrorAction SilentlyContinue
        if ($lines | Where-Object { $_ -match '^## Child Plans[ \t]*$' }) {
            $parentFile = $f
            break
        }
    }
    if (-not $parentFile) { return }

    # Map each ID to the distinct child names that define it.
    $idOwners = @{}
    foreach ($name in $ChildIds.Keys) {
        foreach ($id in $ChildIds[$name]) {
            if ([string]::IsNullOrEmpty($id)) { continue }
            if (-not $idOwners.ContainsKey($id)) {
                $idOwners[$id] = [System.Collections.Generic.List[string]]::new()
            }
            if (-not $idOwners[$id].Contains($name)) { $idOwners[$id].Add($name) }
        }
    }

    foreach ($id in $idOwners.Keys) {
        if ($idOwners[$id].Count -gt 1) {
            $owners = ($idOwners[$id] -join ' ')
            Add-ApsResult -Path $parentFile -Type "warning" -Code "W020" `
                -Message "Work-item ID '$id' defined in multiple child trees: $owners"
        }
    }
}

function Invoke-ApsFileLint {
    param([string]$File, [string[]]$TreeIds = @(), [hashtable]$ChildIds = @{})
    $fileType = Get-ApsFileType -FilePath $File
    Set-ApsFileType -Path $File -FileType $fileType
    Add-ApsFileCount

    switch ($fileType) {
        "index"    { return (Invoke-ApsIndexLint -File $File) }
        "module"   { return (Invoke-ApsModuleLint -File $File -TreeIds $TreeIds -ChildIds $ChildIds) }
        "simple"   { return (Invoke-ApsModuleLint -File $File -TreeIds $TreeIds -ChildIds $ChildIds) }
        "issues"   { return (Invoke-ApsIssuesLint -File $File) }
        "design"   { return (Invoke-ApsDesignLint -File $File) }
        "actions"  { return $true }
        "archive"  { return $true }
        "template" { return $true }
        default {
            Add-ApsResult -Path $File -Type "warning" -Code "W000" -Message "Unknown file type, skipping validation"
            return $true
        }
    }
}

function Invoke-ApsLint {
    param(
        [string]$Target = "plans",
        [switch]$JsonOutput
    )

    if (-not (Test-Path -LiteralPath $Target)) {
        Write-ApsError "Path not found: $Target"
        return $false
    }

    Reset-ApsResults

    # Collect files
    $files = @()
    if (Test-Path -LiteralPath $Target -PathType Leaf) {
        $files = @($Target)
    } else {
        $files = @(Find-ApsFiles -Directory $Target)

        # Also scan designs/ when the target is specifically plans/
        if ($Target -eq "plans" -or $Target -eq "plans/" -or $Target -eq "plans\") {
            if (Test-Path -LiteralPath "designs" -PathType Container) {
                $files += @(Find-ApsFiles -Directory "designs")
            }
        }

        # MONO-002: follow ## Child Plans links so a federated parent root
        # validates its child plan trees as one plan (children live outside
        # the parent dir).
        $files = @(Expand-ApsChildPlans -Files $files)
    }

    if ($files.Count -eq 0) {
        Write-ApsError "No APS files found in: $Target"
        return $false
    }

    # Build the cross-file ID index. For a single-file target, widen the index
    # to the surrounding plan tree so cross-module dependencies still resolve.
    $indexFiles = @($files)
    if (Test-Path -LiteralPath $Target -PathType Leaf) {
        $tdir = (Resolve-Path (Split-Path $Target -Parent)).Path
        $troot = $tdir
        if ((Split-Path $tdir -Leaf) -eq "modules") { $troot = Split-Path $tdir -Parent }
        $indexFiles += @(Find-ApsFiles -Directory $troot)
    }
    $treeIds = Build-ApsIdIndex -Files $indexFiles

    # MONO-002: per-child ID registry feeds prefix-aware W003; W020 surfaces
    # cross-tree ID collisions when a federation parent is in scope.
    $childIds = Build-ApsChildRegistry -Files $files
    Test-ApsCrossTreeCollisions -Files $files -ChildIds $childIds

    # Lint each file
    foreach ($file in $files) {
        $null = Invoke-ApsFileLint -File $file -TreeIds $treeIds -ChildIds $childIds

        # Mark file as valid if no issues were added
        if (-not (Test-ApsFileHasResults -Path $file)) {
            Add-ApsResult -Path $file -Type "ok" -Code "OK" -Message "" -Line ""
        }
    }

    # Output results
    if ($JsonOutput) {
        Write-ApsJsonResults
    } else {
        Write-ApsTextResults
    }

    return ((Get-ApsTotalErrors) -eq 0)
}

Export-ModuleMember -Function 'Invoke-ApsLint'
