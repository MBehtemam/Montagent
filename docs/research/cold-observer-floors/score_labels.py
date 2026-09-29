#!/usr/bin/env python3
"""Scores the cold label transcriptions in transcripts/ against manifest.json. Issue #422."""
import json,glob,os,difflib
T=os.path.dirname(os.path.abspath(__file__))
m=json.load(open(f"{T}/manifest.json"))
truth={v["type_px"]:v["truth"] for v in m.values() if v["kind"]=="label"}
for f in sorted(glob.glob(f"{T}/transcripts/*.txt")):
    px,model=os.path.basename(f)[:-4].split("-"); px=int(px)
    got=[l.strip() for l in open(f) if l.strip()]
    tr=truth[px]; exact=sum(a==b for a,b in zip(got,tr))
    wrong=[(b,a) for a,b in zip(got,tr) if a!=b]
    chars=sum(len(b) for b in tr); err=sum(round((1-difflib.SequenceMatcher(None,a,b).ratio())*max(len(a),len(b))) for a,b in zip(got,tr))
    uns="all flagged unsure" if os.path.exists(f[:-4]+".unsure") else "confident"
    print(f"{px:>2}px {model:<6} exact {exact}/18  ~char errors {err}/{chars}  {uns}")
    for b,a in wrong: print(f"        truth {b!r:40} read {a!r}")
