---
status: accepted
amends: 0007 (a run gains an optional annotation object), 0028 (slot inflation arithmetic is evaluated in exact tenths), 0029 (baseline placement in the presence of slot inflation), 0145 (capability enters via entry test with accepted prototype)
---

# Ruby text annotations are run-level objects with line-inflating slot arithmetic

[#1025](https://github.com/MBehtemam/Montagent/issues/1025) and [#1035](https://github.com/MBehtemam/Montagent/issues/1035), on the map [#1013](https://github.com/MBehtemam/Montagent/issues/1013). Unblocked by [#1014](https://github.com/MBehtemam/Montagent/issues/1014).

## Context

Interlinear text annotations—commonly termed **ruby text** in East Asian typography (such as Furigana over Kanji or Pinyin over Hanzi) and glosses in language education—place secondary explanatory text directly adjacent to a base word or phrase. In language-learning videos, such as the Montagent benchmark fixture (`en/project-part1-A.json` at 1:28), annotations frequently overlap or stack into multiple tiers (e.g., grammatical commentary plus vocabulary definitions over a single base phrase).

Historically, Montagent lacked any interlinear annotation primitive. Authors attempting to place glosses were forced to create separate floating text elements, manually computing spatial coordinates using external scripts (such as Python PIL). This created severe layout fragility:
1. **Vertical Collisions**: In the English Part 1 fixture at 1:28, the gloss `"stood is missing"` (centred at `y: 359`) was painted directly on top of Row 1's ink (`y: 341–374`), because the vertical space between Row 1 and Row 2 accommodated only one tier of text, whereas two tiers were required.
2. **Horizontal Drift**: When fonts change or letter spacing is modified, separate text elements drift out of alignment with the base words they annotate.
3. **Leading Starvation**: Base line slots defined under [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) and [ADR-0028](./0028-text-block-arithmetic-is-exact-tenths.md) reserve space strictly as `size × line_height`, providing no mechanism to expand leading when annotations protrude beyond the slot boundary.

### ADR-0145 Precedent Analysis & Entry Test Compliance

Neither Premiere Pro nor CapCut provides a native interlinear or ruby annotation model. In both applications, interlinear glosses must be manually composed as unlinked title layers or static graphic overlays.

Because no precedent exists in the reference editors, this capability enters Montagent strictly via the **Entry Test** established in [ADR-0145](./0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md):

1. **The Four Invariants**:
   - **Literal Values**: Every annotation tier, gap, size, overhang, and position is declared as an explicit literal. No runtime auto-layout solvers or elastic springs negotiate placement.
   - **Closed Vocabulary**: The annotation schema uses strictly enumerated properties (`position: "over" | "under"`, `align: "center" | "start" | "end"`) and bounded numeric parameters.
   - **Checkable by `validate`**: Slot inflation is governed by a deterministic closed formula evaluated in exact tenths, and horizontal overhang/overlap is checkable statically before rendering.
   - **Exact-String Replace**: The annotation configuration is co-located directly on the annotated `Run` object within `runs[]`, preserving isolated read-modify-write ergonomics.

2. **Rendered Prototype & Formal Owner Acceptance**:
   As required by ADR-0145, a rendered prototype video demonstrating multi-tier stacking and deterministic line-slot inflation was submitted in [#1014](https://github.com/MBehtemam/Montagent/issues/1014):
   - **Artifacts**: `handoff/prototype/ruby-prototype.mp4` (27.5 s, 960×540), `frame-88000-annotated.png`, and `frame-88000-intended.png`.
   - **Evaluation**: The prototype resolved the 1:28 collision by stacking Tier 1 gloss `"a kind of grain"` (`y: 436`) and Tier 2 gloss `"stood is missing"` (`y: 402`) above Row 2, applying `+53 px` of line leading inflation to push subsequent rows downward and completely clear Row 1 ink.
   - **Owner Acceptance Statement (quoted verbatim from [#1014](https://github.com/MBehtemam/Montagent/issues/1014#issuecomment-6146343879))**:
     > *"Owner Acceptance: The rendered prototype video demonstrates deterministic, collision-free typographic placement that upholds Montagent's declarative model. The multi-tier stacking and line-inflation arithmetic satisfy all requirements of ADR-0145. Owner acceptance is hereby granted to graduate Ruby Annotations to ADR specification."*

---

## Decision

### 1. Run-Level Annotation Schema

A `Run` object in a `text` element's `runs[]` array gains an optional `annotation` object ([ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) amended):

```json
{
  "text": "the oats green,",
  "annotation": {
    "text": "stood is missing",
    "font": "body-italic",
    "size": 28,
    "color": "#FFCC00",
    "position": "over",
    "align": "center",
    "tier": 1,
    "gap": 6,
    "overhang": 0
  }
}
```

#### Field Specifications

- **`text`** (String, required):  
  The annotation text content. Must be non-empty. Follows NFC normalization and UTF-8 encoding.
- **`font`** (String, optional):  
  A semantic font chain name declared in the project's `fonts` table. If omitted, inherits the parent run's `font` (or the element's default `font`).
- **`size`** (Integer, optional):  
  Font size of the annotation in integer pixels. If omitted, defaults to `ceil(base_run_size × 0.5)` (half the base run's declared size).
- **`color`** (Colour, optional):  
  The fill colour of the annotation text (`#RRGGBB` or `#RRGGBBAA`). If omitted, inherits the parent run's `color` (or the element's `color`).
- **`position`** (String enum, optional):  
  Placement relative to the base text line. Closed enum: `"over"` (above the line) or `"under"` (below the line). Defaults to `"over"`.
- **`align`** (String enum, optional):  
  Horizontal alignment relative to the base run's advance box. Closed enum: `"center"`, `"start"`, `"end"`. Defaults to `"center"`.
- **`tier`** (Integer, optional):  
  Non-negative integer index (`0, 1, 2, …`) defining the vertical stacking tier. Defaults to `0` (tier closest to base text). Higher tiers stack progressively further away from the base line.
- **`gap`** (Integer, optional):  
  Distance in integer pixels between the base text ink boundary and Tier 0 ink, or between successive stacked tiers. Defaults to `6`.
- **`overhang`** (Integer, optional):  
  Maximum integer pixels that the annotation ink may extend beyond the horizontal start or end bounds of the base run without triggering a validation finding. Defaults to `0`.

---

### 2. Deterministic Line Slot Inflation Arithmetic

Under [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) and [ADR-0028](./0028-text-block-arithmetic-is-exact-tenths.md), each line $l$ reserves a base slot:

$$\text{base\_slot\_height}_l = \left\lceil \frac{\max_{r \in l}(\text{size}_r) \times n}{10} \right\rceil \quad \text{where } n = \text{line\_height} \times 10$$

When a line carries annotations, the slot is inflated along the corresponding direction so that annotations never collide with adjacent lines or clip outside the text block.

#### Inflation Formula (Evaluated in Exact Tenths)

For a given line $l$ and vertical direction $d \in \{\text{over}, \text{under}\}$:

1. **Required Stacking Extent**:  
   Let $T_d$ be the count of active tiers in direction $d$ on line $l$, defined as:
   $$T_d = \begin{cases} 0 & \text{if no annotations in direction } d \\ \max_{a \in \text{ann}_{l, d}}(a.\text{tier}) + 1 & \text{otherwise} \end{cases}$$
   
   For each tier $t \in [0, T_d - 1]$, let $\text{pitch}_t$ be the maximum vertical pitch among annotations on that tier:
   $$\text{pitch}_t = \max_{a \in \text{tier}_{t}} (a.\text{size}) + \max_{a \in \text{tier}_{t}} (a.\text{gap})$$
   The total required annotation clearance is:
   $$\text{required\_extent}_d = \sum_{t=0}^{T_d - 1} \text{pitch}_t$$

2. **Available Leading in Base Slot**:  
   From [ADR-0029](./0029-line-baseline-half-leading.md), the base line slot distributes leading symmetrically as half-leading. The uninflated clearance between font ink and slot boundary is:
   $$\text{available\_leading}_d = \max\left(0, \left\lfloor \frac{\text{base\_slot\_height}_l - (\text{ascent}_l + \text{descent}_l)}{2} \right\rfloor\right)$$
   where $\text{ascent}_l$ and $\text{descent}_l$ are the maximum vertical font metrics across all runs on line $l$.

3. **Extra Leading Required**:  
   $$\text{extra\_leading}_d = \max\left(0, \text{required\_extent}_d - \text{available\_leading}_d\right)$$

4. **Total Inflated Line Slot Height**:  
   $$\text{slot\_height}_l = \text{base\_slot\_height}_l + \text{extra\_leading}_{\text{over}, l} + \text{extra\_leading}_{\text{under}, l}$$

5. **Total Text Block Height**:  
   The cumulative height of the text block is the sum of all inflated line slots:
   $$\text{block\_height} = \sum_{l=1}^{\text{line\_count}} \text{slot\_height}_l$$
   All arithmetic is evaluated in integer tenths and ceiled to integer pixels, upholding the invariant from [ADR-0028](./0028-text-block-arithmetic-is-exact-tenths.md).

---

### 3. Baseline & Placement Geometry

Baseline positioning preserves half-leading symmetry ([ADR-0029](./0029-line-baseline-half-leading.md) amended) while accommodating extra leading:

1. **Base Text Baseline Placement**:  
   The top of line slot $l$ within the element block is:
   $$\text{slot\_top}_l = \sum_{i < l} \text{slot\_height}_i$$
   The base text baseline for line $l$ is placed at:
   $$\text{baseline\_y}_l = \text{slot\_top}_l + \text{extra\_leading}_{\text{over}, l} + \left\lfloor \frac{\text{base\_slot\_height}_l + (\text{ascent}_l - \text{descent}_l)}{2} \right\rfloor$$
   This ensures that `extra_leading_over` displaces the base text downward, guaranteeing that annotations placed `"over"` remain entirely inside the line slot and never encroach upon preceding lines.

2. **Annotation Baseline Placement**:  
   For an annotation $a$ on run $r$ on line $l$:
   - If $a.\text{position} == \text{"over"}$:
     $$\text{ann\_baseline\_y} = \text{baseline\_y}_l - \text{ascent}_l - a.\text{gap} - \sum_{k=0}^{a.\text{tier} - 1} \text{pitch}_k - a.\text{descent}$$
   - If $a.\text{position} == \text{"under"}$:
     $$\text{ann\_baseline\_y} = \text{baseline\_y}_l + \text{descent}_l + a.\text{gap} + \sum_{k=0}^{a.\text{tier} - 1} \text{pitch}_k + a.\text{ascent}$$

3. **Horizontal Alignment**:  
   Let $[x_{\text{start}}, x_{\text{end}}]$ be the shaped horizontal advance of the base run $r$, with width $W_r = x_{\text{end}} - x_{\text{start}}$, and let $W_a$ be the shaped advance width of the annotation text:
   - `align: "center"`:  
     $$x_a = x_{\text{start}} + \left\lfloor \frac{W_r - W_a}{2} \right\rfloor$$
   - `align: "start"`:  
     $$x_a = x_{\text{start}} \quad (\text{in LTR}; \text{right edge in RTL})$$
   - `align: "end"`:  
     $$x_a = x_{\text{end}} - W_a \quad (\text{in LTR}; \text{left edge in RTL})$$

4. **Overhang & Same-Tier Overlap Geometry**:  
   - **Overhang (`R-ANNOTATION-OVERHANG`)**: If $W_a > W_r$, the annotation extends horizontally past the base run by:
     $$\text{overhang}_{\text{left}} = \max(0, x_{\text{start}} - x_a), \quad \text{overhang}_{\text{right}} = \max(0, (x_a + W_a) - x_{\text{end}})$$
     If $\max(\text{overhang}_{\text{left}}, \text{overhang}_{\text{right}}) > a.\text{overhang}$, `validate` reports `R-ANNOTATION-OVERHANG`.
   - **Same-Tier Overlap (`R-ANNOTATION-OVERLAP`)**: For any two annotations $a_1, a_2$ ($a_1 \neq a_2$) on the same `(line, position, tier)`, let their horizontal intervals be $[x_{a_1}, x_{a_1} + W_{a_1}]$ and $[x_{a_2}, x_{a_2} + W_{a_2}]$. If their horizontal intersection is strictly positive:
     $$\text{overlap\_px} = \min(x_{a_1} + W_{a_1}, x_{a_2} + W_{a_2}) - \max(x_{a_1}, x_{a_2}) > 0$$
     `validate` reports `R-ANNOTATION-OVERLAP`.

---

### 3.5. Validation Seam Placement & Performance Floor (`#1035`)

Because horizontal collision (`R-ANNOTATION-OVERLAP`) and overhang (`R-ANNOTATION-OVERHANG`) depend on shaped glyph advance widths ($W_r$ and $W_a$), [#1035](https://github.com/MBehtemam/Montagent/issues/1035) resolves which tool seam owns the check and establishes the performance floor for font shaping in `validate`:

1. **Seam Placement — `validate` (`CheckSet::Document`) Evaluates Verdicts; `measure --layout` Reports Raw Boxes**:
   - **Selected Seam (`validate` inside `CheckSet::Document`)**: Both `R-ANNOTATION-OVERLAP` and `R-ANNOTATION-OVERHANG` execute unconditionally in `validate` (`crates/montagent-core/src/checks/ink.rs`, invoked by `verbs::validate::run_checks` in `CheckSet::Document`), while `measure --layout` exposes the raw derived `annotations[]` bounding boxes (`x`, `y`, `width`, `height`, `tier`, `position` per [ADR-0213](./0213-measure-layout-report-exposes-word-boxes-and-character-ranges-under-conservative-integer-enclosure.md)) with **no verdict**. This mirrors the exact architectural split established by [ADR-0087](./0087-thai-line-height-collision-is-a-font-selection-problem.md), where `measure` reports raw `ink_seams` and `validate` emits `R-LINE-INK-COLLISION`, and satisfies [ADR-0145](./0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md) Invariant 3 (*"Checkable by `validate` from the file and its probed assets alone"*).
   - **Why `measure` (as an advisory finding) is refused**: [ADR-0024](./0024-measure-writes-the-fit-repair-not-the-verdict.md) (unanimous 3/3 court, reaffirmed in [ADR-0087](./0087-thai-line-height-collision-is-a-font-selection-problem.md) and [ADR-0088](./0088-the-contrast-verdict-is-refused-and-validate-checks-for-self-erasure-instead.md)) establishes that `measure` returns derived extents and fit repairs, never verdicts or findings about the document. Furthermore, [ADR-0112](./0112-a-report-names-the-check-sets-that-ran-and-prints-no-zero-it-did-not-earn.md) §2 fixes `measure`'s completed check sets as empty (`check_sets: []`).
   - **Why a dedicated `montagent lint` verb is refused**: [ADR-0011](./0011-tool-surface-reads-checks-renders.md) fixes the minimal verb surface and rejects redundant verbs, while [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) and [ADR-0041](./0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md) reject opt-in lint commands because *"an opt-in check you might not run manufactures false confidence."* Keeping the checks in `validate` ensures they run unconditionally on `validate` and during pre-flight in `render`, `preview`, and `shift` (`verbs::validate::checked`).

2. **Check Registry Specification**:
   Both codes register in `crates/montagent-core/src/registry.rs` with:
   - `classes: &[Class::Review]` — overlapping or overhanging glosses are legible in some contexts and defective in others; `validate` surfaces the exact pixel overlap/overhang for human or agent review without blocking `render` ([ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)).
   - `repair: None` — multiple valid authorial repairs exist (incrementing `tier`, adjusting `align`, `overhang`, or `size`, or splitting runs); per [ADR-0043](./0043-finding-carries-its-repair-only-where-the-document-decides-the-direction.md) and [ADR-0087](./0087-thai-line-height-collision-is-a-font-selection-problem.md), the document does not determine which branch the author intends.
   - `threshold: ThresholdProvenance::Internal` — `R-ANNOTATION-OVERLAP` fires on strictly positive horizontal overlap (`overlap_px > 0`) on the same `(line, position, tier)`, and `R-ANNOTATION-OVERHANG` fires when horizontal extension exceeds the author-declared `a.overhang` integer (`max(overhang_left, overhang_right) > a.overhang`).
   - `sets: &[CheckSet::Document]` — evaluated across `written_letter_spacings(element)` (initial and keyframe endpoint `letter_spacing` values, matching `R-LINE-INK-COLLISION` under [ADR-0151](./0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)).

3. **Permissibility & Performance Floor of Font Shaping in `validate`**:
   - **In-Process Hermetic Font Shaping**: [ADR-0112](./0112-a-report-names-the-check-sets-that-ran-and-prints-no-zero-it-did-not-earn.md) §2 defines `CheckSet::Document` as *"`validate`'s checks that read only the file and its fonts."* Three existing `validate` check modules already open the project's declared font files and invoke `montagent-text` (Parley + Skrifa) in-process with zero subprocess spawns: `checks::fonts` (`E-FONT-NO-GLYPH`, `E-FONT-UNATTESTED`, `N-FONT-CENSUS`, `R-FONT-SWAP` — [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md), [ADR-0057](./0057-font-census-and-swap-review-split.md)), `checks::units` (`E-UNIT-RUN-MERGED`, `R-UNIT-MERGED` — [ADR-0151](./0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)), and `checks::ink` (`R-LINE-INK-COLLISION` — [ADR-0087](./0087-thai-line-height-collision-is-a-font-selection-problem.md)). Under [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md), `fontique`'s system-font discovery is not compiled into `montagent-text`; only fonts declared in the project's `fonts` table are loaded, making shaping in `validate` strictly hermetic and deterministic.
   - **Performance Floor**:
     - *Unannotated fast path (`0 µs` overhead)*: Elements where no `Run` carries `annotation` bypass annotation shaping and interval comparisons via an $O(\text{runs})$ guard.
     - *Shared single-pass shaping*: `checks::ink::check` already calls `montagent_text::measure` for every text element. Shaping $k$ short annotation strings reuses the already-loaded `Fonts` and `LayoutContext<u32>` in memory during the same pass, adding $< 0.1\text{ ms}$ per annotation and $O(k^2)$ integer interval comparisons per `(line, position, tier)` (where $k$ is typically $1\text{–}6$).
     - *Measured floor*: All 21 ink integration tests (`cargo test -p montagent-core --test ink`), including full-engine `validate` over the 22-text-element reference fixture (`fixtures/en-halloween-decorating`), complete in `110 ms` total in unoptimized debug builds (`~5 ms` per `validate` call). Annotation validation must add `0 ms` to unannotated elements and keep `validate` latency under `< 10 ms` per annotated element in debug builds (`< 2 ms` in release builds), with zero external process spawns.

---

### 4. Non-Goals and Architectural Boundaries

1. **No Runtime Layout Negotiation or Automatic Tier Packing**:  
   The renderer never shifts tiers, alters declared gaps, or computes automatic collision avoidance. All tier indices and positions are declared literals. If two annotations on the same tier overlap in space, `validate` reports `R-ANNOTATION-OVERLAP`; the authoring agent resolves the conflict by incrementing `tier` or splitting runs.
2. **No Furigana Phonetics Engine**:  
   Montagent contains no Japanese dictionary, morphological analyzer (e.g., MeCab), or Pinyin generator. All annotation text must be authored explicitly.
3. **No Vertical Writing Modes**:  
   Annotations are defined strictly for horizontal text lines (LTR and RTL). Vertical text layout remains out of scope.
4. **No Multi-Run Spanning Annotations**:  
   An annotation attaches strictly to a single `Run`. If a phrase spans multiple words, the phrase must be authored as a single `Run` in `runs[]`.

---

## Consequences

- **Schema Extension**: `schema/montagent.schema.json` admits the optional `annotation` object on `Run`.
- **Determinism**: Given identical project JSON and fonts, all annotation bounding boxes, baselines, and line-slot inflations resolve to identical integer coordinates across all platforms.
- **Validation & Measurement Split**: `validate` (`CheckSet::Document`) evaluates `R-ANNOTATION-OVERLAP` and `R-ANNOTATION-OVERHANG` (`Class::Review`, `repair: None`, `ThresholdProvenance::Internal`) in-process using `montagent-text`, while `montagent measure --layout` reports raw derived `annotations[]` bounding boxes without verdicts ([ADR-0024](./0024-measure-writes-the-fit-repair-not-the-verdict.md)).
- **Backward Compatibility**: Existing text elements without annotations produce identical line heights, slot heights, and baseline positions with zero validation overhead.
