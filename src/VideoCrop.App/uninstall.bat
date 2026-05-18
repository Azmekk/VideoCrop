@echo off
REM Wrapper that runs uninstall.ps1 with execution policy bypass and
REM moves cmd.exe's working directory away from the install dir so it
REM doesn't pin the folder we're trying to delete.

set "INSTALL_DIR=%~dp0"
cd /d "%TEMP%"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%INSTALL_DIR%uninstall.ps1"
