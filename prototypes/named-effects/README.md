# PROTOTYPE #722 — throwaway, never to merge

The measuring prototype ADR-0156 §6 puts in front of the `posterize`, `glow` and
`directional_blur` build ([#724](https://github.com/MBehtemam/Montagent/issues/724)) and the
`grain` build ([#725](https://github.com/MBehtemam/Montagent/issues/725)). Cut from `main`.

- `make.py` writes every project:
  - `effects.montagent.json` → `out/effects.mp4`: six 3-second scenes, 18 s, 1920×1080, 30 fps.
    1. `grain` on video, `mono` true/false at `size` 1 and 3.
    2. The texture idiom: a grey `rect` with `grain`, blended `overlay` over footage (left half).
    3. `grain` on a flat, a rotating and a scaled `rect`: the cells ride the transform.
    4. `glow` on white and amber text: none, `threshold: 1`, `threshold: 0.5`.
    5. `posterize` 4 and 256 on video carrying `blur`, and on a drifting gradient rect.
    6. `directional_blur` at 0, 45 and 90°, `length` 40 (top) and 0 (bottom).
  - `identity/`: an identity value written against the member removed (and two controls).
  - `shift/`: the same grain elements starting 0, 3 and 30 frames later.
  - `reach/`: one white rect alone, with and without `glow` / `directional_blur`.
  - `cost/`: one full-frame 1080p gradient rect per member, 30 frames.
  - `colour-bound.json`: `tint`, `saturation`, `brightness`, `contrast` beside a `blur`, drifting
    and rotating, for the bound criterion.
  - `dblur-bound.json`: `directional_blur` drifting, rotating, scaled, flipped, at 135° with
    length 120, and between a `blur` and a `shadow`.
- `sweep.sh <project> <dir>`: renders at K×C = 1×1000, 2×5, 3×2, 4×1, 5×4, 6×9, 7×3, 8×3, 9×2,
  10×7 and the default, each with the #652 bounds hint on and off, hashing the raw RGB at the
  encoder's input. Results: `out/painters-sweep.txt` (the clip), `out/dblur-sweep.txt`,
  `out/colour-sweep.txt`; one hash per frame per run in `out/*painters/*.raw`.
- `check.py`: `out/identity.txt`, `out/shift.txt`, `out/reach.txt` (with the crop) and
  `out/reach-no-crop.txt` (before the crop existed).
- `cost.py` → `out/cost.txt`. `probe.sh` → `out/probe.txt` (Skia's view of each filter, the
  hint each layer got, SkSL's integer support, compiles per painter thread).
- `diag_posterize.py` → `out/posterize-identity.txt`; `diag_crop.py` →
  `out/dblur-crop-vs-no-crop.txt`.
- `stills.sh` → `out/stills/`, one full-scale PNG per scene.
- `diag_rotating.py` → `out/transforms-sweep.txt`: each member under rotation, non-uniform
  scale + rotation and a flip, hint on/off at one and four painters.
- `diag_colour.py` → `out/colour-bound-diag.txt`: each colour filter (and posterize) after a
  blur, let into the bound, flat and rotating.
- `out/own-hint/`: the same sweeps from the first form, where posterize's and the directional
  blur's own layers were hinted too. Under rotation (colour filters) and a flip (directional
  blur) that changed 1–2 levels on a few pixels, so the final form leaves those two layers
  unhinted and passes the bound through them.

What the code does (`crates/montagent-render/src/canvas/named_effects.rs` holds the four):

- `posterize`: one runtime colour filter, unpremultiply, `floor(v × (L−1) + 0.5) / (L−1)`,
  premultiply. `levels` rounds half away from zero, clamped to 2–256.
- `glow`: `source Plus gain(blur(bright_pass(source)))`, an image-filter graph in the element's
  own layer. Bright-pass and gain are runtime colour filters; the blur is `blur`'s own.
- `directional_blur`: one runtime shader image filter, `ceil(length) + 1` samples centred on the
  pixel, equal weights, angle in element space; then **cropped** to its input grown by the
  declared reach + 1 px (Skia cannot bound a runtime shader filter itself).
- `grain`: a plain layer like `mask`'s. After the element is drawn into it, a cell-resolution
  noise image is made on the CPU from a SplitMix64 hash of (seed, local instant, cell x, cell y,
  channel), drawn over the layer with nearest sampling under the element's matrix, and applied
  through a runtime **blender** (offset on unpremultiplied colour, clamp, alpha untouched).
- Every `RuntimeEffect` is compiled once per painter thread (`thread_local`), never shared.
- `layer_bound.rs`: the new members are let into the #652 hint (glow as a blur; directional blur
  by its declared reach; grain and posterize pass the bound through).
  `MONTAGENT_PROTO_COLOUR_BOUND` lets the four ADR-0049 colour filters in too.
- Probes, all env-gated: `MONTAGENT_PROTO_UNBOUND`, `MONTAGENT_PROTO_COUNT`,
  `MONTAGENT_PROTO_HASHES`, `MONTAGENT_PROTO_DUMP`, `MONTAGENT_PROTO_COST`,
  `MONTAGENT_PROTO_COMPILE`, `MONTAGENT_PROTO_PROBE_TB`, `MONTAGENT_PROTO_PROBE_INT`,
  `MONTAGENT_PROTO_DBLUR_NO_CROP`, `MONTAGENT_PROTO_SKIA_DBLUR_BOUNDS`.

Left out: every `validate` check (ranges, required `seed`, static parameters), the
`R-GRAIN-SEED-SHARED` review, `query --at`, keyframed parameters, the schema publication.
