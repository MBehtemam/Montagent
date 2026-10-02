# usage: exact.py <project> <rig> <out>: owl parts drawn at their source size, scaled by `scale`
import json, sys, os
d=json.load(open(sys.argv[1])); rig=json.load(open(sys.argv[2])); base=0.6
rigdir=os.path.relpath(os.path.dirname(sys.argv[2]), os.path.dirname(sys.argv[1]))
byfile={os.path.normpath(os.path.join(rigdir,p["file"])):p for p in rig["parts"].values()}
for t in d["tracks"]:
    for e in t["elements"]:
        p=byfile.get(os.path.normpath(e.get("source","")))
        if not p: continue
        assert "scale" not in e
        e["width"],e["height"]=p["width"],p["height"]
        if "joint" in p: e["origin"]=p["joint"]
        e["scale"]=[base,base]
json.dump(d,open(sys.argv[3],"w"),indent=1)
