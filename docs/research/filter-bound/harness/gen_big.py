"""Synthetic edge cases for #649: one element per 1 s slot, x drifting by a fractional amount."""
import json

W, H = 1920, 1080
cases = []


def case(name, el):
    cases.append((name, el))


def mv(x0, dx=13.7):
    return lambda s: [{"t": s, "v": round(x0)}, {"t": s + 1000, "v": round(x0 + dx), "ease": "linear"}]


blur = lambda r: {"name": "blur", "radius": r}
shadow = lambda dx, dy, r, c="#FF9F2E", o=0.8: {"name": "shadow", "dx": dx, "dy": dy, "radius": r, "color": c, "opacity": o}
T = dict(type="text", font="cinzel-bold", size=150, color="#E3C067", align="center", runs=[{"text": "SPY"}])

case("rect-blur-sigma134", dict(type="rect", x=mv(960.4), y=540, origin="center", width=400, height=200, fill="#F2F2F2", effects=[blur(268)]))
case("rect-blur-sigma150-rescaled", dict(type="rect", x=mv(960.4), y=540, origin="center", width=400, height=200, fill="#F2F2F2", effects=[blur(300)]))
case("text-scale3-blur-sigma150", dict(T, x=mv(700.4), y=500, origin="center", width=600, height=225, scale=[3.0, 3.0], effects=[blur(100)]))
case("ellipse-shadow-sigma140-offset", dict(type="ellipse", x=mv(500.4), y=400, origin="center", width=200, height=200, fill="#F2F2F2", effects=[shadow(40, 30, 280)]))
case("rect-blur-sigma350-maxlayerdim", dict(type="rect", x=mv(960.4), y=540, origin="center", width=400, height=200, fill="#F2F2F2", effects=[blur(700)]))

tracks = []
instants = []
for i, (name, el) in enumerate(cases):
    s = i * 1000
    el = dict(el)
    el["id"] = name
    el["start"], el["end"] = s, s + 1000
    if callable(el["x"]):
        el["x"] = el["x"](s)
    if name == "text-scale-anim-blur":
        el["scale"] = [{"t": s, "v": [1.4, 1.4]}, {"t": s + 1000, "v": [0.93, 0.93], "ease": "linear"}]
    tracks.append({"name": f"t{i}", "layer": i, "elements": [el]})
    instants += [(name, s + k) for k in (0, 333, 667, 967)]

project = {
    "frame": {"width": W, "height": H}, "fps": 30, "background": "#101418",
    "duration": len(cases) * 1000, "output": "out/big.mp4",
    "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
    "fontVendor": {"fonts/Cinzel-Bold.ttf": {"licence": "OFL-1.1", "source": "google/fonts ofl/cinzel, instanced wght=700", "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
    "tracks": tracks,
}
json.dump(project, open("big.montagent.json", "w"), indent=1)
with open("instants_big.tsv", "w") as f:
    for name, t in instants:
        f.write(f"{name}\t{t}\n")
print(len(cases), "cases", len(instants), "instants")
