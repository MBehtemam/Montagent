import zlib, struct
def load(path):
    d=open(path,'rb').read(); assert d[:8]==b'\x89PNG\r\n\x1a\n'
    i=8; idat=b''; 
    while i<len(d):
        n=struct.unpack('>I',d[i:i+4])[0]; t=d[i+4:i+8]; c=d[i+8:i+8+n]; i+=12+n
        if t==b'IHDR': w,h,bd,ct,_,_,il=struct.unpack('>IIBBBBB',c)
        elif t==b'IDAT': idat+=c
    assert bd==8 and il==0, (bd,il)
    bpp={6:4,2:3,0:1,4:2}[ct]
    raw=zlib.decompress(idat); stride=w*bpp; out=bytearray(h*stride); prev=bytearray(stride); p=0
    for y in range(h):
        f=raw[p]; p+=1; line=bytearray(raw[p:p+stride]); p+=stride
        if f==1:
            for x in range(bpp,stride): line[x]=(line[x]+line[x-bpp])&255
        elif f==2:
            for x in range(stride): line[x]=(line[x]+prev[x])&255
        elif f==3:
            for x in range(stride): line[x]=(line[x]+((line[x-bpp] if x>=bpp else 0)+prev[x])//2)&255
        elif f==4:
            for x in range(stride):
                a=line[x-bpp] if x>=bpp else 0; b=prev[x]; c=prev[x-bpp] if x>=bpp else 0
                pa=abs(b-c); pb=abs(a-c); pc=abs(a+b-2*c)
                pr=a if pa<=pb and pa<=pc else (b if pb<=pc else c)
                line[x]=(line[x]+pr)&255
        out[y*stride:(y+1)*stride]=line; prev=line
    return w,h,bpp,out
def bbox(path,thr=128):
    w,h,bpp,px=load(path)
    xs=[];ys=[]
    x0=w;y0=h;x1=-1;y1=-1
    for y in range(h):
        row=px[y*w*bpp:(y+1)*w*bpp]
        for x in range(w):
            if row[x*bpp+bpp-1]>=thr:
                if x<x0:x0=x
                if x>x1:x1=x
                if y<y0:y0=y
                y1=y
    return (x0,y0,x1,y1)
