#!/usr/bin/env python3
"""Build the benchmark project, the one scenario `render`'s speed target is stated for.

**The benchmark project** is the project this script writes from the repository's committed
fixtures: 6 minutes of 1920x1080 at 30 fps, with three `video` elements
visible at once for most of the timeline, text, rects, and an audio mix of narration, a
looped bed and the videos' own sound. ADR-0142 states `render`'s target against it, and
`montagent_render::budget::RENDER_TARGET` holds the number. It is not called the
"reference project": `RENDER_REFERENCES` already means reference *measurements*.

The same script builds the two observed variants ADR-0142 names, which are measured and
recorded but carry no number:

    benchmark-1080p30.montagent.json        the benchmark project (judged)
    benchmark-2160p30.montagent.json        the same timeline at 3840x2160 (observed)
    benchmark-1080p30-1video.montagent.json one video element instead of three (observed)

The generated media is not committed. Everything is written into the directory given on
the command line, which is self-contained: every path in the projects is relative to it.

    python3 fixtures/benchmark/make_benchmark.py <out-dir>

**The re-encode is pinned**, so the benchmark does not drift with the ffmpeg version: the
codec, profile, level, GOP, B-frames, preset, CRF and pixel format the sources are written
with are the constants below, and the GOP is closed and fixed (no scene-cut keyframes), so
every source has a keyframe exactly every 10 s. A long GOP is the point: a seek into one
decodes up to 299 frames before it reaches the one asked for, which is what real camera
footage costs and what the committed fixture's short GOP hid. The bytes can still differ
between x264 builds; the structure cannot.

Re-running reuses the sources already in `<out-dir>` when `manifest.json` records the same
encode settings and the same input, and re-encodes otherwise (`--force` re-encodes always).
Generating the media is never part of a timed run (ADR-0142's protocol).

Requires Python >= 3.9 and ffmpeg/ffprobe with libx264 on PATH.
"""

import argparse
import hashlib
import json
import math
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
FIXTURES = HERE.parent
HALLOWEEN = FIXTURES / "en-halloween-decorating"
SOURCE_VIDEO = HALLOWEEN / "reference" / "en-halloween-decorating.mp4"
NARRATION = sorted((HALLOWEEN / "audio").glob("*.mp3"))
FONT = HALLOWEEN / "fonts" / "OpenRunde-Bold.otf"
FONT_LICENCE = HALLOWEEN / "fonts" / "OpenRunde-LICENSE.txt"
FONT_SOURCE = "https://github.com/lauridskern/open-runde"
BED = FIXTURES / "skills" / "media" / "bed.wav"

FPS = 30
DURATION_MS = 6 * 60 * 1000
GOP_SECONDS = 10

# The re-encode, pinned. Change any of these and the benchmark is a different benchmark:
# say so in the ADR that records the next measurement.
VIDEO_ENCODE = [
    "-c:v", "libx264",
    "-profile:v", "high",
    "-pix_fmt", "yuv420p",
    "-preset", "medium",
    "-crf", "20",
    "-g", str(FPS * GOP_SECONDS),
    "-keyint_min", str(FPS * GOP_SECONDS),
    "-sc_threshold", "0",
    "-bf", "3",
    "-x264-params", "open-gop=0",
]
AUDIO_ENCODE = ["-c:a", "aac", "-b:a", "128k", "-ar", "48000", "-ac", "2"]
LEVEL = {1080: "4.1", 2160: "5.1"}

# Three distinct files, so no two video elements share a decoder's input: a real project's
# three visible clips are three files. Each is the committed short at 30 fps, told apart by
# a cheap filter so a frame shows which lane it came from.
LANES = {
    "a": "null",
    "b": "hflip",
    "c": "hue=h=90",
}


def run(*args, capture=False):
    result = subprocess.run(list(args), check=True, capture_output=capture, text=True)
    return result.stdout if capture else None


def ffmpeg(*args):
    run("ffmpeg", "-nostdin", "-y", "-v", "error", *args)


def probe_ms(path):
    out = run(
        "ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0",
        str(path), capture=True,
    )
    return int(float(out.strip()) * 1000)


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def ffmpeg_version():
    return run("ffmpeg", "-version", capture=True).splitlines()[0]


