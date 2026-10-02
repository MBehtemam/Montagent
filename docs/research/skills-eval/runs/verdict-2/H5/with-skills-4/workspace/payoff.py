import json, subprocess
p = json.load(open("h5a.json"))
tr = {t["name"]: t for t in p["tracks"]}
old = next(e for e in tr["captions"]["elements"] if e["id"] == "captions-04")
hl = {r["text"]: r["highlight"] for r in old["runs"] if "highlight" in r}
tr["captions"]["elements"] = [e for e in tr["captions"]["elements"] if e["id"] != "captions-04"]
tr["captions-bg"]["elements"] = [e for e in tr["captions-bg"]["elements"] if e["id"] != "captions-bg-04"]
START, END, Y, SIZE = 6720, 10000, 985, 76
KEY = {"exactly", "one", "thing", "nothing", "else", "moved."}
lines = [["exactly", "one", "thing"], ["and", "prove"], ["nothing", "else", "moved."]]
CLEAR = "#F5F0E600"
def build(show):
    runs = []
    for li, ws in enumerate(lines):
        for wi, w in enumerate(ws):
            r = {"text": w}
            if show == "base":
                if w in KEY: r["color"] = CLEAR
                else: r["highlight"] = dict(hl[w])
            elif w == show:
                h = dict(hl[w]); h["end"] = END; r["highlight"] = h
            else:
                r["color"] = CLEAR
            runs.append(r)
            if wi < len(ws) - 1: runs.append({"text": " "})
        if li < len(lines) - 1: runs.append({"text": "\n"})
    return runs
el = {"runs": build("base"), "font": "bold", "size": SIZE}
m = json.loads(subprocess.run(["montagent", "measure", "h5a.json", "--element", json.dumps(el), "--json"], capture_output=True, text=True).stdout)["measure"]
W = max(l["advance_width"] for l in m["lines"]); H = m["block_height"]
print("payoff block", W, H, [l["text"] for l in m["lines"]])
POP = [0.34, 1.56, 0.64, 1]
sc = [{"t": START, "v": [0.6, 0.6]}, {"t": START + 320, "v": [1.0, 1.0], "ease": POP}]
op = [{"t": START, "v": 0.0}, {"t": START + 120, "v": 1.0, "ease": "ease-out"}]
def text(id_, runs, start, caption):
    e = {"id": id_, "type": "text", "start": start, "end": END, "x": 540, "y": Y, "origin": "center",
        "width": int(W) + 8, "height": H, "font": "bold", "size": SIZE, "color": "#F5F0E6", "align": "center",
        "runs": runs, "scale": sc, "opacity": op}
    if not caption: e["caption"] = False
    return e
bg = {"id": "payoff-bg", "type": "rect", "start": START, "end": END, "x": 540, "y": Y, "origin": "center",
      "width": int(W) + 80, "height": H + 44, "fill": "#101418", "radius": 28, "scale": sc, "opacity": op}
p["tracks"].append({"name": "payoff-bg", "layer": 31, "elements": [bg]})
p["tracks"].append({"name": "payoff", "layer": 32, "elements": [text("payoff", build("base"), START, True)]})
for i, w in enumerate(["exactly", "one", "thing", "nothing", "else", "moved."]):
    n = "payoff-" + w.strip(".")
    p["tracks"].append({"name": n, "layer": 33 + i, "elements": [text(n, build(w), START, False)]})
json.dump(p, open("h5.json", "w"), indent=2)
