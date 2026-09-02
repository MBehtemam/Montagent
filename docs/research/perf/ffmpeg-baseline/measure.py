#!/usr/bin/env python3
import subprocess, time, statistics, json, sys, os, pathlib, render

def run(cmd):
    t = time.perf_counter()
    r = subprocess.run(cmd, capture_output=True, text=True)
    el = time.perf_counter() - t
    if r.returncode != 0:
        raise RuntimeError(f"rc={r.returncode}\n{r.stderr[-2000:]}")
    return el

def bench(name, cmd, reps=3):
    cold = run(cmd)                       # first run: caches unwarmed
    warm = [run(cmd) for _ in range(reps)]
    rec = {"name": name, "cold": round(cold, 3),
           "warm_median": round(statistics.median(warm), 3),
           "warm_min": round(min(warm), 3),
           "warm_all": [round(w, 3) for w in warm]}
    print(f"{name:38s} cold {rec['cold']:7.2f}s   warm {rec['warm_median']:7.2f}s   "
          f"(min {rec['warm_min']:.2f})", flush=True)
    return rec

if __name__ == "__main__":
    which = sys.argv[1] if len(sys.argv) > 1 else "all"
    out = []
    T = render.TOTAL
    print(f"# fixture timeline {T:.3f}s @ {render.FPS}fps = {round(T*render.FPS)} frames "
          f"at {render.W}x{render.H}\n", flush=True)

    if which in ("all", "still"):
        out.append(bench("still: single frame -> PNG", render.build("m_still.png", still=True), reps=4))
    if which in ("all", "null"):
        out.append(bench("full: filter only (no encode)", render.build("x", null=True), reps=2))
    if which in ("all", "x264"):
        out.append(bench("full: libx264 medium crf23", render.build("m_x264.mp4", "libx264"), reps=2))
    if which in ("all", "vt"):
        out.append(bench("full: h264_videotoolbox 4M", render.build("m_vt.mp4", "videotoolbox"), reps=2))
    if which in ("all", "range"):
        out.append(bench("10s range (naive -to): x264", render.build("m_r264.mp4", "libx264", to=10), reps=2))
        out.append(bench("10s range (naive -to): vtb", render.build("m_rvt.mp4", "videotoolbox", to=10), reps=2))

    pathlib.Path(f"results_{which}.json").write_text(json.dumps(out, indent=2))
