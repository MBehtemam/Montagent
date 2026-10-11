#!/usr/bin/env python3
"""Re-derive committed #932 measurements; fail if an ADR numeric claim stops reproducing."""
import json
from pathlib import Path
import statistics

root = Path(__file__).resolve().parent
records = [json.loads(line) for line in (root / "results.jsonl").read_text().splitlines()]
samples, recorded = records[:-1], records[-1]
assert len(samples) == 9
medians = {}
for variant in ("off", "on", "baked"):
    selected = [r for r in samples if r["variant"] == variant]
    assert {r["repeat"] for r in selected} == {0, 1, 2}
    for row in selected:
        assert abs(row["seconds_per_frame"] - row["seconds"] / 90) < 1e-12
    medians[variant] = statistics.median(r["seconds_per_frame"] for r in selected)
assert medians == recorded["medians"]
speedup = medians["off"] / medians["on"]
over_baked = 100 * (medians["on"] / medians["baked"] - 1)
assert speedup == recorded["speedup"] and over_baked == recorded["percent_over_baked"]
assert [round(medians[v], 6) for v in ("off", "on", "baked")] == [0.135482, 0.025700, 0.023482]
assert round(speedup, 2) == 5.27 and round(over_baked, 2) == 9.45
digests = {r["frames_sha256"] for r in samples if r["variant"] in ("off", "on")}
assert digests == {"5aafebb29a1c626f57e387174459cfc37f64c52da923ad52143c004ea97cfefa"}
identity = json.loads((root / "identity.json").read_text())
hashes = [line.split(",")[-1].strip() for line in (root / "frames.framemd5").read_text().splitlines()
          if not line.startswith("#")]
assert identity["all_equal"] and identity["frames"] == len(hashes) == 30
assert identity["frame_md5"] == hashes
print(f"9 samples: {speedup:.2f}x speedup, {over_baked:.2f}% above baked; identity evidence consistent")
