#!/usr/bin/env python3
"""Frame diffs, via ffmpeg (no Python imaging deps). Throwaway."""
import subprocess, sys, struct

def rgb(path):
    w,h = subprocess.run(["ffprobe","-v","error","-select_streams","v:0",
        "-show_entries","stream=width,height","-of","csv=p=0:s=x",path],
        capture_output=True,text=True).stdout.strip().split("x")
    out = subprocess.run(["ffmpeg","-v","error","-i",path,"-f","rawvideo",
        "-pix_fmt","rgb24","-"],capture_output=True).stdout
    return out, int(w), int(h)

def diff(a,b,label):
    A,w,h = rgb(a); B,_,_ = rgb(b)
    n=min(len(A),len(B))
    tot=0; mx=0; over=0
    for i in range(0,n,3):
        d=max(abs(A[i]-B[i]),abs(A[i+1]-B[i+1]),abs(A[i+2]-B[i+2]))
        tot+=A[i]-B[i] if False else d
        if d>mx: mx=d
        if d>8: over+=1
    px=n//3
    print(f"{label:44s} mean={tot/px:7.3f} max={mx:3d} px>8={over/px*100:6.2f}%")

if __name__ == "__main__":
    for a,b,l in [(sys.argv[i],sys.argv[i+1],sys.argv[i+2]) for i in range(1,len(sys.argv),3)]:
        diff(a,b,l)
