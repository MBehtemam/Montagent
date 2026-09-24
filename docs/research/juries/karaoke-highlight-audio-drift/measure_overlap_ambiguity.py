#!/usr/bin/env python3
"""Measures how many audio-bearing elements overlap each text element's
timeline range in the real fixture, as a proxy for how ambiguous a
timeline-overlap heuristic would be for associating a karaoke `highlight`
window with the audio source it was calibrated against.

The fixture has zero actual karaoke highlighting (per ADR-0048), so every
text element stands in as a proxy for a would-be highlight-bearing element.

Run from the repo root:
  python3 docs/research/juries/karaoke-highlight-audio-drift/measure_overlap_ambiguity.py
"""
import json
import collections

FIXTURE = "fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json"


def overlaps(a_start, a_end, b_start, b_end):
    return a_start < b_end and b_start < a_end


def main():
    d = json.load(open(FIXTURE))
    tracks = d["tracks"]

    audio_elements = []
    text_elements = []
    video_elements = []
    for tr in tracks:
        for el in tr["elements"]:
            el2 = dict(el)
            el2["_track"] = tr["name"]
            if el.get("type") == "audio":
                audio_elements.append(el2)
            elif el.get("type") == "text":
                text_elements.append(el2)
            elif el.get("type") == "video":
                video_elements.append(el2)

    print("audio elements:", len(audio_elements))
    print("text elements:", len(text_elements))
    print("video elements (may carry embedded audio):", len(video_elements))

    audio_bearing = audio_elements + video_elements

    print()
    print("--- Per text element: overlapping audio-bearing elements ---")
    overlap_counts = []
    for t in text_elements:
        n = sum(
            1
            for a in audio_bearing
            if overlaps(t["start"], t["end"], a["start"], a["end"])
        )
        overlap_counts.append(n)
        print(f"  {t['id']} [{t['start']}-{t['end']}]: {n} overlapping audio-bearing elements")

    print()
    print("distribution of overlap counts:", collections.Counter(overlap_counts))
    print("exactly 1 overlapping source (heuristic precise):", sum(1 for n in overlap_counts if n == 1))
    print("0 overlapping sources:", sum(1 for n in overlap_counts if n == 0))
    print("2+ overlapping sources (heuristic ambiguous):", sum(1 for n in overlap_counts if n >= 2))


if __name__ == "__main__":
    main()
