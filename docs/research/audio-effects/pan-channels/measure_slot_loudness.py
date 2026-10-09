#!/usr/bin/env python3
"""Why channel operations belong before `audio_effects` (map #795, ADR-0169 'Where routing sits').

Usage: python3 -I measure_slot_loudness.py [path/to/ffmpeg]

Measures integrated loudness (ebur128, I:) of a left-only 1 kHz tone and a correlated stereo
tone through each routing operation. A channel operation moves programme loudness by up to
up to 6 dB either way (measured: left fill on a left-only source +3.0 LU, mono -3.0 LU, balance +0.5 on a centred source -2.0 LU); anything that reads level (normalize_loudness, compressor threshold) therefore must
see the signal AFTER it. Stdlib only.
"""
import re, subprocess, sys
FF = sys.argv[1] if len(sys.argv) > 1 else "ffmpeg"
SR = 48000
LEFT = f"sine=f=1000:r={SR}:d=5,volume=0.1[l];sine=f=1000:r={SR}:d=5,volume=0[r];[l][r]amerge=inputs=2"
CORR = f"sine=f=1000:r={SR}:d=5,volume=0.1[l];sine=f=1000:r={SR}:d=5,volume=0.1[r];[l][r]amerge=inputs=2"
OPS = {
    "none": "anull",
    "left (fill)": "pan=stereo|c0=c0|c1=c0",
    "mono 0.5(L+R)": "pan=stereo|c0=0.5*c0+0.5*c1|c1=0.5*c0+0.5*c1",
    "swap": "pan=stereo|c0=c1|c1=c0",
    "balance +1 (R only)": "pan=stereo|c0=0*c0|c1=1*c1",
    "balance +0.5 linear": "pan=stereo|c0=0.5*c0|c1=1*c1",
}
def lufs(src, af):
    r = subprocess.run([FF, "-hide_banner", "-nostats", "-f", "lavfi", "-i", src, "-af", af + ",ebur128", "-f", "null", "-"],
                       capture_output=True, text=True).stderr
    return float(re.search(r"Integrated loudness:\s*I:\s*(-?[\d.]+) LUFS", r).group(1))
rows = {}
for sname, src in (("left-only", LEFT), ("correlated", CORR)):
    base = lufs(src, "anull")
    for o, af in OPS.items():
        v = lufs(src, af)
        rows[(sname, o)] = v - base
        print(f"{sname:11s} {o:22s} {v:7.2f} LUFS  ({v-base:+.2f} LU vs none)")
ok = abs(rows[("left-only", "left (fill)")] - 3.01) < 0.1 and abs(rows[("left-only", "mono 0.5(L+R)")] + 3.01) < 0.1 \
     and abs(rows[("correlated", "mono 0.5(L+R)")]) < 0.1 and abs(rows[("correlated", "swap")]) < 0.01
print("ok" if ok else "FAIL")
sys.exit(0 if ok else 1)
