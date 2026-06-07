#
# APS CLI Work Item Validation Rules
# Port of lib/rules/workitem.sh
#

# Dependencies (Output, Common) must be imported by the entry point.

# E005: Missing required work item fields (Intent, Expected Outcome, Validation)
function Test-E005RequiredFields {
    param([string]$File, [string]$ItemHeader, [int]$ItemLine)
    $hasErrors = $false
    $content = Get-ApsWorkItemContent -FilePath $File -StartLine $ItemLine
    $contentText = $content -join "`n"

    # Terminal (completed) work items are commonly compacted to Status + a short
    # summary once shipped, with full Intent/Expected Outcome/Validation detail
    # preserved in version history. Exempt them from the required-field checks.
    # Active states (Proposed/Ready/In Progress/Blocked/Draft/Deferred) are still checked.
    if ($contentText -match '(?im)^- \*\*Status:\*\*[ \t]*(done|complete|merged|released|shipped)\b') {
        return $true
    }

    if ($contentText -cnotmatch '(?m)^- \*\*Intent:\*\*') {
        Add-ApsResult -Path $File -Type "error" -Code "E005" -Message "$ItemHeader`: Missing **Intent:** field" -Line "$ItemLine"
        $hasErrors = $true
    }
    if ($contentText -cnotmatch '(?m)^- \*\*Expected Outcome:\*\*') {
        Add-ApsResult -Path $File -Type "error" -Code "E005" -Message "$ItemHeader`: Missing **Expected Outcome:** field" -Line "$ItemLine"
        $hasErrors = $true
    }
    if ($contentText -cnotmatch '(?m)^- \*\*Validation:\*\*') {
        Add-ApsResult -Path $File -Type "error" -Code "E005" -Message "$ItemHeader`: Missing **Validation:** field" -Line "$ItemLine"
        $hasErrors = $true
    }
    return (-not $hasErrors)
}

# W001: Work item ID format check
function Test-W001IdFormat {
    param([string]$File, [string]$ItemHeader, [int]$ItemLine)
    if ($ItemHeader -match '^### ([A-Za-z0-9-]+):') {
        $itemId = $Matches[1]
        if ($itemId -cnotmatch '^[A-Z]+-[0-9]{3}$') {
            Add-ApsResult -Path $File -Type "warning" -Code "W001" `
                -Message "Work item ID '$itemId' should match pattern PREFIX-NNN (e.g., AUTH-001)" -Line "$ItemLine"
        }
    }
}

# W003: Dependency references unknown task ID
# Resolves in-file first, then against the plan-tree index (cross-module
# dependencies and decision references are legitimate).
function Test-W003Dependencies {
    param([string]$File, [int]$ItemLine, [string[]]$AllIds, [string[]]$TreeIds = @())
    $content = Get-ApsWorkItemContent -FilePath $File -StartLine $ItemLine

    foreach ($line in $content) {
        if ($line -match '^- \*\*Dependencies:\*\*') {
            $depIds = [regex]::Matches($line, '[A-Z]+-[0-9]{3}')
            foreach ($dep in $depIds) {
                if ($dep.Value -notin $AllIds -and $dep.Value -notin $TreeIds) {
                    $depLine = Get-ApsLineNumber -FilePath $File -Pattern "Dependencies:.*$([regex]::Escape($dep.Value))"
                    Add-ApsResult -Path $File -Type "warning" -Code "W003" `
                        -Message "Dependency '$($dep.Value)' not found in plan" -Line "$depLine"
                }
            }
            break
        }
    }
}

# W018: Terminal work item missing Validation in an active module
#
# E005 deliberately exempts terminal items from required fields (closeout
# compaction). But a Complete item with no Validation in a module that is
# still active is exactly the "overstated completion" risk DOGFOOD-002
# targets — the audit cannot verify it. Warning only; fully Complete modules
# are archives and stay exempt.
function Test-W018TerminalValidation {
    param([string]$File, [string]$ItemHeader, [int]$ItemLine, [string]$ModuleStatus)

    # Skip when the whole module is terminal (archive compaction is sanctioned)
    if ($ModuleStatus -match '(?i)^(done|complete|merged|released|shipped|archived)') {
        return
    }

    $content = Get-ApsWorkItemContent -FilePath $File -StartLine $ItemLine
    $contentText = $content -join "`n"

    # Terminal status: an explicit Status field is authoritative; the
    # "— Complete <date>" header suffix only counts when no field is present
    $terminal = $false
    if ($contentText -match '(?im)^- \*\*Status:\*\*[ \t]*(\S.*)$') {
        if ($Matches[1] -match '(?i)^(done|complete|merged|released|shipped)\b') {
            $terminal = $true
        }
    } elseif ($ItemHeader -match '(?i)(—|--) *(done|complete|merged|released|shipped)\b') {
        $terminal = $true
    }
    if (-not $terminal) { return }

    if ($contentText -cnotmatch '(?m)^- \*\*Validation') {
        Add-ApsResult -Path $File -Type "warning" -Code "W018" `
            -Message "$ItemHeader`: Complete item has no Validation — completion cannot be audited" -Line "$ItemLine"
    }
}

# Lint all work items in a file
function Invoke-ApsWorkItemLint {
    param([string]$File, [string[]]$TreeIds = @())
    $hasErrors = $false

    # Collect all work item IDs for dependency checking
    $lines = Get-Content -LiteralPath $File -ErrorAction SilentlyContinue
    $allIds = @()
    if ($lines) {
        foreach ($line in $lines) {
            if ($line -match '^### ([A-Z]+-[0-9]+):') {
                $allIds += $Matches[1]
            }
        }
    }

    # Module status gates W018 (terminal modules are exempt archives)
    $moduleStatus = Get-ApsStatus -FilePath $File

    $items = Get-ApsWorkItems -FilePath $File
    foreach ($item in $items) {
        Test-W001IdFormat -File $File -ItemHeader $item.Header -ItemLine $item.LineNumber
        if (-not (Test-E005RequiredFields -File $File -ItemHeader $item.Header -ItemLine $item.LineNumber)) {
            $hasErrors = $true
        }
        Test-W003Dependencies -File $File -ItemLine $item.LineNumber -AllIds $allIds -TreeIds $TreeIds
        Test-W018TerminalValidation -File $File -ItemHeader $item.Header -ItemLine $item.LineNumber -ModuleStatus $moduleStatus
    }

    return (-not $hasErrors)
}

Export-ModuleMember -Function 'Invoke-ApsWorkItemLint'
