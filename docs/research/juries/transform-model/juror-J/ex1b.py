from fractions import Fraction as F
R=lambda x: round(float(x),6)
def val(kfs,t):
    if t<=kfs[0][0]: return F(kfs[0][1])
    if t>=kfs[-1][0]: return F(kfs[-1][1])
    for a,b in zip(kfs,kfs[1:]):
        if a[0]<=t<=b[0]:
            return F(a[1])+(F(b[1])-F(a[1]))*F(t-a[0],b[0]-a[0])
photos=[("photo-05-intro",0,3018,0,15000),("photo-05",3018,17472,3018,18018),
 ("photo-06",17472,30603,17472,32472),("photo-07",30603,42763,30603,45603),
 ("photo-08",42763,53856,42763,57763),("photo-05-quiz",53856,64016,53856,68856),
 ("photo-05-loop",64016,65216,64016,79016)]
print("== case (d): keyframe past own end. value reached at element end, and %% of ramp used")
for n,s,e,k0,k1 in photos:
    v=val([(k0,F('1.0')),(k1,F('1.08'))],e)
    print(" %-15s end=%5d scale@end=%.4f  ramp used %.0f%%"%(n,e,v,100*(e-k0)/(k1-k0)))

print("\n== case (d2): at AFTER element end but BEFORE its last keyframe (photo-05, at=17800 d=2000)")
base=[(3018,F('1.0')),(18018,F('1.08'))]
print(" before: scale at 17472 (last visible frame) = %.6f"%val(base,17472))
mv=[(3018,F('1.0')),(20018,F('1.08'))]
print(" MOVE  : scale at 17472 = %.6f   delta %.6f -> %.1f px on a 1080 frame"%(val(mv,17472), val(base,17472)-val(mv,17472), float(val(base,17472)-val(mv,17472))*1080))
print(" SPLIT : first segment 3018->17800 untouched, scale at 17472 = %.6f (unchanged)"%val([(3018,F('1.0')),(17800,val(base,17800))],17472))
print(" HOLD  : unchanged")

print("\n== case (c): element wholly after at (photo-07, at=20000 d=2000 -> 32603..44763)")
b7=[(30603,F('1.0')),(45603,F('1.08'))]
print(" HOLD (kfs untouched): scale at new start 32603 = %.6f  (was 1.000000) -> visible pop of %.1f px"%(val(b7,32603),float(val(b7,32603)-1)*1080))
print(" MOVE/SPLIT: kfs -> 32603,47603 ; scale at new start = 1.000000")

print("\n== the semantic test: post-shift frame at 22000+x must equal pre-shift frame at 20000+x")
p6=[(17472,F('1.0')),(32472,F('1.08'))]
vsp=val(p6,20000)
S=[(17472,F('1.0')),(20000,vsp),(22000,vsp),(34472,F('1.08'))]
M=[(17472,F('1.0')),(34472,F('1.08'))]
H=p6
for x in (0,1000,5000,8603):
    print("  x=%5d  old@%5d=%.6f | SPLIT %.6f | MOVE %.6f | HOLD %.6f"%(
      x,20000+x,val(p6,20000+x),val(S,22000+x),val(M,22000+x),val(H,22000+x)))
print("\n inserted window [20000,22000): SPLIT holds %.6f (a real pause); MOVE %.6f->%.6f; HOLD %.6f->%.6f"%(
  vsp,val(M,20000),val(M,22000),val(H,20000),val(H,22000)))
