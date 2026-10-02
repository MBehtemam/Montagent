import json, subprocess
P = json.load(open('build/p1.json'))
src = {e['id']: e for t in P['tracks'] for e in t['elements']}
bed = src['bed']
tmp = [src['tmp-0%d' % i] for i in range(1, 6)]
# highlight windows per word, in speaking order, from the caption script's timing pass
words = []
for e in tmp:
    for r in e['runs']:
        if 'highlight' in r: words.append((r['text'], r['highlight']['start'], r['highlight']['end']))

def measure(text, size, lh=None):
    el = {"font": "bold", "size": size, "runs": [{"text": text}]}
    if lh: el["line_height"] = lh
    m = json.loads(subprocess.run(['montagent', 'measure', 'h5.json', '--json', '--element', json.dumps(el)], capture_output=True, text=True).stdout)['measure']
    return m['advance_width'], m['block_height']

def runs_for(lines, ws):
    runs, k = [], 0
    for li, line in enumerate(lines):
        for wi, w in enumerate(line.split(' ')):
            t, s, e = ws[k]; assert t == w, (t, w); k += 1
            if wi: runs.append({"text": " "})
            elif li: runs.append({"text": "\n"})
            runs.append({"text": w, "highlight": {"start": s, "end": e, "color": "#FF5A36"}})
    assert k == len(ws)
    return runs

PAPER, INK, SIGNAL = "#F5F0E6", "#101418", "#FF5A36"
CY = 1060
pages = [  # (lines, start, end)
    (["Here's the trick."], 0, 1280),
    (["Every element in the video", "is one line of text,"], 1280, 4160),
    (["so my agent can change"], 4160, 6480),
]
caps, pills, k = [], [], 0
for i, (lines, s, e) in enumerate(pages, 1):
    n = sum(len(l.split(' ')) for l in lines)
    ws = words[k:k+n]; k += n
    w, h = measure("\n".join(lines), 56)
    caps.append({"id": f"cap-{i:02d}", "type": "text", "start": s, "end": e, "x": 540, "y": CY, "origin": "center",
                 "width": int(w) + 2, "height": h, "font": "bold", "size": 56, "color": PAPER, "align": "center",
                 "runs": runs_for(lines, ws)})
    pills.append({"id": f"cap-pill-{i:02d}", "type": "rect", "start": s, "end": e, "x": 540, "y": CY, "origin": "center",
                  "width": int(w) + 2 + 64, "height": h + 36, "fill": "#101418D9", "radius": 24})

# the payoff: bigger, on a solid Ink card outlined in Signal, popping in on its first word
lines = ["exactly one thing", "and prove", "nothing else moved."]
ws = words[k:]
PS, PE, FADE = 6720, 9960, 9720
w, h = measure("\n".join(lines), 72, 1.1)
PY = 1010
pop = [{"t": PS, "v": [0.86, 0.86]}, {"t": PS + 320, "v": [1.0, 1.0], "ease": [0.34, 1.56, 0.64, 1]}]
fade = [{"t": FADE, "v": 1.0}, {"t": PE - 40, "v": 0.0, "ease": "ease-in"}]
payoff = {"id": "payoff", "type": "text", "start": PS, "end": PE, "x": 540, "y": PY, "origin": "center",
          "width": int(w) + 2, "height": h, "font": "bold", "size": 72, "line_height": 1.1, "color": PAPER, "align": "center",
          "runs": runs_for(lines, ws), "scale": pop, "opacity": fade}
payoff_card = {"id": "payoff-card", "type": "rect", "start": PS, "end": PE, "x": 540, "y": PY, "origin": "center",
               "width": int(w) + 2 + 88, "height": h + 56, "fill": INK, "stroke": SIGNAL, "stroke_width": 6, "radius": 32,
               "scale": pop, "opacity": fade}

# the close: the header lockup travels to the centre and grows into the end card
CS, CE = 9720, 10520
LW, LH = 800, 185
header = {"id": "lockup", "type": "image", "start": 0, "end": 12000, "source": "brand/lockup.png",
          "x": [{"t": 0, "v": 540}], "y": [{"t": CS, "v": 210}, {"t": CE, "v": 675, "ease": [0.65, 0, 0.35, 1]}],
          "origin": "center", "width": LW, "height": LH, "fit": "contain",
          "scale": [{"t": CS, "v": [0.37875, 0.37875]}, {"t": CE, "v": [1.0, 1.0], "ease": [0.65, 0, 0.35, 1]}]}
del header["x"]; header["x"] = 540
presenter = src['presenter']
presenter.update({"end": 9960, "overrun": "hold", "opacity": [{"t": CS, "v": 1.0}, {"t": 9920, "v": 0.0, "ease": "ease-in"}]})

P['tracks'] = [
    {"name": "music", "layer": 0, "elements": [bed]},
    {"name": "presenter", "layer": 10, "elements": [presenter]},
    {"name": "lockup", "layer": 20, "elements": [header]},
    {"name": "captions-bg", "layer": 29, "elements": pills},
    {"name": "captions", "layer": 30, "elements": caps},
    {"name": "payoff-card", "layer": 33, "elements": [payoff_card]},
    {"name": "payoff", "layer": 34, "elements": [payoff]},
]
json.dump(P, open('h5.json', 'w'))
