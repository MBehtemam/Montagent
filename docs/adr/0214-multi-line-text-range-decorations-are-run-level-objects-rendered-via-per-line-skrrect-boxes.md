---
status: accepted
amends: 0007 (a run gains an optional decoration object), 0014 (decorations introduce Pass 0: BackgroundDecorations before glyph outline passes), 0048 (generalizes run-level windowed decorations to multi-line boxes, highlights, and underlines), 0145 (capability enters via Premiere Pro and CSS precedent under the entry test)
---

# Multi-line text range decorations are run-level objects rendered via per-line SkRRect boxes

[#1034](https://github.com/MBehtemam/Montagent/issues/1034), on the map [#1013](https://github.com/MBehtemam/Montagent/issues/1013). Unblocked by [#1016](https://github.com/MBehtemam/Montagent/issues/1016).

## Context

In educational, storybook, and subtitle video production (e.g. Montagent's reference benchmark `en/project-part1-A.json` at 1:28), multi-word phrases frequently break across line wraps and require typographic range decorations—such as dashed phrase boundary boxes, background highlights, or underlines.

Historically, Montagent lacked inline range decoration primitives. Authors attempting to highlight or box a phrase were forced to dismantle sentences into fragmented text elements and manually construct bounding `rect` elements for each line fragment:
1. **Sentence Fragmentation**: In the 1:28 benchmark, the phrase `"out in the countryside."` wraps across lines (`"out in the"` on Line 1, `"countryside."` on Line 2). To draw dashed boxes around each fragment, the sentence was artificially severed into two distinct `text` elements (`c01-01-text` at `y: 129` and `text-l1b` at `y: 208`).
2. **Bounding-Union Blowout**: A naive single bounding box spanning across lines covers $[\min(x_1, x_2), \max(x_1+w_1, x_2+w_2)]$ across the vertical extent $[y_1, y_2+h_2]$. This creates an oversized rectangular box that engulfs unrelated words on both lines and empty inter-line leading space, destroying typographic legibility.
3. **Manual Coordinate Calculation**: Authors had to manually derive advance widths (`691.71` to `836.05` on Line 1; `477.00` to `660.01` on Line 2) and author separate `type: rect` elements (`b-phrase-a`, `b-phrase-b`) with manual opacity keyframes to match speech timing.

### ADR-0145 Precedent Analysis & Entry Test Compliance

As documented in [`docs/research/text-range-decorations.md`](../research/text-range-decorations.md), multi-line range decorations enter Montagent in accordance with [ADR-0145](./0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md):

1. **Precedent in Reference Editors and Web Standards**:
   - **Adobe Premiere Pro**: Premiere's Essential Graphics panel provides a Text Background feature toggled to **`Per-Line`**, which dynamically creates independent background boxes for each individual line conforming to that line's text bounds rather than a single unified bounding box. Premiere's Underline is a character-level property that fragments along the baseline across line wraps.
   - **CSS Fragmentation Module Level 3**: W3C defines `box-decoration-break: clone | slice` for inline boxes fragmented across line wraps. `box-decoration-break: clone` independently wraps every line fragment with complete padding, borders, corner radii, and background fills.
   - **Adobe InDesign**: Paragraph Shading with `Width: Text` does *not* create per-line boxes; typographers use thick character underlines, which fragment across lines but cannot provide four-sided closed borders or corner radii.

2. **Compliance with the ADR-0145 Entry Test**:
   - **Literal Values**: Every decoration parameter (`kind`, `fill`, `stroke`, `stroke_width`, `stroke_dash`, `radius`, `padding`, `decoration_break`, and `window`) is declared as an explicit literal. No runtime auto-layout solvers or elastic springs negotiate placement.
   - **Closed Vocabulary**: The decoration schema uses strictly enumerated properties (`kind: "box" | "highlight" | "underline"`, `decoration_break: "clone" | "slice"`) and bounded numeric parameters.
   - **Checkable by `validate`**: All coordinates, padding, and timed windows are checkable from the project file alone without executing external layout engines or heavy pipelines.
   - **Exact-String Replace**: The decoration configuration is co-located directly on the annotated `Run` object within `runs[]`, preserving isolated read-modify-write ergonomics.

---

## Decision

### 1. Run-Level Decoration Schema

A `Run` object in a `text` element's `runs[]` array gains an optional `decoration` object ([ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) amended):

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
    "decoration_break": "clone",
    "window": {
      "start": 86200,
      "end": 100000,
      "fade_in": 400
    }
  }
}
```

#### Field Specifications

- **`kind`** (String enum, optional):  
  The visual decoration style. Closed enum: `"box"`, `"highlight"`, `"underline"`. Defaults to `"box"`.
  - `"box"`: A closed rectangular or rounded boundary box around the line fragment.
  - `"highlight"`: A solid background fill behind the text ink.
  - `"underline"`: A horizontal rule drawn along the fragment's baseline.
- **`fill`** (Colour, optional):  
  Background fill colour (`#RRGGBB` or `#RRGGBBAA`).
