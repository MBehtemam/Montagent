#!/usr/bin/env python3
"""Re-derive every number in this directory's README and fail if one stops holding.

Run it from anywhere:

    python3 docs/research/visual-token-cost/check_visual_token_cost.py

Exit status is 0 only when every assertion below holds. There are four groups:

  A. The published cost formula and the two resolution tiers, checked against
     every row of Anthropic's own published table (the primary source, quoted
     in README.md). This is the load-bearing group: if it fails, either the
     reference implementation transcribed here is wrong or the documented
     table has changed and the README needs re-reading against it.
  B. Montagent's own claim, as `frame`'s doc string states it. This group is
     the one that already fails against the doc string as written, so the
     assertions encode what is *true*, and the README says which half of the
     doc string is wrong.
  C. Encoding independence, measured against the real binary when it is built:
     the same instant as PNG and as JPEG at the same scale costs identical
     visual tokens while differing by a large factor in bytes.
  D. The caption's share of the cost: the resolved-stack text that every
     `frame` call returns unconditionally, measured over the fixture's 18
     visual states, against a concrete one-sheet answer carrying the same
     identifying information as short labels.

Group C needs `./target/debug/montagent`; it is skipped (loudly, and without
failing) when the binary is absent. Group D runs against the captions committed
next to this script and, when the binary *is* built, additionally asserts that
freshly generated captions still match those bytes — so a change to `frame`'s
output that moves these numbers fails here rather than silently rotting.

Token counts for TEXT are not computed offline: this repo's standing rule is to
not reason from memory about token arithmetic, and there is no offline Claude
tokenizer (`tiktoken` is OpenAI's and is wrong for Claude). Set
ANTHROPIC_API_KEY and the script will call `POST /v1/messages/count_tokens` and
print exact text-token figures; without it, group D reports the exact,
tokenizer-independent quantities (characters, words, lines) and the ratio
between the two designs, which is what the budget question turns on.
"""

from __future__ import annotations

import json
import math
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
BINARY = REPO / "target" / "debug" / "montagent"
# Relative, and every subprocess runs with cwd=REPO: `frame` echoes the project
# path it was given back into its caption, so an absolute path here would make
# the committed captions machine-specific and their character counts unstable.
PROJECT = Path("fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json")
CAPTIONS = HERE / "captions"

FAILURES: list[str] = []
SKIPS: list[str] = []


def check(label: str, got, want) -> None:
    if got == want:
        print(f"  ok    {label}: {got}")
    else:
        print(f"  FAIL  {label}: got {got!r}, want {want!r}")
        FAILURES.append(label)


# ---------------------------------------------------------------------------
# The documented cost model, transcribed from Anthropic's reference
# implementation (see README.md for the URL and the verbatim quote).
# ---------------------------------------------------------------------------

STANDARD_TIER = (1568, 1568)  # (max long edge px, max visual tokens)
HIGH_RES_TIER = (2576, 4784)  # Claude 4.7 and later


def count_image_tokens(width: int, height: int) -> int:
    """Visual tokens consumed by an image: one token per 28x28 pixel patch."""
    return math.ceil(width / 28) * math.ceil(height / 28)


def resized_size(
    width: int, height: int, max_edge: int = 1568, max_tokens: int = 1568
) -> tuple[int, int]:
    """The size Claude resizes an image to before padding. Returns (width, height)."""

    def fits(w: int, h: int) -> bool:
        return (
            math.ceil(w / 28) * 28 <= max_edge
            and math.ceil(h / 28) * 28 <= max_edge
            and count_image_tokens(w, h) <= max_tokens
        )

    if fits(width, height):
        return (width, height)
    if height > width:
        resized_h, resized_w = resized_size(height, width, max_edge, max_tokens)
        return (resized_w, resized_h)

    aspect_ratio = width / height
    lo, hi = 1, width  # lo always fits; hi never fits
    while lo + 1 < hi:
        mid = (lo + hi) // 2
        if fits(mid, max(round(mid / aspect_ratio), 1)):
            lo = mid
        else:
            hi = mid
    return (lo, max(round(lo / aspect_ratio), 1))


