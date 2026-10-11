#!/usr/bin/env python3
"""Measure #932 on a disposable copy of the reproduction kit: binary, kit directory."""
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
import time

binary, kit = sys.argv[1], Path(sys.argv[2]).resolve()
project = json.loads((kit / "project.json").read_text())
plates = {"table", "spread-full"}
background = json.loads(json.dumps(project))
for track in background["tracks"]:
    track["elements"] = [e for e in track["elements"] if e["id"] in plates]
background["tracks"] = [t for t in background["tracks"] if t["elements"]]
(kit / "prefix-background.json").write_text(json.dumps(background))
subprocess.run([binary, "frame", str(kit / "prefix-background.json"), "--at", "1500000",
                "--full", "--out", str(kit / "prefix-baked.png")], check=True, capture_output=True)
baked = json.loads(json.dumps(project))
for track in baked["tracks"]:
    track["elements"] = [e for e in track["elements"] if e["id"] not in plates]
baked["tracks"] = [t for t in baked["tracks"] if t["elements"]]
baked["tracks"].insert(0, {"name": "prefix-baked", "layer": 0, "elements": [{
    "id": "prefix-baked", "type": "image", "start": 0, "end": project["duration"],
    "source": "prefix-baked.png", "x": 0, "y": 0, "origin": "top-left",
    "width": 1920, "height": 1080, "fit": "literal"}]})
(kit / "prefix-baked.json").write_text(json.dumps(baked))
samples = []
for repeat in range(3):
    for variant, source, cache in [("off", "project.json", "off"),
                                   ("on", "project.json", "on"),
                                   ("baked", "prefix-baked.json", "on")]:
        output = kit / f"prefix-{variant}.mp4"
        env = dict(os.environ, MONTAGENT_PREFIX_CACHE=cache, MONTAGENT_PAINTING="1,90")
        started = time.monotonic()
        result = subprocess.run([binary, "render", str(kit / source), "--from", "1500000",
                                 "--to", "1503000", "--output", str(output), "--json"],
                                env=env, capture_output=True, text=True, check=True)
        elapsed = time.monotonic() - started
        md5 = subprocess.run(["ffmpeg", "-v", "error", "-i", str(output), "-f", "framemd5", "-"],
                             capture_output=True, check=True).stdout
        hashes = [line for line in md5.splitlines() if not line.startswith(b"#")]
        assert len(hashes) == 90, f"expected 90 frames, got {len(hashes)}"
        record = dict(repeat=repeat, variant=variant, seconds=elapsed,
                      seconds_per_frame=elapsed / 90,
                      frames_sha256=hashlib.sha256(b"\n".join(hashes)).hexdigest())
        samples.append(record)
        print(json.dumps(record), flush=True)
medians = {v: statistics.median(s["seconds_per_frame"] for s in samples if s["variant"] == v)
           for v in ("off", "on", "baked")}
assert len({s["frames_sha256"] for s in samples if s["variant"] in ("off", "on")}) == 1
print(json.dumps(dict(medians=medians, speedup=medians["off"] / medians["on"],
                      percent_over_baked=100 * (medians["on"] / medians["baked"] - 1))))
