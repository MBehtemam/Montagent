# `long-state` — a still state long enough to look inside

This project exists because **the committed `en-halloween-decorating` fixture gains no infill tile
at any ceiling**: its 18 visual states fill the 18 slots the target rung admits, so
`frame --infill-ceiling` there only exercises the null path.
[ADR-0106](../../docs/adr/0106-the-sheets-opt-ins-are-keyframes-and-infill-ceiling-and-a-keyframe-tile-is-sampled-where-its-change-first-paints.md)
names that as an honest cost, and [#492](https://github.com/MBehtemam/Montagent/issues/492)
commits a document where infill has room.

At 25 fps the grid paints every 40 ms. Over a whole-span `bg`:

| element | track | range | keyframes |
| --- | --- | --- | --- |
| `bg` | `bg` | 0..9000 | none |
| `a` | `a` | 0..1000 | none |
| `b` | `b` | 1000..9000 | `x` at 1000 and 4010 |

The visual states are `0..1000 {bg, a}` and `1000..9000 {bg, b}`, with run tiles at 0 and 1000 ms.
Two tiles leave 16 slots at the target rung. `b.x@4010` is interior to the second state and first
paints at 4040 ms.

| call | infill tiles | disclosed |
| --- | --- | --- |
| `--infill-ceiling 2000` | 3000, 5000, 7000 | `requested 2000 ms, achieved 2000 ms` |
| `--infill-ceiling 200` | 16, every 520 ms from each tile | `requested 200 ms, achieved 520 ms`; 40 tiles `infill-evicted`, 4 and 36 by state |
| `--infill-ceiling 3000 --keyframes` | 4000, 7040 around the keyframe tile at 4040 | `achieved 3000 ms` |
| `--infill-ceiling 3040` | 4040 and 7080; the first shows `b.x@4010` | census `1 tiled, 0 untiled` |
| `--infill-ceiling 60000` | none | `achieved 60000 ms`, the sheet drawn without the flag |

Each infill tile is the latest painted frame no more than the ceiling after the tile before it,
and the last span runs to 9000 ms
([ADR-0130](../../docs/adr/0130-infill-fills-the-rung-not-the-grid-at-the-latest-frame-within-the-ceiling.md)).

It uses no media, and `validate` finds nothing in it.
