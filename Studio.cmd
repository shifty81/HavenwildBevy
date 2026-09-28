@echo off
setlocal
cd /d "%~dp0"
call PCC.cmd run dx12
exit /b %errorlevel%
