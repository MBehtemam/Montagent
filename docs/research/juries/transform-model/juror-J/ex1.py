from fractions import Fraction as F

def lerp_at(kfs, t):
    # kfs list of (time,value), linear, clamped
    if t<=kfs[0][0]: return F(kfs[0][1])
    if t>=kfs[-1][0]: return F(kfs[-1][1])
    for i in range(len(kfs)-1):
        a,b=kfs[i],kfs[i+1]
        if a[0]<=t<=b[0]:
            u=F(t-a[0],b[0]-a[0])
            return F(a[1])+(F(b[1])-F(a[1]))*u
def MOVE(kfs,at,d): return [(t+d if t>=at else t, v) for t,v in kfs]
def HOLD(kfs,at,d): return kfs
def SPLIT(kfs,at,d):
    v=lerp_at(kfs,at)
    out=[(t,v2) for t,v2 in kfs if t<at]+[(at,v),(at+d,v)]+[(t+d,v2) for t,v2 in kfs if t>=at]
    return sorted(out)

at,d=20000,2000
p6=[(17472,F('1.0')),(32472,F('1.08'))]
print("photo-06 base value at 20000:", float(lerp_at(p6,at)), lerp_at(p6,at))
for name,f in [("MOVE",MOVE),("HOLD",HOLD),("SPLIT",SPLIT)]:
    r=f(p6,at,d)
    print(name, [(t,float(v)) for t,v in r], "value at new end 32603:", float(lerp_at(r,32603)))
