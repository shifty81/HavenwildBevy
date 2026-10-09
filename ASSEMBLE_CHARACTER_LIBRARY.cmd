@echo off
setlocal
cd /d "%~dp0"
where py >nul 2>nul
if %errorlevel%==0 (
  py -3 tools\reassemble_character_archive.py
) else (
  python tools\reassemble_character_archive.py
)
if errorlevel 1 (
  echo.
  echo ASSEMBLY FAILED - verify all four downloaded volume ZIPs were extracted into one folder.
  pause
  exit /b 1
)
echo.
echo The original Characters.zip is verified. You may now use PCC.cmd assets hydrate-characters.
pause
