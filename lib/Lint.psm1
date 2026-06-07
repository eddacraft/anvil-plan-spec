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
        foreach ($line in $lines) {
            # Work item headers: ### AUTH-001: title
            if ($line -match '^### ([A-Za-z]+-[0-9]+):') { $null = $ids.Add($Matches[1]) }
            # Decision entries: - **D-026:** text
            if ($line -match '^- \*\*(D-[0-9]+):') { $null = $ids.Add($Matches[1]) }
        }
    }
    return @($ids)
}

function Invoke-ApsFileLint {
    param([string]$File, [string[]]$TreeIds = @())
    $fileType = Get-ApsFileType -FilePath $File
    Set-ApsFileType -Path $File -FileType $fileType
    Add-ApsFileCount

    switch ($fileType) {
        "index"    { return (Invoke-ApsIndexLint -File $File) }
        "module"   { return (Invoke-ApsModuleLint -File $File -TreeIds $TreeIds) }
        "simple"   { return (Invoke-ApsModuleLint -File $File -TreeIds $TreeIds) }
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

    # Lint each file
    foreach ($file in $files) {
        $null = Invoke-ApsFileLint -File $file -TreeIds $treeIds

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
