import json, subprocess, os, sys
os.chdir('/private/var/folders/bk/5qm8ch691nz6vjb_4h98_4vw0000gn/T/montagent-eval-cdbfi247/work')

TOL, SOFT, SPILL = [float(x) for x in os.environ.get("KEY", "0.27,0.05,0.6").split(",")]
MUS, MUS_END = 0.12, 0.45

INK, PAPER, SIGNAL = '#101418', '#F5F0E6', '#FF5A36'
words = json.load(open('presenter/take-2.words.json'))
W = {i: w for i, w in enumerate(words)}

p = json.load(open('project.json'))
p['fonts'] = {"bold": [{"file": "fonts/Inter-Bold.ttf"}], "regular": [{"file": "fonts/Inter-Regular.ttf"}]}
p['tracks'] = []
json.dump(p, open('project.json', 'w'))

VID_END = 9800           # presenter take: source 0..9800 placed at timeline 0
FADE_OUT = (9520, 9760)  # last word ends at 9500
CARD_IN = (9880, 10280)
CAP_END = 10000

def kf(pairs):
    out = []
    for i, (t, v, *e) in enumerate(pairs):
        r = {"t": t, "v": v}
        if i:
            r["ease"] = e[0] if e else "linear"
        out.append(r)
    return out

def measure(specs):
    r = subprocess.run(['montagent', 'measure', 'project.json', '--json', '--elements', json.dumps(specs)],
                       capture_output=True, text=True)
    return [x['ok'] for x in json.loads(r.stdout)['measure']['results']]

cap_fade = kf([(9760, 1.0), (9960, 0.0, "ease-in")])
fade_out = kf([(FADE_OUT[0], 1.0, "step"), (FADE_OUT[1], 0.0, "ease-in")])

# ---- presenter ---------------------------------------------------------------
S = 1.1
VW, VH = round(1920 * S), round(1080 * S)
PX = 540 + round((960 - 940) * S)   # presenter's body centre (~x 940 in source) onto frame centre
presenter = {
    "id": "presenter", "type": "video", "start": 0, "end": VID_END,
    "source": "presenter/take-2.mp4", "source_start": 0, "source_end": VID_END,
    "x": PX, "y": 1350, "origin": "bottom-center", "width": VW, "height": VH, "fit": "contain",
    "scale": kf([(0, [1.0, 1.0]), (9760, [1.035, 1.035], "ease-in-out")]),
    "opacity": fade_out,
    "volume": kf([(9600, 1.3), (9760, 0.0)]),
    "effects": [{"name": "chroma", "color": "#00FF22", "tolerance": TOL, "softness": SOFT, "spill": SPILL}],
}

# ---- background ----------------------------------------------------------------
glow = {"id": "glow", "type": "ellipse", "start": 0, "end": VID_END,
        "x": 540, "y": 600, "origin": "center", "width": 1000, "height": 1000, "fill": "#FF5A3622",
        "opacity": fade_out, "effects": [{"name": "blur", "radius": 140}]}

# ---- brand header while speaking --------------------------------------------------
LH = 56
LW = round(2694 * LH / 623)
header = {"id": "header-lockup", "type": "image", "start": 200, "end": VID_END,
          "source": "brand/lockup-on-dark.png", "x": 540, "y": kf([(200, 76), (700, 92, "ease-out")]),
          "origin": "center", "width": LW, "height": LH, "fit": "contain",
          "opacity": kf([(200, 0.0), (700, 1.0, "ease-out"), (FADE_OUT[0], 1.0, "step"), (FADE_OUT[1], 0.0, "ease-in")])}

# ---- captions ----------------------------------------------------------------------
chunks = [  # (first word, last word, end ms, style)
    (0, 2, 1260, 'plain'),        # Here's the trick.
    (3, 4, 1820, 'plain'),        # Every element
    (5, 7, 2480, 'plain'),        # in the video
    (8, 12, 4168, 'plain'),       # is one line of text,
    (13, 17, 6726, 'plain'),      # so my agent can change
    (18, 20, 7680, 'pay'),        # exactly one thing
    (21, 25, CAP_END, 'pay'),     # and prove / nothing else moved.
]
BREAK_AFTER = {22}
ACCENT = {19, 25}  # "one", "moved."
CY = 1075
specs = []
for a, b, end, style in chunks:
    size = 60 if style == 'plain' else 80
    runs = []
    for i in range(a, b + 1):
        sep = '' if i == b else ('\n' if i in BREAK_AFTER else ' ')
        runs.append({"text": W[i]['word'] + sep})
    specs.append({"runs": runs, "font": "bold", "size": size, "line_height": 1.2 if style == 'plain' else 1.1})
