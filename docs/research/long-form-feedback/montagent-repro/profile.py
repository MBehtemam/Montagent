#!/usr/bin/env python3
"""Reproduces the render-time profile in ../montagent-feedback.md. Run from this folder:  python3 profile.py [--at 1500000] [--len 3000]
Renders the same 3 s slice of project.json (a real book page: table + book plate with a feathered mask, a looping picture with multiply blend,
two circular stickers, karaoke text and about 20 small marks) once per variant, each to its OWN output file, and prints seconds per frame.
Last block: bakes the two static background images into one picture with `montagent frame` and times the slice again."""
import argparse, json, os, re, subprocess, sys, time
ap = argparse.ArgumentParser(); ap.add_argument("--at", type=int, default=1500000); ap.add_argument("--len", type=int, default=3000); a = ap.parse_args()
P = json.load(open("project.json")); os.makedirs("out", exist_ok=True)
def variant(keep=lambda e: True, edit=None):
    q = json.loads(json.dumps(P))
    for t in q["tracks"]:
        t["elements"] = [e for e in t["elements"] if keep(e)]
        if edit:
            for e in t["elements"]: edit(e)
    q["tracks"] = [t for t in q["tracks"] if t["elements"]]; return q
isart = lambda e: e["id"].startswith("art-"); issticker = lambda e: e["id"].startswith("stk-")
ismark = lambda e: e["id"].startswith(("g-", "b-", "u-", "h-", "c-", "nb-", "bk-", "ar-", "br-", "r-", "tr-"))
istext = lambda e: e["id"].endswith("-text") and e["id"].startswith("c0")
def nofx(e): e.pop("effects", None)
def noblend(e): e.pop("blend", None)
TESTS = [("everything", variant()),
 ("drop table plate", variant(lambda e: e["id"] != "table")), ("drop book plate (feathered mask)", variant(lambda e: e["id"] != "spread-full")),
 ("drop table + book plate", variant(lambda e: e["id"] not in ("table", "spread-full"))),
 ("drop picture loop only", variant(lambda e: not isart(e))), ("drop stickers only", variant(lambda e: not issticker(e))),
 ("drop picture loop + stickers", variant(lambda e: not (isart(e) or issticker(e)))),
 ("picture loop without blend", variant(edit=lambda e: noblend(e) if isart(e) else None)),
 ("drop fade rectangle", variant(lambda e: e["id"] != "fade")), ("drop all marks", variant(lambda e: not ismark(e))),
 ("drop karaoke text", variant(lambda e: not istext(e))), ("strip every effect", variant(edit=nofx)), ("strip every blend", variant(edit=noblend))]
def run(q, tag):
    json.dump(q, open(f"_{tag}.json", "w")); out = f"out/_{tag}.mp4"; t0 = time.time()
    r = subprocess.run(["montagent", "render", f"_{tag}.json", "--from", str(a.at), "--to", str(a.at + a.len), "--output", out], capture_output=True, text=True)
    el = time.time() - t0
    if r.returncode: sys.exit("render failed: " + (r.stdout + r.stderr)[-300:])
    os.remove(f"_{tag}.json"); return el / (a.len / 1000 * 30), out
base = None
for i, (label, q) in enumerate(TESTS):
    spf, out = run(q, f"v{i}"); base = base or spf
    print(f"{label:36s} {spf:.4f} s/frame  {1/spf:5.1f} fps  " + ("" if i == 0 else f"saves {100*(base-spf)/base:5.1f}%"), flush=True); os.remove(out)
print("\n-- bake the two static background images into one picture (needs the slice to be after the book has opened)")
bg = variant(lambda e: e["id"] in ("table", "spread-full")); json.dump(bg, open("_bg.json", "w"))
subprocess.run(["montagent", "frame", "--out", "bg-baked.png", "--at", str(a.at), "--full", "_bg.json"], capture_output=True); os.remove("_bg.json")
q = variant(lambda e: e["id"] not in ("table", "spread-full")); q["tracks"].insert(0, {"name": "bg", "layer": 0, "elements": [{"id": "bg", "type": "image", "start": 0, "end": P["duration"], "source": "bg-baked.png", "x": 0, "y": 0, "origin": "top-left", "width": 1920, "height": 1080, "fit": "contain"}]})
spf, out = run(q, "baked"); print(f"{'baked background (1 image)':36s} {spf:.4f} s/frame  {1/spf:5.1f} fps  saves {100*(base-spf)/base:5.1f}%"); os.remove(out)
print("done")
