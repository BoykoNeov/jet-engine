@echo off
cd /d "W:\temp\claude\jet-solver-walls\head\rust"
cargo test --release %* > "W:\temp\claude\jet-solver-walls\head.log" 2>&1
exit /b %errorlevel%
