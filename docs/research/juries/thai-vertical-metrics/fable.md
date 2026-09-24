# Ballot — Fable 5.1

## Verdict

None of #325's three candidates is a repair, and the defect they are aimed at is smaller than
the ticket's own instrument says it is. **Candidate 1 is a no-op on both faces**: the stack
already reads `OS/2` sTypo\* (both faces set `USE_TYPO_METRICS`), and on both faces `hhea`
equals sTypo\* field-for-field, so no pixel could move. **Candidate 2 has no number to
put in the schema**: the first tenth that clears is a property of the face *and the string*
(Noto 1.3 for prose, 1.5 for a string with descender consonants under a below-vowel; Sarabun
1.6 and 1.8 for the same two strings), and a tool that has the fonts can compute the exact
seam for the document in front of it, which dominates any floor. **Candidate 3 is not a
repair either, because the "collision" as #130 and #325 measure it is largely an artefact of
the measurement**: it is a bounding-box seam, and when I rasterised the two lines and counted
pixels where both have ink, Noto prose has zero overlapping pixels at `line_height` 1.0, Sarabun
prose clears at the 1.2 default, and even the artificial worst-case stack clears at 1.1 (Noto)
and 1.3 (Sarabun) — two to three tenths below the bbox floors. What the framing misses is that
the real defect is **single-line and not Thai-specific**: the slot is a function of `size ×
line_height` while ADR-0029's half-leading centres the font's *metric* box in it, so any face
with negative leading spills ink out of the declared box with one line and no second line to
collide with — 12.8 px above and 8.3 px below the box for a Sarabun Thai line at the default,
and 3.3 px below the box for a *Latin* descender in Sarabun at the default. The tools' job is
`measure` reporting real ink and `validate` reporting an x-aware ink overlap as `review`, with
the smallest clearing tenth as the suggested repair; nothing should be refused, and a
bbox-seam check would cry wolf on every Thai document in both faces.

## Disclosures

- I did not open any forbidden file (`docs/research/thai-vertical-metrics.md`, `docs/adr/0087-*`,
  `crates/montagent-text/src/ink.rs`, `crates/montagent-core/src/checks/ink.rs`,
  `crates/montagent-core/tests/ink.rs`).
- **The author's conclusion leaked through permitted files anyway**, and the brief should know
  it: `run.sh`'s `must` lines assert the headline numbers (floors 1.3 / 1.6, "candidate 1 is a
  no-op"); ADR-0024's amendment banner names ADR-0087's title and the code
  `R-LINE-INK-COLLISION`; `engine.rs`/`measure.rs` doc comments say `measure` now reports ink
  seams; and a `grep` over `registry.rs` surfaced two comment lines about ADR-0087. I read none
  of these beyond what the grep/`cat` showed, and every number below is my own measurement,
  from my own harness, with one method the author's probe does not use.
