from fractions import Fraction as F
photos=[("photo-05-intro",0,3018,0,15000),("photo-05",3018,17472,3018,18018),
 ("photo-06",17472,30603,17472,32472),("photo-07",30603,42763,30603,45603),
 ("photo-08",42763,53856,42763,57763),("photo-05-quiz",53856,64016,53856,68856),
 ("photo-05-loop",64016,65216,64016,79016)]
def val(k,t):
    if t<=k[0][0]: return k[0][1]
    if t>=k[-1][0]: return k[-1][1]
    for a,b in zip(k,k[1:]):
        if a[0]<=t<=b[0]: return a[1]+(b[1]-a[1])*F(t-a[0],b[0]-a[0])
def shift_kfs(k,at,d):
    out=[(t+d if t>=at else t,v) for t,v in k]
    if k[0][0]<at<k[-1][0]:
        v=val(k,at)
        ins=[(at,v)]
        if not any(t==at for t,_ in k): ins.append((at+d,v))
        out=sorted(out+ins)
    return out
at,d=20000,2000
added=0
print("applying the rule to all seven animated elements, at=20000 delta=2000")
for n,s,e,k0,k1 in photos:
    k=[(k0,F('1.0')),(k1,F('1.08'))]
    ns,ne=(s,e+d) if s<at<e else ((s+d,e+d) if s>=at else (s,e))
    nk=shift_kfs(k,at,d); added+=len(nk)-len(k)
    # frame preservation check over the element's post-shift visible range
    bad=[]
    for t in range(ns,ne):
        old_t = t if t<at else t-d
        if val(nk,t)!=val(k,old_t): bad.append(t)
    print("  %-15s %5d-%5d -> %5d-%5d  kfs %d->%d  frame-preserving: %s"%(
        n,s,e,ns,ne,len(k),len(nk),"YES" if not bad else "NO (%d frames, first %d)"%(len(bad),bad[0])))
print("total keyframes added to the whole project:",added)
