# Install a pinned, checksum-verified ffmpeg on a Windows CI runner, native to the runner's
# architecture.
#
# Why not Chocolatey, which these legs used before: its `ffmpeg` package is gyan.dev's x64
# build whatever the machine, so on `windows-11-arm` every encode and decode in the suite ran
# under x64 emulation. That made the arm64 leg ~1.5x slower than Linux outside Skia, which is
# enough to miss `preview_budget`'s scrub budget and to time out a painters test, while its
# raster time matched Linux. The x64 leg takes the same pinned build so both Windows legs run
# one ffmpeg, from the builder the Linux legs already use (`ci/install_ffmpeg_floor.sh`), in
# place of gyan.dev's 9.0.2 essentials build, whose libx264 wrote MP4s of different sizes from
# identical frames at the pinned thread count (ADR-0143) in `painters.rs`.
#
# The build is BtbN's GPL static build of the newest ffmpeg release series (9.0) from a
# month-end autobuild, because those survive where the dailies are pruned. The checksums are
# BtbN's own `checksums.sha256` for that release; a mismatch fails the job rather than testing
# a binary nobody chose. Bump the pin to the next series when it is released, so the Windows
# legs keep testing the top of ADR-0115's range.
#
# The PE header of the installed ffmpeg.exe is checked against the runner's architecture, so a
# pin that quietly hands arm64 an x64 binary again fails here, not as a slow test.
#
# Usage: ci/install_ffmpeg_windows.ps1 <install-dir>   (then put <install-dir>\bin on PATH)
param(
    [Parameter(Mandatory = $true)]
    [string] $Dest
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Release = 'autobuild-2026-09-30-13-08'
$Build = 'ffmpeg-n9.0.2-17-g2a571b6068'
$Series = '9.0'

# The OS's architecture, not this process's: an x64 PowerShell under emulation on arm64
# would otherwise ask for the x64 build this script exists to avoid.
$os = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
switch ($os) {
    'X64' {
        $Arch = 'win64'
        $Sha256 = 'a0e45723c72141975f51d8666302e614711745f3102b704ca3f82c897a58d278'
        $Machine = 0x8664  # IMAGE_FILE_MACHINE_AMD64
    }
    'Arm64' {
        $Arch = 'winarm64'
        $Sha256 = '6dcd0626f9f6d6c7323565e57410e9f37ed14a85946ec9ea9caff294ca32f5c0'
        $Machine = 0xAA64  # IMAGE_FILE_MACHINE_ARM64
    }
    default { throw "no pinned Windows ffmpeg build for $os" }
}

$Name = "$Build-$Arch-gpl-$Series"
$Temp = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [System.IO.Path]::GetTempPath() }
$Archive = Join-Path $Temp "$Name.zip"
$Url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/$Release/$Name.zip"

Invoke-WebRequest -Uri $Url -OutFile $Archive -MaximumRetryCount 3 -RetryIntervalSec 5
$actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $Archive).Hash.ToLowerInvariant()
if ($actual -ne $Sha256) {
    throw "$Name.zip: SHA-256 $actual, expected $Sha256"
}
Write-Host "$Name.zip: SHA-256 OK"

# The archive holds one top-level directory, `$Name`, with `bin\` inside it.
$Staging = Join-Path $Temp "$Name-unpacked"
if (Test-Path -LiteralPath $Staging) { Remove-Item -LiteralPath $Staging -Recurse -Force }
Expand-Archive -LiteralPath $Archive -DestinationPath $Staging
$Root = Join-Path $Staging $Name
if (-not (Test-Path -LiteralPath (Join-Path $Root 'bin'))) {
    throw "$Name.zip has no $Name\bin"
}
if (Test-Path -LiteralPath $Dest) { Remove-Item -LiteralPath $Dest -Recurse -Force }
Move-Item -LiteralPath $Root -Destination $Dest

foreach ($tool in 'ffmpeg.exe', 'ffprobe.exe') {
    $exe = Join-Path (Join-Path $Dest 'bin') $tool
    # The DOS header's e_lfanew (at 0x3C) points at "PE\0\0", and the machine follows it.
    $bytes = New-Object byte[] 4096
    $stream = [System.IO.File]::OpenRead($exe)
    try { [void] $stream.Read($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
    $pe = [System.BitConverter]::ToInt32($bytes, 0x3C)
    if ($pe -lt 0 -or $pe + 6 -gt $bytes.Length -or
        [System.BitConverter]::ToUInt32($bytes, $pe) -ne 0x4550) {
        throw "$tool is not a PE image"
    }
    $machine = [System.BitConverter]::ToUInt16($bytes, $pe + 4)
    if ($machine -ne $Machine) {
        throw ('{0} is built for machine 0x{1:X4}, not this runner''s 0x{2:X4}' -f $tool, $machine, $Machine)
    }
}
Write-Host ('ffmpeg.exe and ffprobe.exe are native ({0}, machine 0x{1:X4})' -f $os, $Machine)

if ($IsWindows) {
    $version = & (Join-Path (Join-Path $Dest 'bin') 'ffmpeg.exe') -hide_banner -version
    if ($LASTEXITCODE) { throw "ffmpeg -version exited $LASTEXITCODE" }
    $version | Select-Object -First 1
}
