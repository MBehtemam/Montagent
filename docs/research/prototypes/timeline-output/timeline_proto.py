#!/usr/bin/env python3
"""PROTOTYPE — throwaway. Answers #11: what does `montaget timeline` print?

Emits four candidate renderings of the committed fixture project so they can be
read side by side at a real terminal width. Not production code.
"""
import json, sys, shutil
from collections import defaultdict

PATH = "fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json"
COLS = int(sys.argv[1]) if len(sys.argv) > 1 else 100

doc = json.load(open(PATH))
DUR = doc["duration"]
tracks = doc["tracks"]
els = [(t, e) for t in tracks for e in t["elements"]]


def ts(ms):
    return f"{ms // 60000}:{ms % 60000 / 1000:06.3f}"


def short(e):
    """The one-line 'what is this' for an element."""
    ty = e["type"]
    if ty == "text":
        s = "".join(r["text"] for r in e["runs"]).replace("\n", "⏎")
        return f'"{s}"'
    if ty == "audio":
        return e["source"].split("/")[-1]
    if ty == "image":
        return e["source"].split("/")[-1]
    if ty in ("rect", "shape"):
        return f'{e.get("width")}x{e.get("height")} {e.get("fill","")}'
    return ty


def banner(n, title, claim):
    print("\n" + "=" * COLS)
    print(f"CANDIDATE {n} — {title}")
    print(f"  claim: {claim}")
    print("=" * COLS)


# ---------------------------------------------------------------- A: gantt
def gantt():
    banner("A", "Gantt chart — one row per track, time across the width",
           "the shape ADR-0011's consumers called illegible. Included as evidence, not inherited.")
    label_w = max(len(t["name"]) for t in tracks) + 2
    chart = COLS - label_w - 1
    scale = chart / DUR
    # ruler
    ruler = [" "] * chart
    for sec in range(0, DUR // 1000 + 1, 10):
        c = int(sec * 1000 * scale)
        for i, ch in enumerate(f"{sec}s"):
            if c + i < chart:
                ruler[c + i] = ch
    print(" " * label_w + "".join(ruler))
    for t in tracks:
        row = ["·"] * chart
        for e in t["elements"]:
            a, b = int(e["start"] * scale), int(e["end"] * scale)
            b = max(b, a + 1)
            for c in range(a, min(b, chart)):
                row[c] = "█"
            if a < chart:
                row[a] = "▌"
        print(f'{t["name"]:<{label_w}}' + "".join(row))


# ---------------------------------------------------------- B: cut list
def cutlist():
    banner("B", "Cut list — chronological, every boundary, what enters and leaves",
           "time is a column of numbers, not a horizontal axis. Scales past terminal width.")
    ev = defaultdict(lambda: ([], []))
    for t, e in els:
        ev[e["start"]][0].append((t, e))
        ev[e["end"]][1].append((t, e))
    prev = None
    for ms in sorted(ev):
        ins, outs = ev[ms]
        gapnote = ""
        if prev is not None and len(ins) + len(outs) > 1:
            gapnote = f"   (+{(ms - prev) / 1000:.3f}s)"
        print(f"\n{ts(ms):>10}{gapnote}")
        for t, e in sorted(outs, key=lambda x: x[0]["layer"]):
            print(f'{"":>10}  ─ {t["name"]:<16} {e["id"]:<20} {short(e)[:COLS-54]}')
        for t, e in sorted(ins, key=lambda x: x[0]["layer"]):
            print(f'{"":>10}  + {t["name"]:<16} {e["id"]:<20} {short(e)[:COLS-54]}')
        prev = ms


# ------------------------------------------------------------- C: by group
def bygroup():
    banner("C", "By group — the structure the file already has, one block per group",
           "`group` is the beat the author thinks in. 60 elements collapse to 8 blocks.")
    groups = defaultdict(list)
    for t, e in els:
        groups[e.get("group", "—")].append((t, e))
    order = sorted(groups, key=lambda g: min(e["start"] for _, e in groups[g]))
    for g in order:
        items = sorted(groups[g], key=lambda x: (x[1]["start"], x[0]["layer"]))
        a = min(e["start"] for _, e in items)
        b = max(e["end"] for _, e in items)
        print(f'\n▸ {g}   {ts(a)} → {ts(b)}   ({(b-a)/1000:.3f}s, {len(items)} elements)')
        for t, e in items:
            rel = f'+{(e["start"]-a)/1000:6.3f} +{(e["end"]-a)/1000:6.3f}'
            kf = " ~anim" if any(isinstance(e.get(k), list) and e.get(k) and isinstance(e[k][0], dict) for k in ("scale", "x", "y", "opacity", "rotation")) else ""
            print(f'   {rel}  L{t["layer"]:<3} {t["name"]:<15} {e["id"]:<20} {short(e)[:COLS-62]}{kf}')


# ------------------------------------------------------- D: coverage strip
def coverage():
    banner("D", "Coverage — what is on screen, and where nothing is",
           "answers the question the file cannot: is every instant painted, and is audio contiguous?")
    cuts = sorted({e["start"] for _, e in els} | {e["end"] for _, e in els} | {0, DUR})
    print(f'\n{"span":>22}  {"vis":>3}  {"aud":>3}  stack (front → back)')
    for a, b in zip(cuts, cuts[1:]):
        if a >= DUR:
            break
        live = [(t, e) for t, e in els if e["start"] <= a < e["end"]]
        vis = [(t, e) for t, e in live if e["type"] != "audio"]
        aud = [(t, e) for t, e in live if e["type"] == "audio"]
        top = sorted(vis, key=lambda x: -x[0]["layer"])
        names = " › ".join(e["id"] for _, e in top[:3])
        flag = "  ← NOTHING ON SCREEN" if not vis else ("  ← SILENT" if not aud else "")
        print(f'{ts(a)}–{ts(b):<11} {len(vis):>3}  {len(aud):>3}  {names[:COLS-40]}{flag}')


gantt(); cutlist(); bygroup(); coverage()
