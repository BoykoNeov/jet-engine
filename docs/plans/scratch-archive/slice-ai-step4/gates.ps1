$D = "W:\temp\claude\slice-ai-step4"
function Run($exe, $args_, $wd, $log, $tag) {
  $psi = New-Object System.Diagnostics.ProcessStartInfo
  $psi.FileName = $exe; $psi.Arguments = $args_; $psi.WorkingDirectory = $wd
  $psi.UseShellExecute = $false; $psi.RedirectStandardOutput = $true; $psi.RedirectStandardError = $true
  $p = [System.Diagnostics.Process]::Start($psi); $p.PriorityClass = 'BelowNormal'
  "$tag PID $($p.Id)" | Out-File -Encoding utf8 -Append "$D\gates.pids"
  $o = $p.StandardOutput.ReadToEndAsync(); $e = $p.StandardError.ReadToEndAsync()
  $p.WaitForExit()
  ($o.Result + "`n" + $e.Result + "`n${tag}_EXIT=$($p.ExitCode)") | Out-File -Encoding utf8 $log
}
$t0 = Get-Date
Run "cargo" 'test --release --manifest-path "W:\Claude_projects\jet engine\rust\Cargo.toml"' "W:\Claude_projects\jet engine" "$D\cargo.log" "CARGO"
$t1 = Get-Date
Run "W:\Claude_projects\jet engine\.venv\Scripts\python.exe" "-m pytest" "W:\Claude_projects\jet engine" "$D\pytest.log" "PYTEST"
$t2 = Get-Date
"cargo $(($t1-$t0).ToString()) pytest $(($t2-$t1).ToString())" | Out-File -Encoding utf8 "$D\gates.done"
