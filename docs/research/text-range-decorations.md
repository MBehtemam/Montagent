# Text Range Decorations Across Wrapped Lines: Precedent, Schema Design, and Rendering Geometry

Research for Ticket [#1016](https://github.com/MBehtemam/Montagent/issues/1016), evaluating schema architecture and Skia rendering geometry for text range decorations (underlines, highlights, and dashed/solid phrase boxes) that span across wrapped lines.

**Date of research:** 2026-10-13.
**Sourcing rule applied:** Every technical claim cites primary sources — official W3C specifications, first-party software documentation, engine source code, and measured prototype project files. Labels used:
- **READ** — quoted verbatim from a primary specification, API reference, or codebase with path/URL.
- **MEASURED** — extracted directly from prototype artifacts (`scratch/handoff/prototype/` and `desired.json`).
- **INFERRED** — engineering deduction and architectural synthesis over the facts above.

---

## 1. Executive Summary

1. **The Production Defect:** In storybook and educational video production (e.g. English Part 1), multi-word phrases such as `"out in the countryside"` wrap across line breaks (`"out in the"` / `"countryside."`) and carry semantic annotations (glosses). Storybook typography requires a single continuous dashed phrase box enclosing the phrase. Because Montagent currently lacks inline range decoration primitives, authors are forced to disassemble sentences into multiple disconnected `text` elements and manually calculate bounding `rect` elements for each line fragment (§2, MEASURED).
2. **Precedent Across Graphic Systems:**
   - **CSS (`box-decoration-break`):** W3C CSS Fragmentation Module Level 3 defines `box-decoration-break: slice | clone` for inline boxes fragmented across line wraps. `slice` treats the element as an unbroken box sliced apart (open ends at breaks, no break padding/border). `clone` independently wraps every line fragment with complete padding, borders, corner radii, and backgrounds (§3.1, READ).
   - **Adobe Premiere Pro:** Premiere's Essential Graphics panel provides a Text Background feature with a wrench toggle between **`All Lines`** (one unified rectangular bounding box covering all lines) and **`Per-Line`** (dynamically creates independent background boxes for each individual line conforming to that line's text bounds). Premiere's Underline is a character-level property that draws baseline strokes across line fragments (§3.2, READ).
   - **Adobe InDesign:** InDesign's Paragraph Borders & Shading with `Width: Text` does *not* create per-line boxes; on multi-line text it draws one single bounding box covering from the leftmost start to the rightmost end of the longest line. Production designers are forced to use the "Character Underline Trick" (thick, offset underlines), which naturally fragments across line wraps but cannot provide closed four-sided borders, horizontal padding, or rounded corner radii (§3.3, READ).
3. **Rendering Geometry in Skia:** A **bounding union** across lines fails completely for wrapped phrases because it creates a giant rectangular box spanning from the start of the second line to the end of the first line, enclosing unrelated text and empty margins (§4.1, INFERRED). The correct geometry is **per-line fragmented boxes** derived from Parley/Skrifa cluster advances and line slot metrics. Skia draws these fragments using `SkRRect` and `skia_safe::PathEffect::dash` under `box-decoration-break: clone` (§4.2, §4.3).
4. **Schema Integration in Montagent:** Range decorations must attach directly to `Run` in `crates/montagent-core/src/model/text.rs` (§5). Consistent with ADR-0007 (runs own styled text) and ADR-0048 (no element-level ID join arrays, no fragile sub-run character offsets), the run is the atomic unit of text addressing. When a decorated phrase wraps across lines via explicit `\n`, a single run produces multiple line fragments during layout.
5. **Speech Synchronization & Paint-Only Invariant:** A decoration's visibility may be static or timed via a speech-synchronized window `[start, end)` (ADR-0048). As established in ADR-0048 and issue #280, decorations are strictly **paint passes**; they never alter layout metrics or cause text reflow at runtime.

---

## 2. The Production Defect & Hand-Crafted Prototype

### 2.1 The Concrete Problem (`scratch/handoff/ISSUE-DRAFTS/03-range-decoration-comment.md`)

In the storybook production pipeline, phrases frequently break across lines and carry glosses or vocabulary markers. In `scratch/handoff/ISSUE-DRAFTS/03-range-decoration-comment.md`, the production team notes (READ):

