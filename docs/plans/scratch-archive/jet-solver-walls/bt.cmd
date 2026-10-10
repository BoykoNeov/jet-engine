@echo off
cd /d "W:\Claude_projects\jet engine\rust"
set RUST_BACKTRACE=1
cargo test --release --test cli_golden > "W:\temp\claude\jet-solver-walls\bt.log" 2>&1
exit /b %errorlevel%
