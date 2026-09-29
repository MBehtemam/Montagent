"""The court: three jurors judge the same pairs as the human, from stills. Recorded, never
decisive.

    uv run docs/research/skills-eval/harness/court.py [--phase verdict] [--briefs <brief.md> ...]

For each pair in `judging/<phase>/pairs.json` that a judge would see (not `"auto"`), it draws
one sheet, `judging/<phase>/stills/<pair>.png`: the left clip on the top row and the right on
the bottom, each at the same fixed instants, `(k + 0.5) / N` of the brief's declared length
for k = 0…N-1 (N is `court.stills_per_clip` in `pins.json`). Then it asks each juror in
`pins.json`, independently, in an isolated Claude Code session (project settings only, Read as its only tool), which is better for
the brief, with the same three choices as the human. Each juror's ballots go verbatim to
`judging/<phase>/ballots/court/<model>.json`. Re-running fills only missing ballots.

The jurors see stills only: no motion, no sound. `verdict.py` reports where they disagree
with the human; it never resolves the disagreement.
"""

# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow"]
# ///

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import (  # noqa: E402
    EVAL, JUDGING, PHASES, REPO, brief_duration_ms, ffprobe, load_pins, read_json, rel, sh,
    write_json,
)

THUMB_H = 240
FONT = EVAL / "assets" / "fonts" / "Inter-Bold.ttf"

SCHEMA = {
    "type": "object",
    "properties": {
        "vote": {"type": "string", "enum": ["left", "right", "equal"]},
        "reasoning": {"type": "string"},
    },
    "required": ["vote", "reasoning"],
}

QUESTION = """You are judging two finished videos made for the same brief. You see them as one
sheet of stills, `{sheet}` in this directory: the top row is the LEFT video and the bottom row
is the RIGHT video, each sampled at the same instants ({instants}). You see no motion and
hear no sound; judge what the stills show.

Which is the better finished piece for this brief? Weigh how well each delivers the brief's
beats and checklist, then craft (layout, type, colour, polish) and visible defects. Answer
"equal" when you cannot honestly prefer one. Answer only with your vote and your reasoning.

The brief:

{brief}"""


def instants_ms(duration_ms: int, n: int) -> list[int]:
    return [round((k + 0.5) / n * duration_ms) for k in range(n)]


def still(clip: Path, at_ms: int, dest: Path, clip_s: float) -> Image.Image:
    t = min(at_ms / 1000, max(clip_s - 0.05, 0))
    sh("ffmpeg", "-v", "error", "-y", "-ss", f"{t:.3f}", "-i", clip, "-frames:v", "1",
       "-vf", f"scale=-2:{THUMB_H}", dest)
    return Image.open(dest).convert("RGB")


def sheet(pair: dict, key: dict, duration_ms: int, n: int, dest: Path) -> None:
    font = ImageFont.truetype(str(FONT), 22)
    rows = []
    with tempfile.TemporaryDirectory() as tmp:
        for side in ("left", "right"):
            clip = REPO / key[pair[side]] / "render.mp4"
            clip_s = ffprobe(clip)["duration_s"]
            rows.append([still(clip, t, Path(tmp) / f"{side}{i}.png", clip_s)
                         for i, t in enumerate(instants_ms(duration_ms, n))])
    w = sum(im.width for im in rows[0]) + 8 * (n + 1) + 90
    h = (THUMB_H + 8) * 2 + 8
    img = Image.new("RGB", (w, h), "#202020")
    draw = ImageDraw.Draw(img)
    for r, (label, row) in enumerate(zip(("LEFT", "RIGHT"), rows)):
        y = 8 + r * (THUMB_H + 8)
        draw.text((10, y + THUMB_H // 2 - 12), label, font=font, fill="#ffffff")
        x = 98
        for im in row:
            img.paste(im, (x, y))
            x += im.width + 8
    dest.parent.mkdir(parents=True, exist_ok=True)
    img.save(dest)


def ask(juror: str, pair: dict, sheet_png: Path, brief: str, instants: list[int]) -> dict:
    with tempfile.TemporaryDirectory(prefix="montagent-eval-court-") as tmp:
        tmp = Path(tmp)
        (tmp / "work").mkdir()
        (tmp / "work" / sheet_png.name).write_bytes(sheet_png.read_bytes())
        prompt = QUESTION.format(sheet=sheet_png.name, brief=brief.strip(),
                                 instants=", ".join(f"{t / 1000:g} s" for t in instants))
        env = {"HOME": str(Path.home()), "USER": os.environ.get("USER", ""), "PATH": os.environ["PATH"],
               "DISABLE_AUTOUPDATER": "1"}
        r = subprocess.run(
            ["claude", "-p", prompt, "--model", juror, "--output-format", "json",
             "--json-schema", json.dumps(SCHEMA), "--tools", "Read",
             "--permission-mode", "bypassPermissions", "--strict-mcp-config", "--setting-sources", "project",
             "--no-session-persistence", "--max-budget-usd", "2"],
            cwd=tmp / "work", env=env, capture_output=True, text=True, timeout=600)
    try:
        result = json.loads(r.stdout)
    except json.JSONDecodeError:
        return {"error": (r.stdout + r.stderr)[-2000:]}
    answer = result.get("structured_output")
    if not isinstance(answer, dict):
        try:
            answer = json.loads(result.get("result", ""))
        except (json.JSONDecodeError, TypeError):
            answer = {}
    return {"vote": answer.get("vote"), "reasoning": answer.get("reasoning"),
            "model": juror, "raw": result.get("result"), "cost_usd": result.get("total_cost_usd")}


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--phase", choices=PHASES, default="verdict")
    ap.add_argument("--briefs", type=Path, nargs="*", default=[],
                    help="brief files for the pairs' briefs (development briefs are found in the repo)")
    args = ap.parse_args()

    pins = load_pins()
    out = JUDGING / args.phase
    key = read_json(out / "key.json")
    pairs = [p for p in read_json(out / "pairs.json") if "auto" not in p]
    briefs = {p.stem: p.read_text() for p in (EVAL / "briefs" / "dev").glob("*.md")}
    briefs.update({p.stem: p.read_text() for p in args.briefs})
    n = pins["court"]["stills_per_clip"]

    for p in pairs:
        if p["brief"] not in briefs:
            sys.exit(f"no brief text for {p['brief']}: pass it with --briefs")
        png = out / "stills" / f"{p['id']}.png"
        if not png.exists():
            sheet(p, key, brief_duration_ms(briefs[p["brief"]]), n, png)

    for juror in pins["court"]["jurors"]:
        path = out / "ballots" / "court" / f"{juror}.json"
        cast = read_json(path) if path.exists() else {}
        todo = [p for p in pairs if not (cast.get(p["id"]) or {}).get("vote")]

        def one(p):
            text = briefs[p["brief"]]
            return p["id"], ask(juror, p, out / "stills" / f"{p['id']}.png", text,
                                instants_ms(brief_duration_ms(text), n))

        with ThreadPoolExecutor(max_workers=4) as pool:
            for pid, ballot in pool.map(one, todo):
                cast[pid] = ballot
                write_json(path, cast)
        missing = [p["id"] for p in pairs if not (cast.get(p["id"]) or {}).get("vote")]
        print(f"{juror}: {len(pairs) - len(missing)}/{len(pairs)} ballots in {rel(path)}"
              + (f"; failed: {missing}" if missing else ""))


if __name__ == "__main__":
    main()
