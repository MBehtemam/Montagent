"""Check the migration. Usage: python3 verify.py <old.json> <new.json>

<old.json> is the pre-migration prototype, preserved on main at
  docs/research/sample-project/pre-migration.montaget.json

So the whole migration is re-runnable from a checkout of main:
  python3 docs/research/sample-project-migration/verify.py \
      docs/research/sample-project/pre-migration.montaget.json \
      fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json
"""
import json, collections, sys
old=json.load(open(sys.argv[1])); new=json.load(open(sys.argv[2]))
def flat(d): return [e for t in d["tracks"] for e in t["elements"]]
O,N={e["id"]:e for e in flat(old)},{e["id"]:e for e in flat(new)}
print("elements:",len(O),"->",len(N),"| ids identical:",set(O)==set(N))
print("tracks:",len(old["tracks"]),"->",len(new["tracks"]))
# 1. time model untouched
bad=[i for i in O if (O[i]["start"],O[i]["end"])!=(N[i]["start"],N[i]["end"])]
print("elements with changed start/end:",len(bad))
for k in ("source","source_start","source_end","speed","group","type"):
    d=[i for i in O if O[i].get(k)!=N[i].get(k)]
    print(f"  changed {k}: {len(d)}")
# 2. box -> x/y/w/h round trip for rects
rt=[]
for i,e in O.items():
    if e["type"]=="rect":
        x,y,w,h=e["box"]
        n=N[i]
        if (n["x"],n["y"],n["width"],n["height"],n["origin"])!=(x,y,w,h,"top-left"): rt.append(i)
print("rect box round-trip failures:",len(rt))
# 3. no forbidden fields
for i,e in N.items():
    if e["type"]=="audio":
        assert not ({"x","y","origin","width","height","opacity","scale"} & set(e)), i
    else:
        assert "width" in e and "height" in e, ("missing size",i)
    assert "box" not in e and "weight" not in e, i
    if e["type"]!="text": assert "align" not in e, i
    if e["type"]=="image": assert "gravity" in e and "clip" in e, i
    if e["type"]=="text": assert "runs" in e and "text" not in e and e["font"]=="brand", i
print("field-shape assertions: all pass")
# 4. keyframe schema
for i,e in N.items():
    for r in e.get("scale",[]):
        assert isinstance(r["v"],list) and len(r["v"])==2, i
    if "scale" in e:
        assert "ease" not in e["scale"][0], ("ease on first record",i)
        ts=[r["t"] for r in e["scale"]]; assert ts==sorted(ts) and len(set(ts))==len(ts), i
print("keyframe schema: all pass")
# 5. per-track non-overlap preserved
ov=0
for t in new["tracks"]:
    es=sorted(t["elements"],key=lambda e:e["start"])
    for a,b in zip(es,es[1:]):
        if a["end"]>b["start"]: ov+=1
print("per-track overlaps:",ov)
# 6. one element per line
lines=open(sys.argv[2]).read().split("\n")
el=[l for l in lines if l.strip().startswith('{"id"')]
print("element lines:",len(el),"| all single-line:",len(el)==len(N))
# 7. text box: how many are measured vs derived
meas={"sentence-05","sentence-06","sentence-07","sentence-08","sentence-quiz","chip-text","handle-text"}
txt=[i for i,e in N.items() if e["type"]=="text"]
print(f"text elements: {len(txt)} | box measured from a rect: {len(meas)} | self-derived: {len(txt)-len(meas)}")
# 8. overflow check on the 7 measured boxes (ADR-0007 block formula)
print("ADR-0006 overflow, measured boxes only:")
for i in sorted(meas):
    e=N[i]; lines_=e["runs"][0]["text"].count("\n")+1
    bh=lines_*e["size"]*e["line_height"]
    print(f"  {i:<14} block h {bh:7.1f} vs box {e['height']:>4}  {'OK' if bh<=e['height'] else 'OVERFLOW'}")
