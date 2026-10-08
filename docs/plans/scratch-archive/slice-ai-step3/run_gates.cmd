@echo off
set LOGD=W:\temp\claude\slice-ai-step3
cargo test --release --manifest-path "W:\Claude_projects\jet engine\rust\Cargo.toml" > "%LOGD%\cargo.log" 2>&1
echo CARGO_EXIT=%ERRORLEVEL%>> "%LOGD%\cargo.log"
pushd "W:\Claude_projects\jet engine"
"W:\Claude_projects\jet engine\.venv\Scripts\python.exe" -m pytest > "%LOGD%\pytest.log" 2>&1
echo PYTEST_EXIT=%ERRORLEVEL%>> "%LOGD%\pytest.log"
popd
echo DONE> "%LOGD%\gates.done"
