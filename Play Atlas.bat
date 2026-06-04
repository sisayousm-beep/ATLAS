@echo off
REM ── Play Atlas ──────────────────────────────────────────────────────────────
REM Launches the standalone desktop build of Project Atlas (no install, no dev
REM server). Double-click this file. If the app has not been built yet, run:
REM     cd frontend  &&  npm run tauri build
REM ────────────────────────────────────────────────────────────────────────────
setlocal
set "REL=%~dp0frontend\src-tauri\target\release"

if exist "%REL%\Project Atlas.exe" (
  start "" "%REL%\Project Atlas.exe"
  exit /b 0
)
if exist "%REL%\app.exe" (
  start "" "%REL%\app.exe"
  exit /b 0
)

echo.
echo   Project Atlas has not been built yet.
echo   Build it once with:
echo.
echo       cd frontend  ^&^&  npm run tauri build
echo.
echo   Then double-click "Play Atlas.bat" again.
echo.
pause
endlocal
