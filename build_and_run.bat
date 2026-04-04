@echo off
setlocal enabledelayedexpansion
title FocusFlow - Build and Run Launcher

echo ========================================
echo    FocusFlow - Build and Run Launcher
echo    Millennium Dawn Focus Tree Editor
echo ========================================
echo.

:: =============================================
:: STEP 1: Verify Rust installation
:: =============================================
echo [1/5] Checking Rust installation...

rustc --version >nul 2>&1
if !errorlevel! neq 0 (
    echo    [ERROR] Rust is not installed!
    echo    Install from: https://rustup.rs/
    pause
    exit /b 1
)
cargo --version >nul 2>&1
if !errorlevel! neq 0 (
    echo    [ERROR] Cargo not in PATH!
    pause
    exit /b 1
)

for /f "delims=" %%i in ('rustc --version') do echo    Rust: %%i
for /f "delims=" %%i in ('cargo --version') do echo    Cargo: %%i
echo    [OK]
echo.

:: =============================================
:: STEP 2: Verify project structure
:: =============================================
echo [2/5] Checking project files...

cd /d "%~dp0" 2>nul
if not exist "src\main.rs" (
    echo    [ERROR] src\main.rs not found in %CD%
    pause
    exit /b 1
)
echo    Project: %CD%
echo    [OK]
echo.

:: =============================================
:: STEP 3: Check venezuela.txt
:: =============================================
echo [3/5] Checking MD files...

set "VENEZUELA_PATH=C:\Users\armon\Documents\Paradox Interactive\Hearts of Iron IV\mod\MD\common\national_focus\venezuela.txt"
if exist "%VENEZUELA_PATH%" (
    for %%A in ("%VENEZUELA_PATH%") do echo    Venezuela: FOUND (%%~zA bytes)
) else (
    echo    Venezuela: NOT FOUND (quick-load will need manual path)
)
echo    [OK]
echo.

:: =============================================
:: STEP 4: Build
:: =============================================
echo [4/5] Building...
echo.

:: Remove old exe to force fresh compile
if exist "target\debug\focusflow.exe" del /q "target\debug\focusflow.exe" >nul 2>&1

cargo build 2>&1

if !errorlevel! neq 0 (
    echo.
    echo    ========================================
    echo    BUILD FAILED
    echo    ========================================
    echo.
    echo    If you see "Application Control policy has blocked":
    echo    Your company has Device Guard / HVCI enabled.
    echo    You need IT admin to disable it or whitelist this folder.
    echo.
    echo    Workaround: Try running the standalone test instead:
    echo      rustc tests_standalone.rs -o test_runner.exe
    echo      test_runner.exe
    echo.
    pause
    exit /b 1
)

echo.
echo    [OK] Build successful!
echo.

:: =============================================
:: STEP 5: Run
:: =============================================
echo [5/5] Launching FocusFlow...
echo.
echo    ========================================
echo     Shortcuts:
echo       Ctrl+S Save    Ctrl+Z Undo
echo       Ctrl+Y Redo    E      Edit
echo       Del  Delete    Ctrl+D Duplicate
echo       F5   Reload
echo    ========================================
echo.

cargo run 2>&1

if !errorlevel! neq 0 (
    echo.
    echo    ========================================
    echo    LAUNCH FAILED
    echo    ========================================
    echo.
    echo    If blocked by Application Control policy:
    echo    1. Open Windows Security
    echo    2. Device Security ^> Core Isolation Details
    echo    3. Turn OFF Memory Integrity
    echo    4. Restart and try again
    echo.
    echo    OR run the standalone test (no dependencies):
    echo      rustc tests_standalone.rs -o test_runner.exe
    echo      test_runner.exe
    echo.
) else (
    echo.
    echo    ========================================
    echo     Session ended. See you next time!
    echo    ========================================
)

echo.
pause
