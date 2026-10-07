#!/usr/bin/env python3
"""ADR-0172's measurement, reproduced from scratch against this machine's `ffmpeg`.

Run from anywhere:  python3 docs/adr/keyed_volume_pieces_check.py
Exits non-zero, naming every defect, the moment one of its claims stops holding.
Takes about three minutes, most of it the one graph the ADR rejects.

Why this exists
---------------
ADR-0172 chose how a keyframed `volume`'s timed commands reach the sample they name. It
rests on four claims about `ffmpeg`, each checked here on the case that costs most: a
6-minute AAC tone whose level falls 1 -> 0 linearly, so `render` sends a command on every frame
but the first of its 10,800 at 30 fps: 10,799 (`render::instant_of`'s floored instants).

  1. Today's graph (`asendcmd` on the decoder's 1024-sample frames) hears a command late:
     a step at 266 ms (sample 12768) sounds at 13312.
  2. Uniform 1 ms frames (`asetnsamples=n=48`) in front of one `asendcmd` are exact but
     cost frames x commands: more than 5x today's graph.
  3. The chosen graph (1 ms frames, the commands cut into pieces of 256, each piece its own
     `asendcmd` and `volume`, joined by `concat`, regrouped to 1024) is exact on every one
     of the 17,280,000 samples: source x the step envelope, within float rounding.
  4. And it is no slower than today's graph (with 15% for a loaded machine).

The graph strings mirror `keyed_volume` in `crates/montagent-core/src/verbs/render.rs`;
the render's own behaviour is held by the `a_keyframed_volume_*` tests in
`crates/montagent-core/tests/render.rs`. Needs `ffmpeg` on PATH (ADR-0009) and skips with a
message rather than failing when it is absent.
"""
import array
import os
import shutil
import subprocess
import sys
import tempfile
import time

RATE, FPS, LENGTH_MS, PIECE = 48_000, 30, 360_000, 256


def seconds(ms):
    return f"{ms // 1000}.{ms % 1000:03d}"


def changes():
    """`render`'s commands for the fade: every frame instant whose value moved."""
    out, last, n = [], 1.0, 0
    while (t := n * 1000 // FPS) < LENGTH_MS:
        v = round(1 - t / LENGTH_MS, 6)
        if abs(v - last) > 1e-6:
            out.append((t, v))
            last = v
        n += 1
    return out


def volume(name, start, cmds):
    text = "".join(f"{seconds(t)} volume@{name} volume {v:.6f};" for t, v in cmds)
    return f"asendcmd=c='{text}',volume@{name}=volume={start:.6f}:eval=frame"


def graph(kind, cmds):
    head = (f"[0:a]aformat=sample_rates={RATE}:channel_layouts=stereo,"
            f"atrim=start=0:end={seconds(LENGTH_MS)},asetpts=PTS-STARTPTS,")
    if kind == "today":
        return head + volume("v0", 1.0, cmds) + "[out]"
    if kind == "uniform":
        return head + "asetnsamples=n=48:p=0," + volume("v0", 1.0, cmds) + "[out]"
    pieces = [cmds[i:i + PIECE] for i in range(0, len(cmds), PIECE)]
    labels = "".join(f"[s{k}]" for k in range(len(pieces)))
    g = (head + "asetnsamples=n=48:p=0,asegment=timestamps="
         + "|".join(seconds(p[0][0]) for p in pieces[1:]) + labels + ";")
    start = 1.0
    for k, piece in enumerate(pieces):
        g += f"[s{k}]{volume(f'v0_{k}', start, piece)},asetpts=PTS-STARTPTS[p{k}];"
        start = piece[-1][1]
    g += "".join(f"[p{k}]" for k in range(len(pieces)))
    return g + f"concat=n={len(pieces)}:v=0:a=1,asetnsamples=n=1024:p=0[out]"


def run(work, filtergraph, output, args):
    script = os.path.join(work, "graph.txt")
    with open(script, "w") as f:
        f.write(filtergraph)
    began = time.monotonic()
    done = subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", os.path.join(work, "tone.m4a"),
                           "-/filter_complex", script, "-map", "[out]", *args, output],
                          capture_output=True)
    if done.returncode:
        sys.exit(f"ffmpeg failed: {done.stderr.decode()}")
    return time.monotonic() - began


