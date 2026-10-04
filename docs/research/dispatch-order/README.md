# Dispatch order and the reorder window, #650

Evidence for [How far may painting lead the encoder on a lumpy timeline?](https://github.com/MBehtemam/Montagent/issues/650). Throwaway research branch. The spike patch is never merged.

## What was run

- **Binary:** `research/649-filter-bound` (`7ee09de1`), plus [`spike.diff`](spike.diff):
  - the stills fix from #648 (`to_raster_image`);
  - one `P650 <frame> <ms>` line per frame, the time `produce` spends painting.

  It was built in release with a private `CARGO_TARGET_DIR`.
- **Renders:** two trailer renders on the dev's M1 Pro. `MONTAGENT_FILTER_BOUND=keep:pic` is the shipped bound; `none` is unbounded. Both match all 1,080 pinned frame hashes.
- **Load:** 23–35 throughout, shared with other sessions. The absolute times are inflated and are **not** timings under the map's rules. Only the per-frame *shape* is used.

| run | paint CPU-s | title 919–1067 | wall |
|---|---|---|---|
| `none` (stills fixed) | 228.4 | 127.8 s, 56% | 234 s |
| `keep:pic` (stills fixed + bound) | 145.2 | 62.9 s, 43% | 168 s |

The bound cuts the title section by about half rather than ⅔. Its elements sit low and right on the frame, where the bound saves least.

## Simulation

[`sim650.py`](sim650.py) is a discrete-event model of the ladder's step 3:

- K painters over C-frame chunks;
- one in-order encoder at 30.8 s / 1,080 frames (the profile's encoder-only time);
- W = frames claimed or painted and not yet encoded, at 8.29 MB per 1080p RGBA frame.

It compares two policies:
- **in-order:** the ladder as written;
- **cost-ordered:** the in-order front is kept a lead ahead of the encoder, the most expensive chunks go first within W, and the lead is swept over 16–128 frames with the best kept.

It ignores core contention, so its seconds are a **lower bound**. `×1.3` scales paint to the profile's ~298 CPU-s. The full output is in [`results/sim.txt`](results/sim.txt).

With the bound, at K=6, C=2:

| W | memory | in-order | in-order ×1.3 | cost-ordered (×1.0 / ×1.3) |
|---|---|---|---|---|
| 32 | 0.27 GB | 37.3 s | 41.8 s | — |
| 64 | 0.53 GB | 35.8 s | 39.4 s | 33.6 s |
| 150 | 1.24 GB | 33.4 s | 36.5 s | 30.9 / 32.3 s |
| 300 | 2.5 GB | 30.9 s | 32.3 s | 30.9 s |

- **Without the bound** (×1.3): in-order W=150 gives 54.3 s; cost-ordered gives 50.9 s.
- **Below the floor:** with W < K·C (for example C=8, W=16), in-order collapses to 90 s, because the window starves painters.

```
python3 sim650.py results/paint-per-frame-keep_pic.tsv 1.3
```
