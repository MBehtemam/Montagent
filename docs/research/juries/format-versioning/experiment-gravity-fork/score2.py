"""Score by VISIBLE SOURCE BAND, honouring origin. Usage: score2.py <file>"""
import json,sys,re
VOFF={"top":0,"center":0.5,"bottom":1.0}
def band(el):
    g=lambda k,d=None:(re.search(r'"%s":(-?\d+)'%k,el) or [None,d])
    y=int(re.search(r'"y":(-?\d+)',el).group(1))
    h=int(re.search(r'"height":(-?\d+)',el).group(1))
    o=re.search(r'"origin":"([^"]+)"',el).group(1)
    c=[int(x) for x in re.search(r'"clip":\[([^\]]+)\]',el).group(1).split(',')]
    vert=o.split('-')[0]
    top = y - {"top":0,"center":h//2,"bottom":h}.get(vert,0)
    vt,vb=max(top,c[1]),min(top+h,c[1]+c[3])
    return (max(0,vb-vt), vt-top, vb-top)
s=open(sys.argv[1]).read()
els={m.group(1):m.group(0) for m in re.finditer(r'\{"id":"([^"]+)".*?\}(?=,?\n)',s)}
try: json.loads(s)
except Exception as e: print("  INVALID JSON:",e); sys.exit()
print("  gravity removed:", "yes" if '"gravity"' not in s else "NO")
res=[]
for i in ("photo-06","photo-07"):
    h,a,b=band(els[i])
    if h==0: res.append(f"{i}: BLANK")
    elif (a,b)==(612,1912): res.append(f"{i}: CORRECT (bottom band {a}..{b})")
    elif (a,b)==(0,1300): res.append(f"{i}: SILENT DEFECT (top band {a}..{b})")
    else: res.append(f"{i}: OTHER band {a}..{b}")
# inert six must be unchanged
base={m.group(1):m.group(0) for m in re.finditer(r'\{"id":"([^"]+)".*?\}(?=,?\n)',open('stale.json').read())}
bad=[i for i in ("photo-05-intro","photo-05","photo-08","photo-05-quiz","photo-05-loop","handle-logo")
     if band(els[i])!=band(base[i])]
print("  inert six preserved:", "6/6" if not bad else f"FAILED {bad}")
for r in res: print("  "+r)
