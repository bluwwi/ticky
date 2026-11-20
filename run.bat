@echo off
setlocal
cd /d "%~dp0"
cargo run
if errorlevel 1 pause
