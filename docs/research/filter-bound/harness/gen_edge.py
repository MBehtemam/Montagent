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

case("rect-blur-center", dict(type="rect", x=mv(960.37), y=540, origin="center", width=400, height=200, fill="#F2F2F2", effects=[blur(40)]))
case("ellipse-shadow-offset", dict(type="ellipse", x=mv(800.5), y=500, origin="center", width=300, height=180, fill="#3BA0FF", effects=[shadow(24, 18, 24)]))
case("rect-shadow-frac-offset", dict(type="rect", x=mv(700.1), y=600, origin="center", width=320, height=160, radius=24, fill="#F2F2F2", stroke="#FF3B30", stroke_width=6, effects=[shadow(7.5, -3.25, 20)]))
case("ellipse-glow-zero", dict(type="ellipse", x=mv(1000.3), y=540, origin="center", width=64, height=64, fill="#F2F2F2", effects=[shadow(0, 0, 60, "#FFFFFF", 0.6)]))
case("text-glow-zero", dict(T, x=mv(960.4), y=540, origin="center", width=600, height=225, effects=[shadow(0, 0, 28)]))
case("text-blur-left-edge", dict(T, x=mv(-60.3), y=540, origin="center", width=600, height=225, effects=[blur(14)]))
case("rect-blur-over-right-bottom", dict(type="rect", x=mv(1850.6), y=1040, origin="center", width=300, height=200, fill="#F2F2F2", effects=[blur(30)]))
case("rect-offframe-blur-reaches-in", dict(type="rect", x=mv(-45.5, 4.3), y=300, origin="center", width=40, height=200, fill="#FFFFFF", effects=[blur(60)]))
case("shadow-offset-into-frame", dict(type="rect", x=mv(-80.25, 3.1), y=200, origin="center", width=100, height=100, fill="#FFFFFF", effects=[shadow(70, 0, 30)]))
case("text-scale-2.5-blur", dict(T, x=mv(960.4), y=540, origin="center", width=600, height=225, scale=[2.5, 2.5], effects=[blur(16)]))
case("text-scale-0.4-shadow", dict(T, x=mv(960.4), y=540, origin="center", width=600, height=225, scale=[0.4, 0.4], effects=[shadow(0, 0, 28)]))
case("text-scale-anim-blur", dict(T, x=mv(400.4), y=540, origin="center", width=600, height=225, effects=[blur(14)]))
case("rect-anisotropic-scale-shadow", dict(type="rect", x=mv(960.4), y=540, origin="center", width=200, height=200, scale=[2.0, 0.5], fill="#F2F2F2", effects=[shadow(10, 10, 20)]))
case("rect-rotated-30-blur", dict(type="rect", x=mv(960.4), y=540, origin="center", width=400, height=120, rotation=30, fill="#F2F2F2", effects=[blur(24)]))
case("text-overflows-box-shadow", dict(T, x=mv(960.4), y=540, origin="center", width=60, height=40, effects=[shadow(0, 0, 20)]))
case("text-stroke-shadow", dict(T, x=mv(960.4), y=540, origin="center", width=600, height=225, stroke="#FF3B30", stroke_width=8, effects=[shadow(0, 0, 24)]))
case("opacity-half-shadow", dict(type="ellipse", x=mv(960.4), y=540, origin="center", width=200, height=200, opacity=0, fill="#F2F2F2", effects=[shadow(12, 12, 24)]))
case("blur-then-shadow", dict(type="rect", x=mv(960.4), y=540, origin="center", width=300, height=150, fill="#F2F2F2", effects=[blur(10), shadow(20, 20, 30)]))
case("shadow-then-blur", dict(type="rect", x=mv(960.4), y=540, origin="center", width=300, height=150, fill="#F2F2F2", effects=[shadow(20, 20, 30), blur(10)]))
case("mask-then-blur", dict(type="rect", x=mv(960.4), y=540, origin="center", width=300, height=300, fill="#F2F2F2", effects=[{"name": "mask", "shape": "circle"}, blur(20)]))
case("image-blur-clip", dict(type="image", source="img/boat.jpg", fit="literal", x=mv(960.4), y=540, origin="center", width=640, height=360, clip=[700, 400, 400, 300], effects=[blur(20)]))
case("tiny-sigma-blur", dict(type="rect", x=mv(960.4), y=540, origin="center", width=300, height=150, fill="#F2F2F2", effects=[blur(3)]))

case("text-overflow-tiny-blur", dict(T, x=mv(960.4), y=540, origin="center", width=60, height=40, effects=[blur(2)]))
case("text-wide-stroke-tiny-shadow", dict(T, x=mv(960.4), y=540, origin="center", width=600, height=225, stroke="#FF3B30", stroke_width=24, effects=[shadow(0, 0, 3)]))
case("ellipse-stroke-tiny-blur", dict(type="ellipse", x=mv(960.4), y=540, origin="center", width=300, height=200, fill="#F2F2F2", stroke="#FF3B30", stroke_width=20, effects=[blur(2)]))
case("text-scale-3-tiny-blur-edge", dict(T, x=mv(1880.4), y=1050, origin="center", width=300, height=120, scale=[3.0, 3.0], effects=[blur(1)]))

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
    "duration": len(cases) * 1000, "output": "out/edge.mp4",
    "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
    "fontVendor": {"fonts/Cinzel-Bold.ttf": {"licence": "OFL-1.1", "source": "google/fonts ofl/cinzel, instanced wght=700", "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
    "tracks": tracks,
}
json.dump(project, open("edge.montagent.json", "w"), indent=1)
with open("instants.tsv", "w") as f:
    for name, t in instants:
        f.write(f"{name}\t{t}\n")
print(len(cases), "cases", len(instants), "instants")
