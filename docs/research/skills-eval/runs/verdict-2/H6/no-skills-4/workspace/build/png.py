import zlib,struct
def load(path):
    d=open(path,'rb').read(); assert d[:8]==b'\x89PNG\r\n\x1a\n'
    i=8; idat=b''; 
    while i<len(d):
        n=struct.unpack('>I',d[i:i+4])[0]; t=d[i+4:i+8]; c=d[i+8:i+8+n]; i+=12+n
        if t==b'IHDR': w,h,bd,ct,_,_,il=struct.unpack('>IIBBBBB',c)
        elif t==b'IDAT': idat+=c
    assert bd==8 and il==0, (bd,il)
    bpp={6:4,2:3,0:1,4:2}[ct]
    raw=zlib.decompress(idat); stride=w*bpp; out=bytearray(); prev=bytearray(stride); p=0
    for y in range(h):
        f=raw[p]; p+=1; line=bytearray(raw[p:p+stride]); p+=stride
        for x in range(stride):
            a=line[x-bpp] if x>=bpp else 0; b=prev[x]; cc=prev[x-bpp] if x>=bpp else 0
            if f==1: line[x]=(line[x]+a)&255
            elif f==2: line[x]=(line[x]+b)&255
            elif f==3: line[x]=(line[x]+((a+b)>>1))&255
            elif f==4:
                pp=a+b-cc; pa=abs(pp-a); pb=abs(pp-b); pc=abs(pp-cc)
                pr=a if pa<=pb and pa<=pc else (b if pb<=pc else cc)
                line[x]=(line[x]+pr)&255
        out+=line; prev=line
    return w,h,bpp,bytes(out)
def bbox(path):
    w,h,bpp,px=load(path)
    if bpp!=4: return (0,0,w,h)
    xs=[];ys=[]
    x0,y0,x1,y1=w,h,-1,-1
    for y in range(h):
        row=px[y*w*4:(y+1)*w*4]
        for x in range(w):
            if row[x*4+3]>40:
                if x<x0:x0=x
                if x>x1:x1=x
                if y<y0:y0=y
                y1=y
    return (x0,y0,x1,y1)
