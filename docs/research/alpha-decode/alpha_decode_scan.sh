#!/usr/bin/env bash
# Re-derives the alpha-through-the-decode-path claims. Exits non-zero the moment
# any of them stops reproducing.
#
#   1. ProRes 4444 alpha survives decode.rs's exact ffmpeg invocation.
#   2. VP9-in-WebM alpha is SILENTLY DROPPED by that same invocation.
#   3. Adding `-c:v libvpx-vp9` recovers it.
#   4. ffprobe reports pix_fmt=yuv420p for a VP9-alpha file, so probe.rs's
#      pix_fmt_has_alpha() answers `false` for a file that does carry alpha.
set -euo pipefail
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
cd "$WORK"

GEN="color=c=red:s=320x240:r=25:d=2,format=rgba,geq=r='255':g='0':b='0':a='if(gt(X,80)*lt(X,240)*gt(Y,60)*lt(Y,180),255,0)'"
ffmpeg -hide_banner -loglevel error -y -f lavfi -i "$GEN" -c:v prores_ks -profile:v 4444 -pix_fmt yuva444p10le alpha.mov
ffmpeg -hide_banner -loglevel error -y -f lavfi -i "$GEN" -c:v libvpx-vp9 -pix_fmt yuva420p -auto-alt-ref 0 alpha.webm

# decode.rs:101 verbatim, minus the binary path
decode () { ffmpeg -hide_banner -loglevel error ${2:-} -ss 1.000 -i "$1" -frames:v 1 \
            -vf scale=320:240 -f rawvideo -pix_fmt rgba - 2>/dev/null; }

check () { python3 -c "
import sys
d=sys.stdin.buffer.read(); a=d[3::4]
transparent_corner = a[0] < 32
opaque_center      = a[120*320+160] > 223
got = 'PRESERVED' if (transparent_corner and opaque_center) else 'LOST'
want = '$2'
print(f'  $1: alpha {got} (want {want})')
sys.exit(0 if got == want else 1)
"; }

decode alpha.mov                      | check "ProRes 4444, auto decoder" PRESERVED
decode alpha.webm                     | check "VP9/WebM,   auto decoder" LOST
decode alpha.webm "-c:v libvpx-vp9"   | check "VP9/WebM,   libvpx-vp9  " PRESERVED

pf=$(ffprobe -hide_banner -loglevel error -select_streams v:0 \
     -show_entries stream=pix_fmt -of csv=p=0 alpha.webm)
echo "  VP9-alpha file reports pix_fmt=$pf to ffprobe"
[ "$pf" = "yuv420p" ] || { echo "  UNEXPECTED: probe.rs alpha detection assumption changed"; exit 1; }
echo "ALL CLAIMS REPRODUCE"
