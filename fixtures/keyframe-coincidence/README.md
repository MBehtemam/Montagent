# `keyframe-coincidence` — keyframe change points that share a frame

This project exists because **the committed `en-halloween-decorating` fixture has zero keyframe
change points interior to a visual state on a visible element**, so `frame --keyframes` adds no
tile to it and its census prints `0 tiled, 0 untiled`. #407's `doctored/` project places every
keyframe on the grid, so neither decision it tests has a committed instance.
[ADR-0106](../../docs/adr/0106-the-sheets-opt-ins-are-keyframes-and-infill-ceiling-and-a-keyframe-tile-is-sampled-where-its-change-first-paints.md)
made one a shipping condition for its decision 11, and
[#491](https://github.com/MBehtemam/Montagent/issues/491) commits it.

At 25 fps the grid paints every 40 ms. Over a whole-span `bg`:

| element | track | range | keyframes |
| --- | --- | --- | --- |
| `bg` | `bg` | 0..2000 | none |
| `a` | `a` | 0..1003 | none |
| `b` | `b` | 1003..2000 | `x` at 1003, 1020, 1410 and 1600 |
| `c` | `c` | 0..2000 | `y` at 0 and 1425 |

The visual states are `0..1003 {bg, a, c}` and `1003..2000 {bg, b, c}`. Their run tiles are at
0 and 1040 ms.

| change point | interior? | sampled at | what the sheet does |
| --- | --- | --- | --- |
| `c.y@0` | no, at a state's start | | not in the population |
| `b.x@1003` | no, at a state's start | | not in the population |
| `b.x@1020` | yes | 1040 | the run tile's own frame: it joins that line and the tile stays a run tile (D11), with or without the flag |
| `b.x@1410` | yes | 1440 | one keyframe tile, shared ... |
| `c.y@1425` | yes | 1440 | ... by two elements' change points (D11) |
| `b.x@1600` | yes | 1600 | a keyframe tile on the grid, offset `+0` |

With `--keyframes` the sheet has four tiles, two of them keyframe tiles, and the census reads
`4 tiled, 0 untiled`. Without it, two tiles and `1 tiled, 3 untiled`, the three as
`not-requested` ([ADR-0129](../../docs/adr/0129-an-untiled-keyframe-point-is-named-in-the-census-and-never-in-skipped.md)).

It uses no media, and `validate` finds nothing in it.
