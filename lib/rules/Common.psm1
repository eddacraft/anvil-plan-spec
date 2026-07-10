#
# APS CLI Common Validation Helpers
# Port of lib/rules/common.sh — section detection, metadata parsing
#

function Test-ApsSection {
    param([string]$FilePath, [string]$SectionHeader)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return $false }
    foreach ($line in $lines) {
        if ($line -ceq $SectionHeader) { return $true }
    }
    return $false
}

function Test-ApsMetadataTable {
    param([string]$FilePath)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return $false }
    $limit = [Math]::Min(20, $lines.Count)
    for ($i = 0; $i -lt $limit; $i++) {
        if ($lines[$i] -match '^\| *ID *\|') { return $true }
    }
    return $false
}

function Get-ApsSectionContent {
    param([string]$FilePath, [string]$SectionHeader)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return @() }
    $found = $false
    $content = [System.Collections.ArrayList]::new()
    foreach ($line in $lines) {
        if ($line -ceq $SectionHeader) {
            $found = $true
            continue
        }
        if ($found -and $line -match '^## ') { break }
        if ($found) { $null = $content.Add($line) }
    }
    return @($content)
}

function Test-ApsSectionHasContent {
    param([string]$FilePath, [string]$SectionHeader)
    $content = Get-ApsSectionContent -FilePath $FilePath -SectionHeader $SectionHeader
    foreach ($line in $content) {
        $trimmed = $line.Trim()
        if ($trimmed -eq "") { continue }
        if ($trimmed -match '^<!--.*-->$') { continue }
        if ($trimmed -match '^<!--') { continue }
        return $true
    }
    return $false
}

function Get-ApsWorkItems {
    param([string]$FilePath)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return @() }
    $results = [System.Collections.ArrayList]::new()
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^### [A-Za-z]+-[0-9]+:') {
            $null = $results.Add([PSCustomObject]@{
                LineNumber = $i + 1
                Header     = $lines[$i]
            })
        }
    }
    return @($results)
}

function Get-ApsModuleId {
    param([string]$FilePath)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return "" }
    $foundHeader = $false
    foreach ($line in $lines) {
        if ($line -match '^\| *ID *\|') {
            $foundHeader = $true
            continue
        }
        if ($foundHeader -and $line -match '^\|') {
            if ($line -match '^\| *-+') { continue }
            $cells = ($line -split '\|') | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne '' }
            if ($cells.Count -gt 0) { return $cells[0] }
        }
    }
    return ""
}

function Get-ApsStatus {
    param([string]$FilePath)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return "" }
    $statusCol = -1
    $foundHeader = $false
    foreach ($line in $lines) {
        if ($line -match '^\| *ID *\|') {
            $cols = ($line -split '\|') | ForEach-Object { $_.Trim() }
            for ($i = 0; $i -lt $cols.Count; $i++) {
                if ($cols[$i] -ceq "Status") { $statusCol = $i }
            }
            $foundHeader = $true
            continue
        }
        # Skip the header row and the `| --- | --- |` separator (which may carry
        # a space after the leading pipe). The prior `^\|[^-]` guard mis-read a
        # spaced separator as the data row and returned "------" as the status,
        # so W005/W017/W018 status gating never matched in PowerShell.
        if ($foundHeader -and $statusCol -ge 0 -and $line -match '^\|' -and
            $line -notmatch '^\| *ID *\|' -and $line -notmatch '^[|: -]+$') {
            $vals = ($line -split '\|') | ForEach-Object { $_.Trim() }
            if ($statusCol -lt $vals.Count) { return $vals[$statusCol] }
        }
    }
    return ""
}

# Extract the `Type` column value from the metadata table, or "".
# Mirrors the Rust module_type() / bash get_module_type: find the `| ID |`
# header row, locate the `Type` column, then read the first data row's value.
function Get-ApsModuleType {
    param([string]$FilePath)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return "" }
    $typeCol = -1
    $foundHeader = $false
    foreach ($line in $lines) {
        if (-not $foundHeader -and $line -match '^\| *ID *\|') {
            $cols = ($line -split '\|') | ForEach-Object { $_.Trim() }
            for ($i = 0; $i -lt $cols.Count; $i++) {
                if ($cols[$i] -ceq "Type") { $typeCol = $i }
            }
            $foundHeader = $true
            continue
        }
        if ($foundHeader -and $typeCol -ge 0 -and $line -match '^\|') {
            if ($line -match '^\| *ID *\|') { continue }   # repeated header
            if ($line -match '^[|: -]+$') { continue }      # separator row
            # First data row is authoritative (mirrors Rust module_type, which
            # returns here even when the Type cell is empty).
            $vals = ($line -split '\|') | ForEach-Object { $_.Trim() }
            if ($typeCol -lt $vals.Count) { return $vals[$typeCol] }
            return ""
        }
    }
    return ""
}

# True when a module file carries `Type: Conductor` (case-insensitive).
function Test-ApsConductor {
    param([string]$FilePath)
    return ((Get-ApsModuleType -FilePath $FilePath) -match '^(?i)conductor$')
}

function Get-ApsLineNumber {
    param([string]$FilePath, [string]$Pattern)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return $null }
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match $Pattern) { return ($i + 1) }
    }
    return $null
}

function Get-ApsWorkItemContent {
    param([string]$FilePath, [int]$StartLine)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return @() }
    $content = [System.Collections.ArrayList]::new()
    for ($i = $StartLine; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^###? ') { break }
        $null = $content.Add($lines[$i])
    }
    return @($content)
}

Export-ModuleMember -Function @(
    'Test-ApsSection'
    'Test-ApsMetadataTable'
    'Get-ApsSectionContent'
    'Test-ApsSectionHasContent'
    'Get-ApsWorkItems'
    'Get-ApsModuleId'
    'Get-ApsModuleType'
    'Test-ApsConductor'
    'Get-ApsStatus'
    'Get-ApsLineNumber'
    'Get-ApsWorkItemContent'
)
