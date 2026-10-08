$d = "W:\temp\claude\slice-ai-step6"; $repo = "W:\Claude_projects\jet engine"
$p = Start-Process cargo -ArgumentList 'test','--release','--no-fail-fast' -WorkingDirectory "$repo\rust" -RedirectStandardOutput "$d\rust-gate.log" -RedirectStandardError "$d\rust-gate.err" -PassThru -NoNewWindow
$null = $p.Handle; $p.PriorityClass = 'BelowNormal'; $p.WaitForExit()
Set-Content "$d\rust-gate.exit" "CARGO_EXIT=$($p.ExitCode)"
$q = Start-Process "$repo\.venv\Scripts\python.exe" -ArgumentList '-m','pytest','-q','-p','no:cacheprovider' -WorkingDirectory $repo -RedirectStandardOutput "$d\pytest.log" -RedirectStandardError "$d\pytest.err" -PassThru -NoNewWindow
$null = $q.Handle; $q.PriorityClass = 'BelowNormal'; $q.WaitForExit()
Set-Content "$d\pytest.exit" "PYTEST_EXIT=$($q.ExitCode)"
