---
status: accepted
amends: 0007 (FontFile gains optional static axes for the five OpenType-registered variation axes; TextElement and Run gain optional closed OpenType features; Run gains optional baseline_shift; Faux Italic and All Caps join Faux Bold as explicit refusals), 0029 (Run.baseline_shift shifts a run's glyph placement and measured ink vertically without altering the line's shared baseline_y or slot height), 0151 (Run gains optional static letter_spacing overriding TextElement.letter_spacing per grapheme, with E-RUN-LETTER-SPACING-CONFLICT when TextElement.letter_spacing is animated), 0215 (disposes of the Variable Font Axes, Faux Bold/Italic, All Caps, OpenType features, Baseline Shift, Tsume, and Optical/Manual Kerning rows from the After Effects text census #902 and Premiere Pro titles census on map #896)
---

# Font entries pin static variable axes, and text and runs gain closed OpenType features, baseline_shift, and run letter_spacing

[Which variable-font axes, closed OpenType features and baseline-shift controls enter text and spans?](https://github.com/MBehtemam/Montagent/issues/1066)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0007](0007-text-runs-literal-size-declared-fonts.md) established that a `text` element carries a base style and an ordered
`runs` array, references a project-declared `font` chain by semantic name (`"font": "brand"`), and refuses `weight` and `bold`
on elements and runs while deferring variable-font axes until the text shaper could honour them. Since then,
`montagent-text` migrated to `parley` `0.7.0` + `swash` + `skrifa` `0.40.0`
([ADR-0010](0010-skia-safe-rasterizer-text-beside-it.md)), and the After Effects text and typography census
([#902](https://github.com/MBehtemam/Montagent/issues/902)) and Premiere Pro titles census
([`docs/research/premiere-gap/titles.md`](../research/premiere-gap/titles.md)) catalogued seven character-level typography and
OpenType controls across both reference tools:
- **Variable-font axis coordinates** (`wght`, `wdth`, `slnt`, `ital`, `opsz`, plus custom axes and per-character animator axes
  in After Effects [#902](https://github.com/MBehtemam/Montagent/issues/902) §1 and §2);
- **Faux Bold, Faux Italic, and All Caps** ([#902](https://github.com/MBehtemam/Montagent/issues/902) §1; Premiere Essential
  Graphics *Faux Styles*);
- **Closed OpenType features** (Small Caps, Caps to Small Caps, Superscript, Subscript, Tabular/Oldstyle Figures, Slashed Zero,
  Fractions, Ordinals, and Ligature toggles in [#902](https://github.com/MBehtemam/Montagent/issues/902) §1 and Premiere
  [`titles.md`](../research/premiere-gap/titles.md) §2);
- **Baseline Shift** (per-run vertical offset in pixels in both Premiere and After Effects); and
- **Per-run Tracking (`letter_spacing`), Kerning (Metrics / Optical / manual), and Tsume** ([#902](https://github.com/MBehtemam/Montagent/issues/902)
  §1; Premiere [`titles.md`](../research/premiere-gap/titles.md) §2).

This ADR decides:
1. Where variable-font axis coordinates live (`FontFile.axes` in top-level `fonts`), why animated variable axes are refused,
   and how `validate` (`E-FONT-AXIS`), `parley`, and `skrifa` evaluate them.
2. Which closed OpenType `features` enter `TextElement` and `Run`, how `validate` (`E-FONT-FEATURE`) checks them against the
   resolved face, and why Faux Bold, Faux Italic, and All Caps are refused.
3. How `baseline_shift` enters `Run` and interacts with line slot height ([ADR-0028](0028-text-block-arithmetic-is-exact-tenths.md)),
   `baseline_y` ([ADR-0029](0029-line-baseline-half-leading.md)), and ink measurement.
4. How static `letter_spacing` enters `Run` ([ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md) §1),
   and why Tsume and Optical Kerning are refused.

---

## Decisions

### 1. Variable-font `axes` enter `FontFile` in top-level `fonts`; animated variable axes are refused

1. **Static `axes` on `FontFile` in `fonts`:**
   Each chain entry (`FontFile`) in the project's top-level `fonts` table gains an optional `axes: Option<FontAxes>` object
   alongside `file` and `index`:

   ```json
   "fonts": {
     "inter-regular": [
       {"file": "fonts/InterVariable.ttf", "axes": {"wght": 400}}
     ],
     "inter-display-bold": [
       {"file": "fonts/InterVariable.ttf", "axes": {"wght": 750, "opsz": 32}},
       {"file": "fonts/Vazirmatn-Bold.ttf"}
     ]
   }
   ```

   `FontAxes` is a closed struct (`#[serde(deny_unknown_fields)]`) carrying optional finite `f64` fields for the five
   OpenType-registered variation axes:
   - `wght` — Weight
   - `wdth` — Width
   - `slnt` — Slant
   - `ital` — Italic
   - `opsz` — Optical size

   When `axes` is present on a `FontFile`, at least one axis field must be set; an empty `"axes": {}` object is a schema error.
   Pinning axis coordinates on the `FontFile` entry preserves [ADR-0007](0007-text-runs-literal-size-declared-fonts.md)'s rule
   that a `text` element or `Run` selects a face by its semantic `font` key (`"font": "inter-display-bold"`) rather than
   carrying inline weight/width/style sliders, while allowing a single vendored variable `.ttf` to back multiple named faces in
   `fonts` and letting static fallback entries in the same chain omit `axes`.

2. **Validation against the face's `fvar` table (`E-FONT-AXIS`):**
   `validate` inspects the resolved face of each `FontFile` that declares `axes` via `skrifa`'s `MetadataProvider::axes()`
   (`fvar` table). An entry is rejected with `E-FONT-AXIS` if:
   - the face is not a variable font or does not expose a declared axis tag in its `fvar` table; or
   - a declared coordinate falls outside that axis's `[min_value, max_value]` range in `fvar`, naming the offending tag and the
     face's legal bounds.

3. **Shaping, metrics, and glyph outlines at the pinned variation location:**
   - During `parley` shaping (`crates/montagent-text/src/engine.rs`), the resolved chain entry's `axes` are passed via
     `StyleProperty::FontVariations` so `swash` shapes `GSUB`/`GPOS` and `HVAR` glyph advances at the pinned design-space
     coordinates.
   - `skrifa` outline extraction (`place.rs` `outline_of`), ink measurement (`ink.rs` `line_ink`), and vertical metrics
     (`metrics.rs` `FaceMetrics`) evaluate at `LocationRef::new(run.run().normalized_coords())` (or `axes.location(...)` for
     face metrics) rather than `LocationRef::default()`, and `place.rs`'s glyph outline cache key includes the run's normalized
     variation coordinates so two named fonts backed by the same variable `.ttf` at different `axes` never alias in cache.

4. **Animated variable-font axes (keyframed on `text`, `Run`, or `units`) and arbitrary four-letter custom tags are refused:**
   - **Breaks static box measurement ([ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md) §3):**
     Varying `wght` or `wdth` over time or across `units` changes glyph advances (`HVAR`) and outlines (`gvar`) on every frame,
     making line widths and ink boxes frame-dependent and breaking `measure`, `R-BOX-SLACK`, and `E-TEXT-OVERFLOW` for the same
     reason [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md) §3
     refuses keyframing `size` and `line_height`.
   - **Sanctioned routes:** Stretch animations use `scale`; discrete weight changes across time use a timed style window or run
      referencing a second `fonts` key; kinetic variable-font weight morphs enter as pre-rendered plates via
     `montagent-prerender` ([ADR-0156](0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7).

### 2. Closed OpenType `features` enter `TextElement` and `Run`; Faux Bold, Faux Italic, and All Caps are refused

1. **Closed `features` list on `TextElement` and `Run`:**
   Both `TextElement` and `Run` gain an optional `features: Option<Vec<FontFeature>>` field:

   ```json
   {
     "id": "timer-01",
     "type": "text",
     "font": "inter-regular",
     "size": 64,
     "features": ["tnum", "zero"],
     "runs": [
       {"text": "01:23.45 "},
       {"text": "st", "features": ["ordn"]}
     ]
   }
   ```

   `FontFeature` is a closed enum (`#[serde(rename_all = "lowercase")]`) of nine standard OpenType feature tags:
   - `"tnum"` — Tabular figures (monospaced digits for counters, timers, and tables)
   - `"onum"` — Oldstyle figures
   - `"zero"` — Slashed zero
   - `"smcp"` — Small capitals (lowercase to small caps)
   - `"c2sc"` — Capitals to small capitals
   - `"sups"` — True OpenType superscript glyphs
   - `"subs"` — True OpenType subscript glyphs
   - `"frac"` — Diagonal fractions
   - `"ordn"` — Ordinals

   **Precedence and schema rules:**
   - When `Run.features` is omitted (`None`), the run inherits `TextElement.features`. When `Run.features` is present, it
     replaces `TextElement.features` for that run.
   - An explicit empty array (`"features": []`), duplicate entries within the same `features` array, or mutually exclusive pairs
     in the same `features` array (`"tnum"` alongside `"onum"`, or `"sups"` alongside `"subs"`) are rejected at validation time.
   - Each requested feature is passed to `parley` via `StyleProperty::FontFeatures` with setting `1` alongside the existing
     contextual ligature settings.

2. **Ligatures remain governed by rule, not manual feature toggles:**
   Standard and discretionary ligatures (`liga`, `clig`, `dlig`, `rlig`) are not members of `FontFeature`. They remain governed
   by [ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md) §1 and
   [ADR-0153](0153-a-joined-piece-moves-as-one-and-joining-scripts-keep-their-ligatures-and-take-no-letter-spacing.md) §3:
   optional ligatures (`liga`, `clig`, `dlig`) are switched off automatically on non-joining scripts whenever any element or run
   `letter_spacing` is non-zero or the element staggers by `letter`, and left at the shaper's standard script defaults
   otherwise.

3. **Validation against the resolved face's `GSUB`/`GPOS` tables (`E-FONT-FEATURE`):**
   `validate` checks every glyph's resolved face in `montagent-text` against the active `features` list on its run using
   `skrifa`'s `GSUB` and `GPOS` feature lists. If a face used to shape the run does not declare a requested `FontFeature` tag in
   either `GSUB` or `GPOS`, `validate` emits `E-FONT-FEATURE` naming the element, the face, and the unsupported feature tag.
   Silent no-ops (such as requesting `"smcp"` or `"tnum"` on a font that lacks those tables) are refused.

4. **Faux Bold, Faux Italic, and All Caps are refused:**
   - **Faux Bold and Faux Italic:** [ADR-0007](0007-text-runs-literal-size-declared-fonts.md) already refused `bold` and `weight`
     because synthetic emboldening is a renderer-specific geometric hack. Synthetic obliquing (*Faux Italic* — shearing glyph
     outlines when no italic face is loaded) is refused on the same ground. The sanctioned routes are a real bold/italic font
     file in `fonts`, a variable font with `axes.wght`, `axes.ital`, or `axes.slnt` in `fonts`, or an explicit `stroke` on the
     text/run when an outlined look is intended.
   - **All Caps:** A display-time uppercase transform makes rendered glyphs diverge from the literal string in `runs[].text`,
     breaking exact-string replace and reading from the file. The sanctioned route is writing uppercase characters literally in
     `runs[].text` (combined with `"features": ["c2sc", "smcp"]` when full small caps are desired).

### 3. `baseline_shift` enters `Run` as a static signed integer pixel offset

1. **Schema and sign convention:**
   `Run` gains an optional `baseline_shift: Option<i64>` field in signed integer pixels:

   ```json
   "runs": [
     {"text": "E = mc"},
     {"text": "2", "size": 36, "baseline_shift": 18}
   ]
   ```

   - **Sign convention:** **Positive shifts the run's glyphs upward** (raised above the line's shared baseline, $-y$ in canvas
     coordinates: `y_drawn = baseline_y - baseline_shift`), and **negative shifts the run's glyphs downward** (lowered below the
     baseline, $+y$ in canvas coordinates), matching Premiere Pro, After Effects, and CSS `baseline-shift`.
   - **Default presence rule ([ADR-0030](0030-defaultable-field-presence-is-content.md)):** Omitted means unshifted (`0`).
     Writing an explicit `"baseline_shift": 0` is rejected as a schema error so unshifted runs have one canonical spelling.
   - **Static only:** `baseline_shift` is a static typographic placement offset; animated per-letter or per-word vertical bounce
     belongs to `units` `y` keyframes ([ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)).

2. **Line slot vs. ink arithmetic ([ADR-0028](0028-text-block-arithmetic-is-exact-tenths.md), [ADR-0029](0029-line-baseline-half-leading.md)):**
   - **Line slot height and shared `baseline_y` are unchanged:** A line's slot height remains `ceil(max_run_size × line_height)`
     ([ADR-0028](0028-text-block-arithmetic-is-exact-tenths.md)), and the line's shared `baseline_y` remains
     `slot_centre_y + (ascent - descent) / 2` from the runs' unshifted font metrics
     ([ADR-0029](0029-line-baseline-half-leading.md)). `baseline_shift` never moves the rest of the line or inflates the line
     slot.
   - **Glyph placement, ink bounds, and overflow checks follow the shift:** `place.rs`, `ink.rs` (`line_ink`), `measure`
     (including `--layout` word boxes and range fragments per [ADR-0213](0213-measure-layout-report-exposes-word-boxes-and-character-ranges-under-conservative-integer-enclosure.md)),
     `query --at`, `E-TEXT-OVERFLOW`, and `R-LINE-INK-COLLISION` all evaluate the run's glyphs at `baseline_y - baseline_shift`.
     If a large `baseline_shift` pushes a run's glyph ink outside the element's declared `width × height` box or into an
     adjacent line's ink, `E-TEXT-OVERFLOW` and `R-LINE-INK-COLLISION` catch it automatically.

### 4. Static `letter_spacing` enters `Run`; Tsume and Optical Kerning are refused

1. **Static `letter_spacing` on `Run` ([ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md) §1 amended):**
   `Run` gains an optional static `letter_spacing: Option<i64>` field, measured in thousandths of the run's own `size` (the same
   unit as `TextElement.letter_spacing`):

   ```json
   "runs": [
     {"text": "PRESENTING ", "size": 32, "letter_spacing": 120},
     {"text": "THE ARCHIVE", "size": 72}
   ]
   ```

   - **Per-grapheme resolution:** When `Run.letter_spacing` is present on the run containing a grapheme cluster, the space added
     after that grapheme is `run_size × run_letter_spacing / 1000.0` pixels; when omitted (`None`), the grapheme uses the
     element's `TextElement.letter_spacing`. Consistent with [ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
     §1 and [ADR-0153](0153-a-joined-piece-moves-as-one-and-joining-scripts-keep-their-ligatures-and-take-no-letter-spacing.md) §2,
     no space is added after a line's last grapheme cluster or between two letters of the same joining script, and
     `R-SPACING-SUPPRESSED` fires if a run with only joining-script pairs declares non-zero `letter_spacing`.
   - **Precedence rule against animated element spacing (`E-RUN-LETTER-SPACING-CONFLICT`):**
     [ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md) initially
     deferred `Run.letter_spacing` because a static run override alongside an animated element-level `letter_spacing` keyframe
     list creates a precedence ambiguity. To resolve this cleanly without a second interpolation mode, `Run.letter_spacing` is
     legal only when `TextElement.letter_spacing` is static (or omitted); declaring `Run.letter_spacing` on an element whose
     `TextElement.letter_spacing` is a keyframe list is rejected by `validate` with `E-RUN-LETTER-SPACING-CONFLICT`.
   - **Optional ligatures rule:** `OPTIONAL_LIGATURES_OFF` (`liga`, `clig`, `dlig` disabled on non-joining scripts per
     [ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md) §1 and
     [ADR-0153](0153-a-joined-piece-moves-as-one-and-joining-scripts-keep-their-ligatures-and-take-no-letter-spacing.md) §3)
     activates for the element whenever `TextElement.letter_spacing` has any non-zero value/keyframe, any `Run.letter_spacing`
     on the element is non-zero, or the element staggers with `by: letter`.
   - **Single-character pair kerning:** Adjusting the gap after a single character (Premiere/After Effects manual pair kerning)
     is written by isolating that character in its own `Run` with its own `letter_spacing`, using one unified mechanism rather
     than a separate kerning field.

2. **Tsume and Optical Kerning are refused:**
   - **Optical Kerning:** Computes spacing from proprietary glyph-outline shape heuristics rather than the font's `GPOS`/`kern`
     tables (`Metrics` kerning, which `parley` + `swash` always apply). It has no closed specification outside Adobe's text
     engine.
   - **Tsume:** Reduces CJK glyph sidebearings by a percentage of the glyph's internal whitespace, requiring CJK ideographic
     metrics heuristics not exposed as a standard cross-font OpenType invariant. Proportional or tighter CJK spacing is
     authored via `"features"` or `letter_spacing`.

---

## Why

- **Variable-font `axes` belong on `FontFile`, not on `TextElement` or `Run`:**
  [ADR-0007](0007-text-runs-literal-size-declared-fonts.md) established that font selection in `TextElement` and `Run` is by a
  semantic `font` name (`"font": "heading-bold"`), never by inline weight or style numbers that might silently synthesize or
  mismatch the loaded file. Placing `axes` on `FontFile` in `fonts` lets an author vendor a single variable `.ttf` (such as
  `InterVariable.ttf`) and define named faces at exact design coordinates, validated once against `fvar` (`E-FONT-AXIS`), while
  keeping every `text` and `Run` reference a clean semantic key.
- **Validating `features` against `GSUB`/`GPOS` upholds [ADR-0007](0007-text-runs-literal-size-declared-fonts.md)'s "no silent
  no-op" principle:**
  In GUI editors, toggling Small Caps or Tabular Numbers on a font that lacks the OpenType table either silently does nothing or
  falls back to synthetic scaling. Checking `GSUB`/`GPOS` in `validate` (`E-FONT-FEATURE`) ensures that every declared feature
  in `project.json` corresponds to real glyphs in the resolved face, while `Run.baseline_shift` + `Run.size` remains available
  for explicit manual superscripts/subscripts on any font.
- **`baseline_shift` moves ink without perturbing the line slot:**
  In both Premiere Pro and After Effects, shifting a superscript or trademark symbol upward via Baseline Shift does not push the
  entire line's baseline down or expand the line-to-line leading of the paragraph. Keeping `slot` and `baseline_y` anchored to
  the unshifted metrics while shifting glyph placement and measured `ink` matches editor precedent and relies on existing
  `E-TEXT-OVERFLOW` and `R-LINE-INK-COLLISION` checks to guard the boundaries.
- **Static `Run.letter_spacing` removes the multi-element workaround for mixed-tracking lines:**
  Mixed tracking within one line (such as a wide-tracked small-caps lead-in followed by normal-tracked body text, or a single
  tightened kerning pair) previously forced authors to split one line into separate `text` elements and hand-compute horizontal
  offsets. Admitting static `Run.letter_spacing` while forbidding it alongside animated `TextElement.letter_spacing`
  (`E-RUN-LETTER-SPACING-CONFLICT`) solves mixed-tracking lines with zero interpolation ambiguity.

---

## Considered and refused

- **Inline `axes` on `TextElement` and `Run`, or animatable `axes` keyframes.** Refused: inline axes on elements/runs reintroduce
  [ADR-0007](0007-text-runs-literal-size-declared-fonts.md)'s inline weight/style footgun and complicate mixed static/variable
  fallback chains, while animating `axes` over time changes glyph advances (`HVAR`) and outlines (`gvar`) per frame, violating
  [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md) §3's
  static-box invariant.
- **Open four-character OpenType axis or feature tag strings.** Refused: violates the closed-vocabulary invariant
  ([ADR-0017](0017-closed-schema-no-escape-hatch.md), [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md) §2).
- **Faux Bold, Faux Italic, and All Caps fields.** Refused: synthetic emboldening and synthetic obliquing produce
  renderer-specific geometry where a real face or variable axis belongs, and an All Caps flag causes rendered text to diverge
  from `runs[].text`.
- **`baseline_shift` inflating the line slot or shifting `baseline_y`.** Refused: shifting a superscript on a mixed line must
  not displace the unshifted words on the same line or alter line-to-line spacing.
- **Allowing `Run.letter_spacing` simultaneously with keyframed `TextElement.letter_spacing`.** Refused: requires choosing
  whether a static run override freezes that run during a whole-element tracking animation or adds to it; rejecting the
  combination with `E-RUN-LETTER-SPACING-CONFLICT` keeps precedence unambiguous.
- **Tsume and Optical Kerning.** Refused: renderer-specific glyph-shape heuristics with no closed cross-font arithmetic.

---

## Consequences

- `CONTEXT.md`'s **Face**, **Run**, and **Letter spacing** entries are updated, and **OpenType features** and **Baseline shift**
  entries are added.
- [ADR-0007](0007-text-runs-literal-size-declared-fonts.md),
  [ADR-0029](0029-line-baseline-half-leading.md),
  [ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md), and
  [ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md) gain amendment
  banners pointing here.
- Implementation graduates onto map [#896](https://github.com/MBehtemam/Montagent/issues/896) covering `FontFile.axes`
  (`E-FONT-AXIS`), `TextElement.features` and `Run.features` (`E-FONT-FEATURE`), `Run.baseline_shift`, and `Run.letter_spacing`
  (`E-RUN-LETTER-SPACING-CONFLICT`) across `montagent-core` and `montagent-text`.