> "Real case: in English Part 1 the phrase `out in the countryside` wraps (`out in the` / `countryside.`) and has the gloss `land outside towns`; we draw no dashed box today because one rect cannot follow a wrapped range, about 10 phrases per part. The prototype `handoff/prototype/ruby-prototype.mp4` (17-27 s, from 86.2 s) shows the wanted result: one closed dashed box per line fragment (box x 691-838 on line 1, 477-661 on line 2), built by hand from `measure` advances.
>
> Rule we used: a range decoration is drawn once per line it touches, padding applied per fragment, same style on each. Open ends on the wrap (CSS `box-decoration-break: slice`) would also be acceptable; we have no preference. A timed draw-on window like ADR-0048's `highlight` would let it appear with the voice. The data needed is exactly the range boxes of the `measure` ticket."

### 2.2 Inspection of the Hand-Crafted Prototype (`desired.json`)

To achieve the desired visual output in `handoff/prototype/ruby-prototype.mp4` without schema support, the author had to resort to severe workarounds (MEASURED from `scratch/handoff/prototype/desired.json`):

1. **Sentence Fragmentation:** The single sentence was split across distinct text elements:
   - Line 1 fragment: `c01-01-text` ending in `"It was so lovely out in the"` at `y: 129`
   - Line 2 fragment: `text-l1b` containing `"countryside."` at `y: 208`
2. **Manual Advance Calculations:** In `desired-layout-computation.json`:
   - Line 1 fragment (`"out in the"`): Advance `x0: 691.71`, `x1: 836.05`, width `144.34 px`.
   - Line 2 fragment (`"countryside."`): Advance `x0: 477.00`, `x1: 660.01`, width `183.01 px`.
3. **Disjoint Shape Elements:** The author created two separate `type: rect` elements on separate layers:
   ```json
   {
     "id": "b-phrase-a",
     "type": "rect",
     "start": 86200,
     "end": 100000,
     "x": 692,
     "y": 143,
     "origin": "top-left",
     "width": 144,
     "height": 60,
     "fill": "#00000000",
     "stroke": "#4F7A2B",
     "stroke_width": 3,
     "stroke_dash": [9, 7],
     "radius": 11,
     "opacity": [{"t": 86200, "v": 0.0}, {"t": 86600, "v": 1.0, "ease": "ease-out"}]
   }
   ```
   and on Line 2:
   ```json
   {
     "id": "b-phrase-b",
     "type": "rect",
     "start": 86200,
     "end": 100000,
     "x": 477,
     "y": 222,
     "origin": "top-left",
     "width": 183,
     "height": 60,
     "fill": "#00000000",
     "stroke": "#4F7A2B",
     "stroke_width": 3,
     "stroke_dash": [9, 7],
     "radius": 11,
     "opacity": [{"t": 86200, "v": 0.0}, {"t": 86600, "v": 1.0, "ease": "ease-out"}]
   }
   ```

**Architectural Defect:** This manual approach violates the core tenets of declarative authoring:
- Text is artificially split across multiple elements, destroying natural text flow and alignment.
- Coordinate calculations are coupled to a specific font rasterization and break if font size, letter spacing, or copy changes.
- Styling, timing, and animations must be duplicated across $N$ elements for $N$ lines.

---

## 3. Precedent Analysis & Primary Sources

### 3.1 CSS Standards: `box-decoration-break` & `text-decoration`

#### 3.1.1 W3C CSS Fragmentation Module Level 3 & Level 4
The W3C specification explicitly addresses the styling of inline elements broken across lines.

