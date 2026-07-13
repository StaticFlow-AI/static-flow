@echo off
setlocal EnableExtensions

set "DISTRO=Ubuntu"
set "LINUX_USER=ts_user"
set "LINUX_REPO=/home/ts_user/rust_pro/static_flow"
set "DATA_MOUNT=/mnt/wsl/data4tb"
set "VHD=E:\wsl-disks\data-4tb.vhdx"
set "RC=0"

if /I "%~1"=="--dry-run" goto dry_run

fltmc >nul 2>&1
if errorlevel 1 (
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Start-Process -FilePath '%~f0' -Verb RunAs"
    exit /b
)

if not exist "%VHD%" (
    echo [recover-local][ERROR] Missing VHD: %VHD%
    goto failed
)

echo [recover-local] Mounting %VHD% ...
wsl.exe --mount --vhd %VHD% --partition 1 --type ext4 --name data4tb
if "%ERRORLEVEL%"=="0" goto restore

echo [recover-local] Mount returned a non-zero status; checking whether it is already mounted ...
wsl.exe -d %DISTRO% -u %LINUX_USER% -- /usr/bin/mountpoint -q %DATA_MOUNT%
if not "%ERRORLEVEL%"=="0" goto failed

:restore
echo [recover-local] Restoring StaticFlow, Antigravity Manager, and AI Reviewer ...
wsl.exe -d %DISTRO% -u %LINUX_USER% --cd %LINUX_REPO% -- /usr/bin/env bash ./scripts/recover_local_services_after_reboot.sh
if not "%ERRORLEVEL%"=="0" goto failed

echo.
echo [recover-local] Recovery completed successfully.
echo [recover-local] Open http://127.0.0.1:39180/llm-access to verify the UI.
goto done

:dry_run
wsl.exe -d %DISTRO% -u %LINUX_USER% --cd %LINUX_REPO% -- /usr/bin/env bash ./scripts/recover_local_services_after_reboot.sh --dry-run
if not "%ERRORLEVEL%"=="0" goto failed
echo.
echo [recover-local] Dry-run completed successfully.
goto done

:failed
set "RC=1"
echo.
echo [recover-local][ERROR] Recovery stopped. No further services were started.

:done
echo.
if not "%RECOVERY_NO_PAUSE%"=="1" pause
endlocal & exit /b %RC%
