<#
.SYNOPSIS
  The web sandbox's browser checks -- one program in the gate (rust\test-all.ps1 runs it beside the
  cargo test programs, below-normal like them). docs/plans/sandbox-plan.md § 6.

.DESCRIPTION
  1. Rebuild the browser build and splice a FRESH page into target\test-all\sandbox\; it must equal
     the committed docs\sandbox\turbojet-sandbox.html byte for byte -- a model or template change
     without `build.ps1` (and a commit) fails here, the way tests\visuals.rs regenerates data.json.
  2. check.mjs: the committed page's build against the native model on the check grid, under
     measured bars.
  3. browser.mjs: the committed page in a headless Chrome -- loader, Worker, panels, linked knobs.

  Needs Node (>= 22: global WebSocket) and Chrome. Prints ONE cargo-style "test result:" line, which
  test-all.ps1 counts; the two scripts' own result lines are relabelled so they are not counted twice.

.EXAMPLE
  powershell -File rust\sandbox-wasm\check.ps1
#>
$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot
$rust = Split-Path $here -Parent
$repo = Split-Path $rust -Parent
$work = Join-Path $rust 'target\test-all\sandbox'
New-Item -ItemType Directory -Force $work | Out-Null
$page = Join-Path $repo 'docs\sandbox\turbojet-sandbox.html'
$passed = 0; $failed = 0

function Invoke-Low([string]$dir, [string]$cmdline) {
    Push-Location $dir
    try { cmd /v:on /c "start /belownormal /b /wait $cmdline & exit !errorlevel!" }
    finally { Pop-Location }
    if ($LASTEXITCODE -ne 0) { throw "failed (exit $LASTEXITCODE): $cmdline" }
}

function Report([string]$name, [bool]$ok, [string]$detail = '') {
    if ($ok) { $script:passed++; Write-Host "test $name ... ok" }
    else { $script:failed++; Write-Host "test $name ... FAILED"; if ($detail) { Write-Host "  $detail" } }
}

function Run-Node([string]$name, [string[]]$argv) {
    $out = & node @argv
    $code = $LASTEXITCODE
    $p = 0; $f = 0
    foreach ($line in $out) {
        if ($line -match '^test result: \w+\. (\d+) passed; (\d+) failed') {
            $p = [int]$Matches[1]; $f = [int]$Matches[2]
            Write-Host "  [$name] $($line -replace '^test result:', 'result:')"
        } else { Write-Host $line }
    }
    if ($code -ne 0 -and $f -eq 0) { $f = 1; Write-Host "  [$name] exited $code" }
    $script:passed += $p; $script:failed += $f
}

try {
    # 1. the committed page is what the current code builds
    Invoke-Low $here 'cargo build --release --target wasm32-unknown-unknown --manifest-path Cargo.toml'
    $wasm = Join-Path $here 'target\wasm32-unknown-unknown\release\sandbox_wasm.wasm'
    $fresh = Join-Path $work 'fresh.html'
    Invoke-Low $rust "cargo run --release -q --manifest-path Cargo.toml -- sandbox `"$wasm`" `"$fresh`""
    $same = (Get-FileHash $fresh).Hash -eq (Get-FileHash $page).Hash
    Report 'the_committed_page_is_current' $same 'docs\sandbox\turbojet-sandbox.html is stale: run rust\sandbox-wasm\build.ps1 and commit the page'

    # 2. + 3. the committed page's build, against the native model and in a real browser
    $native = Join-Path $work 'native.txt'
    Invoke-Low $rust "cargo run --release -q --manifest-path Cargo.toml -- sandbox-native > `"$native`""
    Run-Node 'check.mjs' @((Join-Path $here 'check.mjs'), $page, $native)
    Run-Node 'browser.mjs' @((Join-Path $here 'browser.mjs'), $page, $native)
} catch {
    Report 'sandbox_check_ran' $false $_.Exception.Message
}

Write-Host "test result: $(if ($failed) { 'FAILED' } else { 'ok' }). $passed passed; $failed failed; 0 ignored"
if ($failed) { exit 1 }
exit 0
