import subprocess, sys
for f in sys.argv[1:]:
    probe = subprocess.run(["ffprobe","-v","error","-select_streams","v:0","-show_entries","stream=width,height","-of","csv=p=0",f],capture_output=True,text=True).stdout.strip().split(",")
    w,h = int(probe[0]), int(probe[1])
    raw = subprocess.run(["ffmpeg","-v","error","-i",f,"-f","rawvideo","-pix_fmt","rgb24","-"],capture_output=True).stdout
    n=0; worst=0
    for i in range(0,len(raw),3):
        r,g,b = raw[i],raw[i+1],raw[i+2]
        ex = g - max(r,b)
        if ex > 15: n+=1; worst=max(worst,ex)
    print(f, w,h, "greenish px:", n, "max excess:", worst)
