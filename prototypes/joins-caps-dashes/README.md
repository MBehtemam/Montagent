# PROTOTYPE #750 — throwaway, never to merge

The rendered prototype [ADR-0158](../../docs/adr/0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md)
§8 puts in front of any build of joins, caps and dashes
([#750](https://github.com/MBehtemam/Montagent/issues/750)). Cut from `main`. Evidence, not a
starting point: the build carries none of this code.

- `make.py` writes every project:
  - `strokes.montagent.json` → `out/strokes.mp4`: seven scenes, 22 s, 1920×1080, 30 fps. A grey
    1-px rect marks each declared box and a blue one the inset box `[m, w−m] × [m, h−m]`. Five
    elements carry a `shadow`, so the #652 bounds hint has layers to bound.
    1. miter joins at `stroke_miter_limit` 1, 4, 10 on one zigzag (stroke 16);
    2. butt, round and square caps on a 45° open line (stroke 40), and a level square-capped line;
    3. a keyed-`points` miter path (limit 10) whose corner closes from 41° to 0° and back
       between two 41° keys, 4 s;
    4. `stroke_dash` `[60, 20, 10, 20]` on a rect, a rounded rect and an ellipse, start marked;
    5. the seam at `points[0]` on two closed squares and a closed cubic circle, `[50, 30]`;
    6. zero-length dashes: round caps on a line and a cubic, square caps on a diagonal;
    7. marching ants: `stroke_dash_offset` keyed 0 → −240 and 0 → +240 (pattern total 40).
  - `contain/<scene>.json`: each scene's stroked elements alone, on black, untransformed.
  - `errors.json` and `schema/*.json`: one element per new `validate` text.
  - `query.json`: dashed shapes and keyed offsets for `query --at`; `loop.json`: the ants held
    past their last key; `plain.json`: the clip with every new field dropped (for `cost.sh`).
- `check.py` → `out/containment.txt` (ink outside the box, every frame) and `out/seam-join.txt`.
- `sweep.sh` → `out/painters-sweep.txt`: K×C = 1×1000, 2×5, 3×2, 4×1, 5×4, 6×9, 7×3, 8×3, 9×2,
  10×7 and the default, hint on and off, raw RGB hashed at the encoder's input.
- `regress.sh` → `out/regress.txt`: committed fixtures with `rect`, `ellipse` or `path`, main vs
  this branch, frame by frame.
- `loop.sh` → `out/loop.txt`; `stills.sh` → `out/stills/`; `cost.sh` → `out/cost.txt`;
  `gap0.sh` → `out/gap0.txt` (a `[10, 0]` pattern against no pattern).
- `texts.sh` → `out/validate.txt`, `out/schema.txt`, `out/query.txt`.
- `out/side-by-side.mp4`: the clip (left) beside `plain.json` (right, every new field dropped,
  so today's round join, butt cap and no dash), each at half scale. Made with `ffmpeg hstack`.

What the code does:

- `crates/montagent-render/src/canvas.rs`: `StrokeStyle` (join, cap, dash). `path_styled` sets
  Skia's join, miter limit and cap and a `PathEffect::dash` whose phase is the resolved offset
  wrapped into `[0, total)`. `shape_styled`, when dashed, strokes an outline built by
  `shape_outline` with §5's start and direction (lines and conics, clockwise on screen) instead
  of `draw_rect`/`draw_round_rect`/`draw_oval`; undashed shapes draw exactly as before.
  `outline_length` sums Skia's `ContourMeasure` over the same outline.
- `crates/montagent-core`: the five fields in the model (key order guessed, see the issue);
  `animatable::path_reach` (`m = ceil(k × w / 2)`, the square-cap case as the least `m` with
  `2m² ≥ w²`), which also feeds the resolver's clamp; `checks/stroke.rs` and the registry texts;
  `E-PATH-OUTSIDE-BOX` names `k`, `w` and its source; `query --at` gains a `stroke` block.
- Probes, env-gated: `MONTAGENT_PROTO_HASHES`, `MONTAGENT_PROTO_DUMP`, `MONTAGENT_PROTO_UNBOUND`,
  `MONTAGENT_PROTO_COUNT`.

Left out: `shift`'s refusal of a non-whole resolved offset, `format.md`, the committed schema,
tests.
