#!/usr/bin/env python3
"""PROTOTYPE #764: does `query --at` name the letters the frame hides?

Runs `query --at` at every frame instant of the clip (floor(n x 1000 / fps), the instants
`render` paints) and reads the `hidden:` cell of every text carrying `path`. Compares that
against each sweep run's painter log (MONTAGENT_PROTO_HIDDEN: the letters whose glyphs the
painter did not hand to the canvas, on that run's painter count and hint setting).

Usage: hidden.py <project> <sweep-dir>  -> prints the report; writes <sweep-dir>/query.hidden
"""
import glob
import json
import os
import re
import subprocess
import sys

B = os.environ.get("MONTAGENT_BIN", "montagent")
project, sweep = sys.argv[1], sys.argv[2]
doc = json.load(open(project))
fps, duration = doc["fps"], doc["duration"]
frames = duration * fps // 1000
on_path = {e["id"] for t in doc["tracks"] for e in t["elements"]
           if e["type"] == "text" and "path" in e}


def norm(words):
    words = words.strip()
    if words in ("none", ""):
        return "none"
    return " ".join(str(int(n)) for n in re.findall(r"\d+", words))


query = {}
row = re.compile(r"^\s+L\d+\s+(\S+)\s+text\s.*hidden: ([0-9 ]+|none)\s*$")
for n in range(frames):
    t = n * 1000 // fps
    out = subprocess.run([B, "query", project, "--at", str(t)], capture_output=True,
                         text=True).stdout
    for line in out.splitlines():
        m = row.match(line)
        if m and m.group(1) in on_path:
            query[(t, m.group(1))] = norm(m.group(2))
with open(os.path.join(sweep, "query.hidden"), "w") as f:
    for (t, id), words in sorted(query.items()):
        f.write(f"{t} {id} {words}\n")

changes = {}
prev = {}
for (t, id), words in sorted(query.items()):
    if prev.get(id) != words:
        changes[id] = changes.get(id, 0) + 1
    prev[id] = words
print(f"query --at: {len(query)} (instant, element) rows over {frames} frame instants")
print("distinct hidden sets per element over the clip (changes of the set, counting the first):")
for id in sorted(changes):
    sets = {w for (t, i), w in query.items() if i == id}
    print(f"  {id:22s} changes={changes[id]:<4d} distinct={len(sets)}")

print()
for log in sorted(glob.glob(os.path.join(sweep, "*.hidden"))):
    if log.endswith("query.hidden"):
        continue
    painter = {}
    for line in open(log):
        t, id, words = line.split(" ", 2)
        painter[(int(t), id)] = norm(words)
    missing = set(query) - set(painter)
    extra = set(painter) - set(query)
    differ = [k for k in set(query) & set(painter) if query[k] != painter[k]]
    verdict = "AGREE" if not (missing or extra or differ) else "DISAGREE"
    print(f"{os.path.basename(log):34s} rows={len(painter):<5d} missing={len(missing)} "
          f"extra={len(extra)} differ={len(differ)}  {verdict}")
    for k in sorted(differ)[:5]:
        print(f"    {k}: query {query[k]!r} painter {painter[k]!r}")
