$ErrorActionPreference = 'Stop'
$D  = 'W:\temp\claude\slice-aj-preflight'
$PY = 'W:\Claude_projects\jet engine\.venv\Scripts\python.exe'
New-Item -ItemType Directory -Force "$D\out" | Out-Null
$pids = @()
$tests = 'tests/test_rung80.py tests/test_rung81.py tests/test_rung82.py tests/test_rung83.py tests/test_rung84.py'

function Launch($name, $wd, $argline, $mode) {
    $env:PYTHONPATH = "$D\plugin"
    $env:AJ_MODE = $mode
    $env:AJ_OUT = "$D\out\$name"
    $p = Start-Process -FilePath $PY -ArgumentList $argline -WorkingDirectory $wd -PassThru -WindowStyle Hidden `
        -RedirectStandardOutput "$D\out\$name.log" -RedirectStandardError "$D\out\$name.err"
    $p.PriorityClass = 'BelowNormal'
    "$name pid=$($p.Id)"
    return $p
}

$ps = @()
$ps += Launch 'suite_ctl' "$D\ctl" "-m pytest -p ajprobe -n 4 -q -rA $tests" 'census'
$ps += Launch 'suite_mut' "$D\mut" "-m pytest -p ajprobe -n 4 -q -rA $tests" 'light'
foreach ($k in 'r80','r81','r81m','r82','r82r','r82t') {
    foreach ($v in 'ctl','mut') {
        $ps += Launch "k_${k}_$v" $D "kdump.py $D\$v $k $D\out\k_${k}_$v.json" 'none'
    }
}
$ps | ForEach-Object { "$($_.Id)" } | Set-Content -Encoding utf8 "$D\out\pids.txt"
