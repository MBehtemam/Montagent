#!/usr/bin/env python3
"""Re-derive what the two committed reference frames actually are.

`fixtures/en-halloween-decorating/reference/frame-intro.png` and
`frame-05-at-11s.png` are the only two pictures in this repository that can
falsify Montaget: they were extracted from the already-published MP4 by a
pipeline that knows nothing about Montaget, so nothing Montaget does can change
them. `crates/montaget-core/tests/reference_frames.rs` compares `frame` against
both.

Neither PNG carries its instant, and the file names are not a reliable source
for one. This script recovers both numbers from the video itself, and re-derives
the Ken Burns finding that recovery exposed. Everything it prints is a claim the
Rust suite depends on; it exits non-zero the moment one stops reproducing, which
is what `docs/agents/domain.md` asks of a committed numeric claim.

## What it establishes

1. **Which frame of the MP4 each PNG is.** Decode a window of frames, scale each
   to the PNG's own 270x480, and take the mean absolute luma difference. The
   match is exact -- 0.000 at one frame and >= 0.3 at its neighbours -- so this
   is an identification rather than a fit.

   * `frame-intro.png` is frame **10**, t = **400 ms**.
   * `frame-05-at-11s.png` is frame **350**, t = **14 000 ms**. The name counts
     11 s from *item 05's* start at 3 018 ms, not from the beginning of the
     video: 14 000 - 3 018 = 10 982.

2. **That the fixture's Ken Burns pivots about the point the file now names.**
   Recover the photograph's scale and offset at the later frame by searching
   (scale, dx, dy) against a Montaget render taken at the ramp's origin, where
   the scale is exactly 1.0 -- so whatever comes back is the *published* move
   rather than a difference between two moves. The recovered scale must match
   the fixture's declared ramp, and the recovered offset must match what the
   fixture's declared pivot predicts, better than it matches the alternative.

   The pivot, the extents it multiplies and the ramp it is checked against are all
   read out of the project file rather than written down here, so this stops
   reproducing if any of them is changed and this script is not. The pivot is
   checked against **all nine** of ADR-0013's keywords -- the declared one must be
   the nearest of them -- so it fails for any wrong pivot, not only for the
   `top-left` that #276 replaced.

   `docs/research/sample-project-migration/README.md` D3 measured the same thing
   from `reference/kenburns/06.mp4` -- *"the move is a centre-pivot zoom (centre
   beats top at every sample; the joint fit returns dy = 0)"* -- and the
   migration wrote `top-left` into the file anyway. #213's first render against
   the published video found the mismatch and #276 corrected `migrate.py`; this
   is the check that the correction is the one the picture supports.

   It does **not** close to zero. The offset the search recovers is a couple of
   reference pixels from the centre pivot's prediction, which is a residual
   `crates/montaget-core/tests/reference_frames.rs` measures at 8 x 4 project px
   at this instant and names as unexplained. The tolerance below is set to admit
   it deliberately: the claim this script makes is that the centre pivot is the
   right one of the two the fixture could spell, not that it is exact.

## Requirements

`ffmpeg` on PATH (Montaget spawns one rather than bundling one -- ADR-0009), and
Pillow. No numpy: the search below is a few thousand Pillow resizes and
histograms, all of them C.

## Usage

    ci/reference_frame_instants.py                  # assert; exit 0 or 1
    ci/reference_frame_instants.py --verbose        # print every candidate
    ci/reference_frame_instants.py --instants-only  # skip the half that needs a build
    ci/reference_frame_instants.py --binary target/release/montaget
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

try:
    from PIL import Image, ImageChops
except ImportError:  # pragma: no cover - the message is the whole value
    sys.exit("this script needs Pillow: pip install pillow")

REPO = Path(__file__).resolve().parent.parent
FIXTURE = REPO / "fixtures" / "en-halloween-decorating"
REFERENCE = FIXTURE / "reference"
VIDEO = REFERENCE / "en-halloween-decorating.mp4"
PROJECT = FIXTURE / "en-halloween-decorating.montaget.json"

# The *project's* frame duration, which is what an instant handed to `frame` is
# measured in: the fixture declares `"fps": 25`. The published MP4 is 1629
# frames over 65.259 s -- 24.98 fps -- so its own presentation times drift from
# this by a frame over the first thousand. Frames are therefore identified by
# decoded index and only then converted, once, here.
MS_PER_FRAME = 1000.0 / 25.0

# What the Rust suite has recorded, and what this script must reproduce.
EXPECTED = {
    "frame-intro.png": 10,
    "frame-05-at-11s.png": 350,
}

# The photograph's aperture, which is `photo-05`'s own `clip`: [0, 0, 1080, 1300].
CARD_HEIGHT = 1300

# The element the later reference frame is a picture of. Its ramp, its extents and
# its pivot are all read out of the project by `declared()` below rather than
# written down here -- this half of the script exists to check the file against the
# picture, and a constant transcribed from the file cannot do that.
ELEMENT = "photo-05"

# ADR-0013's nine origin keywords, as the (horizontal, vertical) fraction of the
# element's own box that `x`/`y` place and a transform pivots about. The renderer
# spells these in crates/montaget-core/src/verbs/query/geometry.rs; they are
# restated here so this script can be run against a checkout without building it.
ORIGIN_FRACTION = {
    "top-left": (0.0, 0.0), "top-center": (0.5, 0.0), "top-right": (1.0, 0.0),
    "center-left": (0.0, 0.5), "center": (0.5, 0.5), "center-right": (1.0, 0.5),
    "bottom-left": (0.0, 1.0), "bottom-center": (0.5, 1.0), "bottom-right": (1.0, 1.0),
}

# The search steps one reference pixel per axis, which is four of the project's,
# so a recovered offset carries up to 4 px of quantisation error on each of dx and
# dy -- 8 px of Manhattan distance before anything about the picture is involved.
SEARCH_STEP_PX = 1080 / 270
OFFSET_TOLERANCE_PX = 2 * SEARCH_STEP_PX  # both axes at once

# ...and on top of that, the unexplained residual #299 owns, which at this instant
# is 8 x 4 project px. The tolerance has to admit it: this script's claim is that
# the fixture names the right one of the nine keywords, not that the fit is exact.
OFFSET_TOLERANCE_PX += 12


def declared():
    """What the project says about `ELEMENT`: its pivot, its extents and its ramp.

    All of it read out of the file, so that changing any of them and leaving this
    script alone is a failure rather than a silent disagreement. Returns the origin
    keyword, the pivot point in project pixels, and a callable giving the declared
    scale at an instant.
    """
    document = json.loads(PROJECT.read_text())
    element = next(
        e for track in document["tracks"] for e in track["elements"]
        if e["id"] == ELEMENT
    )
    origin = element["origin"]
    fx, fy = ORIGIN_FRACTION[origin]
    width, height = element["width"], element["height"]

    first, last = element["scale"][0], element["scale"][-1]
    # `v` is always a pair (ADR-0012); this fixture zooms both axes together, and a
    # frame's worth of arithmetic below assumes it.
    assert first["v"][0] == first["v"][1] and last["v"][0] == last["v"][1], \
        f"{ELEMENT} scales its axes independently; this script assumes it does not"

    def scale_at(ms):
        span = last["t"] - first["t"]
        travelled = (ms - first["t"]) / span
        return first["v"][0] + (last["v"][0] - first["v"][0]) * travelled

    return origin, (fx * width, fy * height), scale_at, (width, height)


def ramp_origin_ms():
    """The instant `ELEMENT`'s ramp declares a scale of exactly 1.0.

    Read from the file for the same reason as everything else here: it is where the
    search's baseline render is taken, and a baseline taken at the wrong instant
    would make every offset below a difference between two moves.
    """
    document = json.loads(PROJECT.read_text())
    element = next(
        e for track in document["tracks"] for e in track["elements"]
        if e["id"] == ELEMENT
    )
    first = element["scale"][0]
    assert first["v"] == [1.0, 1.0], (
        f"{ELEMENT}'s ramp no longer starts at 1.0, so its first keyframe is not a "
        "baseline the recovered offset can be measured against"
    )
    return first["t"]


def luma(path: Path) -> Image.Image:
    return Image.open(path).convert("L")


def mad(a: Image.Image, b: Image.Image) -> float:
    """Mean absolute difference, 0..255."""
    histogram = ImageChops.difference(a, b).histogram()
    return sum(i * c for i, c in enumerate(histogram)) / sum(histogram)


def extract(window: range, into: Path, width: int, height: int) -> None:
    """Decode a contiguous run of frames, numbered by *decoded frame index*.

    By index and never by timestamp. The published video runs at 1629 frames
    over 65.259 s -- 24.98 fps, not the project's nominal 25 -- so a frame's
    presentation time and `index x 40 ms` drift apart by 40 ms within the first
    thousand frames, and an `-ss` seek computed from the nominal rate silently
    lands one frame off. `select='between(n,lo,hi)'` counts decoded frames,
    which is the thing being identified.
    """
    if (into / f"f{window.stop - 1:05d}.png").exists():
        return
    lo, hi = window.start, window.stop - 1
    subprocess.run(
        [
            "ffmpeg", "-v", "error", "-y",
            "-i", str(VIDEO),
            "-vf", f"select='between(n\\,{lo}\\,{hi})',scale={width}:{height}",
            # Emit exactly the selected frames, unnumbered by time.
            "-vsync", "0",
            "-start_number", str(lo),
            str(into / "f%05d.png"),
        ],
        check=True,
    )


def decoded(index: int, into: Path) -> Image.Image:
    return luma(into / f"f{index:05d}.png")


def identify(name: str, expected: int, scratch: Path, verbose: bool) -> int:
    """Which frame of the MP4 this committed PNG is."""
    committed = luma(REFERENCE / name)
    width, height = committed.size
    # A window around the recorded answer rather than the whole 1629-frame
    # video: identification is exact, so a window that contains the answer and
    # its neighbours proves both that it matches and that nothing near it does.
    window = range(max(expected - 4, 0), expected + 5)
    extract(window, scratch, width, height)
    scores = {i: mad(committed, decoded(i, scratch)) for i in window}
    best = min(scores, key=scores.get)
    if verbose:
        for i in window:
            mark = "  <-- best" if i == best else ""
            print(f"    frame {i:5d}  MAD {scores[i]:.3f}{mark}")

    runner_up = min(v for i, v in scores.items() if i != best)
    ok = best == expected and scores[best] < 0.001 and runner_up > 0.1
    print(
        f"  {name}: frame {best} — t = {best * MS_PER_FRAME:.0f} ms on the project's own "
        f"25 fps timeline (MAD {scores[best]:.3f}; nearest other frame {runner_up:.3f})"
    )
    if not ok:
        print(
            f"    FAILED: expected frame {expected} with an exact match. The "
            f"instant recorded in tests/reference_frames.rs no longer holds.",
            file=sys.stderr,
        )
    return 0 if ok else 1


def rendered(at_ms: int, scratch: Path, binary: str | None) -> Path:
    """One `frame` of the fixture, at true pixels and lossless.

    Through `cargo run` by default, so the script works from a clean checkout
    with nothing built. `--binary` points at one that already exists instead,
    which is what CI passes: the matrix leg has just built and tested the
    workspace for its own target, and a bare `cargo run` there would ignore that
    and produce a second, debug build of everything.
    """
    out = scratch / f"render-{at_ms}.png"
    if out.exists():
        return out
    invocation = (
        [binary] if binary else ["cargo", "run", "--quiet", "-p", "montaget", "--"]
    )
    subprocess.run(
        invocation
        + [
            "frame", str(PROJECT),
            "--at", str(at_ms), "--full", "--png", "--out", str(out),
        ],
        cwd=REPO,
        check=True,
        stdout=subprocess.DEVNULL,
    )
    return out


def pivot(scratch: Path, verbose: bool, binary: str | None) -> int:
    """Recover the published Ken Burns' scale and offset at frame 351."""
    name = "frame-05-at-11s.png"
    at = EXPECTED[name] * MS_PER_FRAME
    committed = luma(REFERENCE / name)
    width, height = committed.size
    _, _, _, (el_width, _) = declared()
    band = round(CARD_HEIGHT * width / el_width)
    # A couple of rows in from the card's own edge, so the cream boundary below
    # the photograph is not itself a feature the search can align on.
    theirs = committed.crop((0, 0, width, band - 4))

    # The render at the ramp's origin, where the declared scale is exactly 1.0 —
    # so whatever scale and offset the search recovers is the *published* move,
    # not a difference between two moves.
    base = luma(rendered(ramp_origin_ms(), scratch, binary)).resize(
        (width, height), Image.LANCZOS
    )
    ours = base.crop((0, 0, width, band))

    best = None
    steps = 29
    for step in range(steps):
        scale = 1.0 + 0.005 * step
        big = ours.resize(
            (round(width * scale), round(band * scale)), Image.LANCZOS
        )
        for dy in range(0, min(big.height - theirs.height, 40) + 1):
            for dx in range(0, min(big.width - theirs.width, 24) + 1):
                window = big.crop((dx, dy, dx + theirs.width, dy + theirs.height))
                score = mad(theirs, window)
                if best is None or score < best[0]:
                    best = (score, scale, dx, dy)
        if verbose:
            print(f"    scale {scale:.3f} -> best so far MAD {best[0]:.3f}")

    score, scale, dx, dy = best
    origin, (pivot_x, pivot_y), scale_at, (el_width, el_height) = declared()
    # Back into the project's own pixels.
    full = el_width / width
    dx_full, dy_full = dx * full, dy * full

    declared_scale = scale_at(at)
    # A pivot fraction f puts the content (f x extent x (scale - 1)) px along as
    # the zoom grows, measured against the same element rendered at scale 1.0.
    predicted = ((scale - 1.0) * pivot_x, (scale - 1.0) * pivot_y)

    def distance(keyword):
        fx, fy = ORIGIN_FRACTION[keyword]
        return (abs(dx_full - (scale - 1.0) * fx * el_width)
                + abs(dy_full - (scale - 1.0) * fy * el_height))

    print(f"  {name} at {at:.0f} ms — the published photograph is the fixture's still at")
    print(f"    scale  {scale:.3f}   (the fixture declares {declared_scale:.4f})")
    print(f"    offset ({dx_full:.0f}, {dy_full:.0f}) px")
    print(f"    the fixture's `{origin}` pivot predicts ({predicted[0]:.0f}, {predicted[1]:.0f})")

    # The scale must land on the declared ramp — that is what says the amplitude
    # and the timing in the file are right, and that the offset below is a
    # statement about the pivot alone.
    scale_ok = abs(scale - declared_scale) <= 0.01

    # **Against all nine keywords, not against `top-left` alone.** Checking only
    # the spelling #276 replaced would pass a fixture declaring any of the other
    # seven: `center-left` at (0, 956) predicts (0, 57) against a recovered
    # (24, 52), which clears the tolerance comfortably. What is asserted instead is
    # that the keyword the file names is the *best* of the nine and within reach of
    # the recovered offset -- so this fails for every wrong pivot, not just one.
    ranked = sorted(ORIGIN_FRACTION, key=distance)
    to_declared = distance(origin)
    runner_up = ranked[1] if ranked[0] == origin else ranked[0]
    pivot_ok = ranked[0] == origin and to_declared <= OFFSET_TOLERANCE_PX

    print(
        f"    -> {to_declared:.0f} px from `{origin}` (tolerance "
        f"{OFFSET_TOLERANCE_PX:.0f}); nearest of the other eight is `{runner_up}` at "
        f"{distance(runner_up):.0f} px"
    )
    if not (scale_ok and pivot_ok):
        print(
            "    FAILED: the Ken Burns finding recorded in "
            "tests/reference_frames.rs no longer reproduces.",
            file=sys.stderr,
        )
    return 0 if (scale_ok and pivot_ok) else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument(
        "--binary",
        metavar="PATH",
        help="an already-built `montaget` to render through, instead of `cargo run`",
    )
    parser.add_argument(
        "--instants-only",
        action="store_true",
        help="check only which frame each PNG is; skip the pivot half, which "
        "renders through the built binary",
    )
    parser.add_argument(
        "--keep",
        metavar="DIR",
        help="scratch directory to reuse, so a second run decodes nothing",
    )
    args = parser.parse_args()

    if shutil.which("ffmpeg") is None:
        sys.exit("this script needs `ffmpeg` on PATH")
    if not VIDEO.exists():
        sys.exit(f"the published video is missing: {VIDEO}")

    scratch = Path(args.keep) if args.keep else Path(tempfile.mkdtemp())
    os.makedirs(scratch, exist_ok=True)

    failures = 0
    print("Which frame of the published MP4 is each committed reference?")
    for name, expected in EXPECTED.items():
        failures += identify(name, expected, scratch, args.verbose)

    if args.instants_only:
        print()
        print("skipped the pivot half (--instants-only)")
    else:
        print()
        print("What move does the published photograph actually make?")
        failures += pivot(scratch, args.verbose, args.binary)

    print()
    if failures:
        print(f"{failures} claim(s) no longer reproduce.")
        return 1
    print("every claim reproduces.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
