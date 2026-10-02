import subprocess, sys
def green(png, x0=180, y0=180, x1=900, y1=1350, thr=20):
    raw = subprocess.run(["ffmpeg","-v","error","-i",png,"-vf",f"crop={x1-x0}:{y1-y0}:{x0}:{y0},format=rgb24","-f","rawvideo","-"],capture_output=True).stdout
    w = x1-x0; n=0; worst=0; pos=None
    for i in range(0, len(raw), 3):
        r,g,b = raw[i], raw[i+1], raw[i+2]
        d = g - max(r,b)
        if d > thr:
            n += 1
            if d > worst: worst=d; pos=((i//3)%w+x0, (i//3)//w+y0)
    return n, worst, pos
if __name__ == "__main__":
    for p in sys.argv[1:]:
        print(p, green(p))
