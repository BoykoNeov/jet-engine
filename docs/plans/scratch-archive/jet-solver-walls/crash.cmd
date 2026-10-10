@echo off
cd /d "W:\temp\claude\jet-solver-walls\crash"
cargo run --release -- %* > "W:\temp\claude\jet-solver-walls\crash.log" 2>&1
exit /b %errorlevel%