- **`stroke`** (Colour, optional):  
  Border or underline stroke colour (`#RRGGBB` or `#RRGGBBAA`). Defaults to element text colour if omitted for underlines.
- **`stroke_width`** (Integer, optional):  
  Stroke thickness in integer pixels. Defaults to `1` for `"box"` and `"underline"`.
- **`stroke_dash`** (Array of integers, optional):  
  Alternating lengths of dashes and gaps in integer pixels (e.g. `[9, 7]`). If omitted, stroke is solid.
- **`radius`** (Integer, optional):  
  Corner radius in integer pixels for rounded boxes. Defaults to `0` (sharp corners).
- **`padding`** (Padding, optional):  
  Outward expansion beyond the glyph advance bounds in integer pixels. May be specified as:
  - Integer scalar: `5` (uniform padding on all four edges).
  - Symmetric object: `{"x": 8, "y": 4}`.
  - Individual object: `{"top": 4, "right": 8, "bottom": 4, "left": 8}`.
  Defaults to `0`.
- **`decoration_break`** (String enum, optional):  
  Behavior when the decorated run spans across line breaks (`\n`). Closed enum:
  - `"clone"` (default): Independently wraps each line fragment with complete padding, borders, corner radii, and background fills.
  - `"slice"`: Renders each fragment as an open-ended slice, omitting border strokes and corner radii along the break edges.
- **`window`** (DecorationWindow, optional):  
  Timed visibility window for speech synchronization ([ADR-0048](./0048-per-word-highlighting-is-a-timed-window-on-the-run.md) generalized):
  - `start`, `end`: Integer milliseconds within the element's active range.
  - `fade_in`, `fade_out`: Optional integer milliseconds for opacity transitions.
  If omitted, the decoration is unconditional (static).

#### Rust Model (`crates/montagent-core/src/model/text.rs`)

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

---

### 2. Per-Line Rendering Geometry in Skia

For wrapped runs, layout geometry is extracted per touched line fragment (leveraging the range fragmentation engine established in [ADR-0213](./0213-measure-layout-report-exposes-word-boxes-and-character-ranges-under-conservative-integer-enclosure.md)).

#### Fragment Box Geometry

For each line fragment $l$ touched by the decorated run:
1. **Advance Bounds**: The unpadded fragment bounds $[x_0, x_1]$ span from the start of the first glyph cluster on line $l$ to the end of the last glyph cluster on line $l$.
2. **Vertical Extents**: The base height $h_{\text{base}}$ spans from line slot top to bottom, or from $(\text{baseline} - \text{ascent})$ to $(\text{baseline} + \text{descent})$.
3. **Padded Box**:
   $$\begin{aligned}
   x_{\text{box}} &= x_0 - \text{pad}_{\text{left}} \\
   y_{\text{box}} &= y_0 - \text{pad}_{\text{top}} \\
   w_{\text{box}} &= (x_1 - x_0) + \text{pad}_{\text{left}} + \text{pad}_{\text{right}} \\
   h_{\text{box}} &= h_{\text{base}} + \text{pad}_{\text{top}} + \text{pad}_{\text{bottom}}
   \end{aligned}$$
