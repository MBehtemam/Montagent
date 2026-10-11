#!/usr/bin/env python3
"""#926 Q5: the ADR-0142 benchmark project, one painter vs forced painters, interleaved.
usage: bench.py <bench-dir> <out.jsonl> <cfg,cfg,...> [reps]   cfg: one | K:C[:W]"""
import hashlib, json, os, re, subprocess, sys, threading, time
d, out, cfgs = sys.argv[1], sys.argv[2], sys.argv[3].split(",")
reps = int(sys.argv[4]) if len(sys.argv) > 4 else 1
proj = f"{d}/benchmark-1080p30.montagent.json"
def sample(stop, res):
    pr = pn = pm = 0
    while not stop.is_set():
        o = subprocess.run(["ps", "-Ao", "rss,comm"], capture_output=True, text=True).stdout
        rss = n = 0
        for l in o.splitlines()[1:]:
            p = l.split(None, 1)
            if len(p) == 2 and p[1].endswith("/ffmpeg"): rss += int(p[0]); n += 1
            if len(p) == 2 and p[1].endswith("/montagent"): pm = max(pm, int(p[0]))
        pr, pn = max(pr, rss), max(pn, n); time.sleep(0.5)
    res.update(peak_ffmpeg_rss_mb=round(pr/1024), peak_ffmpeg_procs=pn, peak_montagent_rss_mb=round(pm/1024))
for rep in range(reps):
    for cfg in cfgs:
        env = dict(os.environ, MONTAGENT_STAGES="1"); env.pop("MONTAGENT_PAINTING", None)
        if cfg != "one": env["MONTAGENT_PAINTING"] = cfg.replace(":", ",")
        o = f"{d}/out-{cfg.replace(':','_')}.mp4"
        for f in (o,): 
            if os.path.exists(f): os.remove(f)
        load = os.getloadavg()[0]; res = {}; stop = threading.Event()
        th = threading.Thread(target=sample, args=(stop, res)); th.start(); t0 = time.time()
        r = subprocess.run(["montagent", "render", proj, "--output", o, "--json"], capture_output=True, text=True, env=env, cwd=d)
        wall = time.time() - t0; stop.set(); th.join()
        st = [l for l in r.stderr.splitlines() if l.startswith("MONTAGENT_STAGES")]
        stages = json.loads(st[-1].split(" ", 1)[1]) if st else {}
        sha = hashlib.sha256(open(o, "rb").read()).hexdigest()[:16] if os.path.exists(o) else None
        rec = dict(rep=rep, cfg=cfg, rc=r.returncode, load_before=round(load,1), load_after=round(os.getloadavg()[0],1),
                   wall_s=round(wall,1), mp4_sha256_16=sha, stages=stages, **res)
        open(out, "a").write(json.dumps(rec) + "\n"); print(json.dumps(rec), flush=True)
