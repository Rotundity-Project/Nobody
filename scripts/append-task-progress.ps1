param(
  [Parameter(Mandatory = $true)]
  [string]$Message,

  [string]$TasksPath = ".kiro/specs/Nobody/tasks_v3.md"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $TasksPath)) {
  throw "Tasks file not found: $TasksPath"
}

$utf8Bom = New-Object System.Text.UTF8Encoding($true)
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)

$content = [System.IO.File]::ReadAllText($TasksPath, $utf8NoBom)
$lines = $content -split "`r?`n"

$maxIndex = 0
foreach ($line in $lines) {
  if ($line -match '^\s*(\d+)\.\s+') {
    $idx = [int]$matches[1]
    if ($idx -gt $maxIndex) {
      $maxIndex = $idx
    }
  }
}

$nextIndex = $maxIndex + 1
$entry = "$nextIndex. $Message"

$newline = if ($content.Contains("`r`n")) { "`r`n" } else { "`n" }
$updated = if ($content.EndsWith("`n")) { "$content$entry$newline" } else { "$content$newline$entry$newline" }

[System.IO.File]::WriteAllText($TasksPath, $updated, $utf8Bom)
Write-Output "APPENDED: $entry"
