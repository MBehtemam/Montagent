# PROTOTYPE #764 — throwaway, never to merge

The rendered prototype [ADR-0161](../../docs/adr/0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md)
§9 puts in front of any build of text on a path
([#764](https://github.com/MBehtemam/Montagent/issues/764)). Cut from `main`. Evidence, not a
starting point: the build carries none of this code.

Fonts are the vendored Oswald SemiBold (`fixtures/benchmark/spy-trailer/fonts/`) with Noto Naskh
Arabic (`fixtures/letter-spacing/fonts/`) behind it. Oswald shapes an `fi` and an `ffl` ligature.

- `make.py` writes every project:
  - `textpath.montagent.json` → `out/textpath.mp4`: six scenes, 24 s, 1920×1080, 30 fps, all
    thirteen cases. A grey 1-px rect marks each text's declared box, a blue one the inset box
    `[m, w−m] × [m, h−m]`, and a grey 2-px `path` element draws the guide with the same points
    (ADR-0161 §2's way to show a guide). Three texts carry a `shadow`, so the #652 bounds hint
    has layers to bound: the circle badge, the letter stagger, and the tight `fi`.
    1. (0–4 s) case 1 arc `align: center, path_offset: 0.5`; case 2 wave sliding on from the
       start end (0–2 s, `align: end`, offset 0 → 1) then off past the far end (2–4 s, a second
       element, `align: start`, offset 0 → 1); case 3 closed circle badge, offset 0 → 1.
    2. (4–8 s) case 4 Arabic on an arc with the defaults; case 11 Arabic with Latin digits;
       case 12 runs of sizes 110, 44, 76 on an element of size 44 (`m` = 110).
    3. (8–12 s) case 5 `by: letter` stagger, `y` −200 → 0; case 7 text stroke 6 on a wave.
    4. (12–16 s) case 6 waving banner (keyed `points`, same shape); case 10 keyed `points`
       that grow and shrink the curve (320 → 892 → 320 px) under a 27-letter line.
    5. (16–20 s) case 8 a semicircle of radius 45 (line height 120) carrying `fi` and the
       joined piece `بسم`; case 9 a line far longer than an open arc and than a closed circle.
    6. (20–24 s) case 13 `align: end` at `path_offset` 0.97; a letter stagger (`y`, `rotation`,
       `opacity`) under `path_offset` 0 → 0.55.
  - `contain/<id>.json`: each text alone, on black, untransformed, its box at (300, 300).
  - `plain.json`: the clip with every `path` and `path_offset` dropped (cost, regress).
  - `errors/*.json`: one minimal bad file per `validate` text; `shift/*.json`: a keyed text;
    `reverse/*.json`: the arc and the circle reversed by hand.
- `sweep.sh` → `out/painters-sweep.txt`: K×C = 1×1000, 2×5, 3×2, 4×1, 5×4, 6×9, 7×3, 8×3, 9×2,
  10×7 and the default, hint on and off, raw RGB hashed at the encoder's input, and the
  painter's own hidden-letter log per run.
- `hidden.py` → `out/hidden.txt`: `query --at` at all 720 frame instants against each of the 22
  painter logs.
- `check.py` → `out/containment.txt`; `lift.sh` → `out/lift.txt`; `regress.sh` →
  `out/regress.txt`; `cost.sh` → `out/cost.txt`; `texts.sh` → `out/validate.txt`,
  `out/query.txt`, `out/shift.txt`; `reverse.sh` → `out/reverse.txt` and
  `out/stills/reverse.png`; `stills.sh` → `out/stills/`. `out/tests.txt`: the `units`,
  `letter_spacing`, `painters`, `path` and `filter_bound` suites, all passing.

What the code does:

- `crates/montagent-core/src/text_path.rs`: `bend` lays nothing out again. It takes the flat
  placement the painter already made, groups its glyphs into rigid bodies with
  `montagent_text::units::bodies(text, By::Letter, …)` (letters, joined pieces, ligature
  clusters), and gives each body one matrix `T(P(d)) · R(θ) · T(−x′, −y₀) · S`: `S` the
  stagger's flat pose, `x′` the posed advance midpoint, `y₀` the line's rest baseline,
  `d = path_offset × L + (x′ − x_anchor)`. A hidden body's glyphs are not handed to the canvas.
  `query --at` calls the same `bend`, so its `hidden:` line and the frame come from one function.
- `crates/montagent-render/src/canvas.rs`: `Curve`, Skia's `ContourMeasure` over the outline a
  `path` element strokes (`pos_tan` per body). The canvas's text drawing is unchanged: the
  matrix rides the existing `UnitDraw`.
- `frame/mod.rs`: a text with `path` draws through `bend` and pivots about its declared box.
- Model: `path: {closed, points}` and `path_offset` (a `Fraction`, range-checked) on `text`; the
  derived animatable list gains `path.points`, so the resolver, its overshoot clamp (to §7's
  `m`) and `shift` treat it as a `path` element's `points`.
- `validate`: the four `E-PATH-*` codes fire on a text's `path` (each naming the type);
  `E-TEXT-PATH-BREAK`, `E-TEXT-PATH-OFFSET-ORPHAN`; `R-BOX-SLACK` skips a text on a path.
- Probes, env-gated: `MONTAGENT_PROTO_HASHES`, `MONTAGENT_PROTO_DUMP`, `MONTAGENT_PROTO_UNBOUND`,
  `MONTAGENT_PROTO_COUNT` (from #750), `MONTAGENT_PROTO_HIDDEN`, `MONTAGENT_PROTO_LIFT`.

Left out: `measure`'s bent ink extent and `hidden:` line, `query --at`'s ink box on a path
(it still measures the flat line), any test of `motion_blur` on a bent text (`bend` reads the sample instant, but no case carries one), `format.md`, the
committed schema, the capability map, and tests.
