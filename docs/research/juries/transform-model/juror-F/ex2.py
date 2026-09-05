# de Casteljau subdivision of the CSS ease-in-out timing curve
# CSS timing functions are cubic-bezier(x1,y1,x2,y2) with P0=(0,0), P3=(1,1)
NAMED = {
 "linear":        (0.0,0.0,1.0,1.0),
 "ease":          (0.25,0.1,0.25,1.0),
 "ease-in":       (0.42,0.0,1.0,1.0),
 "ease-out":      (0.0,0.0,0.58,1.0),
 "ease-in-out":   (0.42,0.0,0.58,1.0),
}
def bez(p,u):
    (x0,y0),(x1,y1),(x2,y2),(x3,y3)=p
    mu=1-u
    x=mu**3*x0+3*mu*mu*u*x1+3*mu*u*u*x2+u**3*x3
    y=mu**3*y0+3*mu*mu*u*y1+3*mu*u*u*y2+u**3*y3
    return x,y
def decasteljau(p,u):
    def L(a,b): return (a[0]+(b[0]-a[0])*u, a[1]+(b[1]-a[1])*u)
    p0,p1,p2,p3=p
    a=L(p0,p1); b=L(p1,p2); c=L(p2,p3)
    d=L(a,b); e=L(b,c); f=L(d,e)
    return (p0,a,d,f),(f,e,c,p3)
def solve_u_for_x(p,X):
    lo,hi=0.0,1.0
    for _ in range(200):
        m=(lo+hi)/2
        if bez(p,m)[0]<X: lo=m
        else: hi=m
    return (lo+hi)/2
def renorm(seg):
    (x0,y0),(x1,y1),(x2,y2),(x3,y3)=seg
    dx=x3-x0; dy=y3-y0
    return ((x1-x0)/dx,(y1-y0)/dy,(x2-x0)/dx,(y2-y0)/dy)
def match(c):
    for n,v in NAMED.items():
        if all(abs(a-b)<5e-4 for a,b in zip(c,v)): return n
    return None

P=((0.0,0.0),(0.42,0.0),(0.58,1.0),(1.0,1.0))
# photo-06 segment 17472..32472, split at at=20000
frac=(20000-17472)/(32472-17472)
u=solve_u_for_x(P,frac)
L,R=decasteljau(P,u)
print(f"ease-in-out = cubic-bezier(0.42, 0, 0.58, 1)")
print(f"split at x = (20000-17472)/15000 = {frac:.7f}  -> bezier parameter u = {u:.7f}")
print(f"  y at split (eased progress) = {bez(P,u)[1]:.7f}")
print(f"  LEFT  raw ctrl pts  {[(round(a,6),round(b,6)) for a,b in L]}")
print(f"  RIGHT raw ctrl pts  {[(round(a,6),round(b,6)) for a,b in R]}")
cl=renorm(L); cr=renorm(R)
print(f"  LEFT  renormalised  cubic-bezier({cl[0]:.6f}, {cl[1]:.6f}, {cl[2]:.6f}, {cl[3]:.6f})  match={match(cl)}")
print(f"  RIGHT renormalised  cubic-bezier({cr[0]:.6f}, {cr[1]:.6f}, {cr[2]:.6f}, {cr[3]:.6f})  match={match(cr)}")
print()
print("nearest named curve, max |y| error over the unit interval:")
def ycurve(c,X):
    p=((0,0),(c[0],c[1]),(c[2],c[3]),(1,1))
    return bez(p,solve_u_for_x(p,X))[1]
for half,c in (("LEFT",cl),("RIGHT",cr)):
    best=None
    for n,v in NAMED.items():
        e=max(abs(ycurve(c,X/100)-ycurve(v,X/100)) for X in range(1,100))
        if best is None or e<best[1]: best=(n,e)
    print(f"  {half}: closest named = {best[0]}, max error {best[1]*100:.2f}% of the value range")

# what that error is worth on photo-06
print()
print("photo-06 scale range 1.0..1.08 over a 1080-wide box:")
for half,c in (("LEFT",cl),("RIGHT",cr)):
    best=min(((n,max(abs(ycurve(c,X/100)-ycurve(v,X/100)) for X in range(1,100))) for n,v in NAMED.items()), key=lambda z:z[1])
    print(f"  {half}: substituting '{best[0]}' misplaces scale by up to {best[1]*0.08:.5f} = {best[1]*0.08*1080:.2f} px of width")

# eased value at `at` for the SPLIT insert
yv=bez(P,u)[1]
print(f"\nSPLIT's interpolated value under ease-in-out: v = 1.0 + 0.08*{yv:.9f} = {1.0+0.08*yv:.9f}")
print(f"  (linear reading gave 1.013483)")
