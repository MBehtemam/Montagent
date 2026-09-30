from PIL import Image
from rig import Rig
rig = Rig(0.5)
poses = {
 "rest": dict(),
 "sides": dict(lift_l=-55, bend_l=8, lift_r=-55, bend_r=8),
 "wave_a": dict(lift_l=75, bend_l=12, lift_r=-45, bend_r=10, mouth="open"),
 "wave_b": dict(lift_l=75, bend_l=68, lift_r=-45, bend_r=10, head=-4),
 "point": dict(lift_l=-45, bend_l=10, lift_r=35, bend_r=5, head=5, blink=True),
 "cheer": dict(lift_l=95, bend_l=12, lift_r=95, bend_r=12, mouth="round"),
}
W = 540*len(poses)
c = Image.new("RGBA", (W, 720), (245,240,230,255))
for i,(k,p) in enumerate(poses.items()):
    q = dict(x=270+540*i, y=690); q.update(p)
    rig.draw(c, q)
c.convert("RGB").save("poses.png")
