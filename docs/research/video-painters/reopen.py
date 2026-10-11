#!/usr/bin/env python3
"""#926 Q1: spawn + seek + first frame of a feed-shaped ffmpeg (decode.rs frames_at's command,
origin 0/0, speed 1, fps 30), at several positions in the GOP. usage: reopen.py <src> <W> <H> <gop> [reps] [parallel]"""
import os, subprocess, sys, time, statistics, shutil, threading

src, W, H, gop = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
reps = int(sys.argv[5]) if len(sys.argv) > 5 else 5
par = int(sys.argv[6]) if len(sys.argv) > 6 else 1
ff = shutil.which("ffmpeg")
FPS = 30


def cmd(first):
    from_ms = first * 1000 // FPS
    w = max(0, from_ms - 200)
    window = f"{w // 1000}.{w % 1000:03d}"
    filt = ("settb=1/1000000,setpts='(ceil((2*(ceil((PTS-1)/1000)-0)-1)*1/(2*1))+0)*1000',"
            f"settb=1/{1000 * FPS},setpts='PTS-{1000 * first}',fps={FPS}:round=up:start_time=0,scale={W}:{H}")
    return [ff, "-hide_banner", "-loglevel", "error", "-threads", "1", "-ss", window, "-copyts", "-i", src,
            "-vf", filt, "-fps_mode", "passthrough", "-f", "rawvideo", "-pix_fmt", "rgba", "-"]


def one(first, nframes=1):
    t0 = time.monotonic()
    p = subprocess.Popen(cmd(first), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    need = W * H * 4
    t_first = None
    for i in range(nframes):
        got = 0
        while got < need:
            b = p.stdout.read(need - got)
            if not b:
                error = p.stderr.read().decode()
                p.wait()
                raise RuntimeError(f"feed ended before a complete frame: {error}")
            got += len(b)
        if i == 0:
            t_first = time.monotonic() - t0
    t_all = time.monotonic() - t0
    p.kill(); p.wait(); p.stdout.close(); p.stderr.close()
    return t_first, t_all


print(f"# {src} {W}x{H} gop={gop} load {os.getloadavg()[0]:.1f} parallel={par}")
for phase in (0, 0.25, 0.5, 0.75, 0.99):
    first = int(gop * phase)
    ts = []
    for _ in range(reps):
        out = []
        ths = [threading.Thread(target=lambda: out.append(one(first)[0])) for _ in range(par)]
        [t.start() for t in ths]; [t.join() for t in ths]
        ts += out
    ts_ms = sorted(t * 1000 for t in ts)
    print(f"GOP phase {phase:4.2f} (frame {first:3d} of GOP): first frame median {statistics.median(ts_ms):6.0f} ms  "
          f"min {ts_ms[0]:5.0f}  max {ts_ms[-1]:5.0f}  (n={len(ts_ms)})", flush=True)
# steady state: 60 frames after the open
rates = []
for _ in range(3):
    tf, ta = one(0, 61)
    rates.append(60 / (ta - tf))
print(f"steady state after open: {statistics.median(rates):.0f} fps (3 runs: {[round(r) for r in rates]})")
