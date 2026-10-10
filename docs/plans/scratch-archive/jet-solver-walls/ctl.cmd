@echo off
cd /d "W:\temp\claude\jet-solver-walls\%1"
cargo run --release --bin controls -- %2 %3 > "W:\temp\claude\jet-solver-walls\ctl_%1.log" 2>&1
exit /b %errorlevel%
