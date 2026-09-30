from PIL import Image
from rig import Rig
rig = Rig(1.0)
poses = [dict(lift_l=55, bend_l=-10, lift_r=-55, bend_r=8, head=-6),
         dict(lift_l=55, bend_l=42, lift_r=35, bend_r=5, head=6),
         dict(lift_l=-55, bend_l=8, lift_r=55, bend_r=30, lean=-6, sx=1.08, sy=0.9)]
tiles=[]
for p in poses:
    c = Image.new("RGBA", (1024,1400), (245,240,230,255))
    q=dict(x=512,y=1330); q.update(p); rig.draw(c,q)
    tiles.append(c.crop((80,400,944,1000)))
out=Image.new("RGB",(864*3,600))
for i,t in enumerate(tiles): out.paste(t.convert("RGB"),(864*i,0))
out.save("joints.png")
