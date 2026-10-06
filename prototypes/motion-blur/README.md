# PROTOTYPE #718: motion blur (ADR-0155)

This is throwaway code, cut from `main` at `48c593e4`. It is evidence and not a starting
point, so the build ([#719](https://github.com/MBehtemam/Montagent/issues/719)) carries none of it.

## What changed in the engine (prototype only)

- `model`: `motion_blur: {shutter 1..360, samples 2..32}` on the six visual element types.
- `verbs/frame/motion_blur.rs` holds the sample instants, which are exact rationals with
  their gcd reduced:
  `t_k = (instant·720·fps·N + shutter·1000·(2k+1−N)) / (720·fps·N)` ms. It also holds the
  still test, which compares every declared animatable property, the transform and every
  `units` pose across the N instants.
- `verbs/frame/mod.rs` widens the painter's time. A new field `t: (i128, i128)` is what
  every keyed value resolves at, through `resolve::at_instant` and
  `animatable::read(…, n, d)`. On today's path `t` is `(instant, 1)`, so the
  whole-millisecond path is unchanged. Presence, transitions, highlight windows and a video's
  source frame stay on the integer `instant`. A moving element is painted N times on a
  transparent full-frame surface, with `blend: normal`. Its effects, mask and `opacity` are
  included in each sample. The samples are summed in `u32`, and the mean
  `(sum + ⌊N/2⌋) / N` is composited once with the element's `blend`. The video frame is
  decoded once per element per frame and held for every sample.
- `montagent-render` `canvas.rs` gains `blank_like`, `clear_transparent`, `accumulate`
  and `composite_mean`. `layer_bound.rs` gains a global `HINT_FIRES` counter.
- Measurement hooks: `MB718_FORCE_SAMPLING=1` sends still elements through the
  accumulation, and `MB718_IDENTITY=1` checks the mean against the first sample's bytes.

## Run

```sh
python3 prototypes/motion-blur/make_projects.py
CARGO_PROFILE_MB718_INHERITS=release cargo build --profile mb718 -p montagent
CARGO_PROFILE_MB718_INHERITS=release cargo test --profile mb718 -p montagent-core --test proto_motion_blur --no-run
MB718_PROJECT=$PWD/prototypes/motion-blur/motion-blur.montagent.json <test binary> --ignored --nocapture --exact motion_blur_sweep
sh prototypes/motion-blur/run_checks.sh <test binary>
sh prototypes/motion-blur/stills.sh <montagent binary>
python3 prototypes/motion-blur/cost.py <montagent binary> 3
```

## Outputs

| File | What it is |
| --- | --- |
| `out/motion-blur.mp4` | The clip: 15 s at 1920×1080, 30 fps, in five scenes |
| `out/no-blur.mp4` | The same document with every `motion_blur` removed |
| `out/side-by-side.mp4` | Sharp on the left, blurred on the right, at half size |
| `out/contact-sheet.png` | Six blurred stills |
| `out/stills/` | `blur-<ms>.png` and `sharp-<ms>.png` at full scale |
| `out/checks/sweep.out` | The byte-identity sweep |
| `out/checks/still-*.jsonl`, `identity.txt` | The still checks and the N-identical-samples checks |
| `out/checks/cost-run*.jsonl` | Cost runs, each with the 1-minute load average before it |
