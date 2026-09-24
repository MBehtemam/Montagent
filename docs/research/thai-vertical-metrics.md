# Thai's `line_height` collision: `OS/2` typo metrics, a script-aware floor, or a font-selection problem

Research for [#325](https://github.com/MBehtemam/Montagent/issues/325), part of the map
[#2](https://github.com/MBehtemam/Montagent/issues/2). Picks up
[#130](https://github.com/MBehtemam/Montagent/issues/130), which measured the defect
against a macOS system font and stopped there by design.

**Date of research:** 2026-09-24. Version numbers and font versions are as of that date.

**Sourcing rule applied**, per [`docs/agents/domain.md`](../agents/domain.md): every claim
cites the source that owns it — the OpenType spec for what a table field means, the crate
source for what the code does, the font binary for what a face declares, and the probe for
what the ink does. Three labels are used throughout and never mixed:

- **MEASURED** — produced by [`prototypes/thai-vertical-metrics/`](prototypes/thai-vertical-metrics/),
  re-executable via `./run.sh`, which re-asserts every headline number below and exits
  non-zero if one stops reproducing.
- **READ** — quoted verbatim from a primary source, with its URL or file path and line.
- **INFERRED** — reasoning over the two above. Never presented as either.

Scope is the **vertical axis only**. Literal `size`
([ADR-0007](../adr/0007-text-runs-literal-size-declared-fonts.md)) is not reopened, and
auto-wrap is settled by [ADR-0008](../adr/0008-line-breaks-belong-to-the-agent.md).

---

> **Amended by two courts — read this first.** Every number below is a **whole-line** ink
> seam: the lowest ink anywhere on one line against the highest ink anywhere on the next.
> That instrument was later measured to over-report by 1–4 tenths of `line_height`, because
> it counts a descender at one end of a line as colliding with a mark at the other. The
> floors in §5 are therefore **1–4 tenths too high**, and the "+9.58 px" headline is a
> bounding-box figure whose x-aware value is +0.61 px. What this document was written to
> establish — that candidate 1 moves nothing and candidate 2 has no constant to carry — is
> unaffected and was confirmed 3/3. See
> [`juries/thai-vertical-metrics/`](juries/thai-vertical-metrics/README.md),
> [`juries/thai-ink-seam-x-aware/`](juries/thai-ink-seam-x-aware/README.md), and
> [ADR-0087](../adr/0087-thai-line-height-collision-is-a-font-selection-problem.md) for what
> shipped.

## 1. Summary

1. **Candidate 1 is already implemented, and it changes nothing.** `skrifa` — the only
   thing in this repo's dependency graph that arbitrates vertical metrics — *already*
   honours the `OS/2` `fsSelection` bit 7 `USE_TYPO_METRICS` flag and *already* prefers
   `sTypoAscender`/`sTypoDescender`/`sTypoLineGap` when it is set (§3, READ). Both
   vendorable Thai faces measured here set bit 7, so the stack is already on the typo path.
   And on both faces `hhea.ascender/descender/lineGap` are **field-for-field identical** to
   the `sTypo*` values (§4, MEASURED), so even a font that did not set the bit would move
   zero pixels. Candidate 1 is a no-op, twice over.
2. **Even the generous reading of candidate 1 fails.** If "read the `OS/2` typo metrics" is
   read as *derive the line slot from them* — which would reverse ADR-0007's literal-size
   decision and is therefore out of scope anyway — it still does not close the gap. On
   Sarabun the typo sum is `1.300 em`, and at `line_height 1.300` real Thai ink still
   collides by **+12.77 px** at size 55 (§5, MEASURED). It falls short by three whole tenths.
   The only metric that would clear Sarabun is `usWinAscent + usWinDescent` (`1.853 em`) —
   and the OpenType spec says in as many words that using the `usWin*` values for line
   spacing is *"strongly discouraged"* (§2, READ).
3. **The collision is real on vendorable fonts, and `1.1` is far below the floor.**
   MEASURED at size 55, ADR-0029's baseline rule, real glyph-outline ink:

   | Face | Thai floor (first clean tenth) | `1.1` seam | `1.2` seam | Latin control floor |
   | --- | --- | --- | --- | --- |
   | Noto Sans Thai 2.002 | **1.3** | +9.58 px | +4.08 px | 1.0 |
   | Sarabun 1.000 | **1.6** | +23.77 px | +18.27 px | 1.0 |

   Positive = ink physically overlaps ink. The Latin control runs *in the same face*, so the
   variable is the script, not the font.
4. **The floor is font-specific, not script-specific.** Two OFL Thai faces, same strings,
   same size, same rule: 1.3 and 1.6. A *script*-aware floor would have to be set at the
   worse of the two to be safe, which is 23 % looser than Noto Sans Thai needs and 60 % looser than Latin — and there
   is no bound on how much worse a third face could be. This is the finding that decides
   the ticket (§6, MEASURED + INFERRED).
5. **A nameable, vendorable face exists and passes this repo's own gate.** `Noto Sans Thai
   2.002`, OFL-1.1, was vendored by the shipped `montagent fonts vendor` binary with
   `licence OFL-1.1 (recognised)` — ADR-0057 bucket 2, no `--licence` declaration needed
   (§4.3, MEASURED). Sarabun 1.000 passes identically.
6. **`validate` is blind to all of it, confirmed against the shipped binary.** A project
   with the colliding Thai element, formatted, validates **0 errors, 0 reviews** (§7,
   MEASURED). `measure` reports the ascent/descent and the two baselines 60.5 px apart, and
   nothing about ink.
7. **The recommendation is candidate 3, sharpened, with a `validate`-side instrument**
   (§8). The gap in the *format* is deliberately left unclosed; the gap in the *tooling* is
   not.

---

## 2. What the OpenType spec actually says

All quotes below were fetched from the Microsoft OpenType specification, version **1.9.1**
as the pages state (`os2` page `ms.date` 2024-05-29; `hhea` 2023-01-10; `recom` 2024-05-29).
READ, verbatim.

### 2.1 `OS/2` — `sTypoAscender` / `sTypoDescender` / `sTypoLineGap`

<https://learn.microsoft.com/en-us/typography/opentype/spec/os2>

> The typographic ascender for this font. This field should be combined with the
> sTypoDescender and sTypoLineGap values to determine default line spacing.
> This field is similar to the ascender field in the 'hhea' table as well as to the
> usWinAscent field in this table. However, legacy platform implementations used those
> fields with platform-specific behaviors. As a result, those fields are constrained by
> backward-compatibility requirements, and they do not ensure consistent layout across
> implementations. The sTypoAscender, sTypoDescender and sTypoLineGap fields are intended to
> allow applications to lay out documents in a typographically correct and portable fashion.

And, critically for this ticket:

> It is not a general requirement that sTypoAscender - sTypoDescender be equal to unitsPerEm.
> These values should be set to provide default line spacing appropriate for the primary
> languages the font is designed to support.

**INFERRED:** that last sentence is a *should*, addressed to the font vendor, with no
conformance test behind it. The spec offers the consumer no guarantee that `sTypoAscender -
sTypoDescender + sTypoLineGap` clears the font's own marks — only a recommendation that the
designer make it so. §5 shows one OFL face where it does and one where it does not.

### 2.2 `OS/2` — `usWinAscent` / `usWinDescent`

Same page. The `usWin*` fields are where the spec puts mark headroom:

> The "Windows ascender" metric. This should be used to specify the height above the
> baseline for a clipping region. […] Some legacy applications use the usWinAscent and
> usWinDescent values to determine default line spacing. **This is strongly discouraged. The
> sTypo\* fields should be used for this purpose.** […] Applications that use the sTypo\*
> fields for default line spacing can use the usWin\* values to determine the size of a
> clipping region. […] For new fonts, the value should be determined based on the primary
> languages the font is designed to support, and should take into consideration **additional
> height that could be required to accommodate tall glyphs or mark positioning.**

(Emphasis added.) On `usWinDescent`:

> the usWinDescent value treats distances below the baseline as positive values; thus,
> usWinDescent is usually a positive value, while sTypoDescender and hhea.descender are
> usually negative.

**INFERRED, and this is the crux of candidate 1:** the spec assigns *mark positioning
headroom* to the `usWin*` pair — the clipping metric — and *default line spacing* to the
`sTypo*` triple, while explicitly discouraging use of the former for the latter. A font that
follows the spec exactly may therefore have `sTypo*` values that do **not** contain its own
stacked marks. That is not a broken font; it is the documented division of labour. §5 finds
exactly that font.

### 2.3 `OS/2` — `fsSelection` bit 7, `USE_TYPO_METRICS`

Same page, from the `fsSelection` bit table:

> USE_TYPO_METRICS | If set, it is strongly recommended that applications use
> OS/2.sTypoAscender - OS/2.sTypoDescender + OS/2.sTypoLineGap as the default line spacing
> for this font.

And from the field's Comments:

> **Bit 7:** Bit 7 was defined in version 4 and is used in many modern applications. For new
> fonts, vendors are encouraged to use a version 4 or later OS/2 table and to have bit 7
> set. […] To minimize such risk, the bit would be set only if using the OS/2.usWin\* metrics
> for line height would yield significantly inferior results than using the OS/2.sTypo\*
> values.

### 2.4 `hhea` — `ascender` / `descender` / `lineGap`

<https://learn.microsoft.com/en-us/typography/opentype/spec/hhea>

> | FWORD | ascender | Typographic ascent—see remarks below. |
> | FWORD | descender | Typographic descent—see remarks below. |
> | FWORD | lineGap | Typographic line gap. Negative lineGap values are treated as zero in
> some legacy platform implementations. |

> The ascender, descender and linegap values in this table are Apple specific; see Apple's
> specification for details regarding Apple platforms. The sTypoAscender, sTypoDescender and
> sTypoLineGap fields in the OS/2 table are used on the Windows platform, and are recommended
> for new text-layout implementations.

### 2.5 The recommendation page

<https://learn.microsoft.com/en-us/typography/opentype/spec/recom> (the page the `OS/2`
field descriptions link to as `recom#tad`):

> The sTypoAscender, sTypoDescender and sTypoLineGap fields are used to specify the
> recommended default line spacing for single-spaced horizontal text. The baseline-to-baseline
> distance is calculated as follows:
>
>     - OS/2.sTypoAscender - OS/2.sTypoDescender + OS/2.sTypoLineGap

and, from *Baseline to Baseline Distances*, the vendor guidance that produces the numbers §4
finds:

> - Set the USE_TYPO_METRICS flag (bit 7) in the fsSelection field of the OS/2 table.
> - Set hhea.ascender = usWinAscent = sTypoAscender.
> - Set hhea.descender = -usWinDescent = sTypoDescender.
> - Set hhea.lineGap and sTypoLineGap to 0.
>
> With either guidance, if the font requires applications to use a larger clipping rectangle,
> then usWinAscent and usWinDescent values can be set to larger values, though different
> behavior will be seen in applications that set line spacing using sTypo\* values (as
> recommended) versus applications that set line spacing using the usWin\* values.

**INFERRED:** that final sentence is the Sarabun case, predicted by the spec three years
before it was measured here.

---

## 3. Which table this repo's stack actually reads

READ from the vendored crate sources on disk, not from documentation.

**The dependency graph.** `Cargo.toml` pins `parley =0.11.1` (with `complex-scripts` in
`crates/montagent-text`) and `skrifa =0.46.2`. `cosmic-text`, `swash`, `ttf-parser` and
`rustybuzz` are **absent from `Cargo.lock` entirely** — so nothing else in the build can
supply vertical metrics. `parley 0.11.1` additionally pulls `skrifa 0.44.0` transitively;
the two versions' `metrics.rs` are byte-identical, so the split changes nothing here.

**parley does no arbitration of its own.** `parley-0.11.1/src/layout/data.rs:411-415`:

```rust
let metrics = {
    let font = &self.fonts[font_index];
    let font_ref = skrifa::FontRef::from_index(font.data.as_ref(), font.index).unwrap();
    skrifa::metrics::Metrics::new(&font_ref, skrifa::prelude::Size::new(font_size), coords)
};
```

It then passes the result through, flipping descent's sign (`data.rs:444-455`).

**skrifa is the single arbitration point, and it already honours bit 7.**
`skrifa-0.46.2/src/metrics.rs`, `Metrics::new`, lines 139-191 — its own comment states the
strategy:

```rust
// We use the same strategy as FreeType:
// 1. Use the OS/2 metrics if the table exists and the USE_TYPO_METRICS
//    flag is set.
// 2. Otherwise, use the hhea metrics.
// 3. If hhea metrics are zero and the OS/2 table exists:
//    3a. Use the typo metrics if they are non-zero
//    3b. Otherwise, use the win metrics
let os2 = font.os2().ok();
let mut used_typo_metrics = false;
if let Some(os2) = &os2 {
    if os2.fs_selection().contains(SelectionFlags::USE_TYPO_METRICS) {
        metrics.ascent = os2.s_typo_ascender() as f32 * scale;
        metrics.descent = os2.s_typo_descender() as f32 * scale;
        metrics.leading = os2.s_typo_line_gap() as f32 * scale;
        used_typo_metrics = true;
    }
```

**Where the repo consumes it.** `crates/montagent-text/src/engine.rs:282-291` takes
`run.metrics().ascent/.descent` and maxes across the runs on a line, per ADR-0029. The slot
itself never sees a font: `engine.rs:334-336`

```rust
let slots: Vec<i128> = shaped
    .iter()
    .map(|line| i128::from(line.size) * i128::from(spec.line_height_tenths))
    .collect();
```

and the baseline sits inside it at `engine.rs:364`:

```rust
baseline_y: pixels(centre) + (shaped.ascent - shaped.descent) / 2.0,
```

**There is an explicit escape hatch, unused.** `read_fonts::tables::os2::Os2::s_typo_ascender`
/ `s_typo_descender` / `s_typo_line_gap` / `fs_selection` are public and reachable through
`skrifa::FontRef::os2()`; nothing in `crates/` calls them for metrics today. An
implementation *could* ask for typo metrics explicitly — it just would not learn anything
new on either face measured here (§4).

**Conclusion for candidate 1, READ not INFERRED: the repair "read `OS/2` typo metrics
instead of whatever `hhea`-derived value skrifa/parley currently reports" rests on a false
premise. There is no `hhea`-derived value being reported for these fonts. skrifa already
reads `OS/2` sTypo* whenever bit 7 is set, and both faces set it.**

---

## 4. The vendorable face, and its actual numbers

### 4.1 Licence, at the source

| Face | Version | Upstream | sha256 (pinned in `run.sh`) |
| --- | --- | --- | --- |
| Noto Sans Thai Regular | 2.002 | [`notofonts/notofonts.github.io`](https://raw.githubusercontent.com/notofonts/notofonts.github.io/main/fonts/NotoSansThai/hinted/ttf/NotoSansThai-Regular.ttf) | `61cf814e…30ee44` |
| Sarabun Regular | 1.000 | [`google/fonts` `ofl/sarabun`](https://raw.githubusercontent.com/google/fonts/main/ofl/sarabun/Sarabun-Regular.ttf) | `226d4f36…43b2f1` |

Licence files fetched from the same repos (`notofonts/thai/OFL.txt`,
`google/fonts/ofl/sarabun/OFL.txt`), both opening:

> This Font Software is licensed under the SIL Open Font License, Version 1.1.

Both are **static TTFs**, which matters: ADR-0007 declines variable-axis fields in v1, so a
variable font would be vendorable but not fully expressible.

### 4.2 The `name` table, which is what the gate reads

MEASURED with `fontTools` (`uvx --from fonttools`):

| | Noto Sans Thai | Sarabun |
| --- | --- | --- |
| name ID 13 | `This Font Software is licensed under the SIL Open Font License, Version 1.1. …` | same text |
| name ID 14 | `https://scripts.sil.org/OFL` | `http://scripts.sil.org/OFL` |

ADR-0057's bucket 2 reads exactly these two records
(`crates/montagent-core/src/fonts/licence.rs`).

### 4.3 The gate, run

MEASURED, using the repo's own shipped binary rather than reading the ADR:

```
$ montagent fonts vendor p.montagent.json …/NotoSansThai-Regular.ttf
0 errors, 0 reviews, 1 note, 0 unchecked, 0 layout, 0 drift — p.montagent.json

VENDORED  fonts/NotoSansThai-Regular.ttf
    licence     OFL-1.1 (recognised)
    sha256      61cf814eec46b294d6ea4401ac295d0cecd5207bd2331dcc5a15e7301d30ee44
    chain entry {"file":"fonts/NotoSansThai-Regular.ttf"}
```

Bucket 2 — *"Known redistributable → copy silently, record the licence"* — with no
`--licence` declaration required. Sarabun produces the identical result.

**So `Noto Sans Thai 2.002` is the nameable, vendorable Thai face the ticket asked for**: it
passes ADR-0057's gate as `recognised-open`, it is static, and it covers the Thai block.

### 4.4 The three metric sets, from the binaries

MEASURED by the probe (`results.txt`, Part 1), read through `read-fonts` off the font files
themselves. Both faces are `unitsPerEm = 1000`, `OS/2` version 4.

| | Noto Sans Thai | Sarabun |
| --- | --- | --- |
| `hhea` ascender / descender / lineGap | 1061 / −450 / 0 | 1068 / −232 / 0 |
| `OS/2` sTypoAscender / Descender / LineGap | 1061 / −450 / 0 | 1068 / −232 / 0 |
| `OS/2` usWinAscent / usWinDescent | 1061 / 450 | **1286 / 567** |
| `fsSelection` bit 7 `USE_TYPO_METRICS` | **set** | **set** |
| `hhea` ≡ `sTypo`? | **yes, all three fields** | **yes, all three fields** |
| sTypo baseline-to-baseline (§2.5's formula) | 1511 = **1.511 em** → 83.11 px at size 55 | 1300 = **1.300 em** → 71.50 px |
| usWin sum | 1511 = 1.511 em → 83.11 px | 1853 = **1.853 em** → 101.92 px |
| what `skrifa::metrics::Metrics::new` resolves at size 55 | ascent 58.35, descent −24.75, leading 0 | ascent 58.74, descent −12.76, leading 0 |

Noto Sans Thai follows §2.5's *first* guidance list with an inflated em
(`sTypo = hhea = usWin`, `1.511 em`). Sarabun follows the *alternative* list and then takes
up the spec's offer of *"a larger clipping rectangle"* — its `usWin*` pair is 43 % taller
than its `sTypo*` triple. **That divergence is exactly the mark headroom §2.2 describes, and
it lives in the table the spec forbids using for line spacing.**

Cross-check, MEASURED: the shipped `montagent measure` reports
`ascent / descent  58.355 / 24.75 px` for a size-55 Noto Sans Thai element — the probe's
numbers to three decimals, confirming the probe measures the same stack `render` does.

---

## 5. Where the ink actually collides

MEASURED. Method: each `\n`-delimited line laid out as its own single-line parley layout
(ADR-0008 owns the partition), baseline placed by ADR-0029's
`slot_centre + (ascent − descent)/2`, ink taken from every glyph's **skrifa outline** rather
than its advance box, swept over ADR-0028's tenths. Full output in
[`prototypes/thai-vertical-metrics/results.txt`](prototypes/thai-vertical-metrics/results.txt).

Samples: #130's two Thai paragraphs verbatim, plus a worst-case full-stack string
`ปู่ปี้ปุ๋ยปิ๊งปู๊ปื้น` (every cluster carries a base + an above-or-below vowel sign + a tone mark;
codepoints include U+0E39 SARA UU below, U+0E34/U+0E35/U+0E37 above, and
U+0E48/U+0E49/U+0E4A/U+0E4B tone marks above those), plus a Latin control **in the same
face**.

### 5.1 Noto Sans Thai, size 55 — the floor is 1.3

Worst ink-to-ink seam, positive = overlap:

| `line_height` | slot | 2-line | 3-line | full stack | Latin control |
| --- | --- | --- | --- | --- | --- |
| 1.0 | 55.00 | +15.08 | +15.08 | +15.08 | −3.19 |
| **1.1** | 60.50 | **+9.58** | **+9.58** | **+9.58** | −8.69 |
| 1.2 | 66.00 | +4.08 | +4.08 | +4.08 | −14.19 |
| **1.3** | 71.50 | **−1.42** | **−1.42** | **−1.42** | −19.69 |
| 1.5 | 82.50 | −12.42 | −12.42 | −12.42 | −30.69 |

Detail at `1.1`, the fixture's own value:

```
line 0: slot=[0.00,60.50] baseline_y=47.05 ascent=58.35 descent=24.75  real ink=[-8.45,61.63]
line 1: slot=[60.50,121.00] baseline_y=107.55 ascent=58.35 descent=24.75  real ink=[52.05,108.10]
seam 0/1: ink-to-ink overlap=+9.58  <-- COLLIDES
```

Rendered: [`frames/noto-thai-fullstack-lh11.png`](prototypes/thai-vertical-metrics/frames/noto-thai-fullstack-lh11.png)
— line 1's lower vowels run into line 2's tone marks — against
[`frames/noto-thai-fullstack-lh13.png`](prototypes/thai-vertical-metrics/frames/noto-thai-fullstack-lh13.png),
which is clean. The collision is visible in the pixels, not only in the arithmetic.

The Latin control clears at every tenth from 1.0. Its seam at 1.1 is **−8.69 px** — the same
figure #130 measured for Helvetica, reproduced here in a different face, which is the
sanity check that the harness has not drifted.

Note also, reproducing #130's second finding: the first line's own ink escapes the **top of
the block** by +8.45 px at 1.1 and is still escaping at 1.4 (+0.20). That is a separate
consequence of the same blindness and is not fixed by clearing the seam.

### 5.2 Sarabun, size 55 — the floor is 1.6

| `line_height` | slot | 2-line | full stack | Latin control |
| --- | --- | --- | --- | --- |
| **1.1** | 60.50 | **+23.77** | **+26.52** | −6.38 |
| 1.2 | 66.00 | +18.27 | +21.02 | −11.88 |
| **1.3** (= Sarabun's `sTypo` sum, 1.300 em) | 71.50 | **+12.77** | **+15.52** | −17.37 |
| 1.5 | 82.50 | +1.77 | +4.52 | −28.37 |
| **1.6** | 88.00 | **−3.73** | **−0.98** | −33.87 |
| 1.8 | 99.00 | −14.73 | −11.98 | −44.87 |

Latin in Sarabun clears at 1.0. Thai in Sarabun needs 1.6.

### 5.3 Answering the ticket's question directly

> The `line_height` at which two consecutive lines actually collide — and whether `1.1` is
> above or below it.

**`1.1` is below the collision floor on both vendorable faces.** The floor is `1.3` on Noto
Sans Thai and `1.6` on Sarabun. It is above the floor (`1.0`) for Latin in both.

---

## 6. Does candidate 1 close the gap?

Three readings of candidate 1, all MEASURED or READ:

**Reading A — "switch the metrics source from `hhea` to `OS/2` sTypo\*."** Zero pixels
change. skrifa already does this when bit 7 is set (§3), both faces set bit 7 (§4.4), and
both faces have `hhea` ≡ `sTypo` field-for-field anyway (§4.4). **Falls short by the entire
gap**: the seam at 1.1 stays at +9.58 px (Noto) and +23.77 px (Sarabun).

**Reading B — "derive the line slot from the `sTypo` baseline-to-baseline formula instead of
`size × line_height`."** This reverses ADR-0007's literal-size decision, which #325 puts out
of scope. Measured anyway, for completeness:

| Face | sTypo-implied `line_height` | seam at that slot |
| --- | --- | --- |
| Noto Sans Thai | 1.511 em | −12.42 px at 1.5 — **clears** |
| Sarabun | 1.300 em | **+12.77 px at 1.3 — still collides** |

**Falls short by 12.77 px on Sarabun**, or three tenths of `line_height`. One face out of two.

**Reading C — "use `usWinAscent + usWinDescent`."** Would clear both (Noto 1.511 em, Sarabun
1.853 em) — and is the one thing the spec says in plain words not to do for line spacing
(§2.2, *"strongly discouraged"*). It would also space Latin in the same face at 1.853 where 1.0 measurably suffices (§5.2).

**INFERRED, from A/B/C together: candidate 1 is not the repair.** Its premise about this
repo's stack is factually wrong; its generous reading fixes one of two faces; and the
variant that fixes both is the one the spec forbids.

**And candidate 2 — a script-aware floor — is refuted by the same table.** A floor keyed on
*script* would have to be `1.6` to be safe on Sarabun, which is 23 % looser than Noto Sans
Thai needs and 60 % looser than Latin in either face. And nothing bounds it: the floor is a
property of a particular face's mark placement, measured at 1.3 and 1.6 across a sample of
two, with no third face measured and no reason to believe 1.6 is the worst. A constant in
the schema, or in the validator, that is derived from a two-font sample and cannot be right
for a font nobody has seen yet is not a check — it is a guess with a version number. This
is the same argument ADR-0007 used to refuse fit-to-box: *"a property whose value is
HarfBuzz's opinion in the version you happen to have installed."*

---

## 7. What the shipped tools say today

MEASURED against `target/release/montagent` on this tree. A project with Noto Sans Thai
vendored, one text element, `size 55`, `line_height 1.1`, two Thai lines — i.e. the exact
configuration §5.1 shows colliding by 9.58 px:

```
$ montagent validate p.montagent.json --verbose
0 errors, 1 review, 1 note, 0 unchecked, 1 layout, 0 drift
  review  R-CAPTION-NO-AUDIO   (no audio under the caption)
  note    N-FONT-ATTESTATION-ORPHANED
  layout  L-LAYOUT             (run `montagent fmt`)
```

Nothing about the collision. Both the `review` and the `layout` finding are about other
things entirely. This is ADR-0006's blindness, confirmed rather than argued: its overflow
check compares declared numbers to declared numbers, and there is no declared number here
that is wrong.

`measure` is closer but still silent:

```
    ascent / descent  58.355 / 24.75 px  (the maximum across every run on a line — ADR-0029)
    block height      121 px
      0  baseline_y    946.552  …  slot 899.5 .. 960
      1  baseline_y   1007.052  …  slot 960 .. 1020.5
```

It reports the resolved ascent, descent and both baselines — everything except the one
quantity that decides the question, which is where the **ink** went. The two baselines are
60.5 px apart; the ink needs 70.08 px of that seam.

---

## 8. Recommendation

**The repair is candidate 3 — accept it as a font-selection problem — but candidate 3 as
stated in #325 is incomplete, and shipping it unchanged would leave the defect silent.**

### 8.1 Why candidate 3, and not 1 or 2

- Candidate 1 is a no-op on the real stack (§3, §6 Reading A), fixes one of two faces in its
  out-of-scope reading (§6 Reading B), and requires a spec-discouraged table to fix both
  (§6 Reading C).
- Candidate 2 needs a constant that the measurements show is a property of the **face**, not
  the **script** — 1.3 vs 1.6 on two OFL Thai faces (§5) — with no upper bound established.
  A schema field carrying a number the format cannot derive from the document is the thing
  ADR-0029 already refused for `ascent`/`descent`, on the rule that *"a field is only
  legitimate when its value is derivable from what the document itself declares."* A floor
  is worse than that field, because it is a guess *about fonts that do not exist yet*.
- Candidate 3 matches where the variance actually lives. `Noto Sans Thai 2.002` needs `1.3`;
  `Sarabun 1.000` needs `1.6`; Latin in either needs `1.0`. The number belongs to the pair
  (face, script), which is exactly the pair the author picks when they choose a font and
  write a `line_height` — and precisely what ADR-0007's literal-size design says the author,
  not the renderer, decides.

### 8.2 Where it lives

**Not a `render`-time computation change.** ADR-0007's slot rule stands and #325 does not
reopen it. Nothing in §3 needs to change: skrifa's arbitration is already correct and
already spec-conformant.

**Not a schema change.** No `line_height` floor field, no per-script table. There is no
number here the document could carry that is derivable from the document.

**An authoring gap the format declines to close — plus a `validate` finding that makes it
visible.** The format keeps its position: `line_height` is the author's literal, and a
literal that is wrong for the chosen font is the author's to fix, by raising it or by
choosing a face whose marks fit. What must change is that today the author gets **no signal
at all** (§7). The recommendation is therefore:

1. **A new `validate` finding, ink-based, `review`-class.** For every multi-line text
   element, compute the real ink extents of adjacent lines the way §5's probe does — the
   renderer already opens the font, already shapes the runs, and already has every glyph
   outline — and report when line *i*'s ink bottom passes line *i+1*'s ink top. This is a
   **fact about the file and the fonts on disk**, which is exactly what ADR-0006 says
   `validate` reports; it is not a rule, and it states no repair, because there are two
   legitimate ones (raise `line_height`, change the face) and the document does not
   determine which. It must be `review`, never `error`: deliberate tight leading is a real
   typographic choice, and ADR-0006's own classification reserves `error` for
   *guaranteed wrong*.
2. **`measure` gains the ink extents it already computes.** `measure` reports `baseline_y`
   per line (ADR-0029) and the block height; it should also report each line's real ink top
   and bottom, and the seam between adjacent lines. That turns the authoring loop ADR-0007
   already makes mandatory into one that can actually answer this question, without
   rendering a frame and inspecting pixels — the same argument ADR-0029 used to add
   `baseline_y` and ADR-0008 used to add break opportunities.
3. **`fonts vendor` is left alone.** It is a licence gate, not a typography adviser, and
   ADR-0057 already refuses to have it re-adjudicate anything at `validate` time.

The gap that stays deliberately unclosed is the **schema's**: the format will not learn to
express "this script needs more leading," because the measurements show that sentence is
false — it is *this font* that needs more leading, and the format already has a field for
that. It is called `line_height`.

### 8.3 What is still not measured

Stated as gaps, not as findings:

- **Only two faces.** 1.3 and 1.6 bound nothing. A third OFL Thai face could need more.
- **One size (55) and one weight (Regular).** The ratios are size-invariant by construction
  (everything scales with em), but this was not checked at a second size.
- **Thai only.** Khmer, Lao, Myanmar, Devanagari and Vietnamese stack differently and are
  untested — #130 flagged the same gap and it is still open.
- **The block-top escape (§5.1) is reported but not repaired here.** ADR-0029's half-leading
  centres on the font's ascent/descent, which on these faces undershoot real Thai ink even
  for a single line. Whether that is a second defect or the same one seen from the other end
  is not settled by these measurements.
