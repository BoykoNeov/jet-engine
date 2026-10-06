<#
.SYNOPSIS
  Build the web sandbox page: compile the model for the browser and splice it into
  docs\sandbox\turbojet-sandbox.html. Everything runs at below-normal priority.

.DESCRIPTION
  1. cargo build --release --target wasm32-unknown-unknown  (this folder's crate -> sandbox_wasm.wasm)
  2. cargo run --release -- sandbox <that .wasm> [OUT]      (the main CLI does the splice)

  Run it after any change that can reach the cycle (src\ of the main crate) or the template, and
  commit the rebuilt page: the gate (rust\test-all.ps1, via check.ps1) fails while the committed
  page is not what this script would build.

.EXAMPLE
  powershell -File rust\sandbox-wasm\build.ps1
  powershell -File rust\sandbox-wasm\build.ps1 -Out W:\temp\claude\sandbox.html
#>
param([string]$Out = '')
$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot
$rust = Split-Path $here -Parent

function Invoke-Low([string]$dir, [string]$cmdline) {
    # cmd's `start /belownormal` puts cargo AND every rustc it spawns below normal from the start;
    # `exit !errorlevel!` (delayed expansion) returns the command's real exit code.
    Push-Location $dir
    try { cmd /v:on /c "start /belownormal /b /wait $cmdline & exit !errorlevel!" }
    finally { Pop-Location }
    if ($LASTEXITCODE -ne 0) { throw "failed (exit $LASTEXITCODE): $cmdline" }
}

Invoke-Low $here 'cargo build --release --target wasm32-unknown-unknown --manifest-path Cargo.toml'
$wasm = Join-Path $here 'target\wasm32-unknown-unknown\release\sandbox_wasm.wasm'
$splice = "cargo run --release -q --manifest-path Cargo.toml -- sandbox `"$wasm`""
if ($Out) { $splice += " `"$Out`"" }
Invoke-Low $rust $splice
