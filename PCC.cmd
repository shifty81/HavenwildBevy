@echo off
setlocal EnableExtensions
cd /d "%~dp0"
chcp 65001 >nul 2>nul
set "PYTHONUTF8=1"
set "PYTHONIOENCODING=utf-8"

set "PCC_PY="
where py >nul 2>nul
if %errorlevel%==0 set "PCC_PY=py -3"
if not defined PCC_PY (
  where python >nul 2>nul
  if %errorlevel%==0 set "PCC_PY=python"
)

if not defined PCC_PY (
  echo [FATAL] Python was not found. Havenwild Bevy PCC cannot start.
  echo Install Python or make python.exe/py.exe available on PATH.
  pause
  exit /b 127
)

if "%~1"=="" (
  %PCC_PY% ProjectControlCenter.py menu
) else (
  %PCC_PY% ProjectControlCenter.py %*
)
set "PCC_RC=%errorlevel%"

if not "%PCC_RC%"=="0" (
  echo.
  echo ========================================================================
  echo  HAVENWILD BEVY PCC EXITED WITH ERROR %PCC_RC%
  echo  The window is being held open so the failure cannot disappear.
  echo  Check .pcc\logs, .pcc\receipts and .pcc\debug.
  echo ========================================================================
  pause
)
exit /b %PCC_RC%