4. **Skia Primitives**:
   - Each fragment is constructed as an `skia_safe::RRect` with corner radius $r = \text{radius}$.
   - Under `decoration_break: clone`, all four corners of each fragment are rounded and bordered.
   - Under `decoration_break: slice`, interior break edges are clipped flat without border strokes.
   - Dashed borders apply `skia_safe::PathEffect::dash(&stroke_dash, 0.0)`.

#### Render Pass Order (`crates/montagent-render/src/canvas.rs`)

To ensure that decoration fills never occlude text ink, text rendering pipeline passes are strictly ordered:

- **Pass 0: Background Decorations (`Pass::BackgroundDecorations`)**:
  - Highlights, phrase box fills, and baseline underlines drawn *prior* to glyph outlines.
- **Pass 1: Glyph Strokes (`Pass::Stroke`)**:
  - Outward glyph stroke contours ([ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md)).
- **Pass 2: Glyph Fills (`Pass::Fill`)**:
  - Solid glyph text fills.
- **Pass 3: Foreground Decorations (`Pass::ForegroundDecorations`)**:
  - Phrase box border strokes (with standard padding $P \ge 4\text{ px}$, strokes sit outside glyph ink anyway).

---

### 3. Invariants & Speech Synchronization

1. **Strict Adherence to ADR-0048 (Paint-Only Invariant)**:
   - Text range decorations are strictly **paint passes** and **never** alter line slot height, baseline placement, or cause text reflow.
   - If vertical clearance is needed between lines for thick borders or padding, the author specifies `line_height` statically on the text element (or uses line-inflating ruby annotations per [ADR-0208](./0208-ruby-text-annotations-are-run-level-objects-with-line-inflating-slot-arithmetic.md)).
2. **Speech Synchronization (`window`)**:
   - The timed window $[t_{\text{start}}, t_{\text{end}})$ allows decorations to light up synchronously with audio narration.
   - For frame instant $t$:
     - If $t < \text{start}$ or $t \ge \text{end}$: opacity is $0.0$ (not rendered).
     - If $t \in [\text{start}, \text{start} + \text{fade\_in}]$: opacity ramps from $0.0$ to $1.0$ (ease-out).
     - If $t \in [\text{start} + \text{fade\_in}, \text{end} - \text{fade\_out}]$: opacity is $1.0$.
     - If $t \in [\text{end} - \text{fade\_out}, \text{end}]$: opacity ramps from $1.0$ to $0.0$.
   - Like ADR-0048 highlights, timed windows evaluate at render time and move no glyphs.
3. **Run-Level Addressing (No Event Arrays or Sub-Run Offsets)**:
   - Consistent with [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) and [ADR-0048](./0048-per-word-highlighting-is-a-timed-window-on-the-run.md), the `Run` is the atomic unit of text addressing.
   - If a word or phrase is decorated, that phrase is authored as a run. When a phrase wraps across lines, it contains an explicit literal `\n`.

---

### 4. Non-Goals

1. **No Runtime Reflow or Elastic Layout Negotiation**:
   The renderer never modifies line breaks or adjusts box padding dynamically to avoid collisions.
2. **No Arbitrary Polygon Hull Union**:
   Wrapped phrases do not construct complex non-convex concave polygon hulls. Discrete per-line fragmented rectangular boxes match typographic precedent and render cleanly across all display densities.
3. **No External Alignment Files**:
   Timed windows are authored directly as integer millisecond literals frozen in the document.

---

## Evidence & Verification

1. **Reference Production Benchmark**:
   - Production failure analyzed from `scratch/handoff/ISSUE-DRAFTS/03-range-decoration-comment.md` and `scratch/handoff/prototype/desired.json`.
   - Verified that the wrapped phrase `"out in the countryside."` from English Part 1 at 1:28 ($t = 86.2\text{ s}$ to $100.0\text{ s}$) produces discrete per-line dashed boxes ($[691, 838]$ on Line 1, $[477, 661]$ on Line 2) exactly matching the accepted prototype video `handoff/prototype/ruby-prototype.mp4`.
2. **Research Synthesis**:
   - Comprehensive comparative survey across Premiere Pro, CSS Fragmentation Module Level 3, and InDesign committed in [`docs/research/text-range-decorations.md`](../research/text-range-decorations.md).
