---
status: accepted
amends: 0151 (`UnitOrder` gains `"center_out"` and `"edges_in"` with the symmetric tie rule, superseding §8's deferral; `Units` and `UnitOverride` gain optional `blur: Animatable<f64>` in `[0, 500]` rendered on the bounded unit body `save_layer` and accounted for in `R-OFF-CANVAS`), 0215 (disposes of the text stagger beyond even sweeps, per-unit animator properties, glyph-to-path conversion, and vertical text rows from the After Effects text census #900 and #1067)
---

# `units` gains `center_out`, `edges_in`, and `blur`, and range selectors, glyph-to-path conversion, and vertical writing mode are refused

[#1067](https://github.com/MBehtemam/Montagent/issues/1067), on the map
[#896](https://github.com/MBehtemam/Montagent/issues/896) (triage
[#908](https://github.com/MBehtemam/Montagent/issues/908)).
The After Effects text census
([#900](https://github.com/MBehtemam/Montagent/issues/900)) and the 3D census
([#902](https://github.com/MBehtemam/Montagent/issues/902)) surfaced five groups of text
capabilities beyond the baseline stagger of
[ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
and the typography controls settled in
[ADR-0223](0223-font-entries-pin-static-variable-axes-and-text-and-runs-gain-closed-opentype-features-baseline-shift-and-run-letter-spacing.md):

1. **Stagger traversal order** beyond `"forward"` and `"reverse"` (`"center_out"`, `"edges_in"`, and `"random"`).
2. **Range-selector envelopes and stacked animators** (AE Range Selector `Shape`: `Square`, `Ramp Up`, `Ramp Down`, `Triangle`, `Round`, `Smooth`; selector `Mode`: `Add`, `Subtract`, `Intersect`, `Min`, `Max`, `Difference`; and multiple `Animator` groups on one text layer).
3. **Additional per-unit animatable properties** in `units` (`blur`, per-unit `swivel` and `tilt`, per-unit `letter_spacing`, per-unit `stroke_width`) and animatable `line_height` on `TextElement`.
4. **Converting text glyph outlines to `path` elements** (AE *Create Shapes from Text* / *Create Masks from Text*).
5. **Vertical text and *Tate-chu-yoko*** (vertical writing direction for CJK/display columns and horizontal-in-vertical digit runs).

Under
[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md),
After Effects is precedent that these rendered results are wanted in the reference class, while
CapCut and Premiere stay first for mechanism and schema shape, and an After Effects mechanism that
conflicts with the format's invariants is reshaped to the existing model, a written recipe in
`skills/montagent-motion/SKILL.md`, or refused on a named invariant.

---

## Decision

### 1. `UnitOrder` gains `"center_out"` and `"edges_in"`; seeded `"random"` stays refused

#### 1.1 `"center_out"` and `"edges_in"` step index arithmetic

[ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
§8 deferred `"center_out"` and `"edges_in"` solely on pre-[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md)
reference-class grounds while noting that *"a tie rule exists (units at an equal distance start
together), so this can enter later by its own ADR."* With After Effects and CapCut/Premiere text
presets providing direct precedent for symmetric centre-split and edge-pinch reveals,
`UnitOrder` (`units.order`) expands from two values to four:

```json
"order": "forward" | "reverse" | "center_out" | "edges_in"
```

Given `count` scheduled units indexed `i ∈ 0..count` in reading order (with `last = count - 1`
when `count >= 1`), let the symmetric edge distance of unit `i` be:

$$d(i) = \min(i,\ \text{last} - i)$$

where $0 \le d(i) \le \lfloor \text{last} / 2 \rfloor$. The integer step multiplier $s(i)$ such
that unit $i$'s derived delay is $\text{delay}(i) = s(i) \times \text{every}$ (and $0$ when
$\text{count} \le 1$) is:

| `order` | Step multiplier $s(i)$ | Maximum step $s_{\max}$ (`count >= 1`) | Behaviour |
|---|---|---|---|
| `"forward"` (default) | $i$ | $\text{last}$ | Unit $0$ starts at $0$; unit $\text{last}$ starts at $\text{last} \times \text{every}$. |
| `"reverse"` | $\text{last} - i$ | $\text{last}$ | Unit $\text{last}$ starts at $0$; unit $0$ starts at $\text{last} \times \text{every}$. |
| `"edges_in"` | $\min(i,\ \text{last} - i)$ | $\lfloor \text{last} / 2 \rfloor$ | Outer pair $(0, \text{last})$ starts together at $0 \times \text{every}$; each symmetric inward pair $(k, \text{last} - k)$ starts together at $k \times \text{every}$, meeting at the middle unit (odd `count`) or middle pair (even `count`). |
| `"center_out"` | $\lfloor \text{last} / 2 \rfloor - \min(i,\ \text{last} - i)$ | $\lfloor \text{last} / 2 \rfloor$ | Middle unit (odd `count`, $i = \text{last}/2$) or middle pair (even `count`, $i \in \{\lfloor \text{last}/2 \rfloor, \lceil \text{last}/2 \rceil\}$) starts at $0 \times \text{every}$; each symmetric outward pair starts together one step of `every` later, reaching the outer pair $(0, \text{last})$ at $\lfloor \text{last} / 2 \rfloor \times \text{every}$. |

Concretely:
- For `count = 5` (`i = 0, 1, 2, 3, 4`, `last = 4`, `last / 2 = 2`):
  - `"edges_in"` steps: `[0, 1, 2, 1, 0]` (delays `0`, `every`, `2 * every`, `every`, `0`).
  - `"center_out"` steps: `[2, 1, 0, 1, 2]` (delays `2 * every`, `every`, `0`, `every`, `2 * every`).
- For `count = 4` (`i = 0, 1, 2, 3`, `last = 3`, `last / 2 = 1`):
  - `"edges_in"` steps: `[0, 1, 1, 0]` (delays `0`, `every`, `every`, `0`).
  - `"center_out"` steps: `[1, 0, 0, 1]` (delays `every`, `0`, `0`, `every`).

**Tie rule and merged/joined bodies.** Under
[ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
§3 and
[ADR-0153](0153-a-joined-piece-moves-as-one-and-joining-scripts-keep-their-ligatures-and-take-no-letter-spacing.md)
§1, letters merged into one ligature glyph or joined into one cursive piece keep their own places
in `count` and move together on the **first** (lowest-index) constituent unit's derived or
overridden delay. That rule is unchanged.

**`frame --keyframes` (`ADR-0151` §7).**
[ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
§7 samples the earliest-starting unit and the latest-starting unit of the stagger so contact sheets
stay bounded at two sets of change points regardless of word/letter count. Under `"forward"` and
`"reverse"`, those extremes sit at $i = 0$ and $i = \text{last}$. Generalized across all four
orders, `frame --keyframes` emits change points for the **first unit in reading order with minimum
step ($s(i) = 0$)** and the **last unit in reading order with maximum step
($s(i) = s_{\max}$)**, labelled with their respective reading-order unit indices `[unit i]` (Plus
any unit carrying an explicit `Run.unit` override). For `"forward"` and `"reverse"`, this reduces
identically to units `0` and `last`.

#### 1.2 Seeded `"random"` order stays refused

[ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
§8's refusal of a seeded `"random"` order stands on its named sight-readability invariant
([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)):
with a PRNG seed, no unit's start time can be read from `project.json` without executing the
generator. When an author wants a non-monotone or bespoke per-unit arrival order (for example,
five letters popping in at `120ms`, `0ms`, `240ms`, `60ms`, `180ms`), splitting the units into runs
with literal `unit.delay` overrides
([ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
§4) states every unit's exact millisecond offset directly in the file.

---

### 2. Range-selector shapes, selector modes, and multiple `units` blocks are refused

After Effects's text animator sweeps a spatial/index selector (`Start`, `End`, `Offset`) shaped by
an envelope (`Square`, `Ramp Up`, `Ramp Down`, `Triangle`, `Round`, `Smooth`) and combines stacked
selectors or `Animator` groups via `Mode` (`Add`, `Subtract`, `Intersect`, `Min`, `Max`,
`Difference`).

These mechanisms are **refused** from the schema:
- In Montagent's time-delayed unit model
  ([ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
  §2), each unit runs the declared keyframe curve in its own local time
  ($t_{\text{local}} = t - \text{delay}(i)$). A spatial range selector is a second, competing model
  of time-to-unit mapping where a single unit's trajectory can only be inferred by inverting a
  moving spatial window through a shape function.
- Every rendered result produced by AE's Range Selector shapes and stacked animators is already
  expressed directly in `units` and `Run.unit`:
  1. **`Ramp Up` / `Ramp Down` (one-way cascade into or out of rest)**: a 2-keyframe curve on
     `units` (e.g. `y: [{"t": 0, "v": 28}, {"t": 280, "v": 0, "ease": "out-cubic"}]`,
     `opacity: [{"t": 0, "v": 0.0}, {"t": 280, "v": 1.0, "ease": "out-cubic"}]`).
  2. **`Triangle` / `Round` / `Smooth` (Mexican-wave pulse travelling across units and returning to rest)**:
     a 3-keyframe return curve on `units` (e.g. `y: [{"t": 0, "v": 0}, {"t": 180, "v": -24, "ease": "out-sine"}, {"t": 360, "v": 0, "ease": "in-out-sine"}]`
     or `scale: [{"t": 0, "v": [1.0, 1.0]}, {"t": 180, "v": [1.25, 1.25], "ease": "out-sine"}, {"t": 360, "v": [1.0, 1.0], "ease": "in-out-sine"}]`)
     delayed by `every` per unit. Linear segments give `Triangle`; sine/cubic-bezier handles give
     `Round` and `Smooth`.
  3. **Multi-property coordination (settle with overshoot, blur-to-crisp fade-in)**: because each
     field on `units` (`x`, `y`, `scale`, `rotation`, `opacity`, `blur`) carries its own independent
     keyframe list and timestamps, a single `units` block can simultaneously run a 2-keyframe fade
     and blur clear over `0..240ms` and a 3-keyframe `y` or `scale` settle over `0..360ms` without
     stacking a second `Animator`.
  4. **Selective unit inclusion/exclusion (`Subtract` / `Intersect` selector modes)**: splitting a
     target word or character into its own `Run` with `"unit": {"delay": ...}` or identity overrides
     (`"unit": {"y": 0, "opacity": 1.0}`) singles out or exempts specific units literally
     ([ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
     §4).

These authoring patterns enter `skills/montagent-motion/SKILL.md` as canonical stagger recipes.

---

### 3. `Units` and `UnitOverride` gain `blur`; per-unit `swivel`/`tilt`, `letter_spacing`, `stroke_width`, and animatable `line_height` are refused

#### 3.1 Per-unit `blur` on `Units` and `UnitOverride`

A signature motion-graphics title transition in both After Effects and CapCut/Premiere presets is a
per-character or per-word **blur-in / evaporate** reveal, where each unit resolves from Gaussian
blur + zero opacity + vertical/scale offset to crisp (`blur: 0`, `opacity: 1`). Element-level
`effects: [{"type": "blur", ...}]` blurs the entire text block uniformly and cannot stagger across
units.

`Units` (`text.units`) and `UnitOverride` (`runs[].unit`) therefore gain an optional animatable
`blur` field:

```json
"units": {
  "by": "char",
  "order": "forward",
  "every": 35,
  "y": [{"t": 0, "v": 14.0}, {"t": 280, "v": 0.0, "ease": "out-cubic"}],
  "opacity": [{"t": 0, "v": 0.0}, {"t": 240, "v": 1.0, "ease": "out-quad"}],
  "blur": [{"t": 0, "v": 16.0}, {"t": 280, "v": 0.0, "ease": "out-cubic"}]
}
```

- **Type and domain**: `Animatable<f64>`, optional, defaulting to `0.0` (`Pose::IDENTITY` carries
  `blur: 0.0`). Every literal or keyframe value must be finite and in `[0.0, 500.0]`, matching
  [ADR-0040](0040-blur-and-backdrop-blur.md)'s `blur` effect range (`E-OUT-OF-RANGE` on `< 0` or
  `> 500`).
- **Override semantics ([ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md) §4)**:
  on a single-unit `Run`, `"unit": {"blur": ...}` replaces `units.blur` for that unit, sampled at
  the unit's local time $t_{\text{local}} = t - \text{delay}$, identically to `x`, `y`, `scale`,
  `rotation`, and `opacity`.
- **Rendering ([ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md) §6)**:
  in `Canvas::text`, a unit body already opens a bounded offscreen `save_layer` between the solid
  stroke pass and the solid fill pass whenever `unit.opacity < 1.0`. That predicate generalizes to
  `unit.opacity < 1.0 || unit.blur > 0.0`:
  - When `unit.blur == 0.0 && unit.opacity >= 1.0`, the unit remains in the `solid` pass and opens
    **no** offscreen layer — preserving exact byte-identity for any unit at rest (`blur: 0`).
  - When `unit.blur > 0.0` (with `unit.opacity > 0.0`), the unit body opens a bounded `save_layer`
    whose bounds are `unit_bounds(&paths, &members)` outset on all four sides by
    $\lceil 3 \times \sigma \rceil$ where $\sigma = \text{blur} / 2$
    ([ADR-0040](0040-blur-and-backdrop-blur.md)), and whose layer `Paint` carries
    `alpha_f = unit.opacity` and
    `image_filters::blur((sigma, sigma), TileMode::Decal, None, None)`. The unit's stroke pass and
    fill pass draw into that layer so a blurred unit blurs as one unified letterform/piece without
    its stroke bleeding through its fill.
- **`validate` (`R-OFF-CANVAS`, [ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md) §7)**:
  the conservative unit excursion pad in `Plan::max_excursion` adds the Gaussian kernel skirt
  $\lceil 3 \times (\max \text{blur} / 2) \rceil$ across `units.blur` and any `Run.unit.blur`
  overrides so a blurred unit remains accounted for in off-canvas bounds checking.

#### 3.2 Refused per-unit and layout-reflowing animations

- **Per-unit `swivel` and `tilt` in `units`**: **refused** under
  [ADR-0167](0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md)
  §3 (*"Text and paths are rasterised flat and projected through the same single path; they are
  never projected as vectors"*). Element-level `swivel`, `tilt`, and `perspective` already animate
  on `TextElement` (`by: "line"` or single-word title cards can be separate projected `text`
  elements when distinct 3D planes are needed).
- **Per-unit `letter_spacing` in `units` and animatable `line_height` on `TextElement`**:
  **refused** under
  [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)
  §3 and
  [ADR-0223](0223-font-entries-pin-static-variable-axes-and-text-and-runs-gain-closed-opentype-features-baseline-shift-and-run-letter-spacing.md)
  §1.4. Animating `line_height` or per-unit tracking changes line slot heights or horizontal
  advances mid-shot, which invalidates static `measure`, `R-BOX-SLACK`, and `E-TEXT-OVERFLOW`.
  Uniform element-level `letter_spacing` already animates on `TextElement`
  ([ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
  §1), and per-unit positional spread without reflow is authored via `units.x` / `units.y`.
- **Per-unit `stroke_width` in `units`**: **refused** under
  [ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
  §2 (*"No unit paint"*). Element-level `stroke_width` already animates on `TextElement`, and static
  per-run `stroke_width` overrides live on `Run`.

---

### 4. Converting text glyph outlines to `path` elements (*Create Shapes from Text*) is refused

After Effects's *Create Shapes from Text* / *Create Masks from Text* converts live text glyphs into
editable vector shape/mask paths so individual contours can be trimmed or vertex-morphed.

Converting text glyph outlines into `PathElement` (`type: "path"`) is **refused** (both as a host
CLI/MCP subcommand and as a Python skill script):

1. **Single-contour `PathElement` invariant ([ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md) §1)**:
   a `PathElement` has a single `points: Animatable<Points>` contour with no interior `holes` or
   compound subpath array. Any glyph with an enclosed counter (`A`, `B`, `D`, `O`, `P`, `Q`, `R`,
   `a`, `b`, `d`, `e`, `g`, `o`, `p`, `q`, `0`, `4`, `6`, `8`, `9`, `&`, `%`, `@`) or disconnected
   contour (`i`, `j`, `!`, `?`, `:`, `;`, diacritics, CJK ideographs) cannot be represented as a
   single filled `path` element with hollow counters.
2. **Integer-pixel vertex/handle grid and sight-readability ([ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md) §1, [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md))**:
   `PathElement` vertices and `in`/`out` handles are integer pixels (`[i64; 2]`). TrueType quadratic
   splines elevated to cubic Beziers ($2/3$ control points) and rounded to integer pixels distort
   stem weights and bowl symmetry at display sizes while dumping hundreds of opaque coordinate rows
   into `project.json`.
3. **Stdlib-only skill script contract ([ADR-0217](0217-motion-bakes-and-keyframe-assistants-enter-in-one-keyframes-script-auto-colour-is-refused-and-a-skill-script-with-its-spec-is-a-template.md) §4.3)**:
   skill scripts run under Python 3 stdlib without third-party font parsers (`fontTools`).

**Authoring boundary (recorded in `skills/montagent-motion/SKILL.md`):**
- Keep typography as `type: "text"`, which already supports animated `stroke_width`, `letter_spacing`,
  per-unit staggers (`units`), soft/hard `mask` wipes, `clip` windows, and static variable-font
  `axes`.
- When a shot specifically calls for a `trim` stroke draw-on of a custom monogram, underline flourish,
  or single-stroke emblem, author that stroke directly as a concise integer-coordinate `path`
  element with `trim`.

---

### 5. Schema-level vertical writing mode (`writing_mode` / *Tate-chu-yoko*) is refused; stacked-column and rotated recipes enter the skill

After Effects offers vertical text direction (top-to-bottom columns, flowing right-to-left for CJK)
and *Tate-chu-yoko* (short horizontal digit/punctuation runs embedded upright inside a vertical
column).

Adding a schema-level `writing_mode` (`"vertical-rl"` / `"vertical-lr"`) or a *Tate-chu-yoko* field
to `TextElement` and `Run` is **refused**:
- `montagent-text` and `parley` `0.7.0` are built around a single exact horizontal line-slot engine:
  line slot height $\text{slot} = \text{max\_size} \times \text{line\_height}$ in exact integer
  twentieths ([ADR-0028](0028-text-block-arithmetic-is-exact-tenths.md)), horizontal `baseline_y`
  half-leading ([ADR-0029](0029-line-baseline-half-leading.md)), horizontal `align`
  (`start` / `center` / `end`), interlinear ruby `annotation`
  ([ADR-0208](0208-ruby-text-annotations-are-run-level-objects-with-line-inflating-slot-arithmetic.md)),
  and horizontal per-line `SkRRect` `decoration` fragments
  ([ADR-0214](0214-multi-line-text-range-decorations-are-run-level-objects-rendered-via-per-line-skrrect-boxes.md)).
  A vertical layout mode would duplicate every metric, box-slack, overflow, annotation, and
  decoration rule along a second orthogonal axis.
- Both reference-class motion-graphics uses of vertical typography are already directly reachable
  with existing `TextElement` primitives and are documented as canonical recipes in
  `skills/montagent-motion/SKILL.md`:
  1. **Upright stacked vertical column (CJK tategaki / vertical display drop-words)**: author one
     upright character per `\n` line inside a narrow `text` element with `align: "center"` and tight
     `line_height` (`1.0`–`1.1`, sized via `measure`). Multi-column vertical CJK titles place one
     narrow `text` element per column ordered right-to-left in `x`. Because each `\n` line is laid
     out horizontally and centered in the column, placing a 2-digit number or punctuation pair on a
     single `\n` line (e.g. `"東\n京\n25\n年"`) naturally produces upright horizontal-in-vertical
     **Tate-chu-yoko** centered in the vertical column, and `units` (`by: "line"` or `by: "char"`)
     staggers the column top-to-bottom.
  2. **Sideways Latin spine / edge label**: author standard horizontal `text` with `rotation: -90`
     (reading bottom-to-top) or `rotation: 90` (reading top-to-bottom).
