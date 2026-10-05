<#
.SYNOPSIS
  THE GATE, run in parallel: every test program of the crate, several at once, all at
  BELOW-NORMAL priority. Same tests as `cargo test --release`; only the scheduling differs.

.DESCRIPTION
  `cargo test` runs its test programs ONE AFTER ANOTHER. Each program runs its own tests on
  every core, but most programs hold a handful of tests, so on 2026-10-05 the full run took
  80 min of mostly idle cores (192 programs; 15 of them three-quarters of the time). This
  script builds them once (`--no-run`, below-normal), then keeps -Jobs programs running at a
  time, the longest first (by the previous run's times, kept in target\test-all\times.tsv), so
  the run ends close to its single longest program instead of the sum of all of them.

  The test programs stay SEPARATE on purpose: editing one test file rebuilds only that program.
  (A change under src\ rebuilds them all either way -- every program uses the library.)

  Each program runs with its working directory at rust\, as cargo runs it. Logs:
  target\test-all\<program>.out / .err. Exit code 0 only if every program passed.

.EXAMPLE
  powershell -File rust\test-all.ps1               # the gate
  powershell -File rust\test-all.ps1 -Only rung8*  # programs whose name matches
  powershell -File rust\test-all.ps1 -Jobs 4       # fewer at once
#>
param(
    [int]$Jobs = 8,          # 8 = the box's PHYSICAL cores; each program also threads internally
    [string]$Only = '*'      # wildcard on the program name (e.g. rung8*, *oracle)
)
$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$logs = Join-Path $root 'target\test-all'
New-Item -ItemType Directory -Force $logs | Out-Null
$clock = [Diagnostics.Stopwatch]::StartNew()

function Start-Low($exe, $argList, $out, $err) {
    $sp = @{ FilePath = $exe; WorkingDirectory = $root; RedirectStandardOutput = $out
             RedirectStandardError = $err; PassThru = $true; NoNewWindow = $true }
    if ($argList.Count -gt 0) { $sp.ArgumentList = $argList }   # Start-Process refuses an empty list
    $p = Start-Process @sp
    try { $p.PriorityClass = 'BelowNormal' } catch {}   # it may already have exited
    $null = $p.Handle                                    # keep the handle, or ExitCode reads empty
    return $p
}

# ---- 1. build every test program once, and read their paths off cargo's JSON messages
Write-Host "building (below-normal)..."
$b = Start-Low 'cargo' @('test', '--release', '--no-run', '--message-format=json-render-diagnostics',
                         '--manifest-path', "`"$root\Cargo.toml`"") "$logs\build.json" "$logs\build.err"
$b.WaitForExit()
if ($b.ExitCode -ne 0) {
    Get-Content "$logs\build.err" -Tail 30
    Write-Host "BUILD FAILED (exit $($b.ExitCode)); full log: $logs\build.err"
    exit 1
}
$programs = @(Get-Content "$logs\build.json" |
    Where-Object { $_ -like '*"executable":"*' } |
    ForEach-Object { $_ | ConvertFrom-Json } |
    Where-Object { $_.reason -eq 'compiler-artifact' -and $_.profile.test } |
    ForEach-Object {
        $name = [IO.Path]::GetFileNameWithoutExtension($_.executable) -replace '-[0-9a-f]{16}$', ''
        if ($_.target.kind -contains 'lib') { $name = "$name (unit tests, lib)" }
        elseif ($_.target.kind -contains 'bin') { $name = "$name (unit tests, bin)" }
        [pscustomobject]@{ Name = $name; Exe = $_.executable; Args = @() }
    })
# the doc tests are not a program of their own; cargo builds and runs them in one step
$programs += [pscustomobject]@{ Name = 'doc-tests'; Exe = 'cargo';
    Args = @('test', '--release', '--doc', '--manifest-path', "`"$root\Cargo.toml`"") }
$programs = @($programs | Where-Object { $_.Name -like $Only })
Write-Host ("built in {0:N0} s; {1} programs to run, {2} at a time" -f $clock.Elapsed.TotalSeconds, $programs.Count, $Jobs)

# ---- 2. longest first, by the last run's times (an unknown program goes first: it may be long)
$timesFile = Join-Path $logs 'times.tsv'
$last = @{}
if (Test-Path $timesFile) {
    Get-Content $timesFile | ForEach-Object { $k, $v = $_ -split "`t"; $last[$k] = [double]$v }
}
$queue = [Collections.Generic.Queue[object]]::new()
$programs | Sort-Object { if ($last.ContainsKey($_.Name)) { -$last[$_.Name] } else { -1e9 } } |
    ForEach-Object { $queue.Enqueue($_) }

# ---- 3. run, keeping $Jobs going
$running = @{}
$results = [Collections.Generic.List[object]]::new()
$done = 0
$runStart = $clock.Elapsed.TotalSeconds
while ($queue.Count -gt 0 -or $running.Count -gt 0) {
    while ($running.Count -lt $Jobs -and $queue.Count -gt 0) {
        $t = $queue.Dequeue()
        $safe = $t.Name -replace '[^A-Za-z0-9_.-]', '_'
        $out = "$logs\$safe.out"; $err = "$logs\$safe.err"
        $p = Start-Low $t.Exe $t.Args $out $err
        $running[$p.Id] = [pscustomobject]@{ P = $p; T = $t; Out = $out; Err = $err; Start = $clock.Elapsed.TotalSeconds }
    }
    foreach ($id in @($running.Keys)) {
        $r = $running[$id]
        if (-not $r.P.HasExited) { continue }
        $r.P.WaitForExit()
        $secs = $clock.Elapsed.TotalSeconds - $r.Start
        $text = (Get-Content $r.Out -Raw) + "`n" + (Get-Content $r.Err -Raw)
        $passed = 0; $failed = 0; $ignored = 0
        foreach ($m in [regex]::Matches($text, 'test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored')) {
            $passed += [int]$m.Groups[1].Value; $failed += [int]$m.Groups[2].Value; $ignored += [int]$m.Groups[3].Value
        }
        $ok = ($r.P.ExitCode -eq 0) -and ($failed -eq 0)
        $results.Add([pscustomobject]@{ Name = $r.T.Name; Secs = $secs; Exit = $r.P.ExitCode;
            Passed = $passed; Failed = $failed; Ignored = $ignored; Ok = $ok; Log = $r.Out; Text = $text })
        $done++
        Write-Host ("[{0,3}/{1}] {2,-6} {3,7:N1} s  {4}" -f $done, $programs.Count, $(if ($ok) { 'ok' } else { 'FAILED' }), $secs, $r.T.Name)
        $running.Remove($id)
    }
    Start-Sleep -Milliseconds 200
}

# ---- 4. summary; the times feed the next run's order (only on a whole run, so -Only cannot thin it)
if ($Only -eq '*') {
    $results | ForEach-Object { "{0}`t{1:F1}" -f $_.Name, $_.Secs } | Set-Content -Encoding ascii $timesFile
}
$bad = @($results | Where-Object { -not $_.Ok })
$sum = ($results | Measure-Object Passed, Failed, Ignored -Sum).Sum
Write-Host ""
Write-Host ("{0} programs: {1} passed, {2} failed, {3} ignored; run {4:N0} s, total {5:N0} s (build included)" -f `
    $results.Count, $sum[0], $sum[1], $sum[2], ($clock.Elapsed.TotalSeconds - $runStart), $clock.Elapsed.TotalSeconds)
foreach ($r in $bad) {
    Write-Host "FAILED: $($r.Name) (exit $($r.Exit)) - log $($r.Log)"
    [regex]::Matches($r.Text, '(?m)^test \S+ \.\.\. FAILED') | ForEach-Object { Write-Host "    $($_.Value)" }
}
if ($bad.Count -gt 0) { exit 1 }
Write-Host "GATE GREEN"
exit 0
