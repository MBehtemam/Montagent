#!/usr/bin/env python3
"""#926 sweep: s/frame against chunk size C for N visible videos on the repro kit.
usage: sweep.py <kit-dir> <out.jsonl> <source> <frames> <reps> <N,N,..> <config,config,..>
config is 'one' (shipped: one painter) or 'K:C[:W]'. Each run records the 1-min load before,
wall, the MONTAGENT_STAGES line, peak sum of ffmpeg RSS and peak ffmpeg process count."""
import json, os, subprocess, sys, threading, time

kit, out, source, frames, reps, ns, cfgs = sys.argv[1:8]
frames, reps = int(frames), int(reps)
ns = [int(x) for x in ns.split(",")]
cfgs = cfgs.split(",")
P = json.load(open(f"{kit}/project.json"))
FROM = 1500000
TO = FROM + round(frames * 1000 / 30)


def variant(n):
    q = json.loads(json.dumps(P))
    vid = None
    for t in q["tracks"]:
        for e in t["elements"]:
            if e["type"] == "video":
                vid = (t, e)
    t0, e0 = vid
    q["tracks"] = [t for t in q["tracks"] if t is not t0]
    for i in range(n):
        e = json.loads(json.dumps(e0))
        e["id"] = f"vid-{i}"
        e["source"] = source
        e["x"] = 300 + 330 * (i % 4)
        e["y"] = 300 + 250 * (i // 4)
        e["width"], e["height"] = 640, 360
        q["tracks"].append({"name": f"vid-{i}", "layer": 22 + i, "elements": [e]})
    return q


def sample(stop, res):
    peak_rss = peak_n = 0
    while not stop.is_set():
        o = subprocess.run(["ps", "-Ao", "rss,comm"], capture_output=True, text=True).stdout
        rss = n = 0
        for l in o.splitlines()[1:]:
            parts = l.split(None, 1)
            if len(parts) == 2 and parts[1].endswith("/ffmpeg"):
                rss += int(parts[0]); n += 1
        peak_rss, peak_n = max(peak_rss, rss), max(peak_n, n)
        time.sleep(0.2)
    res["peak_ffmpeg_rss_mb"] = round(peak_rss / 1024)
    res["peak_ffmpeg_procs"] = peak_n


for rep in range(reps):
    for n in ns:
        q = variant(n)
        pj = f"{kit}/_sweep_{n}.json"
        json.dump(q, open(pj, "w"))
        for cfg in cfgs:
            env = dict(os.environ, MONTAGENT_STAGES="1")
            env.pop("MONTAGENT_PAINTING", None)
            if cfg != "one":
                env["MONTAGENT_PAINTING"] = cfg.replace(":", ",")
            load = os.getloadavg()[0]
            res = {}
            stop = threading.Event()
            th = threading.Thread(target=sample, args=(stop, res))
            th.start()
            t0 = time.time()
            r = subprocess.run(["montagent", "render", pj, "--from", str(FROM), "--to", str(TO),
                                "--output", f"{kit}/_sweep_{n}.mp4", "--json"],
                               capture_output=True, text=True, env=env, cwd=kit)
            wall = time.time() - t0
            stop.set(); th.join()
            if r.returncode:
                open('/tmp/w926/fail.json','w').write(r.stdout)
                print("FAILED", n, cfg, r.returncode, [l.strip() for l in (r.stdout+r.stderr).splitlines() if '"code"' in l or 'E-' in l][:2], r.stdout[:0], r.stderr[-300:].replace(chr(10),' | '), flush=True)
                continue
            st = [l for l in r.stderr.splitlines() if l.startswith("MONTAGENT_STAGES")]
            stages = json.loads(st[-1].split(" ", 1)[1]) if st else {}
            rec = dict(rep=rep, n=n, cfg=cfg, source=os.path.basename(source), frames=frames,
                       load_before=round(load, 1), load_after=round(os.getloadavg()[0], 1),
                       wall_s=round(wall, 2), spf=round(wall / frames, 4), stages=stages, **res)
            open(out, "a").write(json.dumps(rec) + "\n")
            print(f"rep{rep} N={n} {cfg:8s} load {load:5.1f} spf {wall/frames:.4f} "
                  f"opens {stages.get('feeds_opened')}+{stages.get('feeds_reopened')} "
                  f"decode_ms {stages.get('decode_ms')} procs {res['peak_ffmpeg_procs']} "
                  f"rss {res['peak_ffmpeg_rss_mb']}MB", flush=True)
        os.remove(pj)