ms = measure(specs)

caps, plates = [], []
for (a, b, end, style), spec, m in zip(chunks, specs, ms):
    start = W[a]['start']
    base = PAPER if style == 'plain' else INK
    tw = m['extent']['width']; th = m['block_height']
    padx, pady = (34, 16) if style == 'plain' else (44, 24)
    pw, ph = int(tw) + 1 + 2 * padx, th + 2 * pady
    pop = kf([(start, [0.86, 0.86]), (start + 220, [1.0, 1.0], [0.3, 1.5, 0.6, 1.0])]) if style == 'pay' else None
    plate = {"id": f"plate-{a:02d}", "type": "rect", "start": start, "end": end,
             "x": 540, "y": CY, "origin": "center", "width": pw, "height": ph,
             "fill": "#101418E0" if style == 'plain' else PAPER, "radius": 20 if style == 'plain' else 26}
    if pop:
        plate["scale"] = pop
    if end == CAP_END:
        plate["opacity"] = cap_fade
    plates.append(plate)
    # one state per spoken word: words not yet spoken are laid out but transparent,
    # so the line never reflows as it fills in
    for k in range(a, b + 1):
        s0 = W[k]['start']
        s1 = W[k + 1]['start'] if k < b else end
        runs = []
        for j, i in enumerate(range(a, b + 1)):
            col = (SIGNAL if i in ACCENT else base) if i <= k else base + '00'
            r = {"text": spec['runs'][j]['text']}
            if col != base:
                r["color"] = col
            runs.append(r)
        cap = {"id": f"cap-{k:02d}", "group": f"line-{a:02d}", "type": "text", "start": s0, "end": s1,
               "x": 540, "y": CY, "origin": "center", "width": int(tw) + 2, "height": th,
               "font": "bold", "size": spec['size'], "line_height": spec['line_height'], "color": base,
               "align": "center", "runs": runs, "caption": False}
        if pop and s0 < start + 220:
            cap["scale"] = pop
        if s1 == CAP_END:
            cap["opacity"] = cap_fade
        caps.append(cap)

# ---- end card -------------------------------------------------------------------
EW = 780
EH = round(623 * EW / 2694)
ECY = 630
end_lockup = {"id": "end-lockup", "type": "image", "start": CARD_IN[0], "end": 12000,
              "source": "brand/lockup-on-dark.png", "x": 540, "y": ECY, "origin": "center",
              "width": EW, "height": EH, "fit": "contain",
              "scale": kf([(CARD_IN[0], [0.94, 0.94]), (CARD_IN[1] + 300, [1.0, 1.0], "ease-out")]),
              "opacity": kf([(CARD_IN[0], 0.0), (CARD_IN[1], 1.0, "ease-out")])}
tag_spec = {"runs": [{"text": "The video editor AI agents drive."}], "font": "regular", "size": 40, "line_height": 1.2}
tm = measure([tag_spec])[0]
tagline = {"id": "end-tagline", "type": "text", "start": 10200, "end": 12000,
           "x": 540, "y": ECY + EH // 2 + 80, "origin": "center", "width": int(tm['extent']['width']) + 2,
           "height": tm['block_height'], "font": "regular", "size": 40, "line_height": 1.2, "color": PAPER,
           "align": "center", "runs": tag_spec['runs'],
           "opacity": kf([(10200, 0.0), (10600, 1.0, "ease-out")]), "caption": False}

# ---- music ------------------------------------------------------------------------
music = {"id": "music", "type": "audio", "start": 0, "end": 12000,
         "source": "music/bed-120bpm.wav", "source_start": 0, "source_end": 12000,
         "volume": kf([(0, 0.0), (300, MUS, "ease-out"), (9500, MUS, "step"), (10000, MUS_END, "ease-in-out"),
                       (10900, MUS_END, "step"), (11900, 0.0, "ease-in")])}

p['tracks'] = [
    {"name": "background", "layer": 0, "elements": [glow]},
    {"name": "presenter", "layer": 2, "elements": [presenter]},
    {"name": "header", "layer": 3, "elements": [header]},
    {"name": "plates", "layer": 4, "elements": plates},
    {"name": "captions", "layer": 5, "elements": caps},
    {"name": "end-card", "layer": 6, "elements": [end_lockup]},
    {"name": "end-tagline", "layer": 7, "elements": [tagline]},
    {"name": "music", "layer": 0, "elements": [music]},
]
json.dump(p, open('project.json', 'w'), ensure_ascii=False)
subprocess.run(['montagent', 'fmt', 'project.json'])
subprocess.run(['montagent', 'validate', 'project.json'])
