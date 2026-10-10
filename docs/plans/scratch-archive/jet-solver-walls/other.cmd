@echo off
cd /d "W:\temp\claude\jet-solver-walls\crash"
cargo run --release --bin other > "W:\temp\claude\jet-solver-walls\other.log" 2>&1
exit /b %errorlevel%
