import math
SH={'l':(356,736),'r':(666,736)}; EL={'l':(264,776),'r':(753,778)}; HA={'l':(113,868),'r':(908,867)}
NECK=(512,648); ROOT=(512,1322)
def rot(p,c,a):
    a=math.radians(a); x,y=p[0]-c[0],p[1]-c[1]
    return (c[0]+x*math.cos(a)-y*math.sin(a), c[1]+x*math.sin(a)+y*math.cos(a))
def fk(side,torso,ua,fa):
    s=rot(SH[side],ROOT,torso); e=rot(rot(EL[side],SH[side],ua),ROOT,torso)
    h=rot(rot(rot(HA[side],EL[side],fa),SH[side],ua),ROOT,torso)
    return s,e,h
def ik(side,torso,target,elbow_sign):
    # target in drawing coords (pre-root placement)
    s=rot(SH[side],ROOT,torso)
    a=math.dist(SH[side],EL[side]); b=math.dist(EL[side],HA[side]); d=math.dist(s,target)
    A=math.degrees(math.acos(max(-1,min(1,(a*a+d*d-b*b)/(2*a*d)))))
    td=math.degrees(math.atan2(target[1]-s[1],target[0]-s[0]))
    ud=td+elbow_sign*A
    u0=math.degrees(math.atan2(EL[side][1]-SH[side][1],EL[side][0]-SH[side][0]))
    ua=ud-u0-torso
    e=(s[0]+a*math.cos(math.radians(ud)),s[1]+a*math.sin(math.radians(ud)))
    fd=math.degrees(math.atan2(target[1]-e[1],target[0]-e[0]))
    f0=math.degrees(math.atan2(HA[side][1]-EL[side][1],HA[side][0]-EL[side][0]))
    fa=(fd-ud)-(f0-u0)
    n=lambda x:(x+180)%360-180
    return n(ua),n(fa)
