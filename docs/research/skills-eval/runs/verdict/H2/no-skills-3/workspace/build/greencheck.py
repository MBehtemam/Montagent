# counts greenish pixels (g exceeds both r and b by > margin) in a PNG frame
import sys, subprocess
path = sys.argv[1]; margin = int(sys.argv[2]) if len(sys.argv) > 2 else 20
w, h = map(int, subprocess.run(["ffprobe","-v","error","-select_streams","v:0","-show_entries","stream=width,height","-of","csv=p=0",path],capture_output=True,text=True).stdout.strip().split(","))
raw = subprocess.run(["ffmpeg","-loglevel","error","-i",path,"-f","rawvideo","-pix_fmt","rgb24","-"],capture_output=True).stdout
n = 0; pts = []
for i in range(0, len(raw), 3):
    r, g, b = raw[i], raw[i+1], raw[i+2]
    if g > r + margin and g > b + margin:
        n += 1
        if len(pts) < 8: pts.append(((i//3) % w, (i//3)//w, r, g, b))
print(path, "greenish:", n, pts)
