# `spy-trailer` — the paint-heavy yardstick

**SILENT PROTOCOL** is a 36 s, 1920×1080, 30 fps spy-thriller trailer with **no `video`
element**: 181 elements on 114 tracks (104 `text`, 43 `image`, 17 `rect`, 10 `ellipse`,
7 `audio`). An older binary rendered it at 0.13× realtime
([#509](https://github.com/MBehtemam/Montagent/issues/509)), and it is the target of the
paint-speed map ([#641](https://github.com/MBehtemam/Montagent/issues/641)). It carries no
speed number yet; that is the map's to set.

```sh
cd fixtures/benchmark/spy-trailer
uv run --with numpy==2.5.3 --with scipy==1.18.1 python score.py   # writes score.wav, once
montagent render trailer.montagent.json                           # writes out/silent-protocol.mp4
```

## Where it was cut from

`docs/research/skills/prototypes/spy-trailer/` on the `prototype/spy-trailer` branch at
`0b34e45af9ee24b1b15498a86060f9892b983d00`
([#508](https://github.com/MBehtemam/Montagent/issues/508)). The branch's README tells how
each asset was made. This copy keeps the project file and only the assets it references.
The generators (`scene.py`, `plates.py`, `flux.py`, `vo.py`) and the voice timings stay on
the branch. The copy is 8,654,312 bytes (8.7 MB); the branch directory is 27 MB.

What differs from the branch:

- **The six photos in `img/` are JPEG** (ffmpeg `-q:v 3`, 4:2:0), recompressed from the
  generated PNGs at the same 1920×1088. They are opaque, and a render decodes each one once,
  since stills are cached for the whole render. `trailer.montagent.json` differs from the
  branch's only in those six `source` paths.
- **Every alpha plate in `fx/` is the branch's PNG, byte for byte**, including the
  2400×1400 `grain.png`. The grain plate is itself a suspect, so it keeps its size and its
  lossless alpha.
- **`score.wav` is not committed.** The branch never committed it either. `score.py`
  synthesises it from `plan.py`'s timeline with a fixed seed. With numpy 2.5.3 and scipy
  1.18.1 it writes the same bytes on every run, sha256
  `8093372da566e10290329f8d5dd56b7f6e2cc2ca4883a3e166b05be333d5528b`. A committed WAV would
  have added 6.9 MB, almost doubling the fixture, for an element that does no painting.
  `score.wav` is git-ignored.

## The frame hashes

`frames.framemd5` holds the per-frame hashes of the video stream, decoded from one render
of this directory:

```sh
ffmpeg -i out/silent-protocol.mp4 -map 0:v -f framemd5 frames.framemd5
```

They were taken with `montagent` built in release from main at
`8a04f7ed877b6b23936f129d31fe6ffc075f74f8`, on the dev's M1 Pro, with ffmpeg 9.0.2
(`Lavc63.1.102`) and x264 core 165 r3222 b35605a. The render reported `libx264, preset
medium, CRF 20, 5 threads`. There are 1080 frames. A second render from the same binary
gave the same file.

They hash the encoder's output, decoded, as `crates/montagent/tests/render_target.rs` does,
so they hold only while the encoder settings and the x264 build are the same. A render that
paints different pixels changes them. A change small enough to quantize away might not.