def pcm(path):
    a = array.array("f")
    with open(path, "rb") as f:
        a.frombytes(f.read())
    return a


def main():
    if not shutil.which("ffmpeg"):
        print("skipped: no ffmpeg on PATH (ADR-0009)")
        return
    defects = []
    with tempfile.TemporaryDirectory() as work:
        subprocess.run(["ffmpeg", "-v", "error", "-y", "-f", "lavfi", "-i",
                        f"sine=frequency=1000:sample_rate={RATE}:duration={LENGTH_MS // 1000}",
                        "-c:a", "aac", "-b:a", "160k", os.path.join(work, "tone.m4a")], check=True)
        mono = ["-ac", "1", "-f", "f32le"]

        # 1. A step at 266 ms on the decoder's frames.
        step = [(266, 0.25)]
        late = os.path.join(work, "late.f32")
        head = graph("today", step).replace(f"end={seconds(LENGTH_MS)}", "end=1.000")
        run(work, head, late, mono)
        x = pcm(late)
        full = max(abs(s) for s in x[4_800:9_600])
        # The first sample from which a whole period of the tone (48 samples) stays quiet.
        heard = next(i for i in range(9_600, 40_000)
                     if max(abs(s) for s in x[i:i + 48]) < 0.6 * full)
        if abs(heard - 13_312) > 4:
            defects.append(f"claim 1: a step at sample 12768 was heard at {heard}, not 13312")

        cmds = changes()
        if len(cmds) != 10_799:
            defects.append(f"the fade sends {len(cmds)} commands, not 10,799")
        aac = ["-c:a", "aac", "-b:a", "160k"]
        timing = {}
        for kind in ("today", "pieces", "uniform"):
            timing[kind] = run(work, graph(kind, cmds), os.path.join(work, f"{kind}.m4a"), aac)
        print("seconds for the 6-minute fade: "
              + ", ".join(f"{k} {v:.1f}" for k, v in timing.items()))
        if timing["uniform"] < 5 * timing["today"]:
            defects.append(f"claim 2: uniform 1 ms frames took {timing['uniform']:.1f} s, "
                           f"not over 5x today's {timing['today']:.1f} s")
        if timing["pieces"] > 1.15 * timing["today"]:
            defects.append(f"claim 4: the pieces took {timing['pieces']:.1f} s against "
                           f"today's {timing['today']:.1f} s")

        # 3. Every sample of the pieces graph against the source times the envelope.
        out, src = os.path.join(work, "pieces.f32"), os.path.join(work, "src.f32")
        run(work, graph("pieces", cmds), out, mono)
        run(work, f"[0:a]aformat=sample_rates={RATE}:channel_layouts=stereo[out]", src, mono)
        a, b = pcm(out), pcm(src)
        if len(a) != len(b) or len(a) != LENGTH_MS * RATE // 1000:
            defects.append(f"claim 3: {len(a)} samples out of {len(b)} in")
        else:
            points = [(0, 1.0)] + [(t * RATE // 1000, v) for t, v in cmds]
            worst, j = 0.0, 0
            for i, (x, y) in enumerate(zip(a, b)):
                while j + 1 < len(points) and points[j + 1][0] <= i:
                    j += 1
                worst = max(worst, abs(x - y * points[j][1]))
            if worst > 1e-6:
                defects.append(f"claim 3: a sample is {worst:.2e} off source x envelope")
            print(f"pieces graph: {len(a)} samples, worst difference {worst:.1e}")

    for defect in defects:
        print("DEFECT:", defect)
    sys.exit(1 if defects else 0)


if __name__ == "__main__":
    main()