def served(width: int, height: int, tier: tuple[int, int]) -> tuple[int, int, int]:
    """(served_width, served_height, visual_tokens) for an image on `tier`."""
    w, h = resized_size(width, height, tier[0], tier[1])
    return w, h, count_image_tokens(w, h)


# ---------------------------------------------------------------------------
# A. The published formula and tier table.
# ---------------------------------------------------------------------------

# Every row of the "Resolution and token cost" table, transcribed from the
# Vision docs. Columns: (w, h, standard downsize or None, standard tokens,
# high-res downsize or None, high-res tokens).
PUBLISHED_TABLE = [
    (200, 200, None, 64, None, 64),
    (1000, 1000, None, 1296, None, 1296),
    (1092, 1092, None, 1521, None, 1521),
    (1920, 1080, (1456, 819), 1560, None, 2691),
    # The table prints 1269x952 for this row; Anthropic's own reference
    # implementation, transcribed above, returns 1270x952. One pixel of width,
    # and it does not move the token count (ceil(1269/28) == ceil(1270/28) == 46),
    # so the cost model is unaffected and the discrepancy is recorded rather
    # than papered over. See README.md, "Two errors found in the sources".
    (2000, 1500, (1270, 952), 1564, None, 3888),
    (3840, 2160, (1456, 819), 1560, (2576, 1449), 4784),
]

# The worked example from "How Claude resizes and pads images": an A4 page at
# 130 DPI that both tiers treat differently.
A4_130DPI = (1075, 1520)


def group_a() -> None:
    print("\nA. Anthropic's published formula and tier table")
    for w, h, std_size, std_tok, hi_size, hi_tok in PUBLISHED_TABLE:
        sw, sh, st = served(w, h, STANDARD_TIER)
        hw, hh, ht = served(w, h, HIGH_RES_TIER)
        check(f"{w}x{h} standard tokens", st, std_tok)
        check(f"{w}x{h} standard served size", (sw, sh), std_size or (w, h))
        check(f"{w}x{h} high-res tokens", ht, hi_tok)
        check(f"{w}x{h} high-res served size", (hw, hh), hi_size or (w, h))

    # The A4 example, which shows the token limit (not the edge limit) driving
    # the resize: both sides are under 1568 px and it still gets downscaled.
    check("A4@130dpi raw tokens", count_image_tokens(*A4_130DPI), 2145)
    check(
        "A4@130dpi standard served size",
        served(*A4_130DPI, STANDARD_TIER)[:2],
        (924, 1307),
    )
    check(
        "A4@130dpi high-res served size (not resized)",
        served(*A4_130DPI, HIGH_RES_TIER)[:2],
        A4_130DPI,
    )


# ---------------------------------------------------------------------------
# B. Montagent's own claim, as `frame`'s doc string states it.
# ---------------------------------------------------------------------------

HALF = (540, 960)  # `frame`'s default
FULL = (1080, 1920)  # `frame --full`, the fixture's true frame size


