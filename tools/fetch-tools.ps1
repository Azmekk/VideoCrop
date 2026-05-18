# Downloads pinned ffmpeg/ffprobe/mpv binaries into the tools/ directory next to
# the app executable. Run this once after cloning, and as part of CI.
#
# Pinned sources (chosen for clear licensing + Windows convenience):
#   ffmpeg / ffprobe : BtbN Windows builds (LGPL by default).
#   mpv              : official Windows release.

[CmdletBinding()]
param(
    [string]$OutputDir = (Join-Path $PSScriptRoot ''),
    [switch]$Force
)

$ErrorActionPreference = 'Stop'

$ProgressPreference = 'SilentlyContinue'

$FfmpegUrl = 'https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-lgpl.zip'
$MpvUrl    = 'https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20250105/mpv-x86_64-20250105-git-9c84c45.7z'

if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir | Out-Null
}

function Need-Tool([string]$name) {
    $exe = Join-Path $OutputDir $name
    if ($Force) { return $true }
    return -not (Test-Path $exe)
}

function Download-File([string]$url, [string]$dest) {
    Write-Host "Downloading $url ..."
    Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing
}

# --- ffmpeg / ffprobe ---
if ((Need-Tool 'ffmpeg.exe') -or (Need-Tool 'ffprobe.exe')) {
    $zip = Join-Path $env:TEMP "videocrop_ffmpeg_$([guid]::NewGuid().ToString('N')).zip"
    $extract = Join-Path $env:TEMP "videocrop_ffmpeg_extract_$([guid]::NewGuid().ToString('N'))"
    try {
        Download-File -url $FfmpegUrl -dest $zip
        Expand-Archive -LiteralPath $zip -DestinationPath $extract -Force
        $ffmpegExe  = Get-ChildItem -Path $extract -Recurse -Filter 'ffmpeg.exe'  | Select-Object -First 1
        $ffprobeExe = Get-ChildItem -Path $extract -Recurse -Filter 'ffprobe.exe' | Select-Object -First 1
        if ($null -eq $ffmpegExe -or $null -eq $ffprobeExe) {
            throw 'Could not find ffmpeg.exe or ffprobe.exe in extracted archive.'
        }
        Copy-Item $ffmpegExe.FullName  (Join-Path $OutputDir 'ffmpeg.exe')  -Force
        Copy-Item $ffprobeExe.FullName (Join-Path $OutputDir 'ffprobe.exe') -Force
        Write-Host "ffmpeg + ffprobe placed in $OutputDir"
    }
    finally {
        if (Test-Path $zip)     { Remove-Item $zip -Force }
        if (Test-Path $extract) { Remove-Item $extract -Recurse -Force }
    }
} else {
    Write-Host 'ffmpeg.exe and ffprobe.exe already present.'
}

# --- mpv ---
if (Need-Tool 'mpv.exe') {
    Write-Warning "mpv auto-download is not yet wired (7z extraction needs 7-Zip)."
    Write-Warning "Place mpv.exe manually in $OutputDir for now."
} else {
    Write-Host 'mpv.exe already present.'
}

Write-Host 'Done.'
