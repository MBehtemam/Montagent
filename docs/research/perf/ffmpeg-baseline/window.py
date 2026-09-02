#!/usr/bin/env python3
"""Restructured partial render: build a graph covering ONLY [t0,t1].

This is what a tool would have to do to make --from/--to actually cheap on an
FFmpeg backend. The naive -ss/-to costs time proportional to t0 because a
synthesised filtergraph has to be evaluated from zero.
"""
import pathlib, re, subprocess, render

FIX, W, H, FPS, CARD_H, CREAM = render.FIX, render.W, render.H, render.FPS, render.CARD_H, render.CREAM

def shift_ass(t0, t1, src="merged.ass", dst="window.ass"):
    """Re-base the subtitle file onto the window's local clock, dropping non-overlapping lines."""
    def ts(t):
        h=int(t//3600); t-=h*3600; m=int(t//60); t-=m*60
        return f"{h}:{m:02d}:{max(t,0):05.2f}"
    def parse(t):
        h,m,s = t.split(":"); return int(h)*3600+int(m)*60+float(s)
    out, kept = [], 0
    for line in pathlib.Path(src).read_text().splitlines():
        if line.startswith("Dialogue:"):
            m = re.match(r"(Dialogue:\s*[^,]*,)([^,]+),([^,]+),(.*)", line)
            a, b = parse(m.group(2)), parse(m.group(3))
            if b <= t0 or a >= t1: continue
            out.append(f"{m.group(1)}{ts(a-t0)},{ts(min(b,t1)-t0)},{m.group(4)}"); kept += 1
        else:
            out.append(line)
    pathlib.Path(dst).write_text("\n".join(out) + "\n")
    return kept

def build(out, t0, t1, encoder="libx264"):
    dur = t1 - t0
    kept = shift_ass(t0, t1)
    args = ["ffmpeg","-y","-hide_banner","-nostats","-loglevel","error"]
    fc, inputs, n, vlabels, vi = [], [], 0, [], 0

    # only the spans overlapping the window, each trimmed to its overlap
    clock = 0.0
    for i, (img, sdur) in enumerate(render.SPANS):
        s0, s1 = clock, clock + sdur
        clock = s1
        lo, hi = max(s0, t0), min(s1, t1)
        if hi <= lo: continue
        seg = hi - lo
        skip = round((lo - s0) * FPS)          # frames into this span the window starts
        frames = max(1, round(seg * FPS))      # only the window's frames; phase comes from skip
        inputs += ["-i", str(FIX / f"images/{img}.png")]
        crop_h = round(1536 * CARD_H / W)
        z = (f"min(1.0001+0.0009*(on+{skip}),1.12)" if i % 2 == 0
             else f"max(1.12-0.0009*(on+{skip}),1.0001)")
        fc.append(f"[{n}:v]crop=1536:{crop_h}:(iw-1536)/2:(ih-{crop_h})/2,"
                  f"zoompan=z='{z}':d={frames}:s={W}x{CARD_H}:fps={FPS}"
                  f":x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)',setsar=1,"
                  f"trim=duration={seg:.3f},setpts=PTS-STARTPTS[v{vi}]")
        vlabels.append(f"[v{vi}]"); vi += 1; n += 1

    if len(vlabels) > 1:
        fc.append("".join(vlabels) + f"concat=n={len(vlabels)}:v=1:a=0[img]")
    else:
        fc.append(f"{vlabels[0]}null[img]")
    fc.append(f"color=c={CREAM}:s={W}x{H}:r={FPS}:d={dur:.3f}[bg]")
    fc.append("[bg][img]overlay=0:0:shortest=1[base]")
    inputs += ["-i", str(FIX / "brand/logo-en.png")]
    fc.append(f"[{n}:v]scale=64:64[logo]"); n += 1
    fc.append("[base][logo]overlay=470:104[withlogo]")
    fc.append("[withlogo]ass=window.ass[vout]")

    # audio: only clips overlapping the window, re-based
    alabels = []
    for f, at in render.AUDIO:
        probe = subprocess.run(["ffprobe","-v","error","-show_entries","format=duration",
                                "-of","csv=p=0", str(FIX/"audio"/f)], capture_output=True, text=True)
        adur = float(probe.stdout.strip())
        if at + adur <= t0 or at >= t1: continue
        local = at - t0
        inputs += ["-i", str(FIX/"audio"/f)]
        pre = ""
        if local < 0:                      # clip started before the window: trim its head
            pre = f"atrim=start={-local:.3f},asetpts=PTS-STARTPTS,"; local = 0
        fc.append(f"[{n}:a]{pre}adelay={int(local*1000)}|{int(local*1000)},"
                  f"aformat=sample_fmts=fltp:sample_rates=24000:channel_layouts=mono[a{len(alabels)}]")
        alabels.append(f"[a{len(alabels)}]"); n += 1

    if alabels:
        fc.append("".join(alabels) + f"amix=inputs={len(alabels)}:normalize=0:duration=longest,"
                  f"apad,atrim=0:{dur:.3f}[aout]")
    else:
        fc.append(f"anullsrc=r=24000:cl=mono,atrim=0:{dur:.3f}[aout]")

    args += inputs + ["-filter_complex", ";".join(fc), "-map","[vout]","-map","[aout]"]
    if encoder == "libx264":
        args += ["-c:v","libx264","-preset","medium","-crf","23","-pix_fmt","yuv420p"]
    else:
        args += ["-c:v","h264_videotoolbox","-b:v","4M","-pix_fmt","yuv420p"]
    args += ["-c:a","aac","-b:a","64k","-r",str(FPS),out]
    return args, kept
