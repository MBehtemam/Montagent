"""Cold-observer sheets for #422. Picture sheets isolate served tile width by
downscaling ONE 18-tile 6x3 sheet (same content at every width); label sheets
put random realistic ADR-0098 labels at a fixed served type size under 140 px tiles."""
import json, math, os, random
from PIL import Image, ImageDraw, ImageFont
SP = os.path.dirname(os.path.abspath(__file__))  # expects tiles/ (doctored) and clean/ frames beside it; see FINDINGS.md
REPO = os.path.abspath(os.path.join(SP, "..", "..", ".."))
FONT = os.path.join(REPO, "fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf")
INST = [0,3018,5316,10468,17472,22622,30603,35753,42763,47343,53856,56116,57116,58116,59116,60116,61116,64016]
SRC_W, SRC_H = 1080, 1920; LAB_H = int(SRC_H*0.11); COLS, ROWS = 6, 3
def painted(t): return math.ceil(t*25/1000)*40
def core(i, t): p = painted(t); return f"{i+1} {p}ms +{p-t}"
def fitted(d, text, maxh):
    for s in range(int(maxh), 5, -1):
        f = ImageFont.truetype(FONT, s)
        if d.textlength(text, font=f) <= SRC_W*0.94: return f
def authored(tiledir, labels, fixed=None):
    W, H = COLS*SRC_W, ROWS*(SRC_H+LAB_H)
    sh = Image.new("RGB", (W, H), (20,20,24)); d = ImageDraw.Draw(sh)
    for i, t in enumerate(INST):
        c, r = i % COLS, i // COLS; x0, y0 = c*SRC_W, r*(SRC_H+LAB_H)
        sh.paste(Image.open(f"{tiledir}/{t}.png").convert("RGB"), (x0, y0))
        f = ImageFont.truetype(FONT, fixed) if fixed else fitted(d, labels[i], LAB_H*0.78)
        d.text((x0+SRC_W*0.03, y0+SRC_H+LAB_H*0.1), labels[i], font=f, fill=(255,255,255))
        d.rectangle([x0,y0,x0+SRC_W-1,y0+SRC_H+LAB_H-1], outline=(90,90,100), width=3)
    return sh
def out(sheet, tile_w, name):
    k = tile_w/SRC_W; im = sheet.resize((round(sheet.width*k), round(sheet.height*k)), Image.LANCZOS)
    im.save(f"{SP}/obs/{name}.png"); return im.size
os.makedirs(f"{SP}/obs", exist_ok=True)
rnd = random.Random(422); manifest = {}
labels = [core(i,t) for i,t in enumerate(INST)]
doc = authored(f"{SP}/tiles", labels); clean = authored(f"{SP}/clean", labels)
codes = {}
for w in (184, 160, 148, 140, 130, 120, 92):
    name = "p" + "".join(rnd.choice("abcdefghjkmnpqrstuvwxyz") for _ in range(5))
    manifest[name] = dict(kind="picture", tile_w=w, size=out(doc, w, name))
name = "pctrl"+"".join(rnd.choice("abcdefghjkmnpqrstuvwxyz") for _ in range(3))
manifest[name] = dict(kind="control-clean", tile_w=184, size=out(clean, 184, name))
# label sheets: random realistic labels, full ADR-0098 form, fixed served type size at 140 px tiles
fx = json.load(open(os.path.join(REPO, "fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json")))
ids = sorted({e["id"] for t in fx["tracks"] for e in t.get("elements",[])})
for s in (5, 6, 7, 8, 10):
    ts = sorted(rnd.sample(range(0, 65000), 18)); lab = []
    for i, t in enumerate(ts):
        p = painted(t); off = rnd.choice([0, rnd.randint(1, 39)])
        lab.append(f"{i+1} {p}ms +{off} {rnd.choice('+-')}{rnd.choice(ids)}")
    auth = s*SRC_W/140
    probe = ImageDraw.Draw(Image.new("RGB",(1,1)))
    f = ImageFont.truetype(FONT, round(auth))
    lab = [l if probe.textlength(l, font=f) <= SRC_W*0.94 else l.rsplit(" ",1)[0] for l in lab]
    sheet = authored(f"{SP}/tiles", lab, fixed=round(auth))
    name = "l" + "".join(rnd.choice("abcdefghjkmnpqrstuvwxyz") for _ in range(5))
    manifest[name] = dict(kind="label", type_px=s, tile_w=140, size=out(sheet, 140, name), truth=lab)
json.dump(manifest, open(f"{SP}/manifest.json","w"), indent=1)
for k,v in manifest.items(): print(k, v["kind"], v.get("tile_w"), v.get("type_px",""), v["size"])
