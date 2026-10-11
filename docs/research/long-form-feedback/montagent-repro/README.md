# Montagent render-time reproduction kit (from the Danish story-book project)
A 3-second slice of a real 1080p project (1920x1080, 30 fps) with only the 29 elements that are active in that slice, the 4 assets and 3 fonts they use (15 MB). No audio, no text from third parties: story text is Andersen's *Den grimme Ælling* (public domain), pictures are our own.

    cd montagent-repro && python3 profile.py          # about 3 minutes; needs montagent 0.2.x and Python 3

It renders `--from 1500000 --to 1503000` once per variant (each to its own output file), prints seconds per frame, then bakes the two static background images into one picture and times that.

Expected on an Apple M1 Pro (10 cores), montagent 0.2.0, output of a first reference run (your numbers will differ; the ratios are the point):

| variant | s/frame | saves |
|---|---|---|
| everything | 0.138 | |
| drop table plate | 0.102 | 26% |
| drop book plate (feathered mask) | 0.062 | 55% |
| drop table + book plate | 0.035 | 75% |
| drop picture loop only | 0.124 | 10% |
| drop stickers only | 0.134 | 3% |
| drop picture loop + stickers | 0.033 | 76% |
| strip every effect | 0.088 | 36% |
| baked background (1 image) | about 0.03 | about 78% (5x faster) |

What to look at: (1) the two static images cost three quarters of the time; (2) removing both videos saves 76% though each alone saves under 10%; (3) effects (masks, shadows) cost a third.
Files: `project.json` (the slice), `assets/`, `fonts/`, `profile.py`. Details and requests: `../montagent-feedback.md`.

## Running it from this repo (added when the kit was imported, #914)
- Needs the repo's `montagent` on PATH and **ffmpeg 7.1+ with libx264** (a 6.x ffmpeg fails the tool qualification, E-TOOL-UNSUPPORTED).
- `assets/s4-loop-baked.mp4` (12 MB) is not committed. Put the reporter's clip there, or make a stand-in of the same size (any small H.264 loop works for the video tests; the cause is the presence of a `video` element, not its content):
  `ffmpeg -f lavfi -i testsrc2=size=1280x720:rate=30 -t 12 -c:v libx264 -pix_fmt yuv420p assets/s4-loop-baked.mp4`
- Font paths in `project.json` were made relative; the two STIX fonts were not supplied, so they point at `CrimsonText-Regular.ttf` (only changes glyph shapes).
- The `montagent` painter count depends on cores: `available_parallelism - 4`, and one painter when the document has any `video` element (ADR-0144 §7). On a 4-core box it is one painter either way; force it with `MONTAGENT_PAINTING=<painters>,<chunk>` (e.g. `3,2`).
