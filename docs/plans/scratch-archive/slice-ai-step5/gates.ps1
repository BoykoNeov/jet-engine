$ErrorActionPreference = 'Continue'
$d = 'W:\temp\claude\slice-ai-step5'
$t0 = Get-Date
$c = Start-Process -FilePath 'cargo' -ArgumentList 'test','--release','--manifest-path','"W:\Claude_projects\jet engine\rust\Cargo.toml"','--no-fail-fast' -PassThru -NoNewWindow -RedirectStandardOutput "$d\cargo.out" -RedirectStandardError "$d\cargo.err"
$c.PriorityClass = 'BelowNormal'
$c.WaitForExit()
$t1 = Get-Date
"CARGO_EXIT=$($c.ExitCode) minutes=$([math]::Round(($t1-$t0).TotalMinutes,2))" | Out-File -Encoding utf8 "$d\gates.txt"
$p = Start-Process -FilePath 'W:\Claude_projects\jet engine\.venv\Scripts\python.exe' -ArgumentList '-m','pytest','-q','-p','no:cacheprovider' -WorkingDirectory 'W:\Claude_projects\jet engine' -PassThru -NoNewWindow -RedirectStandardOutput "$d\pytest.out" -RedirectStandardError "$d\pytest.err"
$p.PriorityClass = 'BelowNormal'
$p.WaitForExit()
$t2 = Get-Date
"PYTEST_EXIT=$($p.ExitCode) minutes=$([math]::Round(($t2-$t1).TotalMinutes,2))" | Out-File -Encoding utf8 -Append "$d\gates.txt"
