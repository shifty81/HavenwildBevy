@echo off
setlocal
cd /d "%~dp0"
where py >nul 2>nul
if %errorlevel%==0 (
  py -3 ForgePY.py %*
) else (
  python ForgePY.py %*
)
exit /b %errorlevel%
