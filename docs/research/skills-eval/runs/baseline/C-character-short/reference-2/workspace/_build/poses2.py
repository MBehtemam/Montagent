from PIL import Image
from rig import Rig
rig = Rig(0.5)
poses = [
 dict(lift_l=55, bend_l=-10, lift_r=-50, bend_r=8),
 dict(lift_l=55, bend_l=20, lift_r=-50, bend_r=8),
 dict(lift_l=55, bend_l=45, lift_r=-50, bend_r=8),
 dict(lift_l=65, bend_l=0, lift_r=-50, bend_r=8),
 dict(lift_l=65, bend_l=35, lift_r=-50, bend_r=8),
 dict(lift_l=55, bend_l=30, lift_r=55, bend_r=30),
 dict(lift_l=45, bend_l=45, lift_r=45, bend_r=45),
]
c = Image.new("RGBA", (540*len(poses), 720), (245,240,230,255))
for i,p in enumerate(poses):
    q = dict(x=270+540*i, y=690); q.update(p); rig.draw(c, q)
c.convert("RGB").save("poses2.png")
