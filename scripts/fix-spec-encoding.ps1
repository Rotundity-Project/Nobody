param(
  [switch]$CheckOnly
)

$ErrorActionPreference = "Stop"

$targets = @()

$specFiles = Get-ChildItem -Path ".kiro/specs" -Recurse -Filter "*.md" -File -ErrorAction SilentlyContinue
if ($specFiles) {
  $targets += $specFiles
}

$fixedFiles = @(
  "README.md",
  "UI_REDESIGN_GUIDE.md",
  "docs/USER_MANUAL.md",
  "docs/ARCHITECTURE.md",
  "docs/API.md",
  "release/screenshots/README.md",
  "docs/qa/ui-manual-test-v3-2026-02-20.md"
)

foreach ($path in $fixedFiles) {
  if (Test-Path $path) {
    $targets += (Get-Item $path)
  }
}
if (-not $targets -or $targets.Count -eq 0) {
  Write-Output "No markdown files found."
  exit 0
}

$targets = $targets | Sort-Object -Property FullName -Unique

$utf8Bom = New-Object System.Text.UTF8Encoding($true)
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
$foundIssue = $false

foreach ($file in $targets) {
  $raw = [System.IO.File]::ReadAllBytes($file.FullName)
  $hasBom = $raw.Length -ge 3 -and $raw[0] -eq 0xEF -and $raw[1] -eq 0xBB -and $raw[2] -eq 0xBF

  if (-not $hasBom) {
    $foundIssue = $true
    if ($CheckOnly) {
      Write-Output "MISSING_BOM: $($file.FullName)"
      continue
    }

    $text = [System.IO.File]::ReadAllText($file.FullName, $utf8NoBom)
    [System.IO.File]::WriteAllText($file.FullName, $text, $utf8Bom)
    Write-Output "FIXED_BOM: $($file.FullName)"
  }
}

if ($CheckOnly -and $foundIssue) {
  exit 1
}

if (-not $foundIssue) {
  Write-Output "All markdown files already use UTF-8 BOM."
}
