# Writes scene.montagent.json (everything but the owl); bake_rig.py adds the owl.
import json
p = json.load(open('base.montagent.json'))
POP = [0.34, 1.56, 0.64, 1]
def el(**k): return k
def kf(*pairs):
    out = [{"t": pairs[0][0], "v": pairs[0][1]}]
    for t, v, e in pairs[1:]:
        out.append({"t": t, "v": v, "ease": e})
    return out
FADE_OUT = lambda t0=6800, t1=6933: kf((t0, 1.0), (t1, 0.0, "ease-in"))
LT = (760, 430)                       # laptop top-left; its screen is x 120-587, y 41-312
SX, SY = LT[0] + 120, LT[1] + 41      # 880, 471
tracks = []
def track(name, layer, *els): tracks.append({"name": name, "layer": layer, "elements": list(els)})

track("set", 0, el(id="set", type="image", start=0, end=10000, source="character/study.png", x=0, y=0, origin="top-left", width=1920, height=1080, fit="literal"))
# Laptop pops in on "edit".
track("laptop", 4, el(id="laptop", type="image", start=3333, end=7000, source="character/laptop.png", x=LT[0] + 354, y=LT[1] + 283, origin="center", width=708, height=566, fit="literal",
      scale=kf((3333, [0.0, 0.0]), (3600, [1.0, 1.0], POP)), opacity=FADE_OUT()))
# The file, on the laptop's screen.
on = kf((3633, 0.0), (3733, 1.0, "ease-out"), (6800, 1.0, "linear"), (6933, 0.0, "ease-in"))
track("screen", 5, el(id="screen", type="rect", start=3633, end=7000, x=SX + 8, y=SY + 8, origin="top-left", width=451, height=255, fill="#101418", radius=8, opacity=on))
TX, TY = 912, 556
track("file-name", 6, el(id="file-name", type="text", start=3633, end=7000, x=TX, y=SY + 22, origin="top-left", width=150, height=39, font="code", size=32, color="#9AA3AD", runs=[{"text": "hello.json"}], opacity=on, caption=False))
track("select", 6, el(id="select", type="rect", start=3833, end=4133, x=TX + 150, y=TY - 2, origin="top-left", width=127, height=57, fill="#FF5A3666", radius=6))
track("line", 7,
      el(id="line-before", type="text", start=3633, end=4133, x=TX, y=TY, origin="top-left", width=300, height=53, font="code", size=44, color="#F5F0E6", runs=[{"text": "\"Hello, World\""}], opacity=kf((3633, 0.0), (3733, 1.0, "ease-out")), caption=False),
      el(id="line-after", type="text", start=4133, end=7000, x=TX, y=TY, origin="top-left", width=400, height=53, font="code", size=44, color="#F5F0E6", runs=[{"text": "\"Hello, "}, {"text": "Montagent", "color": "#FF5A36"}, {"text": "\""}], opacity=FADE_OUT(), caption=False))
track("render-label", 6, el(id="render-label", type="text", start=4767, end=7000, x=TX, y=632, origin="top-left", width=300, height=39, font="code", size=32, color="#9AA3AD", runs=[{"text": "montagent render"}], opacity=kf((4767, 0.0), (4867, 1.0, "ease-out"), (6800, 1.0, "linear"), (6933, 0.0, "ease-in")), caption=False))
track("bar-track", 6, el(id="bar-track", type="rect", start=4767, end=7000, x=TX, y=684, origin="top-left", width=403, height=14, fill="#2A3038", radius=7, opacity=kf((4767, 0.0), (4867, 1.0, "ease-out"), (6800, 1.0, "linear"), (6933, 0.0, "ease-in"))))
track("bar-fill", 7, el(id="bar-fill", type="rect", start=4867, end=7000, x=TX, y=691, origin="center-left", width=403, height=14, fill="#FF5A36", radius=7, scale=kf((4867, [0.0, 1.0]), (5367, [1.0, 1.0], "linear")), opacity=FADE_OUT()))
# The rendered video comes out of the screen on "renders".
C0 = (SX + 233, SY + 135); C1 = (1290, 232); OUT = [0.25, 1, 0.5, 1]
mv = lambda: dict(x=kf((5400, C0[0]), (5800, C1[0], OUT)), y=kf((5400, C0[1]), (5800, C1[1], OUT)), scale=kf((5400, [0.12, 0.12]), (5800, [1.0, 1.0], OUT)), opacity=FADE_OUT())
track("video-frame", 8, el(id="video-frame", type="rect", start=5400, end=7000, origin="center", width=624, height=358, fill="#F5F0E6", radius=10, effects=[{"name": "shadow", "dx": 0, "dy": 10, "radius": 24, "color": "#101418", "opacity": 0.35}], **mv()))
track("video", 9, el(id="video", type="image", start=5400, end=7000, source="stills/session-02.png", origin="center", width=608, height=342, fit="contain", **mv()))
# Brand.
track("lockup", 30, el(id="lockup", type="image", start=7000, end=10000, source="brand/lockup.png", x=1150, y=450, origin="center", width=864, height=200, fit="contain", scale=kf((7000, [0.0, 0.0]), (7400, [1.0, 1.0], POP))))
track("tagline", 30, el(id="tagline", type="text", start=7300, end=10000, x=1150, y=585, origin="top-center", width=900, height=66, font="bold", size=54, color="#101418", align="center", runs=[{"text": "Edit the file. Get the video."}], opacity=kf((7300, 0.0), (7600, 1.0, "ease-out")), caption=False))
# Sound.
track("voice", 50, el(id="voice", type="audio", start=1200, end=6720, source="character/voice/line-2.wav", source_start=0, source_end=5520))
track("music", 51, el(id="bed", type="audio", start=0, end=9900, source="music/bed-120bpm.wav", source_start=0, source_end=9900,
      volume=kf((0, 0.0), (500, 0.12, "linear"), (6700, 0.12, "linear"), (7100, 0.22, "linear"), (9000, 0.22, "linear"), (9833, 0.0, "linear"))))
p['tracks'] = tracks
json.dump(p, open('scene.montagent.json', 'w'), indent=1)
