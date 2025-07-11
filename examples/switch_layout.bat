@echo off
REM Windows batch script for layout switching

setlocal

if "%1"=="" (
    echo Multi-Viewport Editor Layout Switcher
    echo =====================================
    echo.
    echo Usage: switch_layout.bat ^<layout_name^>
    echo.
    echo Available layouts:
    echo   compact      - Compact layout for small screens
    echo   developer    - Balanced layout for development
    echo   artist       - Viewport-focused for content creation
    echo   dual_monitor - Optimized for dual monitor setups
    echo   minimal      - Clean interface with essentials only
    echo   ultrawide    - Optimized for ultrawide monitors
    echo   detached     - Multi-window workflow
    echo.
    echo Examples:
    echo   switch_layout.bat developer
    echo   switch_layout.bat dual_monitor
    goto :eof
)

set LAYOUT=%1
set EXAMPLES_DIR=%~dp0
set PROJECT_DIR=%EXAMPLES_DIR%..
set TARGET_FILE=%PROJECT_DIR%\editor_layout.json

REM Backup current layout if it exists
if exist "%TARGET_FILE%" (
    copy "%TARGET_FILE%" "%EXAMPLES_DIR%current_layout_backup.json" >nul 2>&1
    if errorlevel 1 (
        echo Warning: Failed to backup current layout
    ) else (
        echo ✓ Current layout backed up
    )
)

REM Determine source file based on layout name
if /i "%LAYOUT%"=="compact" set SOURCE_FILE=%EXAMPLES_DIR%compact_layout.json
if /i "%LAYOUT%"=="developer" set SOURCE_FILE=%EXAMPLES_DIR%developer_layout.json  
if /i "%LAYOUT%"=="artist" set SOURCE_FILE=%EXAMPLES_DIR%artist_layout.json
if /i "%LAYOUT%"=="dual_monitor" set SOURCE_FILE=%EXAMPLES_DIR%dual_monitor_layout.json
if /i "%LAYOUT%"=="minimal" set SOURCE_FILE=%EXAMPLES_DIR%minimal_layout.json
if /i "%LAYOUT%"=="ultrawide" set SOURCE_FILE=%EXAMPLES_DIR%ultrawide_layout.json
if /i "%LAYOUT%"=="detached" set SOURCE_FILE=%EXAMPLES_DIR%detached_workflow.json

if not defined SOURCE_FILE (
    echo Error: Unknown layout '%LAYOUT%'
    echo Use 'switch_layout.bat' without arguments to see available layouts.
    exit /b 1
)

if not exist "%SOURCE_FILE%" (
    echo Error: Layout file '%SOURCE_FILE%' not found.
    exit /b 1
)

REM Copy the layout file
copy "%SOURCE_FILE%" "%TARGET_FILE%" >nul 2>&1
if errorlevel 1 (
    echo Error: Failed to apply layout
    exit /b 1
)

echo ✓ Applied layout: %LAYOUT%
echo   File: %SOURCE_FILE%
echo   Target: %TARGET_FILE%
echo.
echo Restart the editor to see the new layout.