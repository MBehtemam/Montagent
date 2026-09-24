# Ballot — Opus

## Disclosure first

I did **not** open `docs/research/thai-vertical-metrics.md`, `docs/adr/0087-*`,
`crates/montagent-text/src/ink.rs`, `crates/montagent-core/src/checks/ink.rs` or
`crates/montagent-core/tests/ink.rs`. I also did not open the prototype's `results.txt` or
`probe/src/`.

**But this court was not blind, and the brief is what broke it.** Three leaks, in order of
severity:

1. `docs/research/prototypes/thai-vertical-metrics/run.sh` — which the brief tells me to run —
   contains the author's headline numbers *and their interpretation* as literal `must` strings:
   *"Candidate 1 is a no-op"*, *"Noto Thai 2-line floor is 1.3"*, *"Sarabun Thai floor is 1.6"*,
   *"Candidate 1 falls short on Sarabun"*. I read it before I understood what it was.
2. `crates/montagent-text/src/engine.rs` and `place.rs` (not forbidden) cite **ADR-0087 by
   number**, ship `ink_top`/`ink_bottom`/`ink_seams` on `MeasuredLine` and `Measurement`, and
   quote the figure *"the first line's ink escapes `block_top` by 8.45 px at `line_height` 1.1"*.
   The repair is already built and its doc comments narrate it.
3. `ls docs/adr` prints `0087-thai-line-height-collision-is-a-font-selection-problem.md`. The
   verdict is in the filename.

I mitigated the only way available: I wrote my own probe from scratch (parley 0.11.1
`complex-scripts` + skrifa 0.46.2, glyph outlines flattened at 64 steps per curve), never ran
the author's, and treated every leaked number as a claim to falsify. Two of the leaked numbers
reproduce. **One of the leaked framings does not survive**, and the design the leaks reveal is
missing a check I found by measurement — see Q4. Read the ballot knowing I was contaminated.

## Verdict

**Candidate 1 is not merely a no-op on these two faces — it is structurally incapable of
repairing a multi-line collision on *any* face, and #325 does not know this.** The line seam is
`slot + ink_top − ink_bottom`; the half-leading term `(ascent − descent)/2` appears in *both*
baselines and cancels exactly. I proved this by measurement, not algebra: I patched Sarabun's
`hhea` to its `usWin` values and cleared `fsSelection` bit 7, which moved skrifa's ascent by
+11.99 px and descent by +18.43 px at size 55 — and **every** seam, every needed slot and every
`line_height` floor came back bit-identical. Candidate 2's floor is real but is a property of
neither the script nor the face: it is a property of the *string*, and the author's own sample
understates Sarabun's worst orthographic Thai cluster by a full tenth (1.6 measured on the
sample; **1.7** required by `ฏ็์`). Candidate 3 fails on its own terms, because Noto Sans Thai
*is* the canonical vendorable Thai face and it still collides at 1.1. And all three candidates
are aimed at the seam, which is the wrong place: the defect that actually escapes every check in
this repo is **single-line block containment**, where ink escapes the element's own declared box
in both directions, where the half-leading term does *not* cancel — so candidate 1 is the only
lever there and #325 never points it there — and which I measured firing on **plain Latin text
in Sarabun at the format's default `line_height` of 1.2**. This is not a Thai problem and not a
font-selection problem. `R-BOX-SLACK` computes `size × line_height × line_count` with no font
open, and `R-OFF-CANVAS` unions declared `width`/`height`; neither can see a pixel of it.

---

## Q1 — Candidate 1: which table, and would it change a pixel?

### The code path

**READ.** The chain is `montagent-text` → parley → skrifa, and skrifa arbitrates.

- `crates/montagent-text/src/engine.rs:313-314` —
  `ascent = ascent.max(f64::from(run.metrics().ascent))`, same for descent. Parley's
  `RunMetrics`, max across every run on the line (ADR-0029).
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/parley-0.11.1/src/layout/data.rs:410-413`
  builds those from
  `skrifa::metrics::Metrics::new(&font_ref, Size::new(font_size), coords)`
  (and negates descent at line 447, so parley's `descent` is positive-down).
- **The function is `skrifa::metrics::Metrics::new`**, in
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/skrifa-0.46.2/src/metrics.rs:106-190`.
  It implements FreeType's arbitration, documented in its own comment at lines 136-146:
  1. `OS/2` typo metrics **if `fsSelection` bit 7 (`USE_TYPO_METRICS`) is set**;
  2. otherwise `hhea`;
  3. if `hhea` ascent and descent are both zero, `OS/2` typo, else `usWin`.

