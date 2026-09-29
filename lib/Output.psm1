#
# APS CLI Output Module
# Port of lib/output.sh — color output, result tracking, summary formatting
#

# --- State ---

$script:FileResults = [System.Collections.ArrayList]::new()
$script:TotalFiles = 0
$script:TotalErrors = 0
$script:TotalWarnings = 0
$script:FileTypes = @{}

# --- Logging ---

function Write-ApsError {
    param([string]$Message)
    [Console]::Error.Write("error: ")
    [Console]::Error.WriteLine($Message)
}

function Write-ApsWarning {
    param([string]$Message)
    Write-Host "warning: " -ForegroundColor Yellow -NoNewline
    Write-Host $Message
}

function Write-ApsInfo {
    param([string]$Message)
    Write-Host "aps: " -ForegroundColor Green -NoNewline
    Write-Host $Message
}

# --- Result tracking ---

function Reset-ApsResults {
    $script:FileResults.Clear()
    $script:TotalFiles = 0
    $script:TotalErrors = 0
    $script:TotalWarnings = 0
    $script:FileTypes = @{}
}

function Add-ApsResult {
    param(
        [string]$Path,
        [string]$Type,
        [string]$Code,
        [string]$Message,
        [string]$Line = ""
    )
    $null = $script:FileResults.Add([PSCustomObject]@{
        Path    = $Path
        Type    = $Type
        Code    = $Code
        Message = $Message
        Line    = $Line
    })
    if ($Type -eq "error") { $script:TotalErrors++ }
    elseif ($Type -eq "warning") { $script:TotalWarnings++ }
}

function Set-ApsFileType {
    param([string]$Path, [string]$FileType)
    $script:FileTypes[$Path] = $FileType
}

function Add-ApsFileCount {
    $script:TotalFiles++
}

function Get-ApsTotalErrors {
    return $script:TotalErrors
}

function Test-ApsFileHasResults {
    param([string]$Path)
    foreach ($r in $script:FileResults) {
        if ($r.Path -eq $Path) { return $true }
    }
    return $false
}

# Keep cross-tree warnings with their file, matching Rust's report.files order.
function Set-ApsResultOrder {
    param([string[]]$Files)
    $ordered = [System.Collections.ArrayList]::new()
    foreach ($file in $Files) {
        foreach ($result in $script:FileResults) {
            if ($result.Path -ceq $file) { $null = $ordered.Add($result) }
        }
    }
    $script:FileResults = $ordered
}

# --- Text output ---

function Write-ApsTextResults {
    $currentFile = ""
    $fileHasIssues = $false

    foreach ($result in $script:FileResults) {
        if ($result.Path -ne $currentFile) {
            if ($currentFile -ne "") {
                if (-not $fileHasIssues) {
                    Write-Host "  $([char]0x2713) valid" -ForegroundColor Green
                }
                Write-Host ""
            }
            $currentFile = $result.Path
            $fileHasIssues = $false
            Write-Host $currentFile
        }

        if ($result.Code -ne "OK") {
            $fileHasIssues = $true
            $color = if ($result.Type -eq "warning") { "Yellow" } else { "Red" }
            Write-Host "  " -NoNewline
            Write-Host "$($result.Code):" -ForegroundColor $color -NoNewline
            Write-Host " $($result.Message)" -NoNewline
            if ($result.Line) {
                Write-Host " (line $($result.Line))" -ForegroundColor DarkGray
            } else {
                Write-Host ""
            }
        }
    }

    # Last file
    if ($currentFile -ne "" -and -not $fileHasIssues) {
        Write-Host "  $([char]0x2713) valid" -ForegroundColor Green
    }

    Write-Host ""

    # Summary
    $fileWord = if ($script:TotalFiles -ne 1) { "files" } else { "file" }
    $summary = "$($script:TotalFiles) $fileWord checked"

    if ($script:TotalErrors -eq 0 -and $script:TotalWarnings -eq 0) {
        Write-Host "$summary, no issues" -ForegroundColor Green
    } else {
        $parts = @()
        if ($script:TotalErrors -gt 0) {
            $s = if ($script:TotalErrors -ne 1) { "s" } else { "" }
            $parts += @{ text = "$($script:TotalErrors) error$s"; color = "Red" }
        }
        if ($script:TotalWarnings -gt 0) {
            $s = if ($script:TotalWarnings -ne 1) { "s" } else { "" }
            $parts += @{ text = "$($script:TotalWarnings) warning$s"; color = "Yellow" }
        }
        Write-Host "$summary, " -NoNewline
        for ($i = 0; $i -lt $parts.Count; $i++) {
            if ($i -gt 0) { Write-Host ", " -NoNewline }
            Write-Host $parts[$i].text -ForegroundColor $parts[$i].color -NoNewline
        }
        Write-Host ""
    }
}

