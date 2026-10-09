# Install a pinned, checksum-verified ffmpeg on a Windows CI runner, native to the runner's
# architecture.
#
# Why not Chocolatey, which these legs used before: its `ffmpeg` package is gyan.dev's x64
# build whatever the machine, so on `windows-11-arm` every encode and decode in the suite ran
# under x64 emulation. That made the arm64 leg ~1.5x slower than Linux outside Skia, which is
# enough to miss `preview_budget`'s scrub budget and to time out a painters test, while its
# raster time matched Linux. The x64 leg takes a pinned build from the same builder, so both
# Windows legs run BtbN's ffmpeg, the builder the Linux legs already use
# (`ci/install_ffmpeg_floor.sh`), in place of gyan.dev's 9.0.2 essentials build, whose libx264
# wrote MP4s of different sizes from identical frames at the pinned thread count (ADR-0143) in
# `painters.rs`.
#
# The builds are BtbN's GPL static builds of the newest ffmpeg release series (9.0) from
# month-end autobuilds, because those survive where the dailies are pruned. The checksums are
# BtbN's own `checksums.sha256` for each release; a mismatch fails the job rather than testing
# a binary nobody chose. Bump the pins to the next series when it is released, so the Windows
# legs keep testing the top of ADR-0115's range.
#
# The two architectures are pinned to different month-ends on purpose. BtbN's builds from
# 2026-09-23 to 2026-10-03 stripped their static libraries with `llvm-strip --strip-unneeded`,
# which drops COMDAT section symbols from COFF objects, and lld then silently skipped
# relocations against them (BtbN/FFmpeg-Builds 9acad4a). Only winarm64 is linked with
# llvm-mingw, so only it broke: the 2026-09-30 winarm64 ffmpeg.exe holds 162 `bl` instructions
# that branch to themselves (unrelocated calls, encoding 0x94000000), and on windows-11-arm
# `ffmpeg -version` died with 0xC0000005. The win64 build from the same release is linked with
# GNU binutils, has none, and runs. The 2026-08-31 winarm64 build predates the strip and has
# none either. When the 2026-10-31 month-end appears, both architectures can move to it.
#
# The PE header of each installed binary is checked against the runner's architecture, so a
# pin that quietly hands arm64 an x64 binary fails here, not as a slow test. If the native
# arm64 build does not run at all, the script says why (exit code in hex, both binaries'
# `-version`), warns, and installs the x64 pin to run under emulation instead, which is slower
# but is what this leg ran before. On x64 there is no fallback: a build that does not run
# fails the step.
#
# Usage: ci/install_ffmpeg_windows.ps1 <install-dir>   (then put <install-dir>\bin on PATH)
param(
    [Parameter(Mandatory = $true)]
    [string] $Dest
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Pins = @{
    'win64'    = @{
        Release = 'autobuild-2026-09-30-13-08'
        Build   = 'ffmpeg-n9.0.2-17-g2a571b6068'
        Series  = '9.0'
        Sha256  = 'a0e45723c72141975f51d8666302e614711745f3102b704ca3f82c897a58d278'
        Machine = 0x8664  # IMAGE_FILE_MACHINE_AMD64
    }
    'winarm64' = @{
        Release = 'autobuild-2026-08-31-13-27'
        Build   = 'ffmpeg-n9.0.1-11-ge47273f4d9'
        Series  = '9.0'
        Sha256  = '7e6142ae4d04b35123eba48d91bb2c559ef43b49a6b3857f7324bbcf7266d18b'
        Machine = 0xAA64  # IMAGE_FILE_MACHINE_ARM64
    }
}

# NTSTATUS values a crashed Windows process exits with, so a failure reads as a cause.
$NtStatus = @{
    'C0000005' = 'access violation'
    'C000001D' = 'illegal instruction'
    'C00000FD' = 'stack overflow'
    'C0000135' = 'a DLL it needs was not found'
    'C0000139' = 'an entry point it needs was not found in a DLL'
    'C000007B' = 'bad image format (wrong architecture or a corrupt file)'
    'C0000409' = 'stack buffer overrun (fail-fast)'
    'C0000142' = 'DLL initialization failed'
}

function Format-ExitCode([int] $Code) {
    # An NTSTATUS arrives as a negative Int32; '{0:X8}' prints its two's complement.
    $hex = '{0:X8}' -f $Code
    $why = $NtStatus[$hex]
    if ($why) { "$Code (0x$hex, $why)" } else { "$Code (0x$hex)" }
}

function Install-Pin([string] $Arch, [string] $Into) {
    $pin = $Pins[$Arch]
    $name = "$($pin.Build)-$Arch-gpl-$($pin.Series)"
    $temp = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [System.IO.Path]::GetTempPath() }
    $archive = Join-Path $temp "$name.zip"
    $url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/$($pin.Release)/$name.zip"

    Invoke-WebRequest -Uri $url -OutFile $archive -MaximumRetryCount 3 -RetryIntervalSec 5
    $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash.ToLowerInvariant()
    if ($actual -ne $pin.Sha256) {
        throw "$name.zip: SHA-256 $actual, expected $($pin.Sha256)"
    }
    Write-Host "$name.zip ($($pin.Release)): SHA-256 OK"

    # The archive holds one top-level directory, `$name`, with `bin\` inside it.
    $staging = Join-Path $temp "$name-unpacked"
    if (Test-Path -LiteralPath $staging) { Remove-Item -LiteralPath $staging -Recurse -Force }
    Expand-Archive -LiteralPath $archive -DestinationPath $staging
    Remove-Item -LiteralPath $archive -Force
    $root = Join-Path $staging $name
    if (-not (Test-Path -LiteralPath (Join-Path $root 'bin'))) {
        throw "$name.zip has no $name\bin"
    }
    if (Test-Path -LiteralPath $Into) { Remove-Item -LiteralPath $Into -Recurse -Force }
    Move-Item -LiteralPath $root -Destination $Into
    Remove-Item -LiteralPath $staging -Recurse -Force

    foreach ($tool in 'ffmpeg.exe', 'ffprobe.exe') {
        $exe = Join-Path (Join-Path $Into 'bin') $tool
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
        if ($machine -ne $pin.Machine) {
            throw ('{0} is built for machine 0x{1:X4}, not 0x{2:X4} as pinned for {3}' -f $tool, $machine, $pin.Machine, $Arch)
        }
    }
    Write-Host ('ffmpeg.exe and ffprobe.exe are {0} (machine 0x{1:X4})' -f $Arch, $pin.Machine)
}

