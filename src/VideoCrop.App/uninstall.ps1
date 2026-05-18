# VideoCrop uninstaller.
#
# Removes the install directory, Start menu entry, and user data
# (%LOCALAPPDATA%\VideoCrop — logs, settings, downloaded ffmpeg/mpv).
#
# This script ships next to VideoCrop.App.exe. Run it via uninstall.bat
# (which sets -ExecutionPolicy Bypass and moves cmd's CWD away from the
# install dir so deletion can succeed).

param(
    [switch]$Relocated,
    [string]$OrigInstallDir = ''
)

if (-not $Relocated) {
    # Phase 1: copy ourselves to %TEMP% and re-launch from there.
    # PowerShell holds a read lock on $PSCommandPath while it's executing,
    # so we can't delete the install dir from this process.
    $InstallDir = $PSScriptRoot
    $tempScript = Join-Path $env:TEMP "videocrop-uninstall-$([guid]::NewGuid().ToString('N')).ps1"
    Copy-Item -LiteralPath $PSCommandPath -Destination $tempScript -Force
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $tempScript -Relocated -OrigInstallDir $InstallDir
    exit
}

# Phase 2: running from %TEMP%, free to delete $OrigInstallDir.
$ErrorActionPreference = 'Continue'
Write-Host "Uninstalling VideoCrop"
Write-Host "  Install dir: $OrigInstallDir"
Write-Host ""

# Close any running app so its file locks release before we try to delete.
Get-Process VideoCrop.App -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "Closing VideoCrop.App (pid $($_.Id))..."
    try { $_ | Stop-Process -Force -ErrorAction Stop } catch {}
}
Start-Sleep -Milliseconds 500

# Start menu group: %APPDATA%\Microsoft\Windows\Start Menu\Programs\VideoCrop
$startMenuDir = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\VideoCrop'
if (Test-Path -LiteralPath $startMenuDir) {
    Write-Host "Removing Start menu entry: $startMenuDir"
    Remove-Item -LiteralPath $startMenuDir -Recurse -Force -ErrorAction SilentlyContinue
}

# User data: logs, settings, downloaded tools.
$userData = Join-Path $env:LOCALAPPDATA 'VideoCrop'
if (Test-Path -LiteralPath $userData) {
    Write-Host "Removing user data: $userData"
    Remove-Item -LiteralPath $userData -Recurse -Force -ErrorAction SilentlyContinue
}

# Apps & Features registry key (only present on installs that used the old
# registry-based registration; harmless if missing).
$regKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\VideoCrop'
if (Test-Path -LiteralPath $regKey) {
    Write-Host "Removing legacy registry key"
    Remove-Item -LiteralPath $regKey -Recurse -Force -ErrorAction SilentlyContinue
}

# Install directory (must come last — it's where the .bat that launched us lives).
if (Test-Path -LiteralPath $OrigInstallDir) {
    Write-Host "Removing install directory: $OrigInstallDir"
    Remove-Item -LiteralPath $OrigInstallDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "VideoCrop has been removed."
Start-Sleep -Seconds 2

# Self-delete the temp copy.
Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue
