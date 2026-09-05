P=[(0.0,0.0),(0.42,0.0),(0.58,1.0),(1.0,1.0)]
def bez(P,u):
    c=[(1-u)**3,3*(1-u)**2*u,3*(1-u)*u*u,u**3]
    return (sum(ci*p[0] for ci,p in zip(c,P)), sum(ci*p[1] for ci,p in zip(c,P)))
def solve_u(P,xt):
    lo,hi=0.0,1.0
    for _ in range(200):
        m=(lo+hi)/2
        if bez(P,m)[0]<xt: lo=m
        else: hi=m
    return (lo+hi)/2
def decast(P,u):
    mid=lambda A,B: (A[0]+(B[0]-A[0])*u, A[1]+(B[1]-A[1])*u)
    q=[mid(P[0],P[1]),mid(P[1],P[2]),mid(P[2],P[3])]
    r=[mid(q[0],q[1]),mid(q[1],q[2])]
    s=mid(r[0],r[1])
    return [P[0],q[0],r[0],s],[s,r[1],q[2],P[3]]
def norm(seg):
    x0,y0=seg[0]; x3,y3=seg[3]; dx=x3-x0; dy=y3-y0
    return [((p[0]-x0)/dx,(p[1]-y0)/dy) for p in seg]
xf=2528/15000
u=solve_u(P,xf); x,y=bez(P,u)
print("split at x=%.6f -> u=%.6f, y=%.6f"%(xf,u,y))
L,R=decast(P,u); NL,NR=norm(L),norm(R)
f=lambda S:"cubic-bezier(%.6f, %.6f, %.6f, %.6f)"%(S[1][0],S[1][1],S[2][0],S[2][1])
print("LEFT ",f(NL)); print("RIGHT",f(NR))
named={"linear":(0,0,1,1),"ease":(0.25,0.1,0.25,1),"ease-in":(0.42,0,1,1),"ease-out":(0,0,0.58,1),"ease-in-out":(0.42,0,0.58,1)}
def ev(cp,xt):
    Q=[(0,0),(cp[0],cp[1]),(cp[2],cp[3]),(1,1)]
    return bez(Q,solve_u(Q,xt))[1]
for label,S in [("LEFT",NL),("RIGHT",NR)]:
    got=(S[1][0],S[1][1],S[2][0],S[2][1])
    rows=[]
    for n,cp in named.items():
        maxdev=max(abs(ev(cp,i/100)-ev(got,i/100)) for i in range(101))
        rows.append((maxdev,n,max(abs(a-b) for a,b in zip(cp,got))))
    rows.sort()
    print(label,"best named fit:",rows[0][1],"max |Δy| over segment = %.4f"%rows[0][0],"ctrl-pt err %.3f"%rows[0][2])
    print("   all:", ", ".join("%s %.3f"%(n,d) for d,n,_ in rows))
# translate LEFT best-name error into scale/pixels on photo-06
print()
got=(NL[1][0],NL[1][1],NL[2][0],NL[2][1])
best='ease-in'
dev=max(abs(ev(named[best],i/100)-ev(got,i/100)) for i in range(101))
print("photo-06 ramp span 0.08 scale; substituting %s on left half costs up to %.5f scale = %.2f px of a 1080-wide frame"%(best,dev*0.08*(y),dev*0.08*y*1080))
