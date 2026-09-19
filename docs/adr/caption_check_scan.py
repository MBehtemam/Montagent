#!/usr/bin/env python3
"""Re-derives the caption-check numbers ADR-0071 corrects. Exits non-zero if any of them
stops reproducing.

    python3 docs/adr/caption_check_scan.py

ADR-0034 states a metric — grapheme clusters, *"spaces included, `\\n` excluded"* — and,
two paragraphs above it, an evidence table whose character counts include the break. The
two disagree on what fires: on the committed fixture the table's counts put
`quiz-question` at 20.4 cps and over the line, and the metric's counts put it at 19.9 and
under it. This script computes the fixture both ways, so which reading is in force is a
number a reader can re-derive rather than a claim to take on trust.

It asserts **both directions** of each claim, the way
`0013-fitted-extents-floor-and-the-nine-origin-keywords.md`'s scan does: that the metric's
counts fire exactly once, *and* that the table's counts would fire twice — because a scan
that only confirmed the current behaviour would go quiet if the count changed for some
third reason and stop witnessing the disagreement at all.

Grapheme segmentation is not in the standard library. The fixture is Latin text with no
combining marks or emoji — asserted here rather than assumed — so `len(str)` is the
cluster count for this file and this file only. The product counts clusters properly
(`unicode-segmentation`, `crates/montaget-core/src/checks/caption.rs`); this scan is
evidence about one committed document, not a second implementation of the metric.

Run it by hand. It is registered in `.github/workflows/ci.yml` beside the other two
checks, but GitHub Actions is disabled for this repository, so nothing runs it for you. A
failure here is not a bug in the scan — it means ADR-0071's evidence needs revisiting.
"""

import json
import os
import sys
import unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.join(
    HERE, "..", "..", "fixtures", "en-halloween-decorating",
    "en-halloween-decorating.montaget.json",
)

PACE_CPS = 20          # ADR-0034
TOLERANCE_FRAMES = 1   # ADR-0034: one frame at the project's own fps

failures = []


def check(claim, actual, expected):
    if actual != expected:
        failures.append(f"{claim}: expected {expected!r}, got {actual!r}")


def text_elements(project):
    for track in project["tracks"]:
        for element in track["elements"]:
            if element["type"] == "text":
                text = "".join(run["text"] for run in element["runs"])
                yield track["name"], element, unicodedata.normalize("NFC", text)


def main():
    project = json.load(open(FIXTURE))
    fps = project["fps"]
    elements = list(text_elements(project))

    # The premise that makes `len` a cluster count here, checked rather than assumed.
    for _, element, text in elements:
        if any(unicodedata.combining(ch) for ch in text) or any(ord(ch) > 0xFFFF for ch in text):
            failures.append(
                f"{element['id']}: the fixture has grown a combining mark or an astral "
                f"character, so `len` is no longer its cluster count — this scan needs a "
                f"real segmenter"
            )

    fires_by_metric, fires_by_table = [], []
    for _, element, text in elements:
        duration = element["end"] - element["start"]
        # The metric as ADR-0034's **Metric** paragraph states it: `\n` excluded.
        if len(text.replace("\n", "")) * 1000 > PACE_CPS * duration:
            fires_by_metric.append(element["id"])
        # The same arithmetic on the evidence table's counts, which include the break.
        if len(text) * 1000 > PACE_CPS * duration:
            fires_by_table.append(element["id"])

    check("captions firing by ADR-0034's metric", fires_by_metric, ["hook-loop"])
    check(
        "captions firing by ADR-0034's evidence table",
        fires_by_table,
        ["quiz-question", "hook-loop"],
    )

    # The one that separates them, to a hundredth, so the margin is visible.
    quiz = next(e for _, e, _ in elements if e["id"] == "quiz-question")
    quiz_text = next(t for _, e, t in elements if e["id"] == "quiz-question")
    quiz_ms = quiz["end"] - quiz["start"]
    check(
        "quiz-question cps, `\\n` excluded",
        round(len(quiz_text.replace("\n", "")) * 1000 / quiz_ms, 2),
        19.91,
    )
    check(
        "quiz-question cps, `\\n` counted",
        round(len(quiz_text) * 1000 / quiz_ms, 2),
        20.35,
    )

    # ADR-0034 groups byte-identical text **project-wide**, "no dependency on `track` or
    # `group`". Grouped that way the fixture has three disagreements. The ADR reports one,
    # and no scope reproduces that: within the `caption` track alone there are still two.
    def disagreeing(scope):
        groups = {}
        for track, element, text in scope:
            groups.setdefault(text, []).append(element["end"] - element["start"])
        return sorted(
            text
            for text, durations in groups.items()
            if (max(durations) - min(durations)) * fps > 1000 * TOLERANCE_FRAMES
        )

    check(
        "repeated lines whose durations disagree, project-wide",
        disagreeing(elements),
        [
            "I hang cobwebs over the door.",
            "What is this called\nin English?",
            "cobweb  -  cobweb",
        ],
    )
    # Not a defence of the ADR's count either: even scoped to the one track its worked
    # examples come from, the fixture has two. `word-05` holds "cobweb  -  cobweb" for
    # 12156 ms and `word-quiz` for 2900, both in `caption`.
    check(
        "the same, scoped to the `caption` track",
        disagreeing([e for e in elements if e[0] == "caption"]),
        ["What is this called\nin English?", "cobweb  -  cobweb"],
    )
    # The must-not-fire half: one line repeated across two tracks at identical durations.
    check(
        "`string of lights`, one line in two tracks at one duration",
        sorted(
            element["id"]
            for _, element, text in elements
            if text == "string of lights"
        ),
        ["word-08-bridge", "word-08-target"],
    )

    if failures:
        print(f"{len(failures)} claim(s) no longer reproduce:\n")
        for failure in failures:
            print(f"  - {failure}")
        print("\nADR-0071 rests on these numbers. A failure here is an amendment, not a fix.")
        return 1

    print(
        f"OK — {len(elements)} text elements: 1 fires the pace floor as the metric states "
        f"it and 2 as the evidence table counts it; 3 repeated lines disagree project-wide "
        f"and 2 within the `caption` track."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
