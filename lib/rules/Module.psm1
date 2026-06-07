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
    $ageDays = [int]((Get-Date) - $reviewedDate).TotalDays
    if ($ageDays -gt $staleDays) {
        $line = Get-ApsLineNumber -FilePath $File -Pattern '^\*\*Last reviewed:\*\*'
        Add-ApsResult -Path $File -Type "warning" -Code "W017" `
            -Message "Last reviewed $reviewed is $ageDays days old (threshold: $staleDays)" -Line "$line"
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
    param([string]$File, [string[]]$TreeIds = @())
    $hasErrors = $false

    if (-not (Test-E001Purpose -File $File)) { $hasErrors = $true }
    if (-not (Test-E002WorkItems -File $File)) { $hasErrors = $true }
    if (-not (Test-E003Metadata -File $File)) { $hasErrors = $true }

    Test-W004EmptySectionsModule -File $File
    Test-W005ReadyNoItems -File $File
    Test-W017LastReviewed -File $File

    if (Test-ApsSection -FilePath $File -SectionHeader "## Work Items") {
        if (-not (Invoke-ApsWorkItemLint -File $File -TreeIds $TreeIds)) { $hasErrors = $true }
    }

    return (-not $hasErrors)
}

Export-ModuleMember -Function 'Invoke-ApsModuleLint'
