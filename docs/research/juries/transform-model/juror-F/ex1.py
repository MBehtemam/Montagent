from fractions import Fraction as F

def lerp(a,b,t): return a+(b-a)*t

def eval_linear(kfs, t):
    if t<=kfs[0][0]: return float(kfs[0][1])
    if t>=kfs[-1][0]: return float(kfs[-1][1])
    for i in range(len(kfs)-1):
        t0,v0=kfs[i]; t1,v1=kfs[i+1]
        if t0<=t<=t1:
            return float(v0+(v1-v0)*(t-t0)/(t1-t0))

def MOVE(kfs,at,d): return [(t+d if t>=at else t, v) for t,v in kfs]
def HOLD(kfs,at,d): return list(kfs)
def SPLIT(kfs,at,d,ev=eval_linear):
    v=ev(kfs,at)
    out=[(t,v_) for t,v_ in kfs if t<at]+[(at,v),(at+d,v)]+[(t+d,v_) for t,v_ in kfs if t>=at]
    return sorted(out)

def report(name, kfs, start, end, at, d, newstart, newend):
    print(f"\n--- {name}: elem [{start},{end}) kfs {kfs}  shift(at={at},delta={d}) -> elem [{newstart},{newend})")
    for rn,f in (("MOVE",MOVE),("HOLD",HOLD),("SPLIT",SPLIT)):
        nk=f(kfs,at,d)
        vend=eval_linear(nk,newend)
        vstart=eval_linear(nk,newstart)
        print(f"  {rn:6s} {[(t,round(v,6)) for t,v in nk]}  v@newstart={vstart:.6f} v@newend={vend:.6f}")

# fixture
p06=[(17472,1.0),(32472,1.08)]
report("1-base photo-06", p06, 17472,30603, 20000,2000, 17472,32603)

# (b) at exactly on an existing keyframe (== element start == first kf)
report("1b at==first kf(17472)", p06, 17472,30603, 17472,2000, 19472,32603)
# at exactly on last kf
report("1b at==last kf(32472)", p06, 17472,30603, 32472,2000, 17472,30603)

# (c) element entirely after at: photo-07
p07=[(30603,1.0),(45603,1.08)]
report("1c photo-07 entirely after", p07, 30603,42763, 20000,2000, 32603,44763)

# (d) trimmed move: keyframe past own end. photo-06 again, at BETWEEN end and last kf
report("1d at=31000 (after elem end, before last kf)", p06, 17472,30603, 31000,2000, 17472,30603)
# photo-05-loop: kf 13.8s past project end
loop=[(64016,1.0),(79016,1.08)]
report("1d photo-05-loop at=70000", loop, 64016,65216, 70000,2000, 64016,65216)

# on-screen consequence of 1d
def val(kfs,t): return eval_linear(kfs,t)
before=val(p06,30603); after=val(MOVE(p06,31000,2000),30603)
print(f"\n1d ON-SCREEN: photo-06 scale at its own end 30603: before={before:.6f} after MOVE={after:.6f} delta={after-before:.6f}")
print(f"   framing change on a 1080x1300 box: width {1080*before:.1f} -> {1080*after:.1f} px  ({1080*(before-after):.2f} px)")
mid=24037
print(f"   midpoint 24037: before={val(p06,mid):.6f} after={val(MOVE(p06,31000,2000),mid):.6f}")
