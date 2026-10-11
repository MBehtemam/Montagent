#!/usr/bin/env python3
"""Interleaved video-painter evidence on current HEAD.
Usage: measure.py BINARY KIT BENCHMARK OUTPUT_JSONL [--gate]
KIT is a disposable self-contained repro kit; BENCHMARK is make_benchmark.py's
output. Sources are generated before timing. --gate refuses load >= 1.5.
Under-load runs record the exception and report comparisons without asserting time.
"""
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
import threading
import time

binary, kit, benchmark, output = sys.argv[1:5]
binary = str(Path(binary).resolve())
kit, benchmark = Path(kit).resolve(), Path(benchmark).resolve()
gate = '--gate' in sys.argv[5:]
rows = []

def processes(stop, peak):
    while not stop.is_set():
        result = subprocess.run(['ps', '-Ao', 'rss,comm'], capture_output=True, text=True, check=True)
        feeds = [int(line.split()[0]) for line in result.stdout.splitlines()[1:]
                 if line.split()[-1].endswith('/ffmpeg')]
        peak['processes'] = max(peak['processes'], len(feeds))
        peak['rss_mb'] = max(peak['rss_mb'], sum(feeds) / 1024)
        stop.wait(0.1)

def run(kind, project, first, frames, forced, repeat):
    load = os.getloadavg()[0]
    if gate and load >= 1.5:
        raise RuntimeError(f'quiet protocol requires load < 1.5; got {load}')
    env = dict(os.environ, MONTAGENT_STAGES='1')
    env.pop('MONTAGENT_PAINTING', None)
    if '--uncached' in sys.argv:
        env['MONTAGENT_PREFIX_CACHE'] = 'off'
    if forced:
        env['MONTAGENT_PAINTING'] = forced
    dest = project.parent / (project.stem + '-video-painters-measure.mp4')
    peak = dict(processes=0, rss_mb=0)
    stop = threading.Event()
    sampler = threading.Thread(target=processes, args=(stop, peak))
    sampler.start()
    start = time.monotonic()
    try:
        result = subprocess.run([binary, 'render', str(project), '--from', str(first),
                                 '--to', str(first + frames * 1000 // 30), '--output', str(dest),
                                 '--json'], env=env, capture_output=True, text=True, check=True)
        seconds = time.monotonic() - start
    finally:
        stop.set()
        sampler.join()
    answer = json.loads(result.stdout)
    stages = [json.loads(line.split(' ', 1)[1]) for line in result.stderr.splitlines()
              if line.startswith('MONTAGENT_STAGES ')]
    digest = subprocess.run(['ffmpeg', '-v', 'error', '-i', str(dest), '-map', '0:v:0', '-f', 'framemd5', '-'],
                            capture_output=True, check=True).stdout
    hashes = [line for line in digest.splitlines() if not line.startswith(b'#')]
    assert len(hashes) == frames, (len(hashes), frames)
    row = dict(kind=kind, repeat=repeat, forced=forced or 'auto', frames=frames,
               load_before=load, load_after=os.getloadavg()[0], seconds=seconds,
               peak=peak, painting=answer['render']['painting'], stages=stages,
               frame_hash=hashlib.sha256(b'\n'.join(hashes)).hexdigest(),
               mp4_hash=hashlib.sha256(dest.read_bytes()).hexdigest(), quiet=load < 1.5)
    rows.append(row)
    with open(output, 'a') as file:
        file.write(json.dumps(row) + '\n')
    print(json.dumps(row), flush=True)

# Short slices for choosing constants; full benchmark for the one-sided gate.
configs = ['one', None] if gate or '--after' in sys.argv else ['1,240', '2,15', '6,15', '6,2']
for repeat in range(0 if '--decode-only' in sys.argv or '--cap-only' in sys.argv else (1 if '--single' in sys.argv else 3)):
    for config in configs if repeat % 2 == 0 else list(reversed(configs)):
        run('uncached-kit' if '--uncached' in sys.argv else 'kit', kit / 'project.json', 1500000, 240, config, repeat)
for repeat in range(0 if '--cap-only' in sys.argv or '--kit-only' in sys.argv else (1 if '--single' in sys.argv else 3)):
    for config in (['one', None] if repeat % 2 == 0 else [None, 'one']):
        run('decode', benchmark / 'benchmark-1080p30.montagent.json', 0,
            10800 if gate else 240, config, repeat)
if '--cap-only' in sys.argv:
    project = json.loads((kit / 'project.json').read_text())
    videos = [e for t in project['tracks'] for e in t['elements'] if e['type'] == 'video']
    assert len(videos) == 1
    video = videos[0]
    for track in project['tracks']:
        track['elements'] = [e for e in track['elements'] if e['type'] != 'video']
    project['tracks'] = [t for t in project['tracks'] if t['elements']]
    for i in range(4):
        element = dict(video, id=f'cap-video-{i}', x=40 + 480*i, y=300,
                       width=440, height=248, fit='literal', volume=0)
        project['tracks'].append(dict(name=f'cap-video-{i}', layer=920+i, elements=[element]))
    path = kit / 'video-painters-cap.json'
    path.write_text(json.dumps(project))
    for repeat in range(2):
        for config in (['1,240', '5,15', '6,15'] if repeat == 0 else ['6,15', '5,15', '1,240']):
            run('cap', path, 1500000, 240, config, repeat)
for kind in ('kit', 'uncached-kit', 'decode', 'cap'):
    records = [r for r in rows if r['kind'] == kind]
    if not records:
        continue
    assert len({r['frame_hash'] for r in records}) == 1, f'{kind}: frame bytes differ'
    assert len({r['mp4_hash'] for r in records}) == 1, f'{kind}: MP4 bytes differ'
    medians = {c: statistics.median(r['seconds'] for r in records if r['forced'] == c)
               for c in {r['forced'] for r in records}}
    print(json.dumps(dict(kind=kind, medians=medians)), flush=True)
    if gate and kind == 'decode':
        assert medians['auto'] <= 1.1 * medians['one'], medians