# --- JSON output ---

function Write-ApsJsonResults {
    $files = [System.Collections.ArrayList]::new()
    $currentEntries = [System.Collections.ArrayList]::new()
    $currentFile = ""

    foreach ($result in $script:FileResults) {
        if ($result.Path -ne $currentFile) {
            if ($currentFile -ne "") {
                $ft = "unknown"
                if ($script:FileTypes.ContainsKey($currentFile)) {
                    $ft = $script:FileTypes[$currentFile]
                }
                $fileObj = [ordered]@{
                    path     = $currentFile
                    type     = $ft
                    errors   = @($currentEntries | Where-Object { $_.type -eq "error" } | ForEach-Object {
                        $e = [ordered]@{ code = $_.code; message = $_.message }
                        if ($_.line) { $e.line = [int]$_.line }
                        $e
                    })
                    warnings = @($currentEntries | Where-Object { $_.type -eq "warning" } | ForEach-Object {
                        $e = [ordered]@{ code = $_.code; message = $_.message }
                        if ($_.line) { $e.line = [int]$_.line }
                        $e
                    })
                }
                $null = $files.Add($fileObj)
            }
            $currentFile = $result.Path
            $currentEntries.Clear()
        }
        if ($result.Code -ne "OK") {
            $null = $currentEntries.Add(@{
                type    = $result.Type
                code    = $result.Code
                message = $result.Message
                line    = $result.Line
            })
        }
    }

    # Last file
    if ($currentFile -ne "") {
        $ft = "unknown"
        if ($script:FileTypes.ContainsKey($currentFile)) {
            $ft = $script:FileTypes[$currentFile]
        }
        $fileObj = [ordered]@{
            path     = $currentFile
            type     = $ft
            errors   = @($currentEntries | Where-Object { $_.type -eq "error" } | ForEach-Object {
                $e = [ordered]@{ code = $_.code; message = $_.message }
                if ($_.line) { $e.line = [int]$_.line }
                $e
            })
            warnings = @($currentEntries | Where-Object { $_.type -eq "warning" } | ForEach-Object {
                $e = [ordered]@{ code = $_.code; message = $_.message }
                if ($_.line) { $e.line = [int]$_.line }
                $e
            })
        }
        $null = $files.Add($fileObj)
    }

    $output = [ordered]@{
        files   = @($files)
        summary = [ordered]@{
            files    = $script:TotalFiles
            errors   = $script:TotalErrors
            warnings = $script:TotalWarnings
        }
    }

    [Console]::Out.WriteLine(($output | ConvertTo-Json -Depth 10))
}

Export-ModuleMember -Function @(
    'Write-ApsError'
    'Write-ApsWarning'
    'Write-ApsInfo'
    'Reset-ApsResults'
    'Add-ApsResult'
    'Set-ApsFileType'
    'Set-ApsResultOrder'
    'Add-ApsFileCount'
    'Get-ApsTotalErrors'
    'Test-ApsFileHasResults'
    'Write-ApsTextResults'
    'Write-ApsJsonResults'
)