# Runs `-version` of both binaries and prints what each said. Returns $null when both ran,
# otherwise a one-line description of every failure.
function Test-Install([string] $Into) {
    $failures = @()
    # A crash must come back as an exit code to report, not as a terminating error.
    $PSNativeCommandUseErrorActionPreference = $false
    foreach ($tool in 'ffmpeg', 'ffprobe') {
        $exe = Join-Path (Join-Path $Into 'bin') "$tool.exe"
        $ErrorActionPreference = 'Continue'
        $out = & $exe -hide_banner -version 2>&1 | ForEach-Object { "$_" }
        $code = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        if ($code -eq 0) {
            Write-Host "$tool -version: $(@($out) | Select-Object -First 1)"
        } else {
            $failures += "$tool -version exited $(Format-ExitCode $code)"
            Write-Host "$tool -version exited $(Format-ExitCode $code); its output:"
            if ($out) { $out | ForEach-Object { Write-Host "  $_" } } else { Write-Host '  (none)' }
        }
    }
    if ($failures) { return ($failures -join '; ') }
    return $null
}

# The OS's architecture, not this process's: an x64 PowerShell under emulation on arm64
# would otherwise ask for the x64 build this script exists to avoid.
$os = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
$Arch = switch ($os) {
    'X64' { 'win64' }
    'Arm64' { 'winarm64' }
    default { throw "no pinned Windows ffmpeg build for $os" }
}

Install-Pin $Arch $Dest
if (-not $IsWindows) { return }  # The checks below need to run the binaries.

$failure = Test-Install $Dest
if ($failure -and $Arch -eq 'winarm64') {
    $pin = $Pins[$Arch]
    Write-Host ("::warning title=ffmpeg::the native arm64 ffmpeg ($($pin.Build), $($pin.Release)) " +
        "does not run on this runner: $failure. Falling back to the pinned x64 build under " +
        'emulation, which is slower; see ci/install_ffmpeg_windows.ps1.')
    Install-Pin 'win64' $Dest
    $failure = Test-Install $Dest
}
if ($failure) {
    throw "the pinned ffmpeg installed in $Dest does not run: $failure"
}