- **Another juror edited the author's probe on disk during my session** (`probe/src/main.rs`
  gained a "JUROR ADDITION (uncommitted)" block mid-run). I did not use that file for anything
  but a read of the author's method; my harness is in my scratchpad, against my own copies of
  the fonts (sha256 verified against `run.sh`'s pins: Noto `61cf814e…`, Sarabun `226d4f36…`).

## Method (so the numbers can be re-derived)

Independent Rust crate pinned to `skrifa = "=0.46.2"`, `parley = "=0.11.1"` with
`complex-scripts`, `tiny-skia = "0.11"` for rasterising. Each line is its own parley layout
(as `montagent-text/src/engine.rs` does), `ascent`/`descent` are `run.metrics()` maxima over
the line (ADR-0029), baseline is `slot_top + slot/2 + (ascent − descent)/2`, slot is
`55 × line_height`. Three collision measures, from loosest to strictest:

1. **bbox seam** — line 1's lowest ink minus line 2's highest ink (this is #130's / the
   author's method);
2. **x-aware pairs** — per-glyph outline boxes, a pair counts only if it overlaps in x and y;
3. **pixels** — each line rasterised to its own coverage mask at its ADR-0029 baseline,
   counting pixels where both masks have ≥50% coverage ("solid") or any coverage ("any").

Glyph `y` offsets from parley (`positioned_glyphs().y − line baseline`) are applied; a
variant that drops them was also run.

## Q1 — Candidate 1

**READ.** `parley-0.11.1/src/layout/data.rs:414` calls
`skrifa::metrics::Metrics::new(&font_ref, Size::new(font_size), coords)` and copies
`ascent`, `descent` (negated), `leading` into `RunMetrics` (`data.rs:445-447`). Parley adds no
metric logic of its own. `skrifa-0.46.2/src/metrics.rs:141-190`, `Metrics::new`: if the `OS/2`
table exists and `fsSelection` contains `USE_TYPO_METRICS`, ascent/descent/leading are
`sTypoAscender`/`sTypoDescender`/`sTypoLineGap`; otherwise `hhea`; if `hhea` is all zero,
sTypo\* if non-zero, else `usWin*` (negated descent). This is FreeType's order, cited in the
source. `montagent-text/src/engine.rs:311-314` reads `run.metrics().ascent/descent` off parley;
so the repo's line metrics are whatever `Metrics::new` resolves.

**MEASURED** — raw tables, read two ways (skrifa `TableProvider`, and a bare Python `struct`
parse of the table directory; they agree byte-for-byte):

| face | upem | `hhea` asc/desc/gap | `OS/2` v | sTypo asc/desc/gap | usWin asc/desc | fsSelection | bit 7 | `head` yMin/yMax |
|---|---|---|---|---|---|---|---|---|
| Noto Sans Thai 2.002 | 1000 | 1061 / −450 / 0 | 4 | 1061 / −450 / 0 | 1061 / 450 | 0x1c0 | **set** | −433 / 1009 |
| Sarabun 1.000 | 1000 | 1068 / −232 / 0 | 4 | 1068 / −232 / 0 | 1286 / 567 | 0xc0 | **set** | −535 / 1265 |

`skrifa Metrics::new(size 55)` resolves Noto to ascent 58.355 / descent −24.750 / leading 0
(= 1061/−450 × 0.055) and Sarabun to 58.740 / −12.760 / 0 (= 1068/−232 × 0.055): the sTypo
path, as the flag dictates.

**Would candidate 1 change any rendered pixel? No — MEASURED.** The stack already reads
sTypo\*, and even if it read `hhea`, `hhea == sTypo` on both faces, so every ascent, descent,
baseline and glyph position is identical under both readings. Zero pixels.

**Does candidate 1 close the gap in principle? No — MEASURED.** The slot is never derived
from the font (ADR-0007), so which table is read affects only the baseline's position inside
the slot, not the slot. And the sTypo\* numbers do not describe ink: Sarabun's sTypo box is
1.30 em while its tallest glyph reaches 1.265 em *above the baseline alone* (gid 747, an
unencoded GSUB variant; `head.yMax` 1265) and its deepest 0.535 em below (gid 770). Noto's
sTypo box (1.511 em) does happen to enclose its ink (1.009 em up, 0.433 em down), which is why
a font-metric-derived slot would clear on Noto and not on Sarabun — a face property, not a
table-choice property.

**READ (OpenType 1.9.1, `OS/2`).** The spec says which fields *are* meant to describe ink:
`usWinAscent` "should be used to specify the height above the baseline for a clipping region
… If any clipping is unacceptable, then the value should be set greater than or equal to
yMax … should take into consideration additional height that could be required to accommodate
tall glyphs or mark positioning", and using usWin\* for line spacing "is strongly discouraged.
The sTypo\* fields should be used for this purpose." sTypo\* "should be set to provide default
line spacing appropriate for the primary languages the font is designed to support" and "It is
not a general requirement that sTypoAscender − sTypoDescender be equal to unitsPerEm." So
#325's "for scripts with tall stacks the typo metrics are often the ones that account for
them" is not what the spec promises, and Sarabun is a counter-example in hand: its ink is
described by usWin (1.853 em) and `head` bbox (1.80 em), not by sTypo (1.30 em).

## Q2 — Candidate 2

**MEASURED**, size 55, two identical lines, first tenth in 1.0…2.5 with no collision:

| face / string | bbox seam | x-aware pairs | rendered pixels (solid) |
|---|---|---|---|
| Noto, prose (#130's two lines) | 1.3 | 1.1 | **1.0** (10 faint px at 1.0, 0 at 1.1) |
| Noto, stack-A `ปู่ปี้ปุ๋ยปิ๊งปู๊ปื้น` ×2 | 1.3 | 1.2 | **1.1** (51 px at 1.0, 0 solid / 3 faint at 1.1) |
| Noto, prose over stack-B `ปี๋ฎูฏูปื๊ป์ฝี๋` | 1.3 | 1.1 | **1.1** |
| Noto, stack-B ×2 | **1.5** | — | — |
| Sarabun, prose | 1.6 | 1.2 | **1.2** (50 px at 1.0, 25 at 1.1, 0 at 1.2) |
| Sarabun, stack-A ×2 | 1.6 | 1.3 | **1.3** (403 / 254 / 56 px at 1.0 / 1.1 / 1.2) |
| Sarabun, prose over stack-B | 1.6 | 1.3 | **1.3** |
| Sarabun, stack-B ×2 | **1.8** | — | — |
| Latin control, either face | 1.0 | — | — |

Bbox seams at the values in use: Noto prose +9.58 px at 1.1, +4.08 at 1.2, −1.42 at 1.3;
Sarabun prose +23.77 / +18.27 / +12.77 / +7.27 / +1.77 / −3.73 at 1.1 … 1.6. I reproduce the
two numbers `run.sh` pins (Noto −1.42 at 1.3; Sarabun +12.77 at 1.3) from my own code, so the
author's bbox arithmetic and mine agree; the disagreement is about what the bbox means.

**The floor is a property of the face and the string, not the script — MEASURED.**
Same script, same size, same method: two different Thai strings differ by two tenths on each
face (1.3 vs 1.5 on Noto, 1.6 vs 1.8 on Sarabun); the same string differs by three tenths
between two OFL Thai faces. The ink height in em is scale-invariant (Noto stack 1.2741 em at
every size from 24 to 120; Sarabun 1.5821 em), so the tenth does not depend on `size` — but
it depends on which consonants sit under which vowels. The font-wide bound (`head` yMax − yMin:
Noto 1.442 em → 1.5, Sarabun 1.80 em → 1.8) is text-independent but three to six tenths above
what the rendered pixels need.

**What this implies for a schema floor — INFERRED from the above.** There is no number. A floor
keyed on "Thai" would have to be 1.8 to be safe on Sarabun's worst string, which is six tenths
of dead leading on Noto prose that already renders clean at 1.0 in pixels; a floor of 1.3 would
be wrong on Sarabun. A floor keyed on the *face* is not a schema property (the format declares
files, not faces, and the same key can chain several). And a floor keyed on the *script*
requires the tool to decide what script a run is in — a renderer's opinion, exactly what
ADR-0007 keeps out of the document. Since the tool already holds the fonts and the shaper, it
can compute the actual seam for the actual document, which is strictly more accurate than any
floor; the floor is dominated. Candidate 2 should not be taken.

## Q3 — If the format declines to repair it

**READ.** ADR-0006: `error` is "the render is refused or is guaranteed wrong"; `review` is for
what a human must look at; `note` is a fact you will not act on today. ADR-0043: an error's
`repair` is `"none"` (refuse-class) only when the fix depends on knowing what the author
meant. ADR-0061: a threshold is admissible in `validate` when its provenance is the document
and the media on disk. ADR-0024: `measure` writes the repair, never the verdict.

**What the tools should do — INFERRED, from the measurements above and those rules:**

- **`measure`** reports each line's real inked top/bottom (relative to the block) and, for each
  adjacent pair, the x-aware overlap — no verdict. This is the number an author needs to pick a
  `line_height`, and today nothing in the toolchain gives it (the box height is
  `ceil(size × lh × n)`, a tautology of the document, so `measure`'s existing outputs cannot
  disagree with the document about ink).
- **`validate`** gets one **`review`-class** finding, threshold zero overlap, provenance
  internal (fonts on disk + document), emitting the measured overlap in px and the **smallest
  tenth that clears** as its suggested value. Advise-class, because the value is fully
  determined by document + fonts + rendering semantics. `review`, not `error`, for two measured
  reasons: (a) a few pixels of overlap between a below-vowel and a tone mark can be a
  deliberate tight setting, and ADR-0006's "guaranteed wrong" is not met; (b) any
  over-approximation in the measure turns a false positive into a refused render, and the
  cheap measure over-approximates badly (next point).
- **The measure must be x-aware, or pixel-based; a bbox seam is unusable as a check.** Bbox
  says Sarabun collides at the 1.2 default by +18 px and Noto by +4 px; the render has zero
  overlapping pixels in both. A bbox-seam check would fire on essentially every Thai element in
  both faces at every `line_height` below 1.3 / 1.6. The x-aware per-glyph-box test agrees
  with the rasterised answer within one tenth on all six cases and costs nothing the engine does
  not already have (it already walks every glyph outline for placement).
- **`render` refuses nothing.** Overlapping marks are not a document that cannot be painted.
- **Nothing at `fonts vendor`.** The only face-level number available there (`head` bbox, or
  `usWin*`) is three to six tenths conservative; printing "this face needs `line_height` 1.8"
  would be wrong for most Thai text in it. I considered and rejected it.
- **"Nothing" is not the right answer**, because the single-line case (Q4) has no author-visible
  signal at all today, and R-BOX-SLACK actively points the wrong way (Q4).

## Q4 — What #325's framing misses

**1. The instrument overstates the defect — MEASURED.** Everything above. "1.1 collides" is
true for Sarabun (25 solid px) and false for Noto (0 px) in rendered output; the bbox method
says both collide by 9.6–23.8 px. The floors #130's method produces are not the floors a viewer
sees. Any ADR that records "1.3" or "1.6" as *the* floor has recorded a property of the
bounding-box, not of the ink.

**2. The single-line case: ink leaves the declared box with no second line to collide with —
MEASURED.** One line, size 55, ADR-0029 baseline, box = `ceil(55 × lh)`:

| face / string | lh | above box top | below box bottom |
|---|---|---|---|
| Noto, Thai stack or prose | 1.1 | +8.45 px | +0.63 px |
| Noto, Thai stack or prose | 1.2 (default) | +5.70 px | inside |
| Noto, Thai stack or prose | 1.3 | +2.95 px | inside |
| Sarabun, Thai prose | 1.2 (default) | +10.01 px | +8.26 px |
| Sarabun, Thai stack-A | 1.2 (default) | +12.76 px | +8.26 px |
| Sarabun, Thai stack-B | 1.6 | +0.89 px | +8.41 px |
| Sarabun, Thai stack-B | 1.7 | inside | +5.16 px |
| **Sarabun, Latin** | 1.0 | inside | **+8.80 px** |
| **Sarabun, Latin** | 1.1 | inside | **+5.55 px** |
| **Sarabun, Latin** | 1.2 (default) | inside | **+3.30 px** |

**READ.** The renderer does not clip text to the declared box: `montagent-render/src/canvas.rs`
`Canvas::text` paints in `in_element_space(extent, …)` where `extent` is the typographic block,
and its `clip` argument is the element's own crop field, not `width`/`height`;
`montagent-text/src/place.rs` says the declared `height` is "never consulted". So the ink
above is painted outside the box — over whatever card the author sized to that box — and no
check sees it: the only height check, `R-BOX-SLACK` (`checks/box_slack.rs`), compares the
declared height with `text_block_height`, which is the same formula the author used to write
the height, so it cannot detect ink escaping. No `OVERFLOW`-coded check exists in
`registry.rs`. This is the "text overflowing its own box" check ADR-0006 promised, arithmetically
vacuous by construction on the vertical axis.

**3. It is not a Thai problem; it is negative leading plus lopsided metrics — MEASURED,
mechanism INFERRED.** Sarabun's Latin `g`/`y` descenders (0.242 em of ink below the baseline,
against a typo descender of 0.232 em) leave the box bottom by 3.3 px at the default, because
sTypo asc − desc = 1.30 em > 1.2 em slot and ADR-0029 splits the 5.5 px overflow evenly around
the *metric* box, whose centre sits 0.418 em above the baseline while the Latin ink's centre is
much lower. Noto's metrics (1061/−450) are closer to symmetric about its ink, so it does not
show this. ADR-0029 itself recorded (#59) that the fixture font also has negative leading at
size 55; that ADR chose where to put the baseline and never asked whether the slot could hold
the metric box. Thai only makes the overflow large enough to notice.

**4. Candidate 1 names the wrong OS/2 fields — READ + MEASURED.** If a font-only number is
wanted, `usWinAscent`/`usWinDescent` and `head.yMin`/`yMax` are the fields the spec defines as
ink/clipping bounds, and skrifa already exposes the latter as `Metrics::bounds` (computed in
the same `Metrics::new` call the stack already makes). On Sarabun they cover the ink (1.853 em
and 1.80 em vs 1.769 em worst measured); on Noto 1.442 em vs 1.441 em worst. They are a sound
text-independent upper bound, but a conservative one; useful at most as the fallback when a
check cannot shape (a `.notdef` line), not as the check.

**5. `R-BOX-SLACK` punishes the correct fix — MEASURED against `box_slack.rs:97`.** An author
who enlarges a one-line Sarabun box to actually contain the ink (1.5821 em × 55 = 87.0 px →
`height: 88` against a computed 66) is 22 px / 33 % over the `max(2 px, 10 %)` threshold and
gets a `note` telling them their box is oversized. For Noto (70.1 px vs 66, 6 %) it stays
silent. A `note` is not blocking, but it is the only signal the toolchain gives on this axis
today and it points the wrong way.

**6. Mark stacking method — MEASURED here, INFERRED for other faces.** Both faces stack via GSUB
(precomposed raised-mark glyphs; the extreme glyphs are unencoded variants), so dropping
parley's per-glyph `y` offset changes the ink top by 0.00 px on both. The author's probe drops
that offset (`y: baseline_y` for every glyph); the product (`place.rs:162`) keeps it. On a face
that stacks by GPOS mark-to-mark, the probe would under-measure and the product would not; the
two OFL faces in hand cannot show this, so it is recorded, not claimed.

## What blocked me

- Nothing blocked a measurement. The "rendered, not computed" number #325 asked for is the
  pixel column above; the author's probe and #130's did not produce it.
- Two things reduce the ballot's independence and are disclosed above: the headline numbers
  leaking through `run.sh` and ADR-0024's banner, and a co-juror editing the author's probe on
  disk while I worked. Neither changed a number here; all of mine come from my own harness.
