#!/usr/bin/env python3
"""Re-derive ADR-0196's recorded numbers and verify all byte/relative gates."""
import json
import math
from pathlib import Path
import re
import statistics

here = Path(__file__).resolve().parent
text = (here / 'reopen.txt').read_text()
medians = [int(n) for n in re.findall(r'first frame median\s+(\d+) ms', text)]
maxima = [int(n) for n in re.findall(r'max\s+(\d+)\s+\(n=', text)]
assert len(medians) == 5 and len(maxima) == 5
reopen_ms = math.ceil(max(medians) / 100) * 100
assert reopen_ms == 1100 and max(maxima) <= reopen_ms
print(json.dumps(dict(reopen_ms=reopen_ms, worst_phase_median_ms=max(medians),
                      worst_sample_ms=max(maxima))))
for filename in ('before.jsonl', 'cap.jsonl', 'after.jsonl', 'uncached.jsonl', 'uncached-final.jsonl'):
    rows = [json.loads(line) for line in (here / filename).read_text().splitlines()]
    for kind in sorted({r['kind'] for r in rows}):
        selected = [r for r in rows if r['kind'] == kind]
        assert len({r['frame_hash'] for r in selected}) == 1, (filename, kind, 'frame bytes')
        assert len({r['mp4_hash'] for r in selected}) == 1, (filename, kind, 'MP4 bytes')
        timings = {config: statistics.median(r['seconds'] for r in selected if r['forced'] == config)
                   for config in sorted({r['forced'] for r in selected})}
        if filename == 'before.jsonl' and kind == 'kit':
            assert {key: round(value, 3) for key, value in timings.items()} == {
                '1,240': 4.524, '2,15': 4.874, '6,15': 3.633, '6,2': 8.500}
        if filename == 'uncached.jsonl' and kind == 'uncached-kit':
            assert round(timings['auto'] - timings['one'], 2) == 2.08
        if filename == 'after.jsonl':
            expected = {'kit': {'one': 8.173, 'auto': 7.792},
                        'decode': {'one': 3.153, 'auto': 3.659}}[kind]
            assert {k: round(v, 3) for k, v in timings.items()} == expected
        if filename == 'uncached-final.jsonl':
            assert {k: round(v, 3) for k, v in timings.items()} == {'one': 33.388, 'auto': 14.871}
            assert round(timings['one'] / timings['auto'], 2) == 2.25
            assert any(r['forced'] == 'auto' and r['painting']['painters'] > 1 for r in selected)
        if filename == 'cap.jsonl':
            assert {key: round(value, 3) for key, value in timings.items()} == {
                '1,240': 5.240, '5,15': 7.658, '6,15': 6.351}
            for config, count, rss in [('1,240', 5, (633, 637)),
                                       ('5,15', 21, (1250, 1329)),
                                       ('6,15', 25, (1436, 1466))]:
                selected_config = [r for r in selected if r['forced'] == config]
                assert {r['peak']['processes'] for r in selected_config} == {count}
                assert (round(min(r['peak']['rss_mb'] for r in selected_config)),
                        round(max(r['peak']['rss_mb'] for r in selected_config))) == rss
        print(json.dumps(dict(file=filename, kind=kind, medians=timings,
                              load_range=[min(r['load_before'] for r in selected),
                                          max(r['load_before'] for r in selected)])))
rows = [json.loads(line) for line in (here / 'decode-gate.jsonl').read_text().splitlines()]
samples = [r for r in rows if 'pair' in r and not r['warmup']]
assert len(samples) == 10 and {r['pair'] for r in samples} == {1, 2, 3, 4, 5}
medians = {schedule: statistics.median(r['wall_s'] for r in samples if r['schedule'] == schedule)
           for schedule in ('one', 'auto')}
assert medians['auto'] <= medians['one'] * 1.1, medians
assert {k: round(v, 3) for k, v in medians.items()} == {'one': 139.556, 'auto': 143.344}
assert round(medians['auto'] / medians['one'], 6) == 1.027143
assert all(r['under_load_exception'] for r in samples)
assert all(r['answer']['render']['painting']['painters'] == 1 for r in samples)
assert all(r['answer']['render']['painting']['peak_video_feeds'] == 3
           and r['answer']['render']['painting']['feed_cap'] == 20
           for r in samples if r['schedule'] == 'auto')
environment = json.loads((here / 'environment.json').read_text())
assert environment['cpu'] == 'Apple M1 Pro' and environment['cores'] == 10
assert environment['ffmpeg'].startswith('ffmpeg version 9.0.2')
print(json.dumps(dict(decode_medians=medians, ratio=medians['auto'] / medians['one'],
                      under_load_exception=True)))
