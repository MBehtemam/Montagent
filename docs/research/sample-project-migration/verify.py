"""Check the migration. Usage: python3 verify.py <old.json> <new.json>

<old.json> is the pre-migration prototype, preserved on main at
  docs/research/sample-project/pre-migration.montagent.json

So the whole migration is re-runnable from a checkout of main:
  python3 docs/research/sample-project-migration/verify.py \
      docs/research/sample-project/pre-migration.montagent.json \
      fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json
"""
import json, collections, sys, os, struct, subprocess, tempfile
old=json.load(open(sys.argv[1])); new=json.load(open(sys.argv[2]))
NEWDIR=os.path.dirname(os.path.abspath(sys.argv[2]))
HERE=os.path.dirname(os.path.abspath(__file__))

def png_dims(path):
    """Source dimensions from the file itself, not from a table copied out of
    migrate.py -- ADR-0006's question is whether the document agrees with the media on
    disk, and a lookup table cannot answer it."""
    with open(path,"rb") as f: head=f.read(24)
    assert head[:8]==b"\x89PNG\r\n\x1a\n" and head[12:16]==b"IHDR", path
    return struct.unpack(">II", head[16:24])

def cover_int(sw,sh,bw,bh):
    """ADR-0013, restated here so this check is independent of migrate.py."""
    if bw*sh >= bh*sw: return bw,(sh*bw)//sw
    return (sw*bh)//sh, bh
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
# 2b. #276: an image that zooms must pivot about its own centre, and must still occupy the
#     rect the old `box` put it at when the zoom is at 1.0. Two clauses, because the pivot
#     is only meaningful if it re-spells the same rectangle -- moving the element and
#     calling it a pivot would satisfy either clause alone. Restated here rather than
#     imported, so a `migrate.py` that got this wrong cannot verify itself.
FRACTION={"top-left":(0.0,0.0),"top-center":(0.5,0.0),"top-right":(1.0,0.0),
          "center-left":(0.0,0.5),"center":(0.5,0.5),"center-right":(1.0,0.5),
          "bottom-left":(0.0,1.0),"bottom-center":(0.5,1.0),"bottom-right":(1.0,1.0)}
piv=[]
for i,e in O.items():
    if e["type"]!="image": continue
    n=N[i]; x,y,_,_=e["box"]
    fx,fy=FRACTION[n["origin"]]
    # Where the drawn rect's top-left lands at scale 1.0, through the declared origin.
    at_rest=(n["x"]-fx*n["width"], n["y"]-fy*n["height"])
    if at_rest!=(x,y): piv.append((i,"moved",at_rest,(x,y)))
    elif ("scale" in n)!=(n["origin"]=="center"):
        piv.append((i,"pivot",n["origin"],"scale" in n))
print("image pivot failures:",len(piv))
for p in piv: print("  ",p)
assert not piv, "an image element's pivot or its resting rect disagrees with #276"
# 3. no forbidden fields
for i,e in N.items():
    if e["type"]=="audio":
        assert not ({"x","y","origin","width","height","opacity","scale"} & set(e)), i
    else:
        assert "width" in e and "height" in e, ("missing size",i)
    assert "box" not in e and "weight" not in e, i
    if e["type"]!="text": assert "align" not in e, i
    # ADR-0068: the bare `mask` key retires; a mask is an `effects` member, and the
    # param-less form means the inscribed shape. Both directions, per this project's
    # fire/must-not-fire discipline.
    assert "mask" not in e, ("ADR-0068 retired the bare `mask` key; use `effects`",i)
    for fx in e.get("effects",[]):
        assert fx["name"] in ("blur","shadow","mask","tint","saturation",
                              "brightness","contrast"), ("closed effect vocabulary",i,fx)
        if fx["name"]=="mask":
            assert fx["shape"] in ("circle","rect","ellipse"), ("closed shape vocabulary",i,fx)
    # ADR-0015: `gravity` is retired, `fit` is required, and its value set is closed.
    # This assertion was inverted by ADR-0015 -- it previously required `gravity`.
    if e["type"]=="image":
        assert "gravity" not in e, ("ADR-0015 retired `gravity`",i)
        assert "clip" in e and "fit" in e, i
        assert e["fit"] in ("cover","contain","literal"), ("closed fit vocabulary",i,e["fit"])
        if e["fit"]!="literal": assert "clip" in e, ("cover/contain require clip",i)
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
# 9. ADR-0013: every image element's width/height recomputed from the media on disk.
#    Before this check a green verify.py said nothing about the number #44 was about.
print("ADR-0013 fitted extents, recomputed from the source files:")
bad=unchecked=0
for i,e in sorted(N.items()):
    if e["type"]!="image": continue
    p=os.path.join(NEWDIR,e["source"])
    if not os.path.exists(p):
        print(f"  {i:<16} UNCHECKED  source not on disk: {e['source']}"); unchecked+=1; continue
    sw,sh=png_dims(p); c=e["clip"]
    want=cover_int(sw,sh,c[2],c[3]); got=(e["width"],e["height"])
    covers = got[0]>=c[2] and got[1]>=c[3]
    ok = want==got
    if not ok: bad+=1
    print(f"  {i:<16} src {sw}x{sh} box {c[2]}x{c[3]} -> rule {want}, declared {got}"
          f"  {'OK' if ok else 'DEVIATES'}{'' if covers else '  APERTURE NOT COVERED'}")
    assert covers, ("aperture coverage is an error under ADR-0013", i)
print(f"  deviations: {bad} | unchecked: {unchecked}")
assert bad==0, "a declared extent disagrees with ADR-0013's rule"
# 10. the migration script still produces exactly the committed file.
#     Nothing else in the toolchain would catch migrate.py drifting from the fixture
#     it claims to produce -- and a green check on the other nine sections would not
#     notice, which is the ADR-0006 mistake this repo is cleaning up after.
mig=os.path.join(HERE,"migrate.py")
if os.path.exists(mig):
    with tempfile.NamedTemporaryFile(suffix=".json",delete=False) as tf: out=tf.name
    subprocess.run([sys.executable,mig,"linear",sys.argv[1],out],check=True,
                   stderr=subprocess.DEVNULL)
    same=open(out,"rb").read()==open(sys.argv[2],"rb").read()
    os.unlink(out)
    print(f"migrate.py regenerates the committed file byte-for-byte: {same}")
    assert same, "migrate.py no longer reproduces the committed fixture"
