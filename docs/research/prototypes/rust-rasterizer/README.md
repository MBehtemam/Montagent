# rust-rasterizer — measuring skia-safe against tiny-skia

Built as a throwaway prototype for [#34](https://github.com/MBehtemam/Montagent/issues/34).
**The answer that came out of it is in [FINDINGS.md](FINDINGS.md), which is left verbatim.**

It is no longer throwaway. [ADR-0010](../../../adr/0010-skia-safe-rasterizer-text-beside-it.md)
keeps it in-tree as the **correctness oracle** that makes the `tiny-skia` exit real rather
than aspirational, and as the **golden-frame guard** — *"that guard is not optional"* —
since a `skia-safe` bump can quietly shift resampling and antialiasing with nothing
erroring. [#189](https://github.com/MBehtemam/Montagent/issues/189) wired it in:
`cargo test -p rast-bench` runs it, and CI runs it on all six desktop tier-1 targets.

One Rust binary with two rasterizer arms behind a `Backend` trait, driven by a port of
#6's shared ops list so the only thing that varies is the rasterizer.

| file | |
| --- | --- |
| `src/ops.rs` | port of #6's `ops.js` — (scene, frame) → a flat ops list |
| `src/text.rs` | parley layout + skrifa outlines, **shared by both arms** |
| `src/media.rs` | FFmpeg subprocess decode and encode, **shared by both arms** |
| `src/repo.rs` | where the repository is, so scene assets resolve anywhere |
| `src/skia_arm.rs`, `src/tiny_arm.rs` | the only difference between the arms |
| `scene.json` | #6's scene, verbatim |
| `scene-video.json` | the same, plus one real video clip on the timeline |
| `tests/oracle.rs` | the oracle and the golden-frame guard, as tests |
| `frames/oracle/` | the golden frames the guard compares against |
| `run.sh` | every number in FINDINGS.md |
| `compare.py`, `seek.sh` | frame diffing and decoder-seek timing |
| `frames/*.png` | #34's verification frames, kept as that ticket's evidence |

Needs the fixture at `fixtures/en-halloween-decorating/`. `ffmpeg` on `PATH` is needed for
`run.sh` and for the one oracle test that puts a decoded video frame on the timeline;
the rest of the oracle runs without it.

## What #189 changed, and why the numbers still line up

Three things, none of them in the rasterizer arms:

- **Asset paths are repository-relative.** #34 hardcoded absolute paths, which was fine
  for a throwaway and is not a property something running on six targets can have.
- **The font is the vendored file, not a system family.** #34 shaped against
  `SF Pro Rounded`, which exists on one of ADR-0064's six targets — and which
  [#143](https://github.com/MBehtemam/Montagent/issues/143) established is not in the
  repository and is not redistributable. The shaper now registers
  `fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf` with system fonts switched
  off and shapes everything in it, which is also what ADR-0010 asks of the real renderer:
  open nothing outside the declared font chain.
- **It is a workspace member**, so `skia-safe`'s version and feature set stay pinned in
  exactly one place ([#36](https://github.com/MBehtemam/Montagent/issues/36)). Its own
  `Cargo.lock` is superseded by the workspace lock; the versions #34 measured against are
  in `FINDINGS.md` and in this directory's history.

The font change moves the picture, so `frames/oracle/` was rendered fresh and
`frames/*.png` is left alone as #34's evidence. What did **not** move is the thing
ADR-0010 actually asserts: the arms still agree to **mean Δ 0.054 at `t=0` and 0.070 at
`t=40`**, against the ADR's recorded "antialiasing noise (mean Δ 0.05–0.07 of 255)".

`frames/oracle/` is self-confirming — this harness rendered it, so it can only catch a
regression and can never falsify the format. Only
`fixtures/en-halloween-decorating/reference/frame-*.png`, extracted from the published
video, can do that, and that is `frame`'s ticket rather than this one.
