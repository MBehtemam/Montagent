#!/usr/bin/env python3
"""Re-derive every number ADR-0016 claims. Exits non-zero if any stops reproducing.

Run from the repo root:  python3 docs/research/juries/format-versioning/versioning_scan.py
"""
import json, math, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = subprocess.run(["git","rev-parse","--show-toplevel"],capture_output=True,text=True).stdout.strip()
FIX  = "fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json"
fails, checks = [], 0

def check(label, got, want):
    global checks; checks += 1
    if got != want:
        fails.append(f"{label}: got {got!r}, want {want!r}")
        print(f"  FAIL {label}: {got!r} != {want!r}")
    else:
        print(f"  ok   {label}: {got!r}")

def show(rev, path):
    return subprocess.run(["git","-C",ROOT,"show",f"{rev}:{path}"],capture_output=True,text=True).stdout

def elements(s):
    return {m.group(1): m.group(0) for m in re.finditer(r'\{"id":"([^"]+)".*?\}(?=,?\n)', s)}

print("\n[1] M1 — three consecutive accepted revisions, one fixture blob")
blobs = {}
for c in ("afc12d24","ed3db37c","2f8e9405","3b795255"):
    blobs[c] = subprocess.run(["git","-C",ROOT,"rev-parse",f"{c}:{FIX}"],
                              capture_output=True,text=True).stdout.strip()
check("ADR-0012 blob", blobs["afc12d24"], "cd8e797109b833e234c7a4b1ed19339b28182612")
check("ADR-0013 blob == ADR-0012", blobs["ed3db37c"], blobs["afc12d24"])
check("ADR-0014 blob == ADR-0012", blobs["2f8e9405"], blobs["afc12d24"])
check("ADR-0015 blob differs",     blobs["3b795255"] != blobs["afc12d24"], True)

print("\n[2] M2 — a removal migration is not invertible from the published ADRs")
pre = show("3b795255^", FIX)
grav = dict(re.findall(r'"id":"([^"]+)"[^}]*?"gravity":"([^"]+)"', pre))
check("elements carrying gravity", len(grav), 8)
check("handle-logo gravity", grav.get("handle-logo"), "center")
check("the other seven are 'top'", sorted(set(v for k,v in grav.items() if k!="handle-logo")), ["top"])
adrs = "".join(show("origin/main", f"docs/adr/{f}") for f in
               os.listdir(os.path.join(ROOT,"docs/adr")))
check("no ADR records gravity:'center'", 'gravity":"center' in adrs, False)

