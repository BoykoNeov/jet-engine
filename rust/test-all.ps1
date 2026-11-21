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
  powershell -File rust\test-all.ps1 -Features pin-default   # built with a cargo feature (rung 104: the old pin)
#>
param(
    [int]$Jobs = 8,          # 8 = the box's PHYSICAL cores; each program also threads internally
    [string]$Only = '*',     # wildcard on the program name (e.g. rung8*, *oracle)
    [string]$Features = ''   # cargo features to build the tests with (the browser check builds its own, without)
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
$feat = @()
if ($Features) { $feat = @('--features', $Features) }
# DISK (2026-11-22): every build CONFIGURATION gets its own full set of ~250 test programs (~1 GB) in
# target\release\deps, and cargo never deletes an old set -- by then three sets (default + the two
# feature gates) and an older one had grown target\ to 13 GB. So a -Features build goes to a throwaway
# target dir, deleted at the end of the run (logs stay in target\test-all), and a default build drops
# the superseded copies of the programs it just built (step 1b).
$featDir = $null
if ($Features) {
    $featDir = Join-Path $root ('target\feat-' + ($Features -replace '[^A-Za-z0-9_-]', '_'))
    $env:CARGO_TARGET_DIR = $featDir
}
$b = Start-Low 'cargo' (@('test', '--release', '--no-run', '--message-format=json-render-diagnostics',
                          '--manifest-path', "`"$root\Cargo.toml`"") + $feat) "$logs\build.json" "$logs\build.err"
$b.WaitForExit()
if ($b.ExitCode -ne 0) {
    Get-Content "$logs\build.err" -Tail 30
    Write-Host "BUILD FAILED (exit $($b.ExitCode)); full log: $logs\build.err"
    if ($featDir) { Remove-Item -LiteralPath $featDir -Recurse -Force -ErrorAction SilentlyContinue }
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
    Args = @(@('test', '--release', '--doc', '--manifest-path', "`"$root\Cargo.toml`"") + $feat) }
# the web sandbox's browser checks: its own wasm build, Node and a headless Chrome, so not a cargo
# test program -- rust\sandbox-wasm\check.ps1 (docs/plans/sandbox-plan.md section 6, user decision 9.5)
$programs += [pscustomobject]@{ Name = 'sandbox-browser'; Exe = 'powershell';
    Args = @('-NoProfile', '-File', "`"$root\sandbox-wasm\check.ps1`"") }
$programs = @($programs | Where-Object { $_.Name -like $Only })

# ---- 1b. a default build: delete the older copies (another hash) of every test program just built --
# left by a toolchain or profile change, or by a feature build from before 2026-11-22. `turbojet` is
# skipped: its stem is shared by the lib/bin unit tests AND the CLI binary that `cargo run` uses.
if (-not $Features) {
    $deps = Join-Path $root 'target\release\deps'
    $keep = @{}; $stems = @{}           # from ALL built programs, not just -Only's
    Get-Content "$logs\build.json" | Where-Object { $_ -like '*"executable":"*' } |
        ForEach-Object { ($_ | ConvertFrom-Json).executable } | Where-Object { $_ } | ForEach-Object {
            $base = [IO.Path]::GetFileNameWithoutExtension($_)
            $keep[$base] = $true
            $stems[($base -replace '-[0-9a-f]{16}$', '')] = $true
        }
    $freed = 0L
    Get-ChildItem $deps -File | Where-Object { $_.Extension -in '.exe', '.pdb', '.d' } | ForEach-Object {
        $base = $_.BaseName
        if ($base -notmatch '^(.+)-[0-9a-f]{16}$') { return }
        $stem = $Matches[1]
        if ($stem -eq 'turbojet' -or -not $stems.ContainsKey($stem) -or $keep.ContainsKey($base)) { return }
        $freed += $_.Length
        Remove-Item -LiteralPath $_.FullName -Force
    }
    if ($freed -gt 0) { Write-Host ("pruned {0:N0} MB of superseded test programs" -f ($freed / 1048576)) }
}
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
        # the browser check builds its own (no features) and reads its paths under the usual target dirs
        if ($featDir) { if ($t.Name -eq 'sandbox-browser') { Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue } else { $env:CARGO_TARGET_DIR = $featDir } }
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

# ---- 4. summary; the times feed the next run's order (only on a whole default run, so -Only cannot thin it)
if ($Only -eq '*' -and -not $Features) {
    $results | ForEach-Object { "{0}`t{1:F1}" -f $_.Name, $_.Secs } | Set-Content -Encoding ascii $timesFile
}
if ($featDir) {   # the throwaway feature build (step 1); its logs are in target\test-all
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $featDir -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "removed the feature build $featDir"
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
