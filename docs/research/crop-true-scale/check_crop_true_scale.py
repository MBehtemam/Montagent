#!/usr/bin/env python3
"""Re-derive every number in this directory's README and fail if one stops holding.

Run it from anywhere:

    python3 docs/research/crop-true-scale/check_crop_true_scale.py

Exit status is 0 only when every assertion below holds. There are four groups:

  A. The scale rule ADR-0101 adopts, read off the real binary's own caption: a
     `--crop` comes back at true scale with no `--full`, the uncropped default
     is still half scale, and `--crop --full` is byte-identical to `--crop` —
     redundant rather than illegal.
  B. The equivalence that makes the fix meaningful: `--crop --png` is
     pixel-identical to cropping a `--full --png` whole frame externally, which
     is the workaround #402's reporting agent fell back to. If this fails the
     fix has not delivered what the flag's doc string promises.
  C. The colour decomposition, and the correction it forces on #402 as filed.
     The issue attributed a 573 -> 10514 distinct-colour blowup to "downscale
     resampling", comparing a PNG true-scale crop against a JPEG half-scale
     one — two variables at once. Separated, resampling is the *small* term and
     JPEG is the large one, and JPEG at true scale is worse than JPEG at half
     scale. This group is the load-bearing one for the README's claim that true
     scale alone does not make `--crop` fit for pixel analysis.
  D. The whole-frame-crop consequence: `--crop 0,0,W,H` is byte-identical to
     `--full`, so the rule opens no discount on the half-scale default — it is
     `--full` spelled longer. ADR-0011's published formula is re-derived for the
     two pairs this repo has measured; the standard tier's 819x1456 -> 1560 is
     cited from #405 rather than recomputed, because the serving downscale is
     not a published long-edge rule and a guessed reimplementation would be a
     number this repo does not stand behind.

Groups A-C need `./target/debug/montagent` and Pillow; they are skipped
(loudly, and without failing) when either is absent, so this script still
checks group D in a bare checkout. Nothing here reasons about token counts from
memory: group D applies the formula ADR-0011 publishes and the tier caps
ADR-0097 records, both quoted in README.md.
"""

from __future__ import annotations

import hashlib
import math
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
BINARY = REPO / "target" / "debug" / "montagent"
FIXTURE = (
    REPO
    / "fixtures"
    / "en-halloween-decorating"
    / "en-halloween-decorating.montagent.json"
)

# The instant and region #402 filed, unchanged.
AT = 50000
CROP = (48, 1300, 984, 340)
FRAME = (1080, 1920)

failures: list[str] = []
skipped: list[str] = []


def check(claim: str, ok: bool, detail: str = "") -> None:
    print(f"  {'PASS' if ok else 'FAIL'}  {claim}" + (f"  [{detail}]" if detail else ""))
    if not ok:
        failures.append(claim)


def frame(out: Path, *extra: str) -> str:
    """Run `frame` at the fixed instant and return its stdout."""
    argv = [
        str(BINARY), "frame", str(FIXTURE),
        "--at", str(AT), "--out", str(out), *extra,
    ]
    done = subprocess.run(argv, capture_output=True, text=True, check=True)
    return done.stdout


def caption(stdout: str) -> str:
    """The one FRAME line the scale and served dimensions are published on."""
    for line in stdout.splitlines():
        if line.startswith("FRAME"):
            return line
    raise AssertionError("no FRAME line in the answer")


def tokens(width: int, height: int, cap: int) -> int:
    """ADR-0011's `ceil(w/28) * ceil(h/28)` on the dimensions a tier serves.

    `cap` is a **long-edge** cap, which is the only serving rule this script
    models — and it is enough for every pair asserted below, because each one is
    served untouched by the tier it is checked against. It is deliberately *not*
    used to re-derive the standard tier's 1080x1920 -> 819x1456 -> 1560: that
    downscale is not a long-edge cap (a long-edge cap would give 881x1568 and
    1792), so reproducing it would mean guessing at an unpublished rule. That
    figure is #405's measurement, cited in README.md and checked by
    `docs/research/visual-token-cost/`.
    """
    long_edge = max(width, height)
    if long_edge > cap:
        ratio = cap / long_edge
        width, height = int(width * ratio), int(height * ratio)
    return math.ceil(width / 28) * math.ceil(height / 28)


