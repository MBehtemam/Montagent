# proxy-preview-savings — measuring proxy-resolution preview at 4K/8K

Prototype for [#87](https://github.com/MBehtemam/Montagent/issues/87), graduated from
[ADR-0021](../../../adr/0021-preview-budget-and-graceful-degradation.md), part of the map
[#2](https://github.com/MBehtemam/Montagent/issues/2). **Throwaway. The answer is in
[FINDINGS.md](FINDINGS.md).**

Reuses [#34](https://github.com/MBehtemam/Montagent/issues/34)'s compiled
`rast-bench` binary (`../rust-rasterizer/`) by relative path — no fork of the Rust
crate, no source changes. The binary's existing `--scale=k` flag already decouples
destination size (what `k` scales) from source-decode size (what the scene's own
native resolution and the actual asset resolution fix), which is exactly the two
variables this ticket needs to pull apart.

| file | |
| --- | --- |
| `scene-4k.json`, `scene-8k.json` | new scenes at true 2160×3840 / 4320×7680 canvases |
| `gen_scenes.py` | one-off generator that produced the two scene files above (not run by `run.sh`) |
| `generate-media.sh` | generates the synthetic 4K/8K stills + video into `media/` (not committed — see FINDINGS.md) |
| `run.sh` | every number in FINDINGS.md |
| `frames/` | correctness-check frames (skia-safe vs tiny-skia, committed) |

Needs `ffmpeg`/`ffprobe` on `PATH` and a built `../rust-rasterizer/target/release/rast-bench`
(`run.sh` builds it if missing).

```
./run.sh
```
