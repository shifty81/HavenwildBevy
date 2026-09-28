@echo off
setlocal
cd /d "%~dp0"
call PCC.cmd %*
exit /b %errorlevel%