**MEASURED.** The workspace resolves *two* skrifa versions (`Cargo.lock:2009, 2019`): parley
0.11.1 depends on `skrifa 0.44.0`, while the repo pins `=0.46.2` for its own use.
`diff skrifa-0.44.0/src/metrics.rs skrifa-0.46.2/src/metrics.rs` is **empty** — byte-identical,
so both halves of the stack arbitrate identically. The prototype's `Cargo.toml` asserts this;
I confirmed it.

So the answer to *"hhea or OS/2 typo"* is: **neither, unconditionally — skrifa picks, and on
both faces it picks OS/2 typo, because both set bit 7.**

### The tables

**MEASURED** — pure-Python parse of the `head`/`hhea`/`OS/2` tables, on the sha256-pinned
binaries (`61cf814e…` Noto, `226d4f36…` Sarabun, both matching `run.sh`'s pins).

| | Noto Sans Thai 2.002 | Sarabun 1.000 |
|---|---|---|
| `head.unitsPerEm` | 1000 | 1000 |
| `hhea` ascender / descender / lineGap | **1061 / −450 / 0** | **1068 / −232 / 0** |
| `OS/2` sTypoAscender / sTypoDescender / sTypoLineGap | **1061 / −450 / 0** | **1068 / −232 / 0** |
| `OS/2` usWinAscent / usWinDescent | 1061 / 450 | **1286 / 567** |
| `OS/2` version | 4 | 4 |
| `fsSelection` | `0b0000000111000000` (0x01C0) | `0b0000000011000000` (0x00C0) |
| **bit 7 `USE_TYPO_METRICS`** | **set** | **set** |
| hhea == sTypo, field for field | **yes** (asc, desc, gap all) | **yes** (asc, desc, gap all) |
| sum / em: hhea & sTypo | 1511 → **1.511 em** | 1300 → **1.300 em** |
| sum / em: usWin | 1511 → 1.511 em | 1853 → **1.853 em** |

### Would candidate 1 change any rendered pixel?

**No, twice over, and the second reason is the one #325 is missing.**

**(a) On these two faces it is a double no-op. MEASURED.** Bit 7 is set on both, so skrifa is
*already* on the typo path — "switch to OS/2 typo" is a description of current behaviour, not a
change. And even if it were not set, `hhea` and `sTypo` are identical field-for-field on both
faces, so the hhea branch yields the same three numbers. Two independent reasons for zero.
At size 55 skrifa returns ascent 58.355 / descent −24.750 (Noto) and 58.740 / −12.760 (Sarabun).

**(b) On *any* face, it cannot move a line seam at all. MEASURED.**

The algebra, from code I read: `baseline_i = slot_centre_i + (ascent − descent)/2`
(`engine.rs:383`), and `slot_centre_i` comes from `size × line_height` alone
(`engine.rs:362-382`) with no font input. Two consecutive lines in one font at one size
therefore have baselines exactly `slot` apart, *whatever* the metrics are. The seam between
line 1's ink bottom and line 2's ink top is

```
seam = (baseline_2 + ink_top) − (baseline_1 + ink_bottom) = slot + ink_top − ink_bottom
```

and `(ascent − descent)/2` cancels identically. Ink offsets from the baseline are outline
geometry — they do not depend on a metric table at all.

I did not leave that as algebra. I built a perturbed face: Sarabun with `hhea` ascender/descender
overwritten to its own `usWin` values (1286 / −567) and `fsSelection` bit 7 **cleared**, forcing
skrifa down the hhea branch to different numbers. Result at size 55:

| | Sarabun | Sarabun-PERTURBED |
|---|---|---|
| skrifa ascent / descent | 58.740 / −12.760 | **70.730 / −31.185** |
| half-leading term `(asc−desc)/2` | 22.990 | **19.7725** |
| full-stack ink rel. baseline | top −68.750, bottom +18.266 | **top −68.750, bottom +18.266** |
| needed slot (2-line, whole-line extents) | 87.016 px | **87.016 px** |
| first clear tenth | 1.6 | **1.6** |
| worst-cluster sweep floor (orthographic) | 1.6651 | **1.6651** |

A 12–18 px swing in the metrics; **zero** change to every collision quantity. Candidate 1 is
not "a no-op on the faces we happened to measure" — it is the wrong lever, permanently, for the
problem #325 points it at.

**Where it *does* change pixels — and #325 never looks there.** The perturbation moved the
half-leading term by 3.2175 px, which translates the entire block's ink relative to its declared
box. Single-line Sarabun full-stack Thai at `line_height` 1.1: ink escaping the slot top went
**15.51 px → 18.73 px**, escaping the bottom **11.01 px → 7.79 px**. That is candidate 1's only
real effect, it is on the single-line containment defect of Q4, and on these two faces it is
still zero because the tables agree.

---

## Q2 — Candidate 2: the lowest non-colliding `line_height` tenth at size 55

**MEASURED.** Method: parley `complex-scripts` shaping, one line, `break_all_lines(None)`
(the renderer never wraps, ADR-0007); per-glyph skrifa outlines, unhinted, flattened; ink
expressed as y-down offsets from the parley baseline. Two consecutive identical lines, baselines
`55 × line_height` apart. *Whole-line extents* = the conservative bounding-box test;
*column-overlap* = clear unless two glyphs whose x-ranges actually intersect also intersect
vertically — the honest "real ink" test.

Sample string `กุ้ ปู่ ญี๊ ฟุ๋ ฬื้` (base + lower vowel + upper vowel + tone), Latin control
`Handgloves Typography`.

| face / string | ink height (px) | floor (em) | **first clear tenth, extents** | first clear tenth, column |
|---|---|---|---|---|
| Noto, Thai full-stack | 70.078 | 1.2741 | **1.3** (slot 71.50, clears by 1.42) | 1.2 |
| Noto, natural Thai sentence | 56.047 | 1.0190 | **1.1** | 1.1 |
| Noto, Latin control | 39.266 | 0.7139 | **1.0** | 1.0 |
| Sarabun, Thai full-stack | 87.016 | 1.5821 | **1.6** (slot 88.00, clears by 0.98) | 1.5 |
| Sarabun, natural Thai sentence | 66.547 | 1.2099 | **1.3** | 1.2 |
| Sarabun, Latin control | 54.125 | 0.9841 | **1.0** | 1.0 |

1.3 / 1.6 / 1.0 reproduce the leaked headline numbers independently. **1.1 is below every Thai
floor and at or above every Latin one**, which is the finding.

But the two column-overlap columns already show the answer is soft: Noto's *real* ink collision
clears at **1.2**, not 1.3, and Sarabun's at **1.5**, not 1.6. A face-level floor stated to the
tenth is already sensitive to which test you run.

### Is the floor a property of the script or of the face?

**Neither. It is a property of the string.** MEASURED, by exhaustive sweep: all 46 Thai
consonants U+0E01–U+0E2E × {7 above-base vowels/signs} × {3 below-base vowels} × {5 tone
marks/thanthakhat}, shaped one cluster at a time, plus the orthographic subset
base + one vowel + one tone.

| | Noto Sans Thai | Sarabun |
|---|---|---|
| author's sample string | 1.2741 → **1.3** | 1.5821 → **1.6** |
| worst **orthographic** cluster | `ฎ็่` 1.2832 → **1.3** | **`ฏ็์` 1.6651 → 1.7** |
| worst 4-level stack | `ฦู็่` 1.5190 → **1.6** | `ฎู็์` 1.8770 → **1.9** |
| Latin control | 0.7139 → 0.8 | 0.9841 → 1.0 |

**The leaked "Sarabun floor is 1.6" is wrong by a tenth.** An ordinary, orthographically
well-formed Thai cluster — ปฏัก's `ฏ` with mai taikhu and thanthakhat — needs **1.7**. A floor
calibrated on a sample string is a floor that ships the next collision.

**Size-independent. MEASURED:** the identical sweep at size 110 returns identical em floors
(1.5190, 1.2831, 1.8770, 1.6651), confirming that unhinted outlines scale linearly and a floor
is a pure em ratio.

### What that implies about a schema floor

**It kills the schema floor as #325 frames it, on three counts.**

1. **A "script-aware" floor has no script to be aware of.** The floor varies 1.28 → 1.88 em
   *within* Thai on one face, and 1.28 → 1.67 em across two faces for the same orthographic
   class. A constant that covers the worst case (1.9) is 46% taller than what Noto's ordinary
   text needs — it would visibly wreck every conforming document to protect a cluster most of
   them do not contain.
2. **The format cannot compute it.** ADR-0029 already settled the governing principle: *"a field
   is only legitimate when its value is derivable from what the document itself declares, and
   font vertical metrics are not."* An ink floor is strictly worse than a metric — it needs the
   font *and* the shaped string *and* the size. A schema field carrying it would be a frozen
   measurement, invalidated by exactly the font swap ADR-0007's census already exists to flag.
3. **A floor enforced as a minimum would repeal ADR-0030.** `line_height`'s *presence* is
   content; an omitted 1.2 and an explicit 1.2 are different documents. A floor that silently
   raises 1.1 to 1.3 is the normalisation ADR-0030 forbids `fmt` from doing, relocated into the
   renderer where it is worse.

**Candidate 2 is right about the quantity and wrong about where to put it.** The number is real,
per-element, and computable — by `measure`, at authoring time, frozen into the file as a
literal, which is ADR-0007's own settled pattern for exactly this shape of problem (*"wherever
this format refuses a convenience, the convenience belongs in an authoring-time tool whose
output is inert"*).

---

## Q3 — Candidate 3: if the format declines to repair it, what should the tools do?

**Not "nothing".** Three things, and nothing refused.

**`measure` — report the ink.** Per line: `ink_top` and `ink_bottom`, absolute, in the block's
own coordinates, `null` for a line that draws nothing (an empty line still reserves its slot;
that is a different fact from zero-height ink at the baseline). Per adjacent inked pair: the
seam, signed, negative meaning overlap. Per block: `ink_top`/`ink_bottom`, **not** clamped to
the block — clamping is what makes the Q4 defect invisible.

*Why `measure` and not a new tool:* ADR-0029 set this precedent explicitly for `baseline_y` —
*"a quantity the renderer must compute anyway, now made checkable without executing code in the
agent's head"* — and ADR-0007 already makes `measure` mandatory in the text authoring loop. The
layout pass has the outlines in hand (`place.rs:143-168` already walks every positioned glyph
and draws its outline); reporting their extents costs one pass it already makes.
*(Disclosure: `engine.rs` shows this is already built. I would have proposed it regardless —
Q2's result forces it, because the floor is only knowable by measuring.)*

**`validate` — a `review`-class finding on ink-to-ink overlap between adjacent lines, carrying a
`line_height` repair.** `review`, not `error`, on ADR-0006's own line: an error is *guaranteed*
wrong, and overlapping ink is not — a one-line element cannot trip it, and deliberate overlap
is a real typographic effect. It belongs with the other font-dependent judgments rather than
with the `.notdef` error, which has no intent it could express. The repair is the tenth my Q2
sweep computes: measured, per-element, exact, and it is what makes this a finding an agent can
act on rather than a note it reads and ignores.

**Nothing should be refused.** ADR-0043's refuse class is non-bypassable and carries no
alternative; a legibility judgment that depends on what the author is trying to draw is not
that, and ADR-0067's legibility refusal is about a *floor on the render itself*, not about one
element's leading.

**And candidate 3 as stated is false.** Pushing this onto font vendoring assumes a "good" Thai
face exists that does not collide. **Noto Sans Thai is that face** — the Noto project's own
release binary, OFL-1.1, the obvious thing ADR-0057's licence gate would suggest as a Thai
substitute — and it **collides at 1.1, 1.2 and, for its worst orthographic cluster, up to 1.5**
(MEASURED). There is no vendoring decision that repairs a `line_height` the author chose. Worse,
the face with the *better* metric hygiene by conventional standards (Noto: `usWin` == `sTypo`
== `hhea`, all consistent at 1.511 em) is the one whose collision is *hardest* to see, because
its metrics honestly advertise 1.511 em while the document reserves 1.1.

---

## Q4 — The question none of the three asks

### The defect: a single line's ink escapes its own declared box, and every check in this repo is blind to it

**MEASURED.** A single line has no seam, so all three candidates are silent about it by
construction. But ADR-0007's block height is `size × line_height` and ADR-0029 places the
baseline at `slot_centre + (ascent − descent)/2` — an **asymmetric** offset with no relation to
where the ink actually is. The ink therefore hangs outside the block, in both directions, on one
line.

Size 55, single line, ink measured against its own slot:

| face / string | `line_height` | escapes slot top | escapes slot bottom |
|---|---|---|---|
| Noto, natural Thai | 1.1 | **+8.45 px** | — |
| Noto, natural Thai | 1.2 (default) | **+5.70 px** | — |
| Noto, Thai full-stack | 1.1 | **+8.45 px** | **+1.13 px** |
| Sarabun, Thai full-stack | 1.1 | **+15.51 px** | **+11.01 px** |
| Sarabun, Thai full-stack | 1.3 | **+10.01 px** | **+5.51 px** |
| **Sarabun, Latin `Handgloves Typography`** | **1.2 (the format default)** | — | **+3.30 px** |
| Sarabun, Latin `Handgloves Typography` | 1.3 | — | +0.55 px |

**The last two rows are the finding.** That is plain Latin text, in a Google-hosted OFL face, at
the `line_height` ADR-0007 makes the default, with 3.3 px of descender ink outside the block the
document declares. **This is not a Thai defect and not a font-selection problem.** Sarabun's
`sTypoDescender` is −232 (Latin-sized) while its `sTypoAscender` is 1068 (Thai-sized), so
half-leading shoves the baseline 22.99 px — 0.418 em — below the slot centre, and the `g`/`y`/`p`
descenders land 36.30 px below centre against a slot half of 33. Any face that supports Thai
*and* Latin inherits this on its Latin.

**Containment needs a strictly higher `line_height` than the seam does** — derived from the same
measurements, so the seam floor is not merely incomplete but *actively misleading* as a floor:

| case | seam floor (2 lines) | **containment floor (1 line)** |
|---|---|---|
| Noto, Thai full-stack | 1.2741 → 1.3 | **1.4072 → 1.5** |
| Noto, worst orthographic `ฎ็่` | 1.2832 → 1.3 | **1.5532 → 1.6** |
| Sarabun, Thai full-stack | 1.5821 → 1.6 | **1.6640 → 1.7** |
| Sarabun, worst orthographic `ฏ็์` | 1.6651 → 1.7 | **1.8481 → 1.9** |
| Sarabun, Latin control | 0.9841 → 1.0 | **1.3201 → 1.4** |

A `line_height` floor set to clear the seam leaves the block under-sized by one to four tenths in
every single case I measured. **Candidate 2, implemented exactly as #325 frames it, would ship a
document that still overflows its own box.**

### Why nothing catches it. READ.

- **`R-BOX-SLACK`** (`crates/montagent-core/src/checks/box_slack.rs`) is documented *"No font,
  no I/O"* and computes `(size × line_height×10 × line_count + 9) // 10`. It compares a declared
  `height` against a number that has never seen a glyph. A text element with
  `size: 55, line_height: 1.1, height: 61` is *exactly* right by this check while 8.45 px of ink
  sits above it.
- **`R-OFF-CANVAS`** (`checks/canvas.rs:50`) unions `x`, `y`, `origin` and the declared
  `width`×`height`. Ink can be off-frame with the check silent.
- **Nothing clips text to its box.** `clip` in `montagent-render/src/canvas.rs` is a separate
  element-level frame-space rectangle, not the text box. So the overshoot is not cropped — it is
  *drawn*, into whatever neighbours it.

Three independent checks, none of which can see the pixel.

### The inversion this produces

Candidate 1 moves ink relative to the block by exactly `Δ(ascent − descent)/2` — **MEASURED**:
3.2175 px on my perturbed Sarabun, which is 21% of its 15.51 px top escape. So:

| | multi-line seam (what #325 aims at) | single-line containment (what #325 misses) |
|---|---|---|
| candidate 1 (metric table) | **provably zero effect, any face** | **the only lever besides `line_height`** |
| candidate 2 (`line_height` floor) | works, floor is string-specific | works, needs a *different, higher* floor |
| candidate 3 (font selection) | fails — Noto collides at 1.1 | fails — breaks on **Latin** |

**#325 points each candidate at the one case it cannot serve.** That is the defect in the
framing, and it is not visible from any of the three questions it asks.

### A second, smaller miss: stroke

**INFERRED, from READ, not measured.** ADR-0014 makes `stroke_width` paint that falls *outside*
the glyph contour, and `MeasuredLine::extent_width` adds `2 × stroke_width` to the advance for
exactly that reason. Ink extents are contour geometry and carry no such term
(`engine.rs:397-399`). An ink seam therefore over-states clearance by `2 × stroke_width` — one
stroke from the line above, one from the line below — and a block-containment test understates
the escape by `stroke_width` at each edge. I did not measure this: the fixture sets
`Outline: 0` on all 35 styles (ADR-0007 records it), so there is no stroked corpus to measure
against, and I declined to invent one. It is a live arithmetic gap, not a hypothesis.

---

## What I could not do

- **I could not read #130 or #325 themselves.** This machine reports the working directory as not
  a git repository and I did not attempt network access, so both tickets are known to me only
  through the brief's summary. If either names a fourth candidate or constrains the floor
  question, I did not see it.
- **I did not measure the fallback-chain case.** ADR-0007 allows a chain, and a Thai cluster
  whose base resolves in one face and whose marks resolve in another would have its GPOS mark
  attachment computed across a font boundary. I believe that is a distinct Thai-specific defect
  and I have no measurement of it. Stated as unexamined, not as a finding.
- **My column-overlap test buckets by glyph bounding box, not by rasterised coverage.** It is
  tighter than whole-line extents and looser than true pixel intersection. Both numbers are
  reported above; I did not rasterise.
