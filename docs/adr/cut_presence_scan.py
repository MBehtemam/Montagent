#!/usr/bin/env python3
"""Re-derives the two measured claims ADR-0074 rests on. Exits non-zero if either stops
reproducing.

    python3 docs/adr/cut_presence_scan.py

ADR-0074 settles #250 — that the cut list's presence set is *every* element and not only
the on-screen ones — on an argument with a measurable half: the information travels one
way. A caller handed every element can filter to the visual cut list; a caller handed the
visual one cannot recover where the narration started, because those instants are not in
the answer at all.

Both claims are about the fixture, and a claim about a fixture is exactly the kind that
quietly stops being true when the fixture is edited:

  1. audio is a *third* of the fixture's elements, so "on-screen" is not a rounding of the
     presence set but a removal of a large, structured part of it;
  2. the audio-inclusive cut list has boundaries the visual-only one does not, and the
     majority of the fixture's boundaries are of that kind — so the visual-only answer is
     not a coarser view of the same clock, it is a different clock;
  3. the worked case holds in the shape the ADR states it: `vo-quiz-answer`'s start is a
     visual boundary too, and its end is not, which is why the visual answer reports a
     constant interval it is wrong about rather than a boundary it says is missing.

Run it by hand when you edit the fixture. It is registered in `.github/workflows/ci.yml`
beside the amendment-banner check, but GitHub Actions is disabled for this repository, so
nothing runs it for you. A failure here is not a bug in the scan — it means ADR-0074's
prose cites numbers the fixture no longer produces, and the ADR needs an amendment.
"""

import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.join(
    HERE,
    "..",
    "..",
    "fixtures",
    "en-halloween-decorating",
    "en-halloween-decorating.montagent.json",
)

# The figures ADR-0074's prose states. Changing one here without changing it there is the
# failure this script exists to make loud.
EXPECTED = {
    "elements": 60,
    "audio_elements": 20,
    "boundaries_every_element": 47,
    "boundaries_visual_only": 19,
    "boundaries_audio_only": 28,
}

# ADR-0011's worked drift scenario — "19 of 20 elements at/after 30603 moved +800 ms;
# vo-quiz-answer did not" — is the concrete case ADR-0074 leans on, and it lands only in an
# asymmetric shape: the element it names must be audio, its *start* must still be a visual
# boundary (the card and its captions begin there too), and its *end* must not be. The
# asymmetry is the point and is asserted in both directions below — see ADR-0074's
# "the shape of what is lost is worth being exact about, because it is not symmetric".
DRIFT_ELEMENT = {"id": "vo-quiz-answer", "type": "audio", "start": 61116, "end": 63300}

failures = []


def elements(document):
    """Every element in the document, in the order a traversal meets them.

    Flat across tracks on purpose: ADR-0001's element list is the population the presence
    set is drawn from, and which lane an element sits in is a constraint on it (ADR-0004),
    never a filter on whether it is present.
    """
    for track in document.get("tracks", []):
        for element in track.get("elements", []):
            yield element


def boundaries(population):
    """Every instant at which some member of `population` starts or ends.

    Deduplicated, because two elements turning over at one instant is one boundary and not
    two — which is the same rule `cuts.rs` applies, and the reason a count of boundaries is
    smaller than twice a count of elements.
    """
    instants = set()
    for element in population:
        if isinstance(element.get("start"), int) and isinstance(element.get("end"), int):
            instants.add(element["start"])
            instants.add(element["end"])
    return instants


def main():
    with open(FIXTURE, encoding="utf-8") as handle:
        document = json.load(handle)

    every = list(elements(document))
    audio = [element for element in every if element.get("type") == "audio"]
    visual = [element for element in every if element.get("type") != "audio"]

    every_boundary = boundaries(every)
    visual_boundary = boundaries(visual)
    audio_only = every_boundary - visual_boundary

    measured = {
        "elements": len(every),
        "audio_elements": len(audio),
        "boundaries_every_element": len(every_boundary),
        "boundaries_visual_only": len(visual_boundary),
        "boundaries_audio_only": len(audio_only),
    }

    for name, expected in EXPECTED.items():
        if measured[name] != expected:
            failures.append(
                f"{name}: ADR-0074 states {expected}, the fixture now has {measured[name]}"
            )

    # Claim 1, as a proportion rather than as two counts — the ADR says "a third", and a
    # fixture that grew by a hundred visual elements would keep the counts' ratio wrong
    # while both counts still matched something.
    if measured["audio_elements"] * 3 != measured["elements"]:
        failures.append(
            "audio is no longer exactly a third of the fixture's elements: "
            f"{measured['audio_elements']} of {measured['elements']}"
        )

    # Claim 2, in the direction that carries the argument: the loss is not a minority
    # rounding. If this ever flips, the "different clock, not a coarser view" sentence is
    # the one that stops being supported.
    if measured["boundaries_audio_only"] * 2 <= measured["boundaries_every_element"]:
        failures.append(
            "audio-only boundaries are no longer the majority: "
            f"{measured['boundaries_audio_only']} of {measured['boundaries_every_element']}"
        )

    # And the direction the argument does *not* claim, asserted so the scan fails if the
    # relationship ever reverses: dropping audio can only ever remove boundaries, never add
    # one. A visual-only boundary absent from the full set would mean the two answers
    # disagree about the document rather than one being a subset of the other.
    invented = visual_boundary - every_boundary
    if invented:
        failures.append(
            f"visual-only boundaries absent from the full set: {sorted(invented)}"
        )

    # The worked case. Both its boundaries must be audio-only, or the sentence "the one
    # element the scenario is about is not in the answer, and neither is either of its
    # boundaries" overstates what the fixture supports.
    drift = next(
        (e for e in every if e.get("id") == DRIFT_ELEMENT["id"]),
        None,
    )
    if drift is None:
        failures.append(f"{DRIFT_ELEMENT['id']} is no longer in the fixture")
    else:
        for key in ("type", "start", "end"):
            if drift.get(key) != DRIFT_ELEMENT[key]:
                failures.append(
                    f"{DRIFT_ELEMENT['id']}.{key}: ADR-0074 states "
                    f"{DRIFT_ELEMENT[key]!r}, the fixture now has {drift.get(key)!r}"
                )
        # Asymmetric on purpose, and the ADR says so: the start is shared with the card and
        # its captions, the end is reachable only through the narration. Both halves are
        # asserted, because the argument is about the end and the honesty is about the start.
        if drift.get("start") not in visual_boundary:
            failures.append(
                f"{DRIFT_ELEMENT['id']}'s start {drift.get('start')} is no longer a visual "
                "boundary too — ADR-0074 says the visual cut list still cuts there"
            )
        if drift.get("end") in visual_boundary:
            failures.append(
                f"{DRIFT_ELEMENT['id']}'s end {drift.get('end')} is now a visual boundary — "
                "ADR-0074's worked case needs it reachable only through audio"
            )

    if failures:
        print("ADR-0074's evidence no longer reproduces:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print(
        "ADR-0074 reproduces: "
        f"{measured['audio_elements']}/{measured['elements']} elements are audio; "
        f"{measured['boundaries_every_element']} boundaries over every element against "
        f"{measured['boundaries_visual_only']} over the visual ones, "
        f"{measured['boundaries_audio_only']} of them reachable only through audio."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
