"""PROTOTYPE -- label legibility at tile scale (issue #396, question 3)."""
import sys, os, math; sys.path.insert(0,'.')
from make_sheets import *
from PIL import Image, ImageDraw, ImageFont

def fitted(draw, text, font_path, box_w, max_h):
    """Largest size at which `text` fits box_w; None if even 8px overflows."""
    for s in range(int(max_h), 5, -1):
        f = ImageFont.truetype(font_path, s)
        if draw.textlength(text, font=f) <= box_w * 0.94:
            return f, s
    return None, None

def build_labelled(name, instants, cols, rows, label_of, crop=None, tier=STD,
                   label_frac=0.11):
    src_w, src_h = (1080,1920) if crop is None else (crop[2],crop[3])
    lab_h=int(src_h*label_frac); cell_w,cell_h=src_w,src_h+lab_h
    W,H=cols*cell_w,rows*cell_h; sw,sh,t=served(W,H,tier)
    sheet=Image.new("RGB",(W,H),(20,20,24)); d=ImageDraw.Draw(sheet)
    sizes=[]
    for i,ms in enumerate(instants):
        c,r=i%cols,i//cols
        im=Image.open(os.path.join(TILES,f"{ms}.png")).convert("RGB")
        if crop: im=im.crop((crop[0],crop[1],crop[0]+crop[2],crop[1]+crop[3]))
        x0,y0=c*cell_w,r*cell_h
        sheet.paste(im,(x0,y0))
        txt=label_of(i,ms)
        f,s=fitted(d,txt,FONT,cell_w,lab_h*0.78)
        sizes.append(s)
        d.text((x0+cell_w*0.03, y0+src_h+lab_h*0.1), txt, font=f, fill=(255,255,255))
        d.rectangle([x0,y0,x0+cell_w-1,y0+cell_h-1],outline=(90,90,100),
                    width=max(2,src_w//300))
    out=sheet.resize((sw,sh),Image.LANCZOS); out.save(os.path.join(OUT,f"{name}.png"))
    scale=sw/W
    served_pt=[round(s*scale,1) for s in sizes]
    print(f"{name:<24} {len(instants):>2}t {cols}x{rows} served {sw}x{sh} {t:>4}tok "
          f"tile {sw/cols:.0f}px  label authored {min(sizes)}-{max(sizes)}px -> "
          f"SERVED {min(served_pt)}-{max(served_pt)}px")

SHORT = lambda i,ms: f"{i+1} {ms}"
LONG  = lambda i,ms: f"{i+1}  {ms}ms  {LABEL.get(ms,'')}"

for n,(c,r) in [(18,(6,3)),(30,(10,3)),(12,(6,2))]:
    picks=(INSTANTS*2)[:n]
    build_labelled(f"E-short-{n}",picks,c,r,SHORT)
    build_labelled(f"E-long-{n}",picks,c,r,LONG)
band=(0,1300,1080,360)
build_labelled("E-crop-long-18",INSTANTS,3,6,LONG,crop=band)
