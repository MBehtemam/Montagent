# rust-rasterizer — measuring skia-safe against tiny-skia

Throwaway prototype for [#34](https://github.com/MBehtemam/Montaget/issues/34).
**The answer is in [FINDINGS.md](FINDINGS.md).**

Nothing in here is a proposed design. One Rust binary with two rasterizer arms behind a
`Backend` trait, driven by a port of #6's shared ops list so the only thing that varies is
the rasterizer.

| file | |
| --- | --- |
| `src/ops.rs` | port of #6's `ops.js` — (scene, frame) → a flat ops list |
| `src/text.rs` | parley layout + skrifa outlines, **shared by both arms** |
| `src/media.rs` | FFmpeg subprocess decode and encode, **shared by both arms** |
| `src/skia_arm.rs`, `src/tiny_arm.rs` | the only difference between the arms |
| `scene.json` | #6's scene, verbatim |
| `scene-video.json` | the same, plus one real video clip on the timeline |
| `run.sh` | every number in FINDINGS.md |
| `compare.py`, `seek.sh` | frame diffing and decoder-seek timing |
| `frames/` | the verification frames, committed |

Needs `ffmpeg` on `PATH` and the fixture at `fixtures/en-halloween-decorating/`.
