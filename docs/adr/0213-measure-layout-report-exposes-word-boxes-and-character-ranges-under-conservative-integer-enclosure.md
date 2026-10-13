---
status: accepted
amends: 0008 (measure reports break opportunities alongside layout), 0011 (measure gains layout report and range queries), 0028 (bounding box rounding follows conservative outward integer enclosure), 0029 (lines and fragments report baseline_y alongside slot extents)
---

# Measure layout report exposes word boxes and character ranges under conservative integer enclosure

[#1033](https://github.com/MBehtemam/Montagent/issues/1033), on the map [#1013](https://github.com/MBehtemam/Montagent/issues/1013). Unblocked by [#1015](https://github.com/MBehtemam/Montagent/issues/1015).

## Context

Authoring sophisticated text layouts—such as interlinear ruby glosses ([ADR-0208](./0208-ruby-text-annotations-are-run-level-objects-with-line-inflating-slot-arithmetic.md)), synchronized phrase highlight cards ([ADR-0048](./0048-text-element-runs-accept-window-decorations.md)), and wrapped text decorations—requires knowing the exact spatial bounding boxes of words and arbitrary character ranges.

Historically, `measure` ([ADR-0011](./0011-tool-surface-reads-checks-renders.md), [ADR-0024](./0024-measure-writes-the-fit-repair-not-the-verdict.md)) reported only whole-block extents and whole-line vertical ink bounds (`ink_top`, `ink_bottom`, `baseline_y`). Authors needing individual word positions were forced into two brittle paths:
1. **External PIL Fallbacks**: Authors used Python's `PIL.ImageFont` to measure words. Because basic PIL layout lacks OpenType GPOS table processing, kerning pairs, and subpixel fractional advances, errors accumulate horizontally across the line. In the benchmark fixture (`en/project-part1-A.json` at 1:28), PIL measurements diverged from Montagent's true rendered ink by up to **3.56 px** in advance coordinates and **5.00 px** in ink bounds.
2. **Prefix Batching Workaround**: Authors issued up to 58 separate `measure --elements` calls with successive substring prefixes (`text[:start]` and `text[:end]`), then manually queried font metrics tables. This imposed a 29× layout overhead, severed shaping context across word boundaries, and structurally failed on phrases wrapping across line breaks (such as `"out in the countryside."` spanning lines 0 and 1).

Furthermore, Montagent's project schema ([`schema/montagent.schema.json`](../../schema/montagent.schema.json)) requires element coordinates (`x`, `y`, `width`, `height`) to be strict integers (`E-SCHEMA`). Naive floating-point truncation (`as i64` or `round()`) routinely clips glyph ink and descenders, while loose padding creates visual collisions.

A native, single-pass layout report is required to expose exact typographic geometry while upholding the four invariants of [ADR-0145](./0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md).

---

## Decision

### 1. Unified Layout Report Schema

`measure` (and `query --at` on `text` elements) expands to support full typographic layout inspection. In the CLI, this is enabled via `--layout` (and queried via `--ranges <json>`); in MCP, via optional boolean `layout` and array `ranges` parameters.

The layout report returns:

1. **`lines[]`**: Detailed per-line geometry:
   - `index`: 0-indexed line position in the element.
   - `text`: Text string of the line.
   - `start`, `end`: Byte offsets in the element's full concatenated text.
   - `char_start`, `char_end`: Unicode scalar value offsets.
   - `box`: Integer bounding box `{x, y, width, height}` under conservative integer enclosure.
   - `advance_box`: Subpixel floating-point bounds `{x0, y0, x1, y1}`.
   - `ink_box`: Subpixel bounds of actual glyph ink `{x, y, width, height}`.
   - `baseline_y`: Absolute vertical coordinate of the baseline ([ADR-0029](./0029-line-baseline-half-leading.md)).
   - `ascent`, `descent`: Typographic vertical metrics for the line.
   - `slot_top`, `slot_height`: Slot extents in exact tenths ([ADR-0028](./0028-text-block-arithmetic-is-exact-tenths.md)).
   - `rtl`: Base line direction ([ADR-0133](./0133-a-runs-dir-is-an-isolate-and-start-and-end-follow-the-lines-own-direction.md)).
   - `break_opportunities`: Byte offsets of valid wrap positions within the line.

2. **`words[]`**: Lexical word tokens extracted via Unicode Standard Annex #29:
   - `text`: Lexical word string.
   - `start`, `end`: Byte offsets in the element text.
   - `char_start`, `char_end`: Unicode character offsets.
   - `line_index`: Index into `lines[]`.
   - `run_index`: Index into `runs[]` of the word's initial character.
   - `box`: Integer bounding box `{x, y, width, height}`.
   - `advance_box`: Subpixel floating-point bounds `{x0, y0, x1, y1}`.
   - `ink_box`: Subpixel bounds of actual glyph ink `{x, y, width, height}`.
   - `center_x`: Horizontal center point `(x0 + x1) / 2.0` for center-aligned annotation placement.
   - `baseline_y`: Baseline coordinate of the parent line.

3. **`ranges[]`**: Queryable arbitrary character spans `[start, end)`:
   - When requested with character spans (e.g. `[{"id": "phrase-1", "start": 17, "end": 39}]`), returns a list of range results.
   - Each result contains a `fragments[]` array with **one bounding box per touched line**:
     - `line_index`: Index into `lines[]`.
     - `text`: Line fragment substring.
     - `box`: Integer bounding box `{x, y, width, height}`.
     - `advance_box`: Subpixel bounds `{x0, y0, x1, y1}`.
     - `ink_box`: Subpixel ink bounds `{x, y, width, height}`.
     - `baseline_y`: Baseline coordinate for this fragment.
   - This cleanly resolves multi-line wrapped phrases by producing distinct per-line highlight geometries rather than an invalid single rectangular bounding box.

4. **`break_opportunities`**: Explicit break segmentation ([ADR-0008](./0008-line-breaks-belong-to-the-agent.md)):
   - `mandatory_breaks`: Offsets and terminators (`\n`, `\r\n`).
   - `opportunities`: Offsets of valid word-wrapping points with line associations.

```json
{
  "measure": {
    "asked": { ... },
    "line_count": 2,
    "advance_width": 349.0,
    "ascent": 36.25,
    "descent": 12.08,
    "block_top": 129.0,
    "block_bottom": 287.4,
    "block_height": 158,
    "extent": { "width": 349.0, "height": 158.4 },
    "lines": [ ... ],
    "words": [ ... ],
    "ranges": [ ... ],
    "break_opportunities": [ ... ],
    "segmenter": {
      "name": "icu_segmenter",
      "version": "2.3.0",
      "data": "icu_segmenter_data 2.3.0",
      "model": "dictionary"
    }
  }
}
```

---

### 2. Conservative Outward Integer Enclosure

Because Montagent project JSON requires integer coordinates, all reported `box` objects (`lines[].box`, `words[].box`, and `ranges[].fragments[].box`) follow the **Conservative Outward Integer Enclosure** rule:

$$\begin{aligned}
x &= \lfloor x_0 \rfloor \\
y &= \lfloor y_0 \rfloor \\
\text{width} &= \lceil x_1 \rceil - \lfloor x_0 \rfloor \\
\text{height} &= \lceil y_1 \rceil - \lfloor y_0 \rfloor
\end{aligned}$$

**Guaranteed Invariants:**
1. `box.x <= x0` (the left edge is never clipped).
2. `box.y <= y0` (the top edge is never clipped).
3. `box.x + box.width >= x1` (the right edge is never clipped).
4. `box.y + box.height >= y1` (the bottom edge is never clipped).
5. For any non-negative span $x_1 \ge x_0$, $\text{width} \ge 0$.

For center-aligned ruby annotations ([ADR-0208](./0208-ruby-text-annotations-are-run-level-objects-with-line-inflating-slot-arithmetic.md)), the continuous center is $c_x = (x_0 + x_1) / 2.0$. When converted to an integer coordinate for document placement, standard half-up rounding applies:
$$x = \lfloor c_x + 0.5 \rfloor$$

---

### 3. Single-Pass Engine Extraction Path

Extraction is executed directly from `montagent_text::place::Placement` in $O(\text{clusters})$ time during Pass 1:
- `montagent-text` already shapes text into visual clusters, glyph runs, advances, and letter spacing shifts, mapping author byte ranges into layout offsets.
- Word boundaries are identified via `unicode-segmentation` over the full element text. The clusters spanning each word are unioned in a single pass to yield `advance_box` and outline ink bounds.
- Character range queries intersect the query spans with each line's clusters, producing one fragment per touched line without invoking secondary shaping passes.
- Redundant shaper calls are eliminated: measurement and layout are evaluated in $O(1)$ shaping passes per element.

---

## Evidence & Verification

1. **Benchmark Fixture & Precision Validation**:
   - Primary evidence committed in [`docs/research/measure-layout-report/comparison.json`](../research/measure-layout-report/comparison.json): 29 words across 6 lines from `en/project-part1-A.json` (t = 79 s–89.5 s) rendered with Gentium Book Plus at size 33 px.
   - Worst absolute disagreement between Montagent advance/ink and real painted ink is **$\le 0.896\text{ px}$** (pure subpixel rasterization discretization).
   - In contrast, PIL advance drift reaches **$3.56\text{ px}$** and PIL ink error reaches **$5.00\text{ px}$**.
2. **Re-executable Invariant Check**:
   - Validated by [`docs/research/measure-layout-report/check_measure_layout_enclosure.py`](../research/measure-layout-report/check_measure_layout_enclosure.py).
   - Asserts that all five conservative integer enclosure invariants hold strictly across every benchmark word with zero clipping.
3. **Research Analysis**:
   - Comprehensive architectural survey and schema specification committed in [`docs/research/measure-layout-report.md`](../research/measure-layout-report.md).