def group_b() -> None:
    print("\nB. frame's doc string: '2691 at 1080x1920 against 700 at 540x960'")

    # The half-scale figure holds on both tiers: 540x960 is inside every limit,
    # so it is served at its own dimensions and the bare formula applies.
    check("540x960 raw tokens", count_image_tokens(*HALF), 700)
    check("540x960 standard served", served(*HALF, STANDARD_TIER), (540, 960, 700))
    check("540x960 high-res served", served(*HALF, HIGH_RES_TIER), (540, 960, 700))

    # The full-scale figure does NOT hold on both tiers. 2691 is the bare
    # formula's answer and it survives only on the high-resolution tier.
    check("1080x1920 raw tokens (bare formula)", count_image_tokens(*FULL), 2691)
    check("1080x1920 high-res served", served(*FULL, HIGH_RES_TIER), (1080, 1920, 2691))
    # On the standard tier the long edge (1920) exceeds 1568, so it is
    # downscaled and costs 1560 -- not 2691. This is the doc string's error.
    check("1080x1920 standard served", served(*FULL, STANDARD_TIER), (819, 1456, 1560))
    check(
        "--full is 3.84x --half on high-res tier",
        round(2691 / 700, 2),
        3.84,
    )
    check(
        "--full is only 2.23x --half on standard tier",
        round(1560 / 700, 2),
        2.23,
    )

    # The contact-sheet arithmetic. A 4x4 grid of 135x240 tiles fills a
    # 540x960 sheet exactly.
    tile = (135, 240)
    check("4x4 grid of 135x240 tiles fills 540x960", (4 * 135, 4 * 240), HALF)
    check("one 540x960 sheet", count_image_tokens(*HALF), 700)
    check("one 135x240 tile as its own image", count_image_tokens(*tile), 45)
    check("16 separate 135x240 images", 16 * count_image_tokens(*tile), 720)
    # N small images are strictly worse than one sheet of the same total
    # pixels, because each one pays its own ceil() padding to a 28px multiple.
    if 16 * count_image_tokens(*tile) <= count_image_tokens(*HALF):
        print("  FAIL  tiling is not cheaper than N images of the same total pixels")
        FAILURES.append("tiling cheaper than N images")
    else:
        print("  ok    tiling beats 16 separate tiles by 20 tokens (720 -> 700)")

    # And the honest version of "16 for the price of one": 16 instants at
    # frame's *default* resolution is 16x700, not 700. The sheet is one image's
    # pixel budget divided 16 ways, so each instant is served at 45 tokens.
    check("16 instants at frame's default scale", 16 * 700, 11200)
    check("ratio a sheet actually saves (16 frames -> 1 sheet)", 11200 // 700, 16)
    check("resolution each instant loses", round(700 / 45, 1), 15.6)

    # The fixture's actual case: 18 visual states, 4x5 grid, 540x960 sheet.
    tile18 = (540 // 4, 960 // 5)
    check("4x5 grid tile size in a 540x960 sheet", tile18, (135, 192))
    check("one 135x192 tile as its own image", count_image_tokens(*tile18), 35)
    check("18 separate 135x192 images", 18 * count_image_tokens(*tile18), 630)
    check("one sheet holding all 18", count_image_tokens(*HALF), 700)
    # Here N small images are *cheaper* in visual tokens than the sheet, because
    # two of the twenty grid cells are empty and the sheet pays for them anyway.
    # The sheet still wins overall -- see group D -- but it wins on the caption,
    # not on the pixels. Assert the direction so this cannot be misread later.
    if 18 * count_image_tokens(*tile18) >= count_image_tokens(*HALF):
        print("  FAIL  expected 18 loose tiles to undercut the sheet on pixels alone")
        FAILURES.append("18-tile pixel direction")
    else:
        print(
            "  ok    18 loose 135x192 tiles cost 630 against the sheet's 700: "
            "tiling does not save visual tokens at equal tile size"
        )
    check("18 instants at frame's default scale", 18 * 700, 12600)
    check("18 frames -> 1 sheet, visual tokens saved", 18 * 700 - 700, 11900)


# ---------------------------------------------------------------------------
# C. Encoding independence, measured against the real binary.
# ---------------------------------------------------------------------------

FRAME_HEADER = re.compile(
    r"FRAME\s+at (\d+) — (\w+), (\S+) scale, (\d+)x(\d+) from a (\d+)x(\d+) frame"
)
WRITTEN = re.compile(r"written to\s+(\S+) \((\d+) bytes\)")


def run_frame(at: int, out: Path, *flags: str) -> tuple[str, dict]:
    proc = subprocess.run(
        [str(BINARY), "frame", str(PROJECT), "--at", str(at), "--out", str(out), *flags],
        capture_output=True,
        text=True,
        cwd=REPO,
    )
    if proc.returncode != 0:
        raise SystemExit(f"montagent frame failed: {proc.stderr or proc.stdout}")
    text = proc.stdout
    header = FRAME_HEADER.search(text)
    written = WRITTEN.search(text)
    if not header or not written:
        raise SystemExit(f"could not parse frame output:\n{text}")
    meta = {
        "encoding": header.group(2),
        "scale": header.group(3),
        "served": (int(header.group(4)), int(header.group(5))),
        "frame": (int(header.group(6)), int(header.group(7))),
        "bytes": int(written.group(2)),
    }
    # Normalize the absolute out path so captions are comparable across machines.
    return text.replace(str(out), "OUT." + out.suffix.lstrip(".")), meta


def group_c(tmp: Path) -> None:
    print("\nC. Encoding independence, measured (JPEG vs PNG)")
    if not BINARY.exists():
        print(f"  SKIP  {BINARY} is not built (cargo build); group C not measured")
        SKIPS.append("C")
        return

    _, jpg = run_frame(30000, tmp / "half.jpg")
    _, png = run_frame(30000, tmp / "half.png", "--png")
    check("jpeg served size", jpg["served"], HALF)
    check("png served size", png["served"], HALF)
    check("project frame size", jpg["frame"], FULL)
    check(
        "identical visual tokens for both encodings",
        (count_image_tokens(*jpg["served"]), count_image_tokens(*png["served"])),
        (700, 700),
    )
    if png["bytes"] <= jpg["bytes"] * 3:
        print(
            f"  FAIL  expected PNG to be >3x the JPEG bytes; "
            f"got png={png['bytes']} jpeg={jpg['bytes']}"
        )
        FAILURES.append("png/jpeg byte ratio")
    else:
        print(
            f"  ok    same 700 tokens, {png['bytes'] / jpg['bytes']:.1f}x the bytes "
            f"(png {png['bytes']}, jpeg {jpg['bytes']})"
        )

    # Bytes matter for the request, not for the token count: 18 full-scale PNGs
    # would blow the 32 MB request limit that the same 18 images cannot reach
    # as JPEG.
    _, full_png = run_frame(30000, tmp / "full.png", "--png", "--full")
    _, full_jpg = run_frame(30000, tmp / "full.jpg", "--full")
    check("full-scale served size", full_png["served"], FULL)
    check(
        "identical visual tokens at full scale for both encodings",
        (
            count_image_tokens(*full_png["served"]),
            count_image_tokens(*full_jpg["served"]),
        ),
        (2691, 2691),
    )
    b64 = 4 / 3  # base64 expansion; the API's size limits are on encoded bytes
    over = 18 * full_png["bytes"] * b64 > 32 * 1024 * 1024
    under = 18 * full_jpg["bytes"] * b64 < 32 * 1024 * 1024
    if not (over and under):
        print(
            "  FAIL  expected 18 full PNGs to exceed, and 18 full JPEGs to fit, "
            f"the 32 MB request limit (png {full_png['bytes']}, jpeg {full_jpg['bytes']})"
        )
        FAILURES.append("32 MB request-limit crossover")
    else:
        print(
            f"  ok    18 full-scale PNGs base64 to "
            f"{18 * full_png['bytes'] * b64 / 1024 / 1024:.1f} MB (over the 32 MB "
            f"request limit); 18 JPEGs to "
            f"{18 * full_jpg['bytes'] * b64 / 1024 / 1024:.1f} MB (under) — same tokens"
        )


# ---------------------------------------------------------------------------
# D. The caption's share of the cost.
# ---------------------------------------------------------------------------


def visual_state_onsets() -> list[int]:
    """The instants where the *visual* presence set changes, from the cut list.

    Collapses the audio-only boundaries (`vo-*` elements) exactly as the two
    trial agents did by hand, per issue #395.
    """
    proc = subprocess.run(
        [
            str(BINARY),
            "query",
            str(PROJECT),
            "--from",
            "0",
            "--to",
            "65216",
        ],
        capture_output=True,
        text=True,
        cwd=REPO,
    )
    if proc.returncode != 0:
        raise SystemExit(f"montagent query failed: {proc.stderr or proc.stdout}")
    intervals = []
    for line in proc.stdout.splitlines():
        m = re.match(r"\s+(\d+)\.\.(\d+)\s+\d+ ms\s+(.*)$", line)
        if m:
            ids = [s.strip() for s in m.group(3).split(",")]
            intervals.append((int(m.group(1)), tuple(i for i in ids if not i.startswith("vo-"))))
    check("cut-list intervals over the whole fixture", len(intervals), 46)
    onsets, prev = [], None
    for start, visual in intervals:
        if visual != prev:
            onsets.append(start)
            prev = visual
    return onsets


EXPECTED_ONSETS = [
    0, 3018, 5316, 10468, 17472, 22622, 30603, 35753, 42763,
    47343, 53856, 56116, 57116, 58116, 59116, 60116, 61116, 64016,
]


def sheet_answer(onsets: list[int], labels: dict[int, str]) -> str:
    """One contact-sheet answer carrying the same identifying information.

    Deliberately minimal but complete: the disclosure the map requires (the
    selection rule, the tile scale, anything skipped) plus one label per tile
    naming the instant and what produced it. This is the text side of the
    alternative being costed, built from the same data the captions carry, so
    the comparison is between two texts rather than between a text and a guess.
    """
    lines = [
        "0 errors, 0 reviews, 0 notes, 0 unchecked, 0 layout, 0 drift — "
        f"{PROJECT}",
        "",
        f"SHEET  [0, 65216) — {len(onsets)} tiles, 4x5 grid, 135x192 per tile "
        "from a 1080x1920 frame",
        "  selection   one instant per visual-state boundary in the cut list; "
        "audio-only boundaries collapsed",
        "  skipped     nothing",
        "  written to  OUT.jpg",
    ]
    for i, ms in enumerate(onsets, start=1):
        lines.append(f"  {i:>2}  {ms:>6} ms  {labels[ms]}")
    lines += [
        "",
        "NOT CHECKED",
        "  This file was not compared against any prior version or instruction.",
        "  validate verifies that the file is internally legal; it cannot tell you",
        "  whether it says what you meant it to say.",
    ]
    return "\n".join(lines) + "\n"


def caption_labels(captions: dict[int, str]) -> dict[int, str]:
    """The short label a tile would carry: the instant's non-chrome elements.

    Taken from each caption's own `painted` line, minus the persistent chrome
    that is present for the whole duration and so says nothing about the tile.
    """
    chrome = {
        "chip-panel", "handle-panel", "flag-field", "handle-logo", "handle-text",
        "flag-bar-h", "flag-bar-v", "chip-text",
    }
    labels = {}
    for ms, text in captions.items():
        m = re.search(r"painted\s+\d+ elements \(back to front\): (.*)$", text, re.M)
        if not m:
            raise SystemExit(f"caption for {ms} has no painted line")
        ids = [s.strip() for s in m.group(1).split(",")]
        labels[ms] = ", ".join(i for i in ids if i not in chrome)
    return labels


def count_tokens(texts: dict[str, str]) -> dict[str, int] | None:
    """Exact text-token counts via the Messages API, when a key is available."""
    key = os.environ.get("ANTHROPIC_API_KEY")
    if not key:
        return None
    import urllib.error
    import urllib.request

    out = {}
    for name, text in texts.items():
        body = json.dumps(
            {
                "model": "claude-opus-5",
                "messages": [{"role": "user", "content": text}],
            }
        ).encode()
        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages/count_tokens",
            data=body,
            headers={
                "x-api-key": key,
                "anthropic-version": "2023-06-01",
                "content-type": "application/json",
            },
        )
        try:
            with urllib.request.urlopen(req) as resp:
                out[name] = json.load(resp)["input_tokens"]
        except urllib.error.HTTPError as e:
            print(f"  SKIP  count_tokens failed for {name}: {e.read().decode()[:200]}")
            return None
    return out


def group_d(tmp: Path) -> None:
    print("\nD. The unconditional caption's share of the cost")

    committed = {
        int(p.stem): p.read_text() for p in sorted(CAPTIONS.glob("*.txt"))
    }
    check("committed captions", len(committed), 18)
    check("committed caption instants", sorted(committed), EXPECTED_ONSETS)

    if BINARY.exists():
        onsets = visual_state_onsets()
        check("visual-state onsets derived from the cut list", onsets, EXPECTED_ONSETS)
        for ms in onsets:
            fresh, _ = run_frame(ms, tmp / f"{ms}.jpg")
            if fresh != committed[ms]:
                print(f"  FAIL  fresh caption at {ms} differs from the committed one")
                FAILURES.append(f"caption drift at {ms}")
        if not any(f.startswith("caption drift") for f in FAILURES):
            print("  ok    all 18 fresh captions match the committed bytes")
    else:
        print(f"  SKIP  {BINARY} is not built; captions not regenerated")
        SKIPS.append("D-regen")

    labels = caption_labels(committed)
    sheet = sheet_answer(EXPECTED_ONSETS, labels)
    (tmp / "sheet-answer.txt").write_text(sheet)

    per_caption = {ms: len(t) for ms, t in committed.items()}
    total_caption = sum(per_caption.values())
    check("chars in one caption at 30603", per_caption[30603], 1530)
    check("chars in 18 captions", total_caption, 28410)
    check(
        "bytes in 18 captions (UTF-8; the em dashes cost 3 each)",
        sum(len(t.encode()) for t in committed.values()),
        28518,
    )
    check("chars in one equivalent sheet answer", len(sheet), 1413)
    ratio = total_caption / len(sheet)
    check("caption/sheet character ratio", round(ratio, 1), 20.1)

    words = sum(len(t.split()) for t in committed.values())
    check("words in 18 captions", words, 3588)
    check("words in the sheet answer", len(sheet.split()), 183)
    check("lines in 18 captions", sum(t.count("\n") for t in committed.values()), 456)

    exact = count_tokens(
        {"18 captions": "\n".join(committed[ms] for ms in EXPECTED_ONSETS), "sheet": sheet}
    )
    if exact is None:
        print(
            "  note  set ANTHROPIC_API_KEY to get exact text-token counts via\n"
            "        POST /v1/messages/count_tokens. Characters and words above are\n"
            "        exact and tokenizer-independent; the ratio is the load-bearing\n"
            "        number and both texts are the same register, so it carries over."
        )
        SKIPS.append("D-tokens")
    else:
        print(f"  ok    exact text tokens: {exact}")
        if exact["18 captions"] <= exact["sheet"] * 5:
            print("  FAIL  expected the 18 captions to cost >5x the sheet answer")
            FAILURES.append("caption/sheet token ratio")


def main() -> int:
    tmp = Path(os.environ.get("TMPDIR", "/tmp")) / "montagent-visual-token-check"
    tmp.mkdir(parents=True, exist_ok=True)
    print(f"repo      {REPO}")
    print(f"binary    {BINARY} ({'built' if BINARY.exists() else 'ABSENT'})")
    print(f"scratch   {tmp}")

    group_a()
    group_b()
    group_c(tmp)
    group_d(tmp)

    print()
    if SKIPS:
        print(f"skipped: {', '.join(SKIPS)}")
    if FAILURES:
        print(f"FAILED ({len(FAILURES)}): {', '.join(FAILURES)}")
        return 1
    print("all assertions hold")
    return 0


if __name__ == "__main__":
    sys.exit(main())