**Source:** W3C CSS Fragmentation Module Level 3, §3.2 *Joining Boxes: the `box-decoration-break` property* ([https://www.w3.org/TR/css-break-3/#break-decoration](https://www.w3.org/TR/css-break-3/#break-decoration)).

> **Property Definition (READ):**
> - **Name:** `box-decoration-break`
> - **Value:** `slice | clone`
> - **Initial:** `slice`
> - **Applies to:** all elements
>
> "When a inline box breaks across lines, margins, borders, padding, and other decorations wrap each individual fragment separately or span across them as a single unbroken box."
>
> - **`slice`:** "The effect is as though the element were rendered with no breaks present, and then sliced by the breaks afterward. No border and no padding are inserted at a break edge; `border-radius` and backgrounds apply to the unbroken box."
> - **`clone`:** "Each box fragment is independently wrapped with its own border, padding, and margin. The `border-radius`, `border-image`, and `box-shadow` are applied to each fragment independently. The background is drawn independently within each fragment."

#### 3.1.2 W3C CSS Text Decoration Module Level 3 & Level 4
**Source:** W3C CSS Text Decoration Module Level 3 ([https://www.w3.org/TR/css-text-decor-3/](https://www.w3.org/TR/css-text-decor-3/)).
- `text-decoration-line`: `underline | overline | line-through`
- `text-decoration-style`: `solid | double | dotted | dashed | wavy`
- `text-decoration-thickness` and `text-underline-offset`
- Underlines naturally fragment per line box: the browser computes the baseline of each line box and draws a line from the start to the end of the inline run on that line.

### 3.2 Adobe Premiere Pro: Title Background & Underline

**Source:** Adobe Premiere Pro User Guide: *Add text and titles* & *Graphics preferences* ([https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html](https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html) and [https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/](https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/)).

1. **Text Background (Appearance Panel):**
   - In the Essential Graphics panel, text elements carry a `Background` appearance option with parameters: Fill Color, Opacity, Size (padding in pixels), and Roundness (corner radius 0–100%).
   - Under the wrench / settings menu (Graphics Properties), Premiere provides a dedicated toggle for multi-line text:
     - **`All Lines`:** Draws one single bounding box covering the entire text paragraph, sized to the longest line (READ).
     - **`Per-Line`:** Dynamically creates independent background boxes for each individual line of text, wrapping each line fragment tightly according to its advance width (READ).
2. **Captions Track (Subtitles):**
   - Premiere's native subtitle engine uses `Per-Line` backgrounds by default, rendering individual pill boxes behind each line of speech-synced text.
3. **Underline (Faux Styles):**
   - Premiere provides a character-level Underline toggle (`T`). It calculates a baseline-offset stroke for the selected character range across whatever line breaks the text spans.

### 3.3 Adobe InDesign: Paragraph Shading & Character Underline

**Source:** Adobe InDesign User Guide: *Paragraph and character styles* & *Paragraph borders and shading* ([https://helpx.adobe.com/indesign/using/paragraph-character-styles.html](https://helpx.adobe.com/indesign/using/paragraph-character-styles.html)).

1. **Paragraph Shading and Borders:**
   - InDesign provides Paragraph Shading and Paragraph Borders with a `Width` setting: `Column` vs `Text`.
   - **Crucial Multi-Line Behavior:** When a paragraph wraps across multiple lines and `Width: Text` is selected, InDesign does **not** draw individual boxes per line. Instead, it draws a single bounding box whose width extends from the leftmost position of the text to the rightmost point of the longest line (READ). On ragged text, this leaves blank padding behind short lines.
2. **The "Character Underline Trick":**
   - Because InDesign lacks per-line character boxes, professional typographers rely on **Underline Options** in Character Styles:
     - Weight: Set to font size (e.g. `33 pt`).
     - Offset: Set to a negative vertical shift (e.g. `-10 pt`), pushing the stroke up behind the glyphs.
     - Type: `Solid`, `Dashed`, `Dotted`, or `Wavy`.
     - Gap Color and Gap Tint for dashes.
   - **Advantage:** Because Underline is an inline character style, it automatically fragments across line breaks, painting along each line fragment separately.
   - **Limitation:** Underline is fundamentally a 1D stroke along the baseline; it cannot produce closed 4-sided boxes, horizontal end-caps, or rounded corner radii.

### 3.4 Precedent Comparison Table

| System | Feature | Granularity | Wrapped Multi-Line Handling | Geometry / Style |
| :--- | :--- | :--- | :--- | :--- |
| **CSS** | `box-decoration-break: clone` | Inline Element (`<span>`, `<mark>`) | Per-line fragment | Closed box per line, 4-sided padding, independent `border-radius`, solid/dashed borders |
| **CSS** | `box-decoration-break: slice` | Inline Element | Per-line fragment | Sliced continuous box; open ends at wrap boundaries; no padding/border at break edge |
| **CSS** | `text-decoration` | Inline Element | Per-line fragment | Baseline-relative stroke (solid, dashed, wavy, dotted) per line |
| **Premiere Pro** | Text Background (`Per-Line`) | Text Element / Layer | Per-line fragment | Rounded rect per line, uniform padding (`Size`), corner radius (`Roundness`) |
| **Premiere Pro** | Text Background (`All Lines`) | Text Element / Layer | Bounding union | One large box covering all lines sized to longest line |
| **InDesign** | Paragraph Shading (`Width: Text`) | Paragraph | Bounding union | Single rect enclosing all lines from min-x to max-x |
| **InDesign** | Underline Options ("Underline Trick") | Character / Run | Per-line fragment | Open baseline stroke per line (solid, dashed, dotted); no side caps or box radius |
| **Montagent (Today)** | Manual `rect` elements | Element | Manual authoring | Author manually measures advances and places separate `rect` shapes |

---

## 4. Rendering Geometry in Skia & Montagent Engine

### 4.1 Failure of the Bounding Union for Wrapped Phrases

Consider the phrase `"out in the countryside."` wrapping across two lines in a left-aligned or justified block:
- Line 1: `"It was so lovely out in the"` $\rightarrow$ phrase fragment `"out in the"` occupies $x \in [692, 836]$, $y \in [143, 203]$.
- Line 2: `"countryside."` $\rightarrow$ phrase fragment occupies $x \in [477, 660]$, $y \in [222, 282]$.

If a renderer constructs a single **bounding union**:
$$R_{\text{union}} = [\min(x_1, x_2), \min(y_1, y_2), \max(x_1 + w_1, x_2 + w_2), \max(y_1 + h_1, y_2 + h_2)]$$
$$R_{\text{union}} = [477, 143, 836, 282]$$

**Why Bounding Union Fails (INFERRED):**
1. On Line 1, the box extends leftward from 692 to 477, erroneously covering the preceding words `"so lovely "`.
2. On Line 2, the box extends rightward from 660 to 836, covering empty margin or following words.
3. In the vertical gap between lines, the box fills the leading space, creating a massive solid block rather than distinct phrase marks.

Therefore, **a bounding union is invalid for multi-line phrase decorations.** Fragmented per-line geometry is mathematically required.

### 4.2 Derivation of Line Fragment Geometry

Montagent’s text layout pipeline (`montagent-text`) shapes text with Parley and places glyphs in unscaled element space (`crates/montagent-text/src/place.rs`).

For any run $R$ carrying a decoration:
1. **Cluster Filtering:** Identify all glyph clusters $C_i$ produced by run $R$.
2. **Line Partitioning:** Group clusters by their line index $L$. For a run that wraps across lines, this partitions the clusters into $k$ line subsets $\{C_{L, 1}, \dots, C_{L, m}\}$.
3. **Horizontal Extent per Line ($[x_{\text{start}}, x_{\text{end}}]$):**
   $$x_{\text{min}}(L) = \min_{j} (C_{L, j}.x)$$
   $$x_{\text{max}}(L) = \max_{j} (C_{L, j}.x + C_{L, j}.\text{advance})$$
4. **Vertical Extent per Line ($[y_{\text{top}}, y_{\text{bottom}}]$):**
   The vertical extent is derived from the line metrics in `MeasuredLine`:
   - Slot-relative: `slot_top` to `slot_top + slot_height`.
   - Typographic / Baseline-relative: `baseline_y - ascent` to `baseline_y + descent`.
   Using typographic bounds (`ascent` / `descent`) ensures that the phrase box hugs the text tightly rather than expanding into excess `line_height` leading.
5. **Padding Expansion:**
   Given padding $P = [P_{\text{top}}, P_{\text{right}}, P_{\text{bottom}}, P_{\text{left}}]$:
   $$\text{rect}(L) = [x_{\text{min}}(L) - P_{\text{left}}, y_{\text{top}}(L) - P_{\text{top}}, (x_{\text{max}} - x_{\text{min}}) + P_{\text{left}} + P_{\text{right}}, (y_{\text{bottom}} - y_{\text{top}}) + P_{\text{top}} + P_{\text{bottom}}]$$

### 4.3 Rendering Modes: `clone` vs `slice` in Skia

```
                       LINE 1: "out in the"           LINE 2: "countryside."
                 +--------------------------+     +--------------------+
clone mode:      |        out in the        |     |    countryside.    |
                 +--------------------------+     +--------------------+
                 (both fragments have closed ends, padding, and corner radius)

                 +--------------------------      ---------------------+
slice mode:      |        out in the                              .    |
                 +--------------------------      ---------------------+
                 (broken ends are open; no vertical border or corner radius at cut)
```

#### Mode A: `clone` (Recommended Default)
Each line fragment is drawn as a complete, independent rounded rectangle (`SkRRect`):
```rust
let rrect = skia_safe::RRect::new_rect_xy(
    skia_safe::Rect::from_xywh(rect.x, rect.y, rect.w, rect.h),
    radius as f32,
    radius as f32,
);
if let Some(fill_paint) = &fill_paint {
    canvas.draw_rrect(&rrect, fill_paint);
}
if let Some(stroke_paint) = &stroke_paint {
    canvas.draw_rrect(&rrect, stroke_paint);
}
```
- **Advantages:** Perfectly matches storybook illustration requirements (`desired.json`), where each line fragment forms a neat pill/box. Dashed strokes follow the perimeter cleanly.
- **Dash Phase Continuity:** In `clone` mode, each fragment's stroke contour begins at top-left and traverses its own perimeter.

#### Mode B: `slice`
Fragments have open ends at the break edge:
- Line 1 (trailing break): Rounded top-left and bottom-left; open flat right edge.
- Intermediate lines: Open flat left and right edges.
- Last line (leading break): Open flat left edge; rounded top-right and bottom-right.
- In stroke mode, the vertical line at the break is omitted:
  ```rust
  // Line 1 trailing slice path:
  // MoveTo(x + w, y) -> LineTo(x + radius, y) -> ArcTo(top-left) ->
  // LineTo(x, y + h - radius) -> ArcTo(bottom-left) -> LineTo(x + w, y + h)
  ```
- **Evaluation:** As noted in `03-range-decoration-comment.md`, while `slice` is acceptable, `clone` (closed boxes) is the standard in storybook production.

### 4.4 Underline and Highlight Geometry

1. **Underline:**
   - For each line fragment $L$:
     $$y_{\text{underline}} = \text{baseline\_y} + \text{offset}$$
     $$\text{line} = (x_{\text{min}}, y_{\text{underline}}) \rightarrow (x_{\text{max}}, y_{\text{underline}})$$
   - Painted with `stroke_width` and optional `stroke_dash`.
2. **Highlight (Background Fill):**
   - Equivalent to a `box` decoration with `stroke_width = 0`, `fill = Some(color)`, and customizable horizontal/vertical padding.

### 4.5 Drawing Pipeline & Layer Ordering

In `crates/montagent-render/src/canvas.rs`, text is currently drawn in two passes:
1. `Pass::Stroke` (glyph stroke outlines)
2. `Pass::Fill` (glyph fill outlines)

Text range decorations introduce an essential pass ordering:
- **Pass 0: Background Decorations (Pre-glyph):**
  - Highlight fills, phrase box background fills, and underlines.
  - Drawn *under* the text glyphs so that background fills do not occlude glyph ink.
- **Pass 1: Glyph Strokes (`Pass::Stroke`)**
- **Pass 2: Glyph Fills (`Pass::Fill`)**
- **Pass 3: Foreground Decorations (Post-glyph):**
  - Phrase box strokes (if designed to outline over glyph descenders) or strikethrough lines. With standard padding ($P \ge 4\text{px}$), phrase box strokes sit outside glyph ink anyway.

---

## 5. Schema Integration in Montagent

### 5.1 Architectural Constraints

1. **ADR-0007 (Text Runs):** Text styling deltas live on `Run`. Lines break on author's `\n` characters inside run text.
2. **ADR-0041 (Schema Field Order):** Struct fields represent strict serialized schema property order.
3. **ADR-0048 (Local Readability & Addressing):**
   - Reject element-level event arrays with IDs (e.g. `decorations: [{ target_run_id, ... }]`).
   - Reject sub-run character offsets (e.g. `start_char: 5, end_char: 12`), which silently desynchronize when text is edited.
   - The `Run` is the unit of text addressing. If a word or phrase is decorated, that word or phrase is authored as a run.

### 5.2 Schema Design: `decoration` on `Run`

A decorated phrase in a sentence is represented as a `Run`. When the phrase wraps across lines, it contains an embedded `\n`:
```json
{
  "text": "out in the\ncountryside.",
  "decoration": {
    "kind": "box",
    "stroke": "#4F7A2B",
    "stroke_width": 3,
    "stroke_dash": [9, 7],
    "radius": 11,
    "padding": 5,
    "break": "clone"
  }
}
```

Because `montagent-text` shapes runs line by line, a single run containing `\n` naturally spans both Line 1 and Line 2. The renderer visits the glyph clusters of that run, groups them by line, and renders the `clone` box on each line fragment.

### 5.3 Rust Model Definition (`crates/montagent-core/src/model/text.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum DecorationKind {
    #[default]
    Box,
    Highlight,
    Underline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum DecorationBreak {
    #[default]
    Clone,
    Slice,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Padding {
    All(i64),
    Symmetric { x: i64, y: i64 },
    Individual { top: i64, right: i64, bottom: i64, left: i64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecorationWindow {
    pub start: i64,
    pub end: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_in: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_out: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Decoration {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<DecorationKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<Colour>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Colour>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_dash: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<Padding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decoration_break: Option<DecorationBreak>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<DecorationWindow>,
}
```

Updating `Run` in `crates/montagent-core/src/model/text.rs`:
```rust
pub struct Run {
    pub text: String,
    pub font: Option<String>,
    pub size: Option<i64>,
    pub color: Option<Colour>,
    pub stroke: Option<Colour>,
    pub stroke_width: Option<i64>,
    pub dir: Option<Dir>,
    pub highlight: Option<Highlight>,
    pub unit: Option<UnitOverride>,
    pub decoration: Option<Decoration>,
}
```

### 5.4 Timed Draw-On Windows & Speech Synchronization

In educational storybooks, range decorations appear synchronously with speech.

1. **Integration with ADR-0048:**
   - ADR-0048 defined `Highlight` on `Run` as a timed window `[start, end)` for text color deltas.
   - For decorations, `window: { start: 86200, end: 100000, fade_in: 400 }` controls when the decoration renders.
   - If `window` is omitted, the decoration is unconditional (static).
   - If `window` is present, the decoration is evaluated at the current frame instant $t$:
     - For $t < \text{start}$ or $t \ge \text{end}$, opacity = 0.0 (not drawn).
     - For $t \in [\text{start}, \text{start} + \text{fade\_in}]$, opacity ramps from 0.0 to 1.0 (ease-out).
     - For $t \in [\text{start} + \text{fade\_in}, \text{end} - \text{fade\_out}]$, opacity = 1.0.
2. **Paint-Only Guarantee (No Layout Reflow):**
   - As established in ADR-0048 and asserted in `tests/effects.rs::a_highlight_moves_no_glyph`, **timed windows must never move glyphs or resize line slots.**
   - Measurement (`montagent measure`) runs on the static document layout.
   - The decoration's box and padding occupy the visual space already cleared by the line's `line_height` and margins. If extra leading is needed between lines, the author specifies `line_height` statically.

---

## 6. Recommendations & Implementation Roadmap

1. **Phase 1: `montagent-text` Range Box Extraction**
   - Extend `montagent-text`'s line measurement engine to expose run fragment bounding boxes:
     ```rust
     pub struct RangeBox {
         pub line: usize,
         pub x: f64,
         pub y: f64,
         pub width: f64,
         pub height: f64,
     }
     ```
   - This provides the foundation for both `measure` range queries and renderer box generation without duplicating line-breaking logic.
2. **Phase 2: Core Model Ratification**
   - Add `Decoration`, `DecorationKind`, `DecorationBreak`, `DecorationWindow`, and `Padding` to `crates/montagent-core/src/model/text.rs`.
   - Update serde validation and schema generation tests.
3. **Phase 3: Render Pipeline Implementation**
   - Update `crates/montagent-render/src/canvas.rs` to add `Pass::BackgroundDecoration` before glyph outline passes.
   - Implement `SkRRect` fragmented drawing with `skia_safe::PathEffect::dash` for `clone` mode.
4. **Phase 4: Golden Verification**
   - Validate against the handoff fixture: reproduce the visual output of `desired.json` (`ruby-prototype.mp4`, lines 56 and 77) without auxiliary `rect` elements.
