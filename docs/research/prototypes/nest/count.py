#!/usr/bin/env python3
"""Count what a Montagent project spends: elements (nests apart), keyframes, bytes, reviews.

    python3 count.py <project.montagent.json> [--prefix owl]

Keyframes are every list of {"t": ...} records on any element or nest. With --prefix, the
count is split into the character's tracks (the part images and nests) and the rest, and
the character's into body (everything not a mouth or blink) and face (mouths and blinks).
`validate` is asked for its reviews through the montagent binary on PATH.
"""
import json, os, subprocess, sys
from collections import Counter


def walk(tracks, depth=0):
    for track in tracks:
        for el in track["elements"]:
            yield track["name"], el, depth
            if el["type"] == "nest":
                yield from walk(el["tracks"], depth + 1)


def keyframes(el):
    return sum(len(v) for k, v in el.items()
               if isinstance(v, list) and v and isinstance(v[0], dict) and "t" in v[0])


def main(argv):
    path = argv[0]
    prefix = argv[argv.index("--prefix") + 1] if "--prefix" in argv else None
    doc = json.load(open(path))
    rows = list(walk(doc["tracks"]))
    nests = [r for r in rows if r[1]["type"] == "nest"]
    leaves = [r for r in rows if r[1]["type"] != "nest"]
    print(f"elements {len(rows)} ({len(nests)} nests, {len(leaves)} others), "
          f"keyframes {sum(keyframes(r[1]) for r in rows)}, bytes {os.path.getsize(path)}, "
          f"deepest nest {max((r[2] for r in nests), default=-1) + 1}")
    if prefix:
        mine = [r for r in rows if r[1]["id"].startswith(prefix + "-")]
        face = [r for r in mine if "-mouth-" in r[1]["id"] or "-blink-" in r[1]["id"]]
        body = [r for r in mine if r not in face]
        for label, group in (("body", body), ("face", face)):
            print(f"  {label}: {len(group)} elements, {sum(keyframes(r[1]) for r in group)} keyframes")
    out = subprocess.run(["montagent", "validate", path, "--json"], capture_output=True, text=True)
    findings = json.loads(out.stdout)["findings"]
    print("findings:", dict(Counter(f["code"] for f in findings)))


if __name__ == "__main__":
    main(sys.argv[1:])
