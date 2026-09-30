#!/usr/bin/env python3
"""Regenerate the skills fixture set from nothing but ffmpeg and this file (#513).

The drift guard (`crates/montagent/tests/skills.rs`) validates every project snippet in
`skills/` against these files, and runs every skill script against them. Snippets name them
by these paths, so a project written against the set sees real probed durations, a real
font and a real licence attestation.

Everything here is synthetic except the font, which is copied unmodified from the eval pack.
Re-running the script overwrites the set; the bytes may differ between ffmpeg versions, but
nothing the guard checks depends on them.

    python3 fixtures/skills/make_fixtures.py
"""

import json
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
PACK_FONTS = HERE.parent.parent / "docs/research/skills-eval/assets/fonts"


def ffmpeg(*args):
    subprocess.run(["ffmpeg", "-y", "-v", "error", *args], check=True)


def font():
    (HERE / "fonts").mkdir(exist_ok=True)
    for name in ("Inter-Bold.ttf", "Inter-LICENSE.txt"):
        shutil.copyfile(PACK_FONTS / name, HERE / "fonts" / name)


def media():
    out = HERE / "media"
    out.mkdir(exist_ok=True)
    # A still with something to look at, small enough to be a badge or a backdrop.
    ffmpeg("-f", "lavfi", "-i", "testsrc2=s=360x360", "-frames:v", "1", str(out / "still.png"))
    # 6 s of a quiet two-note bed: long enough for the Router skill's 4 s starter.
    ffmpeg(
        "-f", "lavfi", "-i", "sine=f=220:d=6:sample_rate=22050",
        "-f", "lavfi", "-i", "sine=f=330:d=6:sample_rate=22050",
        "-filter_complex", "amix=inputs=2,volume=0.3",
        "-ac", "1", str(out / "bed.wav"),
    )
    # ~30 s of green screen with a moving subject and a tone: a stand-in for recorded
    # footage, so chroma, trims, overrun and probed-duration checks run for real.
    ffmpeg(
        "-f", "lavfi", "-i", "color=c=0x00B140:s=320x180:r=25:d=30",
        "-f", "lavfi", "-i", "testsrc2=s=80x80:r=25:d=30",
        "-f", "lavfi", "-i", "sine=f=440:d=30",
        "-filter_complex", "[0][1]overlay=x='40+t*6':y=50[v]",
        "-map", "[v]", "-map", "2",
        "-c:v", "libx264", "-crf", "38", "-pix_fmt", "yuv420p",
        "-c:a", "aac", "-b:a", "32k", "-shortest", str(out / "clip.mp4"),
    )
    # Word timings and visemes over the bed, in the eval pack's own shapes
    # (`voiceover.words.json`, `character/voice/line-1.json`).
    words = ["Made", "with", "Montagent", "and", "nothing", "else"]
    timings = [{"word": w, "start": 500 + i * 600, "end": 500 + i * 600 + 500} for i, w in enumerate(words)]
    (out / "bed.words.json").write_text(json.dumps(timings, indent=2) + "\n")
    visemes = [[t, v] for t, v in [(0, 0), (500, 21), (650, 1), (1100, 0), (1250, 3), (1700, 0), (2300, 21), (2500, 1), (3000, 0)]]
    (out / "bed.visemes.json").write_text(
        json.dumps({"voice": "fixture", "duration_ms": 6000, "visemes": visemes}) + "\n"
    )


# A three-level rig in the eval pack's `character/rig.json` shape: every part is a solid
# block padded so its pivot is the canvas centre, which is what the rig bake relies on.
RIG = {
    # name: (colour, block width, block height, block offset from pivot (x, y), pivot, parent)
    "torso": ("0x4060A0", 80, 140, (-40, -140), (100, 280), None),
    "arm": ("0x305090", 20, 70, (-10, 0), (130, 160), "torso"),
    "head": ("0xD0A060", 90, 80, (-45, -80), (100, 140), "torso"),
    "mouth_open": ("0x401010", 24, 16, (-12, -30), (100, 140), "head"),
    "mouth_closed": ("0x401010", 24, 4, (-12, -24), (100, 140), "head"),
    "eyes_closed": ("0x201008", 50, 4, (-25, -56), (100, 140), "head"),
}


def rig():
    out = HERE / "rig"
    (out / "parts").mkdir(parents=True, exist_ok=True)
    parts = {}
    for name, (colour, w, h, (dx, dy), pivot, parent) in RIG.items():
        # Pad the block so the pivot sits at the canvas centre.
        half_w = max(abs(dx), abs(dx + w))
        half_h = max(abs(dy), abs(dy + h))
        cw, ch = 2 * half_w, 2 * half_h
        ffmpeg(
            "-f", "lavfi", "-i", f"color=c={colour}:s={w}x{h}",
            "-vf", f"format=rgba,pad={cw}:{ch}:{half_w + dx}:{half_h + dy}:color=0x00000000",
            "-frames:v", "1", str(out / "parts" / f"{name}.png"),
        )
        parts[name] = {"file": f"parts/{name}.png", "width": cw, "height": ch, "pivot": list(pivot), "parent": parent}
    visemes = {str(v): ("closed" if v == 21 else None if v == 0 else "open") for v in range(22)}
    doc = {"drawing": {"width": 200, "height": 300}, "draw_order": list(RIG), "parts": parts, "visemes": visemes}
    (out / "rig.json").write_text(json.dumps(doc, indent=1) + "\n")


if __name__ == "__main__":
    font()
    media()
    rig()
