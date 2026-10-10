@echo off
cd /d "W:\Claude_projects\jet engine\rust"
cargo test --release %* > "W:\temp\claude\jet-solver-walls\last.log" 2>&1
exit /b %errorlevel%
