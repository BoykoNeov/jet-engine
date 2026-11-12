# Number check for the plain-language notes in this folder (roadmap A4). Reports; it is not part of the gate.
# Each "## Rung N" (or "## Part") section is checked against docs/rungN-spec.md plus every rung that section
# names, plus docs/sandbox/EXPERIMENTS.md if it says "experiment" and the page template if it says "page".
# A number passes only if it appears there as a whole number (2.7 does not match 2.75 or 12.7).
# Run: powershell -NoProfile -File docs\notes\check-numbers.ps1   (expect "misses: 0").
# Proven 2026-11-12: it flagged two wrong numbers planted in a copy of the notes.
param([string]$Repo = (Split-Path (Split-Path $PSScriptRoot -Parent) -Parent), [string]$Notes = "")
$docs = Join-Path $Repo "docs"
$exp = Get-Content -Raw -Encoding UTF8 (Join-Path $docs "sandbox\EXPERIMENTS.md")
$page = Get-Content -Raw -Encoding UTF8 (Join-Path $docs "sandbox\template.html")
$cache = @{}
function Spec([int]$n) {
  if (-not $cache.ContainsKey($n)) {
    $p = Join-Path $docs "rung$n-spec.md"
    $cache[$n] = if (Test-Path $p) { Get-Content -Raw -Encoding UTF8 $p } else { "" }
  }
  $cache[$n]
}
$total = 0
if (-not $Notes) { $Notes = Join-Path $docs "notes" }
foreach ($f in Get-ChildItem $Notes -Filter *.md | Where-Object { $_.Name -ne 'README.md' }) {
  $text = Get-Content -Raw -Encoding UTF8 $f.FullName
  $family = [regex]::Matches($f.BaseName, '\d+') | ForEach-Object { [int]$_.Value }
  $lo = $family[0]; $hi = if ($family.Count -gt 1) { $family[1] } else { $family[0] }
  $sections = [regex]::Split($text, '(?m)^(?=## )')
  foreach ($s in $sections) {
    $head = ($s -split "`n")[0].Trim()
    $own = [regex]::Match($head, '^## Rung (\d+)')
    $nums = New-Object System.Collections.Generic.HashSet[int]
    if ($own.Success) { [void]$nums.Add([int]$own.Groups[1].Value) } else { $lo..$hi | ForEach-Object { [void]$nums.Add($_) } }
    foreach ($m in [regex]::Matches($s, '(?i)rungs? (\d+)(?:\p{Pd}(\d+))?')) {
      $a = [int]$m.Groups[1].Value; $b = if ($m.Groups[2].Success) { [int]$m.Groups[2].Value } else { $a }
      $a..$b | ForEach-Object { [void]$nums.Add($_) }
    }
    $src = ($nums | ForEach-Object { Spec $_ }) -join "`n"
    if ($s -match '(?i)experiment') { $src += $exp }
    if ($s -match '(?i)page') { $src += $page }
    # Strip rung references and experiment numbers before collecting numbers.
    $body = [regex]::Replace($s, '(?i)rungs? \d+(?:\p{Pd}\d+)?(?:(?:, | and | or )\d+(?:\p{Pd}\d+)?)*', '')
    $body = [regex]::Replace($body, '(?i)experiments? \d+|rungN|rung\d+-spec|\d+-\d+-[a-z-]+\.md', '')
    $miss = [regex]::Matches($body, '(?<![\w.])\d+(?:\.\d+)?(?![\w])') | ForEach-Object { $_.Value } | Sort-Object -Unique |
      Where-Object { -not [regex]::IsMatch($src, '(?<![\d.])' + [regex]::Escape($_) + '(?!\.?\d)') }
    if ($miss) { $total += @($miss).Count; "{0} :: {1}`n    {2}" -f $f.Name, $head, (($miss) -join '  ') }
  }
}
"misses: $total"