def main() -> int:
    scratch = Path(__file__).resolve().parent / ".scratch"
    scratch.mkdir(exist_ok=True)

    have_binary = BINARY.is_file() and FIXTURE.is_file()
    try:
        from PIL import Image, ImageChops  # noqa: F401

        have_pillow = True
    except ImportError:
        have_pillow = False

    print("\nA. The scale rule ADR-0101 adopts, off the binary's own caption")
    if not have_binary:
        skipped.append("A (no ./target/debug/montagent — run `cargo build`)")
        print("  SKIP  needs ./target/debug/montagent")
    else:
        x, y, w, h = CROP
        spec = f"{x},{y},{w},{h}"

        cropped = caption(frame(scratch / "crop.png", "--png", "--crop", spec))
        check(
            "a `--crop` is served at true scale with no `--full`",
            f"full scale, {w}x{h}" in cropped,
            cropped.strip(),
        )

        plain = caption(frame(scratch / "plain.png", "--png"))
        check(
            "the uncropped default is still half scale — ADR-0011's default stands",
            f"half scale, {FRAME[0] // 2}x{FRAME[1] // 2}" in plain,
            plain.strip(),
        )

        both = caption(
            frame(scratch / "crop-full.png", "--png", "--crop", spec, "--full")
        )
        check(
            "`--crop --full` reports the same scale and dimensions as `--crop`",
            both == cropped,
        )
        check(
            "`--crop --full` is byte-identical to `--crop` — redundant, not illegal",
            hashlib.sha256((scratch / "crop.png").read_bytes()).digest()
            == hashlib.sha256((scratch / "crop-full.png").read_bytes()).digest(),
        )

    print("\nB. The crop equals an external crop of a full frame")
    if not (have_binary and have_pillow):
        skipped.append("B (needs the binary and Pillow)")
        print("  SKIP  needs ./target/debug/montagent and Pillow")
    else:
        from PIL import Image, ImageChops

        x, y, w, h = CROP
        frame(scratch / "whole-full.png", "--png", "--full")
        whole = Image.open(scratch / "whole-full.png").convert("RGB")
        external = whole.crop((x, y, x + w, y + h))
        got = Image.open(scratch / "crop.png").convert("RGB")
        check(
            "`--crop --png` is pixel-identical to cropping `--full --png` externally",
            ImageChops.difference(external, got).getbbox() is None,
            "the workaround #402's agent fell back to is now unnecessary",
        )

    print("\nC. The colour decomposition, and #402's own premise")
    if not (have_binary and have_pillow):
        skipped.append("C (needs the binary and Pillow)")
        print("  SKIP  needs ./target/debug/montagent and Pillow")
    else:
        from PIL import Image

        x, y, w, h = CROP
        spec = f"{x},{y},{w},{h}"

        def distinct_of(path: Path) -> int:
            return len(Image.open(path).convert("RGB").getcolors(maxcolors=10**7))

        def distinct(name: str, *extra: str) -> int:
            out = scratch / name
            frame(out, *extra)
            return distinct_of(out)

        # The half-scale arm is no longer reachable through `--crop` — that is the
        # whole point of the fix — so it is the pre-fix binary's own output,
        # committed in `before/` rather than re-derived with a resampling filter
        # this script would have to guess at.
        before = Path(__file__).resolve().parent / "before"
        region_png_half = distinct_of(before / "region-png-half-scale.png")
        region_png_true = distinct("c-png-true.png", "--crop", spec, "--png")
        region_jpeg_true = distinct("c-jpeg-true.jpg", "--crop", spec)

        print(f"        region PNG  true scale 984x340: {region_png_true:7d}")
        print(f"        region PNG  half scale 492x170: {region_png_half:7d}  (pre-fix, committed)")
        print(f"        region JPEG true scale 984x340: {region_jpeg_true:7d}")

        check(
            "the committed pre-fix artifact is the half-scale region #402 filed",
            Image.open(before / "region-png-half-scale.png").size == (w // 2, h // 2),
        )
        check(
            "on a flat-ink region, resampling fabricates colour: half scale holds "
            "MORE distinct values than true scale, on a quarter of the pixels",
            region_png_half > region_png_true,
            f"{region_png_true} -> {region_png_half}",
        )
        check(
            "at identical dimensions, encoding is the dominant fabricator — one "
            "variable changed, an order of magnitude past the resampling term",
            region_jpeg_true > 10 * region_png_half,
            f"resampling +{region_png_half - region_png_true}, "
            f"JPEG +{region_jpeg_true - region_png_true}",
        )
        check(
            "so #402's stated mechanism is wrong: JPEG at TRUE scale fabricates "
            "more than resampling ever did, which is why true scale alone does "
            "not make `--crop` fit for pixel analysis",
            region_jpeg_true > region_png_half,
            f"JPEG true {region_jpeg_true} > PNG half {region_png_half}",
        )

        # The honesty guard on the claim above: it is about flat ink, not images.
        whole_true = distinct("c-whole-true.png", "--png", "--full")
        whole_half = distinct("c-whole-half.png", "--png")
        print(f"        whole PNG true 1080x1920:       {whole_true:7d}")
        print(f"        whole PNG half 540x960:         {whole_half:7d}")
        check(
            "and the resampling claim is SCOPED to flat ink, not general: on the "
            "photographic whole frame it inverts, so this evidence must never be "
            "cited as 'downscaling always adds colours'",
            whole_half < whole_true,
            f"whole frame {whole_true} -> {whole_half}",
        )

    print("\nD. The whole-frame crop opens no discount (arithmetic only)")
    w, h = FRAME
    # Only the pairs this repo has actually measured are re-derived here. The
    # standard tier's 819x1456 -> 1560 is #405's measured datum, checked by
    # `docs/research/visual-token-cost/`; it is cited, not recomputed, because
    # the serving downscale is not a published long-edge rule and a guessed
    # reimplementation of it would be a number this repo does not stand behind.
    check(
        f"ADR-0011's formula gives 2691 for a {w}x{h} frame served untouched "
        "(the high-resolution tier, which caps at 2576 px)",
        tokens(w, h, 2576) == 2691,
        f"got {tokens(w, h, 2576)}",
    )
    check(
        "and 700 for the half-scale default, on either tier — so the default "
        "this rule leaves alone is untouched",
        tokens(w // 2, h // 2, 2576) == 700 == tokens(w // 2, h // 2, 1568),
        f"got {tokens(w // 2, h // 2, 2576)}",
    )
    if not have_binary:
        skipped.append("D's whole-frame-crop identity (needs the binary)")
        print("  SKIP  the whole-frame-crop identity needs ./target/debug/montagent")
    else:
        whole_crop = scratch / "d-whole-crop.png"
        whole_full = scratch / "d-whole-full.png"
        frame(whole_crop, "--png", "--crop", f"0,0,{w},{h}")
        frame(whole_full, "--png", "--full")
        check(
            f"`--crop 0,0,{w},{h}` is byte-identical to `--full`, so it costs the "
            "same on every tier: the rule is `--full` spelled longer, never a "
            "cheaper route to it",
            hashlib.sha256(whole_crop.read_bytes()).digest()
            == hashlib.sha256(whole_full.read_bytes()).digest(),
        )

    print()
    for note in skipped:
        print(f"SKIPPED: {note}")
    if failures:
        print(f"\n{len(failures)} claim(s) no longer hold:")
        for claim in failures:
            print(f"  - {claim}")
        return 1
    print("Every claim checked here holds.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
