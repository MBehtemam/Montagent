photos=[("photo-05-intro",0,15000),("photo-05",3018,18018),("photo-06",17472,32472),
 ("photo-07",30603,45603),("photo-08",42763,57763),("photo-05-quiz",53856,68856),
 ("photo-05-loop",64016,79016)]
sc=[];pr=[]
for n,a,b in photos:
    sc.append('"scale":[{"t":%d,"v":1.0},{"t":%d,"v":1.08,"ease":"ease-in-out"}]'%(a,b))
    pr.append('"scale":[{"t":%d,"v":[1.0,1.0]},{"t":%d,"v":[1.08,1.08],"ease":"ease-in-out"}]'%(a,b))
for s,p in zip(sc,pr): print(s); print(p); print()
print("scalar total bytes:",sum(map(len,sc)),"  pair total bytes:",sum(map(len,pr)),
      "  delta:",sum(map(len,pr))-sum(map(len,sc)),"bytes over 14 keyframes =",
      (sum(map(len,pr))-sum(map(len,sc)))/14,"per keyframe")