print("\n[3] C1 — a semantics-only revision is NOT byte-invisible in general")
check("float 103*(1920/103) floors to 1919", math.floor(103*(1920/103)), 1919)
check("exact integer arithmetic gives 1920", (103*1920)//103, 1920)
# NOTE: an earlier draft cited ~5.8% from a mis-specified formula. WITHDRAWN.
# The project's own re-derivable figure is ADR-0013's, quoted from the ADR itself.
adr13 = show("origin/main","docs/adr/0013-fitted-extents-floor-and-the-nine-origin-keywords.md")
check("ADR-0013 publishes 4.466%", "4.466" in adr13, True)
check("ADR-0013 publishes 31,402,800", "31,402,800" in adr13, True)

print("\n[4] The fixture is at the fixed point — why gravity was inert there")
cur = show("origin/main", FIX)
els = elements(cur)
p6 = els["photo-06"]
h  = int(re.search(r'"height":(\d+)',p6).group(1))
cl = [int(x) for x in re.search(r'"clip":\[([^\]]+)\]',p6).group(1).split(',')]
check("photo-06 rect height", h, 1912)
check("photo-06 clip", cl, [0,0,1080,1300])
check("y=0 shows the top band, so gravity:'top' was inert", (0, min(h,cl[3])), (0,1300))
check("bottom band would need y", cl[3]-h, -612)
check("clip hides this many px", h-cl[3], 612)

print("\n[5] Experiment 1 — the gravity fork (6 agents, 2 conditions)")
VOFF={"top":0,"center":None,"bottom":None}
def band(el):
    y=int(re.search(r'"y":(-?\d+)',el).group(1)); hh=int(re.search(r'"height":(\d+)',el).group(1))
    o=re.search(r'"origin":"([^"]+)"',el).group(1).split('-')[0]
    c=[int(x) for x in re.search(r'"clip":\[([^\]]+)\]',el).group(1).split(',')]
    top=y-{"top":0,"center":hh//2,"bottom":hh}.get(o,0)
    vt,vb=max(top,c[1]),min(top+hh,c[1]+c[3])
    return (vt-top, vb-top)
G=os.path.join(HERE,"experiment-gravity-fork")
correct={}
for cond in ("m1","m2","m3","mg1","mg2","mg3"):
    p=os.path.join(G,cond,"repaired.json")
    if not os.path.exists(p): continue
    s=open(p).read(); e=elements(s)
    ok = all(band(e[i])==(612,1912) for i in ("photo-06","photo-07"))
    correct[cond]=ok
check("condition M  (messages only) correct",      sum(correct[c] for c in ("m1","m2","m3")), 2)
check("condition M+G (messages+glossary) correct", sum(correct[c] for c in ("mg1","mg2","mg3")), 0)
check("the glossary entry made it worse", correct["m1"] and correct["m3"] and not any(
      correct[c] for c in ("mg1","mg2","mg3")), True)

print("\n[6] Experiment 2 — three arms (9 agents)")
A=os.path.join(HERE,"experiment-three-arm")
def visible_band(el):
    """Source rows of a photo element that survive its aperture, honouring origin.
    Robust to reformatting: works on the parsed object, not the text."""
    y,hh = el["y"], el["height"]
    o = el.get("origin","top-left").split("-")[0]
    top = y - {"top":0,"center":hh//2,"bottom":hh}.get(o,0)
    c = el.get("clip")
    if c is None:                       # aperture removed
        vt,vb = max(top,0), min(top+hh,1920)
    else:
        vt,vb = max(top,c[1]), min(top+hh,c[1]+c[3])
    return (vt-top, vb-top)
def arm(d):
    raw = open(os.path.join(A,d,"shipped.json")).read()
    doc = json.loads(raw)
    els = [e for t in doc["tracks"] for e in t["elements"]]
    photos = [e for e in els if e["id"].startswith("photo-0")]
    rects  = [t["name"] for t in doc["tracks"]
              if re.search(r"photo-(mask|mat|crop)", t["name"])]
    return dict(version=doc.get("montaget"),
                fit=sum(1 for e in els if "fit" in e),
                clip=sum(1 for e in els if "clip" in e),
                compensating_track=bool(rects),
                heights=sorted({e["height"] for e in photos}),
                bands=sorted({visible_band(e) for e in photos}),
                lines=raw.count("\n")+1)
r={d:arm(d) for d in sorted(os.listdir(A)) if os.path.isdir(os.path.join(A,d))}
for d,x in r.items():
    print(f"     {d:10} ver={str(x['version']):4} fit={x['fit']} clip={x['clip']} "
          f"rect={'Y' if x['compensating_track'] else 'n'} h={x['heights']} lines={x['lines']}")
check("arm A: the only version downgrade is a-fable",
      [d for d,x in r.items() if x["version"]==2], ["a-fable"])
check("arm A opus+sonnet preserved every key",
      (r["a-opus"]["fit"], r["a-sonnet"]["fit"]), (8,8))
check("preserved-and-escalated files (A-opus, A-sonnet, B-opus)",
      sorted(d for d,x in r.items() if x["fit"]==8 and x["version"]!=2),
      ["a-opus","a-sonnet","b-opus"])
check("compensating rect track invented independently",
      sorted(d for d,x in r.items() if x["compensating_track"]),
      ["b-fable","b-sonnet","c-fable","c-opus"])
check("arm B and arm C each produced two",
      (sum(1 for d,x in r.items() if d[0]=="b" and x["compensating_track"]),
       sum(1 for d,x in r.items() if d[0]=="c" and x["compensating_track"])), (2,2))
check("c-sonnet is the lone squash (photo height 1912 -> 1300)",
      r["c-sonnet"]["heights"], [1300])
check("nobody stripped the aperture without compensating",
      sorted(d for d,x in r.items()
             if x["clip"]==0 and not x["compensating_track"] and x["heights"]!=[1300]), [])
print("\n[6b] Formatting — ADR-0005's one-element-per-line convention")
reflowed = sorted(d for d,x in r.items() if x["lines"] > 300)
check("agents who pretty-printed the file", reflowed, ["b-fable","b-sonnet","c-sonnet"])
check("i.e. 3 of 9 destroyed exact-string replace", len(reflowed), 3)
check("committed fixture is 154 lines", show("origin/main",FIX).count(chr(10)), 154)

print("\n[7] The unknown-key policy does not exist (C4)")
ctx = show("origin/main","CONTEXT.md")
hits = re.findall(r'additionalProperties|unknown (?:key|field|propert)|unrecognis|unrecogniz',
                  adrs+ctx, re.I)
check("occurrences across 15 ADRs + CONTEXT.md", len(hits), 0)

print("\n[8] migrate.py regenerates the fixture from main alone")
check("origin file is on main",
      os.path.exists(os.path.join(ROOT,"docs/research/sample-project/pre-migration.montaget.json")), True)
check("migrate.py docstring still names the unmerged branch",
      "prototype/sample-project-file" in show("origin/main","docs/research/sample-project-migration/migrate.py"), True)

print(f"\n{checks} checks, {len(fails)} failed")
if fails:
    print("\nFAILURES:"); [print("  "+f) for f in fails]; sys.exit(1)
print("all reproducing")
