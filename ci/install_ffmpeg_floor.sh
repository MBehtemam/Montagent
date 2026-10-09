#!/usr/bin/env bash
# Install the ffmpeg at Montagent's floor on a Linux CI runner, verified by checksum.
#
# ADR-0115 (#477 §8): the floor is ffmpeg 7.1 with libx264, and the floor stated is the
# floor CI tests. Ubuntu 24.04's apt ships 6.1, which is below it, so the Linux legs take a
# pinned BtbN static build instead. macOS keeps installing the newest release, and Windows
# a pinned build of the newest release series (`ci/install_ffmpeg_windows.ps1`), so the
# other end of the range is tested too.
#
# A month-end BtbN autobuild, because those survive where the daily ones are pruned. The
# checksums are BtbN's own `checksums.sha256` for that release, cross-checked against the
# GitHub asset digests when the pin was set; a mismatch fails the job rather than testing
# a binary nobody chose.
#
# Usage: ci/install_ffmpeg_floor.sh <install-dir>   (then put <install-dir>/bin on PATH)
set -euo pipefail

RELEASE="autobuild-2026-07-31-14-10"
BUILD="ffmpeg-n7.1.5-12-g1fdbca85aa"

case "$(uname -m)" in
  x86_64)
    ARCH="linux64"
    SHA256="c1e6caf48923dd8e6bc5e54d51ba70c321175b8162ae9c414c392990e72f0e79"
    ;;
  aarch64 | arm64)
    ARCH="linuxarm64"
    SHA256="a9a50c5782ef5e45306d58d1a9a819015b472d8da30ab6a77f15f571c861a71b"
    ;;
  *)
    echo "no pinned floor build for $(uname -m)" >&2
    exit 1
    ;;
esac

DEST="${1:?usage: $0 <install-dir>}"
NAME="${BUILD}-${ARCH}-gpl-7.1"
ARCHIVE="${RUNNER_TEMP:-/tmp}/${NAME}.tar.xz"

curl -fsSL --retry 3 -o "$ARCHIVE" \
  "https://github.com/BtbN/FFmpeg-Builds/releases/download/${RELEASE}/${NAME}.tar.xz"
echo "${SHA256}  ${ARCHIVE}" | sha256sum -c -

mkdir -p "$DEST"
tar -xJf "$ARCHIVE" -C "$DEST" --strip-components=1
"$DEST/bin/ffmpeg" -hide_banner -version | head -1
