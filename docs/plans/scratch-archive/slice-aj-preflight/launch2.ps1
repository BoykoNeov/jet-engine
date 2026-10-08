$D  = 'W:\temp\claude\slice-aj-preflight'
$PY = 'W:\Claude_projects\jet engine\.venv\Scripts\python.exe'
$ids = @()
foreach ($k in 'r80','r81','r81m','r82','r82r','r82t') {
    $p = Start-Process -FilePath $PY -ArgumentList "kcount.py $D\mut $k $D\out\c_${k}_mut.json" -WorkingDirectory $D -PassThru -WindowStyle Hidden -RedirectStandardOutput "$D\out\c_$k.log" -RedirectStandardError "$D\out\c_$k.err"
    $p.PriorityClass = 'BelowNormal'
    $ids += $p.Id
}
$ids | Set-Content -Encoding utf8 "$D\out\pids2.txt"
$ids -join ' '
