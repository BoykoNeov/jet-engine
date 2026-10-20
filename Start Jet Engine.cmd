@echo off
rem Double-click this file to open the turbojet sandbox in your web browser.
rem The page is one self-contained file: the Rust model runs inside it, no server or install needed.
set "PAGE=%~dp0docs\sandbox\turbojet-sandbox.html"
if not exist "%PAGE%" (
  echo The sandbox page was not found:
  echo   %PAGE%
  echo Rebuild it with:  powershell -File rust\sandbox-wasm\build.ps1
  pause
  exit /b 1
)
start "" "%PAGE%"
