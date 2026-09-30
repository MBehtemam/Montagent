# `keyframe-cross-boundary` — a keyframe change point with no frame left in its state

This project exists because **the committed `en-halloween-decorating` fixture has zero keyframe
change points interior to a visual state on a visible element**, and #407's `doctored/` project
places every keyframe on the grid.
[ADR-0106](../../docs/adr/0106-the-sheets-opt-ins-are-keyframes-and-infill-ceiling-and-a-keyframe-tile-is-sampled-where-its-change-first-paints.md)
made a constructed instance of its decision 12 a shipping condition, covering both branches, and
[#491](https://github.com/MBehtemam/Montagent/issues/491) commits it.

At 25 fps the grid paints every 40 ms. Over a whole-span `bg`:

| element | track | range | keyframes |
| --- | --- | --- | --- |
| `bg` | `bg` | 0..2000 | none |
| `a` | `a` | 0..1030 | `x` at 0 and 1020 |
| `p` | `p` | 500..2000 | `y` at 500 and 1015 |
| `c` | `c` | 1030..2000 | none |

The visual states are `0..500 {bg, a}`, `500..1030 {bg, a, p}` and `1030..2000 {bg, p, c}`.
Their run tiles are at 0, 520 and 1040 ms. Frames paint at 1000 and 1040 ms, so no painted
frame is left in `500..1030` after 1000 ms.

Both interior change points sample at 1040 ms, which is the next state's run tile:

| change point | on screen at 1040? | over `[0, 2000)` | over `[0, 1030)` |
| --- | --- | --- | --- |
| `p.y@1015` | yes | `tiled` by the run tile at 1040 | `untiled`, `no-grid-frame`: the next state is outside the range |
| `a.x@1020` | no, `a` ends at 1030 | `untiled`, `no-grid-frame` | `untiled`, `no-grid-frame` |

Neither ever adds a tile, with or without `--keyframes`, and neither is a finding: an untiled
change point is disclosure only
([ADR-0129](../../docs/adr/0129-an-untiled-keyframe-point-is-named-in-the-census-and-never-in-skipped.md)).

`validate` reports one `R-KEYFRAME-UNREACHED` at `review` on `a`: no sampled frame inside
`0..1030` reaches the 500 its keyframe at 1020 declares. That is the same fact seen from the
document, and it is `validate`'s finding, not the sheet's.

It uses no media.
