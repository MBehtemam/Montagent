"""Simulate K painters over C-frame chunks feeding one in-order encoder, #650.

    python3 sim650.py <results/paint-per-frame-*.tsv, or stderr with P650 lines> [scale]

Paint cost per frame comes from the P650 lines (ms, one painter, observed under load),
multiplied by `scale`. The encoder takes ENC seconds per frame, in order, and starts frame f
once f is painted and f-1 is encoded. W caps frames painted (or being painted) but not yet
encoded, i.e. the RGBA buffered; 1080p RGBA is 8.29 MB a frame.

Policies:
  inorder  chunks handed out in timeline order; a chunk may start only if its last frame is
           within W of the encoder's next frame.
  cost     the in-order front is kept LEAD frames ahead of the encoder (and is always
           admissible there); otherwise, while the buffer has room for a whole chunk, the most
           expensive unclaimed chunk is taken first.
"""
import heapq
import sys

ENC = 30.8 / 1080
LEAD = 32  # frames
MB = 1920 * 1080 * 4 / 1e6


def load(path):
    p = {}
    for line in open(path, errors="replace"):
        f = line.rstrip("\n").split("\t")
        if f[0] == "P650":
            f = f[1:]
        if len(f) == 2 and f[0].isdigit():
            p[int(f[0])] = float(f[1]) / 1000
    return [p[i] for i in range(len(p))]


def sim(cost, K, C, W, policy):
    n = len(cost)
    chunks = [(a, min(a + C, n)) for a in range(0, n, C)]
    ccost = [sum(cost[a:b]) for a, b in chunks]
    unclaimed = set(range(len(chunks)))
    by_cost = sorted(unclaimed, key=lambda i: -ccost[i])
    painted_at = [None] * n
    enc_next, enc_free = 0, 0.0  # next frame to encode; time the encoder is free
    held = 0  # frames claimed (painting or painted) and not yet encoded
    peak = 0
    t = 0.0
    idle = list(range(K))
    events = []  # (time, painter, chunk)

    def encode_until(now):
        nonlocal enc_next, enc_free, held
        while enc_next < n and painted_at[enc_next] is not None and painted_at[enc_next] <= now:
            start = max(enc_free, painted_at[enc_next])
            if start > now:
                break
            enc_free = start + ENC
            enc_next += 1
            held -= 1

    def pick():
        if not unclaimed:
            return None
        first = min(unclaimed)
        a, b = chunks[first]
        if policy == "inorder":
            return first if b - enc_next <= W else None
        # just in time: keep the in-order front LEAD frames ahead of the encoder
        if a - enc_next < LEAD or held + (b - a) > W:
            return first if (a - enc_next < LEAD or held + (b - a) <= W) else None
        for i in by_cost:
            if i in unclaimed:
                a2, b2 = chunks[i]
                return i if held + (b2 - a2) <= W else first
        return first

    while True:
        encode_until(t)
        while idle:
            i = pick()
            if i is None:
                break
            unclaimed.discard(i)
            a, b = chunks[i]
            held += b - a
            peak = max(peak, held)
            p = idle.pop()
            heapq.heappush(events, (t + ccost[i], p, i))
        if enc_next >= n:
            return enc_free, peak
        # next event: a painter finishing, or the encoder freeing up
        cand = []
        if events:
            cand.append(events[0][0])
        if enc_next < n and painted_at[enc_next] is not None:
            cand.append(max(enc_free, painted_at[enc_next]) + ENC)
        t2 = min(cand)
        while events and events[0][0] <= t2:
            te, p, i = heapq.heappop(events)
            a, b = chunks[i]
            for f in range(a, b):
                painted_at[f] = te
            idle.append(p)
        t = max(t2, t)
        encode_until(t)


if __name__ == "__main__":
    cost = load(sys.argv[1])
    scale = float(sys.argv[2]) if len(sys.argv) > 2 else 1.0
    cost = [c * scale for c in cost]
    tot = sum(cost)
    title = sum(cost[919:1068])
    print(f"frames {len(cost)}  paint {tot:.1f} CPU-s  title 919-1067 {title:.1f} s ({title / tot:.0%})")
    print(f"floors: encoder {ENC * len(cost):.1f} s; paint/K at K=6 {tot / 6:.1f} s")
    for K in (4, 6, 8):
        for C in (2, 8):
            for W in (16, 32, 64, 150, 300, 700):
                r = {"inorder": sim(cost, K, C, W, "inorder")}
                best = None
                for lead in (16, 32, 64, 128):
                    globals()["LEAD"] = lead
                    x = sim(cost, K, C, W, "cost")
                    if best is None or x[0] < best[0]:
                        best = x
                r["cost"] = best
                print(
                    f"K={K} C={C} W={W:4d}  inorder {r['inorder'][0]:5.1f} s peak {r['inorder'][1] * MB / 1000:4.2f} GB"
                    f" | cost {r['cost'][0]:5.1f} s peak {r['cost'][1] * MB / 1000:4.2f} GB"
                )
