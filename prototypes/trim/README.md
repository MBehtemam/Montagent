# PROTOTYPE #760 — throwaway, never to merge

The rendered prototype [ADR-0160](../../docs/adr/0160-a-stroke-draws-a-window-of-its-outline-measured-in-fractions-of-its-length.md)
§8 puts in front of any build of trim ([#760](https://github.com/MBehtemam/Montagent/issues/760)).
Cut from the stroke prototype's branch, `prototype-joins-caps-dashes-750` at `fd573eed`, because
trim needs ADR-0158's caps and dashes; that tip is the baseline for every untrimmed comparison.
Evidence, not a starting point: the build carries none of this code. `out/` is force-added so the
clip, stills and texts are on the branch.

- `make.py` writes every project:
  - `trim.montagent.json` → `out/trim.mp4`: seven scenes, 23 s, 1920×1080, 30 fps. A grey
    1-px rect marks each declared box, a blue one a path's inset box, a red dot the outline's
    start point. Six elements carry a `shadow`, so the #652 bounds hint has layers to bound.
    1. draw-on: `trim_end` keyed 0 → 1 under a round cap, on a cubic wave (linear) and a
       zigzag (ease-in-out); the first frame draws nothing;
    2. ring loaders: window `[0, 0.25]`, `trim_offset` keyed 0 → 3 turns, linear, on an
       `ellipse`, a `rect` (start = top-left corner) and a `rect` with `radius` 90;
    3. a closed octagon under a square cap, stroke 30, inset 22, its 45° edges ending on the
       inset box (window 0.2–0.45 orbiting two turns), and a closed diamond drawing on, then off;
    4. keyed `trim_start` 0 → 0.7 → 0.2 and `trim_end` 1 → 0.3 → 0.9 under a round cap: they
       cross at ~1.04 s and ~1.98 s and nothing is drawn in between;
    5. `[0.34, 1.56, 0.64, 1]` on `trim_end` 0 → 1 and on `trim_start` 0.6 → 0, clamped;
    6. `[60, 30]` drawing on and `[0, 40]` dots under a round cap drawing off, each over a dim
       untrimmed copy, so a dash that moved would show beside its ghost;
    7. the full window `[0, 1]` under offsets: an ellipse (keyed 0 → 2.5), a rect, a dashed
       rounded rect and a square-capped closed path (0.37).
  - `contain/<scene>.json`: each scene's stroked elements alone, on black, untransformed.
  - `full-trimmed.json` / `full-plain.json`: the full-window elements, plus a dashed closed path
    with an offset keyed −1.25 → 3.5, against the same elements with every trim field dropped.
  - `probe/seam-*.json`: windows `[0.75, 0.25]` straddling the start point, against untrimmed.
  - `probe/anchor-*.json`: the dashed lines frozen at 0.5, against untrimmed.
  - `errors.json`, `schema/*.json`: one element per `validate` text. `query.json`: the
    `query --at` captures. `plain.json`: the clip with every trim field dropped.
- `all.sh` runs everything below with `MONTAGENT_BIN` (this branch) and `BASELINE_BIN`
  (`fd573eed`, built as is):
  - `check.py` → `out/containment.txt` (ink outside the box on every frame; per frame for the
    square-cap scene), `out/empty-frames.txt`, `out/seam.txt`, `out/dash-anchor.txt`;
  - `sweep.sh` → `out/painters-sweep.txt` (the clip) and `out/painters-sweep-full.txt` (the
    full-window project): K×C = 1×1000, 2×5, 3×2, 4×1, 5×4, 6×9, 7×3, 8×3, 9×2, 10×7 and the
    default, hint on and off, raw RGB hashed at the encoder's input;
  - `fullwin.sh` → `out/fullwin.txt`; `regress.sh` → `out/regress.txt`; `texts.sh` →
    `out/validate.txt`, `out/schema.txt`, `out/query.txt`; `stills.sh` → `out/stills/`;
    `cost.sh` → `out/cost.txt`;
  - `out/side-by-side.mp4`: the clip (left) beside `plain.json` (right, every trim field
    dropped), each at half scale, with `ffmpeg hstack`.

What the code does:

- `crates/montagent-render/src/canvas.rs`: `Trim` and `Drawn`; `trim_window`, the one window
  rule the painter and `query` share (clamp, empty, full, the offset wrapped in turns);
  `stroke_window`, which cuts the window from the outline's first contour with
  `ContourMeasure::getSegment`. A window crossing the start point is one contour: the piece to
  the outline's end, then the piece from its start appended **without a move**, so the stroker
  joins them with the paint's join. A dashed window is dashed with its phase advanced by the
  distance its piece starts at; a dashed crossing window is two pieces. An empty window skips
  the stroke; a full window falls through to the untrimmed drawing. Rects and ellipses are cut
  from #750's explicit outline (`shape_outline`).
- `crates/montagent-core`: the three fields on `path`, `rect` and `ellipse` (`TrimFraction`
  bounds `trim_start`/`trim_end` to [0, 1] in the schema, keyframe values included); the
  resolver clamps both to [0, 1]; `path_reach` counts a cap where `trim_start` or `trim_end` is
  present; `checks/stroke.rs` adds `E-TRIM-EMPTY` and `E-TRIM-OFFSET` and widens
  `E-STROKE-NO-STROKE` and `E-STROKE-CAP-UNDRAWN`; `query --at` gains a `trim` block, raw
  `trim_start`/`trim_end` rows and a `drawn:` cell.
- Probes, env-gated: #750's `MONTAGENT_PROTO_HASHES`, `MONTAGENT_PROTO_DUMP`,
  `MONTAGENT_PROTO_UNBOUND`, `MONTAGENT_PROTO_COUNT`, and `MONTAGENT_PROTO_SPLIT_SEAM` (the
  seam probe's negative control: the second piece starts with a move).

Left out: `shift`, motion blur over a keyed trim, `format.md`, the committed schema, tests.
