from fractions import Fraction

# ---------- Exercise 1: three readings on photo-06 and hard cases ----------
# photo-06: start 17472 end 30603, scale [[17472,1.0],[32472,1.08]]
# shift(at=20000, delta=2000); ADR-0005 stretches end to 32603.
at, delta = 20000, 2000
k1t, k1v, k2t, k2v = 17472, 1.0, 32472, 1.08

def lerp(t, t1, v1, t2, v2): return v1 + (v2-v1)*(t-t1)/(t2-t1)

v_at = lerp(at, k1t,k1v,k2t,k2v)
print("value at 20000 (linear):", repr(v_at), "->", round(v_at,6))

# MOVE: ramp 17472->34472 (17000ms). value at element end 32603:
print("MOVE: value at new end 32603:", round(lerp(32603,17472,1.0,34472,1.08),4))
print("MOVE: value at 19000 (BEFORE at):", round(lerp(19000,17472,1.0,34472,1.08),6),
      "orig:", round(lerp(19000,k1t,k1v,k2t,k2v),6))
# HOLD: ramp unchanged, value at old-content-time check:
print("HOLD: frame that used to show at 25000 (v=%.6f) should now show at 27000; HOLD shows %.6f at 27000"
      % (lerp(25000,k1t,k1v,k2t,k2v), lerp(27000,k1t,k1v,k2t,k2v)))
# SPLIT check: at 27000 with keys [{17472,1.0},{20000,v_at},{22000,v_at},{34472,1.08}]
print("SPLIT: value at 27000:", round(lerp(27000,22000,v_at,34472,1.08),6),
      "== orig at 25000:", round(lerp(25000,k1t,k1v,k2t,k2v),6))
print("SPLIT: value at new end 32603:", round(lerp(32603,22000,v_at,34472,1.08),6),
      "orig at old end 30603:", round(lerp(30603,k1t,k1v,k2t,k2v),6))
print()

# ---------- Exercise 2: de Casteljau split of ease-in-out ----------
# CSS ease-in-out = cubic-bezier(0.42,0,0.58,1). P0=(0,0) P1=(.42,0) P2=(.58,1) P3=(1,1)
P0=(0.0,0.0); P1=(0.42,0.0); P2=(0.58,1.0); P3=(1.0,1.0)
def bez(t,P0,P1,P2,P3):
    u=1-t
    return tuple(u**3*P0[i]+3*u*u*t*P1[i]+3*u*t*t*P2[i]+t**3*P3[i] for i in (0,1))
# split point: x0 = fraction of segment where at falls = (20000-17472)/15000
x0 = (at-k1t)/(k2t-k1t)
print("x0 =", x0)
# solve x(t)=x0 by bisection
lo,hi=0.0,1.0
for _ in range(200):
    mid=(lo+hi)/2
    if bez(mid,P0,P1,P2,P3)[0] < x0: lo=mid
    else: hi=mid
tstar=(lo+hi)/2
xs,ys = bez(tstar,P0,P1,P2,P3)
print("t* =", round(tstar,10), " point=(%.8f, %.8f)"%(xs,ys))
print("=> eased value at 20000 =", round(1.0+0.08*ys,6))

def dc_split(t,P0,P1,P2,P3):
    def L(A,B): return ((1-t)*A[0]+t*B[0],(1-t)*A[1]+t*B[1])
    A=L(P0,P1); B=L(P1,P2); C=L(P2,P3)
    D=L(A,B); E=L(B,C); F=L(D,E)
    return (P0,A,D,F),(F,E,C,P3)
left,right = dc_split(tstar,P0,P1,P2,P3)
def renorm(seg):
    a,b = seg[0], seg[3]
    dx, dy = b[0]-a[0], b[1]-a[1]
    return [((p[0]-a[0])/dx, (p[1]-a[1])/dy if dy!=0 else 0) for p in seg]
Lr = renorm(left); Rr = renorm(right)
print("LEFT  half renormalised control points:", [(round(p[0],6),round(p[1],6)) for p in Lr])
print("RIGHT half renormalised control points:", [(round(p[0],6),round(p[1],6)) for p in Rr])
named = {"linear":(1/3,1/3,2/3,2/3),"ease":(0.25,0.1,0.25,1.0),"ease-in":(0.42,0,1,1),
         "ease-out":(0,0,0.58,1),"ease-in-out":(0.42,0,0.58,1)}
for tag,cps in (("LEFT",Lr),("RIGHT",Rr)):
    c=(cps[1][0],cps[1][1],cps[2][0],cps[2][1])
    print(tag,"as cubic-bezier(%.6f, %.6f, %.6f, %.6f)"%c)
    for n,(a,b,cc,d) in named.items():
        if max(abs(c[0]-a),abs(c[1]-b),abs(c[2]-cc),abs(c[3]-d))<1e-4:
            print("   MATCHES", n)
