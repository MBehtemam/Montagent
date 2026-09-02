#!/usr/bin/env python3
"""Build the incumbent-style FFmpeg filtergraph for the fixture short.

This is deliberately written the way the current pipeline works: one big
filtergraph, Ken Burns via zoompan, text via libass, audio via adelay+amix.
It exists to be *measured*, not to be a design.
"""
import pathlib, argparse

FIX = pathlib.Path("/Users/mohammedehtemam/projects/github/Montaget/fixtures/en-halloween-decorating")
W, H, FPS = 1080, 1920, 30
CARD_H = 1300                 # image occupies the top 1300px; cream card below
CREAM  = "0xFBF3E3"

# (image id, duration on screen) — item 05 carries intro+hook, and returns for the quiz
SPANS = [("05", 17.472), ("06", 13.131), ("07", 12.160), ("08", 11.093), ("05", 11.360)]
TOTAL = sum(d for _, d in SPANS)

# (file, absolute start) from reference/beats.json
AUDIO = [
    ("intro-2.mp3", 0.0), ("hook-2.mp3", 3.02),
    ("05-cobweb.mp3", 5.32), ("sentence-05-cobweb.mp3", 10.47),
    ("06-spider.mp3", 17.47), ("sentence-06-spider.mp3", 22.62),
    ("07-skeleton.mp3", 30.6), ("sentence-07-skeleton.mp3", 35.75),
    ("08-lights.mp3", 42.76), ("sentence-08-lights.mp3", 47.33),
    ("quiz-2.mp3", 53.85), ("sentence-05-cobweb.mp3", 61.11),
]

def build(out, encoder="libx264", ss=None, to=None, still=False, null=False, threads=None):
    args = ["ffmpeg", "-y", "-hide_banner", "-nostats", "-loglevel", "error"]
    if threads: args += ["-threads", str(threads)]

    fc, inputs, n = [], [], 0
    # --- visual spans: crop to the card's aspect, then Ken Burns within the headroom
    vlabels = []
    for i, (img, dur) in enumerate(SPANS):
        frames = max(1, round(dur * FPS))
        # ONE input frame: zoompan's d= is frames-per-input-frame, so a looped
        # input multiplies instead of animating.
        inputs += ["-i", str(FIX / f"images/{img}.png")]
        crop_h = round(1536 * CARD_H / W)          # 1536x1849 window inside the 1536x2720 still
        zdir = "in" if i % 2 == 0 else "out"
        z = ("min(1.0001+0.0009*on,1.12)" if zdir == "in"
             else "max(1.12-0.0009*on,1.0001)")
        fc.append(
            f"[{n}:v]crop=1536:{crop_h}:(iw-1536)/2:(ih-{crop_h})/2,"
            f"zoompan=z='{z}':d={frames}:s={W}x{CARD_H}:fps={FPS}"
            f":x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)',"
            f"setsar=1[v{i}]"
        )
        vlabels.append(f"[v{i}]"); n += 1

    fc.append("".join(vlabels) + f"concat=n={len(SPANS)}:v=1:a=0[img]")
    # cream background, image laid into the top card
    fc.append(f"color=c={CREAM}:s={W}x{H}:r={FPS}:d={TOTAL:.3f}[bg]")
    fc.append("[bg][img]overlay=0:0:shortest=1[base]")

    # channel badge in the header
    inputs += ["-i", str(FIX / "brand/logo-en.png")]
    logo_idx = n; n += 1
    fc.append(f"[{logo_idx}:v]scale=64:64[logo]")
    fc.append("[base][logo]overlay=470:104[withlogo]")

    # every text and shape, burned by libass
    fc.append("[withlogo]ass=merged.ass[vout]")

    # --- audio: 12 placements mixed onto one bed (skipped for a still — nothing to map it to)
    alabels = []
    if still:
        args += inputs + ["-filter_complex", ";".join(fc)]
        args += ["-map", "[vout]", "-frames:v", "1", "-update", "1", out]
        return args
    for j, (f, at) in enumerate(AUDIO):
        inputs += ["-i", str(FIX / "audio" / f)]
        fc.append(f"[{n}:a]adelay={int(at*1000)}|{int(at*1000)},aformat=sample_fmts=fltp:sample_rates=24000:channel_layouts=mono[a{j}]")
        alabels.append(f"[a{j}]"); n += 1
    fc.append("".join(alabels) + f"amix=inputs={len(AUDIO)}:normalize=0:duration=longest,"
              f"apad,atrim=0:{TOTAL:.3f}[aout]")

    args += inputs + ["-filter_complex", ";".join(fc)]

    args += ["-map", "[vout]", "-map", "[aout]"]
    if ss is not None: args += ["-ss", str(ss)]
    if to is not None: args += ["-to", str(to)]

    if null:
        args += ["-f", "null", "-"]      # rasterize + filter, no encode
        return args

    if encoder == "libx264":
        args += ["-c:v", "libx264", "-preset", "medium", "-crf", "23", "-pix_fmt", "yuv420p"]
    else:
        args += ["-c:v", "h264_videotoolbox", "-b:v", "4M", "-pix_fmt", "yuv420p"]
    args += ["-c:a", "aac", "-b:a", "64k", "-r", str(FPS), "-movflags", "+faststart", out]
    return args

if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--out", default="out.mp4"); p.add_argument("--encoder", default="libx264")
    p.add_argument("--ss", type=float); p.add_argument("--to", type=float)
    p.add_argument("--still", action="store_true"); p.add_argument("--null", action="store_true")
    p.add_argument("--print", action="store_true")
    a = p.parse_args()
    cmd = build(a.out, a.encoder, a.ss, a.to, a.still, a.null)
    if a.print:
        import shlex; print(" ".join(shlex.quote(c) for c in cmd))