def encode_settings(height):
    """Everything that decides a source's bytes, as recorded in the manifest."""
    return {
        "input": SOURCE_VIDEO.name,
        "input_sha256": sha256(SOURCE_VIDEO),
        "video": VIDEO_ENCODE + ["-level:v", LEVEL[height]],
        "audio": AUDIO_ENCODE,
        "fps": FPS,
        "size": [height, height * 16 // 9],
        "lanes": LANES,
    }


def sources(out, height, force):
    """The three lane sources at `height`p portrait (width `height`, height 16:9 of it)."""
    media = out / "media"
    media.mkdir(parents=True, exist_ok=True)
    manifest = out / "manifest.json"
    recorded = json.loads(manifest.read_text()) if manifest.exists() else {}
    want = encode_settings(height)
    key = f"sources-{height}"
    files = {lane: media / f"lane-{lane}-{height}.mp4" for lane in LANES}
    if not force and recorded.get(key) == want and all(f.exists() for f in files.values()):
        print(f"reusing the {height}p sources: same settings, same input", file=sys.stderr)
        return files
    width, tall = height, height * 16 // 9
    for lane, look in LANES.items():
        print(f"encoding lane {lane} at {width}x{tall} (this is not timed)", file=sys.stderr)
        ffmpeg(
            "-i", str(SOURCE_VIDEO),
            "-vf", f"fps={FPS},scale={width}:{tall}:flags=lanczos,{look}",
            *want["video"], *want["audio"],
            "-movflags", "+faststart",
            str(files[lane]),
        )
    recorded[key] = want
    manifest.write_text(json.dumps(recorded, indent=2) + "\n")
    return files


def copy_shared(out):
    """Copy what the projects use, unchanged. A file already there with the same bytes is
    left alone, so a re-run does not touch its mtime and invalidate the probe cache."""
    shared = [(FONT, "fonts"), (FONT_LICENCE, "fonts"), (BED, "audio")]
    shared += [(clip, "audio") for clip in NARRATION]
    for src, sub in shared:
        dst = out / sub / src.name
        dst.parent.mkdir(parents=True, exist_ok=True)
        if not dst.exists() or dst.read_bytes() != src.read_bytes():
            shutil.copyfile(src, dst)


def block(size):
    """A one-line text box's height at `line_height` 1.1, rounded up to a whole pixel."""
    return math.ceil(size * 11 / 10)


def segments(start, end, first, length):
    """Cut `[start, end)` into back-to-back pieces: one of `first` ms, then `length` each."""
    cuts, t, size = [], start, first
    while t < end:
        cuts.append((t, min(t + size, end)))
        t, size = t + size, length
    return cuts


def lane(name, lane_id, source, video_ms, span, first, length, offsets, x, k):
    """One lane of back-to-back `video` elements, each cut in at its own source offset."""
    elements = []
    for n, (start, end) in enumerate(segments(*span, first, length)):
        source_start = offsets[n % len(offsets)]
        source_end = source_start + (end - start)
        assert source_end <= video_ms, (name, n, source_end, video_ms)
        elements.append({
            "id": f"{name}-{n + 1:02}", "type": "video", "start": start, "end": end,
            "source": source, "source_start": source_start, "source_end": source_end,
            "x": x * k, "y": 540 * k, "origin": "center",
            "width": 576 * k, "height": 1024 * k, "fit": "literal",
            "volume": 0.15,
        })
    return {"name": name, "layer": 10 + lane_id, "elements": elements}


def project(height, files, video_ms, lanes):
    """The project at `height`p (1080 or 2160) with `lanes` of video visible at once."""
    k = height // 1080
    media = {lane_name: f"media/{path.name}" for lane_name, path in files.items()}
    tracks = []

    # The centre lane covers the whole timeline; the outer two cover all but the first and
    # last 15 s, so three videos are visible for 330 of the 360 s. Their cuts are staggered,
    # so no two lanes reopen a source on the same frame.
    tracks.append(lane("centre", 1, media["a"], video_ms, (0, DURATION_MS), 60_000, 60_000,
                       [0, 2_500, 5_000], 960, k))
    if lanes == 3:
        tracks.append(lane("left", 2, media["b"], video_ms, (15_000, DURATION_MS - 15_000),
                           40_000, 55_000, [5_000, 0, 10_000], 320, k))
        tracks.append(lane("right", 3, media["c"], video_ms, (15_000, DURATION_MS - 15_000),
                           25_000, 55_000, [10_000, 5_000, 0], 1600, k))

    # The lower-third strip, a progress bar whose `x` travels the whole timeline (so one
    # rect changes on every frame), and the card behind the opening title.
    tracks.append({"name": "panels", "layer": 20, "elements": [
        {"id": "strip", "type": "rect", "start": 0, "end": DURATION_MS,
         "x": 0, "y": 900 * k, "origin": "top-left", "width": 1920 * k, "height": 140 * k,
         "fill": "#1E344C", "opacity": 0.85},
    ]})
    tracks.append({"name": "progress", "layer": 21, "elements": [
        {"id": "progress-bar", "type": "rect", "start": 0, "end": DURATION_MS,
         "x": [{"t": 0, "v": -1920 * k}, {"t": DURATION_MS - 1_000, "v": 0, "ease": "linear"}],
         "y": 1040 * k, "origin": "top-left", "width": 1920 * k, "height": 12 * k,
         "fill": "#E8743B"},
    ]})
    tracks.append({"name": "title-card", "layer": 22, "elements": [
        {"id": "title-card", "type": "rect", "start": 0, "end": 15_000,
         "x": 960 * k, "y": 540 * k, "origin": "center", "width": 1200 * k, "height": 300 * k,
         "fill": "#FBF3E3", "radius": 24 * k},
    ]})

    # A new lower-third line every 10 s: 36 text elements, each painted on 300 frames, the
    # way a captioned talk is. Plus the opening title.
    texts = []
    for n, start in enumerate(range(0, DURATION_MS, 10_000)):
        minutes, seconds = divmod(start // 1000, 60)
        texts.append({
            "id": f"line-{n + 1:02}", "type": "text", "start": start, "end": start + 10_000,
            "x": 960 * k, "y": 970 * k, "origin": "center", "width": 1800 * k,
            "height": block(56 * k), "font": "brand", "size": 56 * k, "line_height": 1.1,
            "color": "#FFF8E8", "align": "center",
            "runs": [{"text": f"Part {n + 1} of 36, from {minutes}:{seconds:02}"}],
            "caption": False,
        })
    tracks.append({"name": "lines", "layer": 30, "elements": texts})
    tracks.append({"name": "title", "layer": 31, "elements": [
        {"id": "title", "type": "text", "start": 0, "end": 15_000,
         "x": 960 * k, "y": 540 * k, "origin": "center", "width": 1100 * k,
         "height": block(96 * k), "font": "brand", "size": 96 * k, "line_height": 1.1,
         "color": "#1E344C", "align": "center", "runs": [{"text": "Benchmark project"}],
         "caption": False},
    ]})

    # The rest of the mix: a narration line every 20 s, cycling the fixture's eleven, and a
    # 6 s bed looped under the whole timeline.
    voices = []
    for n, start in enumerate(range(2_000, DURATION_MS, 20_000)):
        clip = NARRATION[n % len(NARRATION)]
        length = probe_ms(clip)
        voices.append({"id": f"voice-{n + 1:02}", "type": "audio", "start": start,
                       "end": start + length, "source": f"audio/{clip.name}",
                       "source_start": 0, "source_end": length})
    bed_ms = probe_ms(BED)
    tracks.append({"name": "voice", "layer": 40, "elements": voices})
    tracks.append({"name": "bed", "layer": 41, "elements": [
        {"id": "bed", "type": "audio", "start": 0, "end": DURATION_MS,
         "source": f"audio/{BED.name}", "source_start": 0, "source_end": bed_ms,
         "overrun": "loop", "volume": 0.4},
    ]})

    name = f"benchmark-{height}p{FPS}" + ("" if lanes == 3 else "-1video")
    font = f"fonts/{FONT.name}"
    return name, {
        "frame": {"width": 1920 * k, "height": 1080 * k},
        "fps": FPS,
        "background": "#101418",
        "duration": DURATION_MS,
        "output": f"out/{name}.mp4",
        "fonts": {"brand": [{"file": font}]},
        "fontVendor": {font: {"licence": "OFL-1.1", "source": FONT_SOURCE,
                              "sha256": sha256(FONT)}},
        "tracks": tracks,
    }


def canonical(document):
    """The document in the house layout (ADR-0042): one element per line, keys in schema
    order, so `montagent fmt --check` has nothing to say about a generated file."""
    lines = ["{"]
    for key, value in document.items():
        if key != "tracks":
            lines.append(f'  "{key}": {json.dumps(value, ensure_ascii=False)},')
    lines.append('  "tracks": [')
    for n, track in enumerate(document["tracks"]):
        lines += ["    {", f'      "name": {json.dumps(track["name"])},',
                  f'      "layer": {track["layer"]},', '      "elements": [']
        elements = [
            "        " + json.dumps(e, separators=(",", ":"), ensure_ascii=False)
            for e in track["elements"]
        ]
        lines += [line + "," for line in elements[:-1]] + elements[-1:]
        lines += ["      ]", "    }," if n + 1 < len(document["tracks"]) else "    }"]
    lines += ["  ]", "}"]
    return "\n".join(lines) + "\n"


def write(out, name, document):
    path = out / f"{name}.montagent.json"
    path.write_text(canonical(document))
    print(path)


def main():
    parser = argparse.ArgumentParser(
        description="Build the benchmark project and its observed variants (ADR-0142).",
        epilog="Requires Python >= 3.9 and ffmpeg/ffprobe with libx264 on PATH.",
    )
    parser.add_argument("out", type=Path, help="directory to write into (created)")
    parser.add_argument("--only-1080", action="store_true",
                        help="skip the 2160p30 variant, whose sources take longest to encode")
    parser.add_argument("--force", action="store_true", help="re-encode the sources")
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)

    copy_shared(out)
    print(f"ffmpeg: {ffmpeg_version()}", file=sys.stderr)
    for height in (1080,) if args.only_1080 else (1080, 2160):
        files = sources(out, height, args.force)
        video_ms = min(probe_ms(f) for f in files.values())
        if height == 1080:
            write(out, *project(height, files, video_ms, lanes=1))
        write(out, *project(height, files, video_ms, lanes=3))


if __name__ == "__main__":
    main()
