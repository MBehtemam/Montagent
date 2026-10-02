import subprocess, json
def load(path):
    out=subprocess.run(['ffprobe','-v','error','-select_streams','v:0','-show_entries','stream=width,height','-of','json',path],capture_output=True,text=True).stdout
    s=json.loads(out)['streams'][0]; w,h=s['width'],s['height']
    raw=subprocess.run(['ffmpeg','-v','error','-i',path,'-f','rawvideo','-pix_fmt','rgba','-'],capture_output=True).stdout
    return w,h,raw
