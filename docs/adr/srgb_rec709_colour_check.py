#!/usr/bin/env python3
"""ADR-0216's colour-conversion and tagging claims, reproduced against `ffmpeg`/`ffprobe`.

Run from anywhere:  python3 docs/adr/srgb_rec709_colour_check.py
Exits non-zero, naming every defect, the moment one of its claims stops holding.

Why this exists
---------------
ADR-0216 rests on four measurable facts about `ffmpeg`'s `swscale` and filtergraph colour
negotiation across `libx264`, `prores_ks`, and `dnxhd`:

  1. Untagged encode drift: piping `rgb24` into `libx264` (`yuv420p`) with no `scale`
     matrix and no colour flags converts RGB -> YUV with `swscale`'s default Rec.601
     (`bt601`) matrix and writes no stream colour tags (`unknown`). Decoding that HD
     stream as Rec.709 (`in_color_matrix=bt709`, as QuickTime, browsers, Premiere, and
     After Effects do for untagged HD video) shifts swatch `#DC3C28` (`RGB(220, 60, 40)`)
     to `RGB(235, 74, 36)` — a 15-code-value error on red and 14 on green.
  2. Filtergraph `unspecified` overwrite on `pipe:0`: passing `-colorspace bt709
     -color_primaries bt709 -color_trc bt709 -color_range tv` on the output side alone
     (without `setparams` in `-vf`) leaves `color_primaries` and `color_transfer` absent
     (`unknown`) in `ffprobe` because `rawvideo` on `pipe:0` negotiates `unspecified`
     primaries and transfer characteristics into the encoder.
  3. Encode chain across all deliverable codecs: chaining
     `scale=in_range=pc:out_color_matrix=bt709:out_range=tv,setparams=colorspace=bt709:color_primaries=bt709:color_trc=bt709:range=tv`
     with `-colorspace bt709 -color_primaries bt709 -color_trc bt709 -color_range tv`
     stamps all four tags (`bt709`, `bt709`, `bt709`, `tv`) and keeps round-trip RGB
     within `+-2/255` across `libx264` MP4, `prores_ks` MOV, `dnxhd` MOV, and `dnxhd` MXF.
  4. Decode tag-honouring vs untagged fallback:
     - bare `scale=W:H` on an untagged Rec.709 stream falls back to `bt601` and shifts
       `(220, 60, 40)` to `(206, 45, 43)` (15-code-value error);
     - unconditional `scale=W:H:in_color_matrix=bt709` on a tagged `smpte170m` (Rec.601)
       stream overrides the stream tag and shifts `(220, 60, 40)` to `(235, 75, 36)`
       (15-code-value error);
     - honouring the stream tag when present (`in_color_matrix=auto:out_range=pc`) and
       passing `in_color_matrix=bt709:out_range=pc` when `color_space` is `unknown`
       recovers `(220, 60, 40)` within `+-2/255` across tagged `bt709` (`tv` and `pc`),
       tagged `smpte170m` (`tv`), and untagged Rec.709 streams.
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile

SWATCH = (220, 60, 40)
ENCODE_VF = (
    "scale=in_range=pc:out_color_matrix=bt709:out_range=tv,"
    "setparams=colorspace=bt709:color_primaries=bt709:color_trc=bt709:range=tv"
)
ENCODE_TAGS = [
    "-colorspace", "bt709",
    "-color_primaries", "bt709",
    "-color_trc", "bt709",
    "-color_range", "tv",
]


def run(args, stdin=None):
    res = subprocess.run(args, input=stdin, capture_output=True)
    if res.returncode != 0:
        raise RuntimeError(f"command failed ({' '.join(args)}): {res.stderr.decode()}")
    return res.stdout


def probe_tags(path):
    out = run([
        "ffprobe", "-v", "error", "-select_streams", "v:0",
        "-show_entries", "stream=color_space,color_primaries,color_transfer,color_range",
        "-of", "json", path,
    ])
    stream = json.loads(out)["streams"][0]
    return {
        "color_space": stream.get("color_space", "unknown"),
        "color_primaries": stream.get("color_primaries", "unknown"),
        "color_transfer": stream.get("color_transfer", "unknown"),
        "color_range": stream.get("color_range", "unknown"),
    }


def decode_rgba(path, width, height, vf):
    out = run([
        "ffmpeg", "-v", "error", "-i", path,
        "-vf", vf, "-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgba", "-",
    ])
    return tuple(out[:3])


def max_delta(got, want=SWATCH):
    return max(abs(a - b) for a, b in zip(got, want))


def main():
    if not (shutil.which("ffmpeg") and shutil.which("ffprobe")):
        print("skipped: ffmpeg/ffprobe not on PATH (ADR-0009)")
        return 0

    failures = 0
    work = tempfile.mkdtemp()
    try:
        rgb_64 = bytes(SWATCH) * (64 * 64)

        # 1. Untagged encode drift when decoded as Rec.709
        untagged_mp4 = os.path.join(work, "untagged_encode.mp4")
        run([
            "ffmpeg", "-v", "error", "-y",
            "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", "64x64", "-r", "25", "-i", "pipe:0",
            "-c:v", "libx264", "-crf", "10", "-pix_fmt", "yuv420p",
            "-movflags", "+faststart", untagged_mp4,
        ], stdin=rgb_64)
        tags_untagged = probe_tags(untagged_mp4)
        shifted_px = decode_rgba(untagged_mp4, 64, 64, "scale=64:64:in_color_matrix=bt709")
        if all(v == "unknown" for v in tags_untagged.values()) and max_delta(shifted_px) >= 14:
            print(f"  ok: untagged encode writes {tags_untagged} and shifts {SWATCH} -> {shifted_px} "
                  f"(max delta {max_delta(shifted_px)}/255) under Rec.709 decode")
        else:
            print(f"  FAIL: expected untagged encode drift >= 14/255, got tags={tags_untagged}, px={shifted_px}")
            failures += 1

        # 2. Output flags alone (without setparams in -vf) drop primaries/trc on rawvideo pipe:0
        flags_only_mp4 = os.path.join(work, "flags_only.mp4")
        run([
            "ffmpeg", "-v", "error", "-y",
            "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", "64x64", "-r", "25", "-i", "pipe:0",
            "-vf", "scale=in_range=pc:out_color_matrix=bt709:out_range=tv",
            "-c:v", "libx264", "-crf", "10", "-pix_fmt", "yuv420p",
            *ENCODE_TAGS, "-movflags", "+faststart", flags_only_mp4,
        ], stdin=rgb_64)
        tags_flags_only = probe_tags(flags_only_mp4)
        if tags_flags_only["color_primaries"] == "unknown" or tags_flags_only["color_transfer"] == "unknown":
            print(f"  ok: output flags without setparams leave primaries/trc as {tags_flags_only}")
        else:
            print(f"  FAIL: expected output flags without setparams to drop primaries/trc, got {tags_flags_only}")
            failures += 1

        # 3. Full encode chain across libx264 MP4, prores_ks MOV, dnxhd MOV, dnxhd MXF (at 1920x1080 for DNxHR)
        rgb_1080 = bytes(SWATCH) * (1920 * 1080)
        deliverables = [
            ("h264.mp4", 64, 64, rgb_64, ["-c:v", "libx264", "-preset", "ultrafast", "-crf", "10", "-pix_fmt", "yuv420p"]),
            ("prores.mov", 64, 64, rgb_64, ["-c:v", "prores_ks", "-profile:v", "3", "-pix_fmt", "yuv422p10le"]),
            ("dnxhr.mov", 1920, 1080, rgb_1080, ["-c:v", "dnxhd", "-profile:v", "dnxhr_hq", "-pix_fmt", "yuv422p"]),
            ("dnxhr.mxf", 1920, 1080, rgb_1080, ["-c:v", "dnxhd", "-profile:v", "dnxhr_hq", "-pix_fmt", "yuv422p"]),
        ]
        want_tags = {
            "color_space": "bt709",
            "color_primaries": "bt709",
            "color_transfer": "bt709",
            "color_range": "tv",
        }
        for name, w, h, buf, enc_args in deliverables:
            out_path = os.path.join(work, name)
            run([
                "ffmpeg", "-v", "error", "-y",
                "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{w}x{h}", "-r", "25", "-i", "pipe:0",
                "-vf", ENCODE_VF,
                *enc_args, *ENCODE_TAGS, out_path,
            ], stdin=buf)
            tags = probe_tags(out_path)
            px = decode_rgba(out_path, w, h, f"scale={w}:{h}:out_range=pc")
            delta = max_delta(px)
            if tags == want_tags and delta <= 2:
                print(f"  ok: {name:10s} tags={tags}, round-trip={px} (max delta {delta}/255 <= 2)")
            else:
                print(f"  FAIL: {name:10s} tags={tags} (want {want_tags}), round-trip={px} (delta {delta})")
                failures += 1

        # 4. Decode normalisation across tagged bt709 (tv & pc), tagged smpte170m, and untagged bt709
        def make_source(name, matrix, rng, tag_space=None, tag_rng=None):
            path = os.path.join(work, name)
            vf = f"scale=in_range=pc:out_color_matrix={matrix}:out_range={rng},format=yuv444p"
            if tag_space and tag_rng:
                vf += f",setparams=colorspace={tag_space}:color_primaries={tag_space}:color_trc={tag_space}:range={tag_rng}"
            else:
                vf += ",setparams=colorspace=unknown:color_primaries=unknown:color_trc=unknown:range=unknown"
            run([
                "ffmpeg", "-v", "error", "-y",
                "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", "64x64", "-r", "25", "-i", "pipe:0",
                "-vf", vf, "-c:v", "libx264", "-crf", "0", path,
            ], stdin=rgb_64)
            return path

        src_709_tv = make_source("src_709_tv.mp4", "bt709", "tv", "bt709", "tv")
        src_709_pc = make_source("src_709_pc.mp4", "bt709", "pc", "bt709", "pc")
        src_601_tv = make_source("src_601_tv.mp4", "bt601", "tv", "smpte170m", "tv")
        src_untagged = make_source("src_untagged.mp4", "bt709", "tv", None, None)

        # Bare scale=64:64 fails on untagged Rec.709 (falls back to bt601)
        bare_untagged_px = decode_rgba(src_untagged, 64, 64, "scale=64:64")
        if max_delta(bare_untagged_px) >= 14:
            print(f"  ok: bare scale=64:64 on untagged Rec.709 shifts {SWATCH} -> {bare_untagged_px} "
                  f"(max delta {max_delta(bare_untagged_px)}/255)")
        else:
            print(f"  FAIL: expected bare scale=64:64 to shift untagged Rec.709, got {bare_untagged_px}")
            failures += 1

        # Unconditional in_color_matrix=bt709 fails on tagged smpte170m
        forced_601_px = decode_rgba(src_601_tv, 64, 64, "scale=64:64:in_color_matrix=bt709:out_range=pc")
        if max_delta(forced_601_px) >= 14:
            print(f"  ok: unconditional in_color_matrix=bt709 on tagged smpte170m shifts {SWATCH} -> {forced_601_px} "
                  f"(max delta {max_delta(forced_601_px)}/255)")
        else:
            print(f"  FAIL: expected unconditional bt709 to shift tagged smpte170m, got {forced_601_px}")
            failures += 1

        # Tag-aware decode rule recovers all four within +-2/255
        for label, src in [
            ("tagged bt709 tv", src_709_tv),
            ("tagged bt709 pc", src_709_pc),
            ("tagged smpte170m tv", src_601_tv),
            ("untagged bt709 tv", src_untagged),
        ]:
            probed = probe_tags(src)
            vf = (
                "scale=64:64:in_color_matrix=bt709:out_range=pc"
                if probed["color_space"] in ("unknown", "unspecified")
                else "scale=64:64:out_range=pc"
            )
            px = decode_rgba(src, 64, 64, vf)
            delta = max_delta(px)
            if delta <= 2:
                print(f"  ok: tag-aware decode on {label:20s} -> {px} (max delta {delta}/255 <= 2)")
            else:
                print(f"  FAIL: tag-aware decode on {label:20s} -> {px} (max delta {delta}/255 > 2)")
                failures += 1

        print()
        print("ADR-0216 holds." if failures == 0 else f"ADR-0216 does NOT hold: {failures} defect(s).")
        return failures
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
