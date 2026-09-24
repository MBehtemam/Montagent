# Jury: does Thai's `line_height` collision have a repair inside the format?

Three jurors (Opus, Sonnet, Fable), each a different model, each answering
[`BRIEF.md`](BRIEF.md) independently and forbidden the author's write-up, ADR-0087 and the
implementation. The subject is [#325](https://github.com/MBehtemam/Montagent/issues/325) and
the decision recorded in
[ADR-0087](../../../adr/0087-thai-line-height-collision-is-a-font-selection-problem.md).

**Read the contamination section before the verdict.** This court was not as blind as it was
supposed to be, and that is the author's fault, not the jurors'.

## Verdict

| | Sonnet | Opus | Fable |
| --- | --- | --- | --- |
| Candidate 1 (`OS/2` typo metrics) is a no-op | ✅ | ✅ | ✅ |
| Candidate 2 (script-aware floor) has no number to carry | ✅ | ✅ | ✅ |
| Finding's class | `error` | **`review`** | **`review`** |
| All three candidates miss single-line containment | ✅ | ✅ | ✅ |

**The two rejections hold 3/3.** The class holds **2–1**. Two findings the ADR did not
have are recorded below, and one of them is a defect in what shipped.

## What the court confirmed

**Candidate 1 is a no-op**, on every juror's own measurement: both faces set `fsSelection`
bit 7, skrifa is therefore already on the `OS/2` sTypo path, and on both faces `hhea` equals
`sTypo` field-for-field (Noto 1061/−450/0; Sarabun 1068/−232/0). All three reached this
without seeing each other or the write-up.

**Candidate 2 has no constant to carry.** All three measured the floor and all three found it
varies by face.

## What the court found that the ADR did not

### 1. Candidate 1 is not merely useless here — it cannot work anywhere (Opus)

The ADR rejects candidate 1 on the grounds that the two tables *happen* to be identical on
these two faces. That is much weaker than the truth. The half-leading term cancels out of the
seam:

```
baseline_y  = slot_centre + (ascent − descent)/2        engine.rs:383
ink_bottom  = baseline_y + ink.bottom                   engine.rs:399
ink_top     = baseline_y + ink.top                      engine.rs:398
overlap     = ink_bottom(above) − ink_top(below)        ink.rs:149
            = ink.bottom − ink.top − slot
```

Ascent and descent appear in both baselines and subtract away. **No choice of metric table
can move a line seam on any face**, for lines of equal style. Opus did not stop at the
algebra: it patched Sarabun's `hhea` to its `usWin` values and cleared bit 7, moving skrifa's
ascent by +11.99 px and its descent by +18.43 px at size 55, and every seam, needed slot and
floor came back bit-identical.

The caveat the ballot did not state: this holds where adjacent lines share ascent and
descent. Mixed per-line metrics — a run at a different size or in a different face — do not
cancel.

### 2. The floor belongs to the string, not the face (Opus, Fable)

The ADR says the floor is a property of the face. An exhaustive sweep of Thai consonants ×
above-marks × below-marks × tones puts Sarabun's worst orthographic cluster (`ฏ็์`) a full
tenth above the write-up's sample. Reproduced by the author against the shipped engine:

| | write-up's sample | worst cluster `ฏ็์` |
| --- | --- | --- |
| Noto Sans Thai | 1.3 | 1.3 |
| Sarabun | **1.6** | **1.7** |

This *strengthens* the rejection of candidate 2 — there is no constant at any granularity,
not even per-face — and makes ADR-0087's own table an understatement.

### 3. The shipped seam over-reports (Fable)

The one finding that is a defect rather than an improvement, and the subject of the
[second court](../thai-ink-seam-x-aware/BRIEF.md). The seam compares **whole-line** extents,
so a descender at one x counts as colliding with a mark at another. Fable rasterised the
glyphs; the author reproduced it independently with per-x-column ink profiles off `place()`:

| case, size 55 | shipped seam | x-aware |
| --- | --- | --- |
| Noto, full stack, 1.1 | +9.58 | **+0.61** |
| Noto, full stack, 1.2 | +4.08 | **−4.89** (fires on a clean render) |
| Sarabun, full stack, 1.3 | +15.52 | **−1.31** (fires on a clean render) |
| either face, ordinary prose | — | identical |

It over-reports and never misses, and the over-report vanishes on dense prose. ADR-0011's
warning about nominal metrics being *"a false-positive generator"* applies one level down.

### 4. Single-line containment, missed by all three candidates — 3/3

Every juror found it independently, and it is larger than #325 or the ADR frames it.
`R-LINE-INK-COLLISION` needs two lines; a single line has no seam, so nothing fires while ink
hangs outside the declared box. Measured at the format's **default** `line_height` of 1.2,
and **not only in Thai** — Sarabun Latin descenders leave the box by ~3.3 px. Filed as
[#334](https://github.com/MBehtemam/Montagent/issues/334), which was written before these
ballots arrived and understates it as a Thai problem.

Fable adds that `R-BOX-SLACK` will *note* the author who sizes the box to hold the real ink,
which is the correct fix — the two checks disagree.

## The dissent

**Sonnet argued `error`, refuse-class**, on the grounds that the collision is a computed fact
rather than an intent-dependent one. Opus and Fable both reached `review` on ADR-0006's
*"guaranteed wrong"* line, which is where the ADR landed: a deliberately tight `line_height`
is a real typographic choice, and `error` would refuse to render a legal document.

Recorded rather than resolved away, because Sonnet's premise is correct even though its
conclusion did not carry — the fact *is* computed. What `error` turns on in ADR-0006 is
consequence, not derivability.

Both Opus and Fable proposed the finding carry a computed `line_height` repair. That is
unavailable: ADR-0043 makes `repair` an axis of `error` alone, so a `review` cannot hold one.
Neither juror had reason to know that.

## Contamination — read this before trusting the margins

**The court was not blind, and the brief is what broke it.** Opus diagnosed it; Fable
independently reported the same leaks; Sonnet reported seeing filenames.

1. **`run.sh`, which the brief instructed jurors to run**, hard-codes the author's headline
   numbers *and their interpretation* as `must` grep assertions — including the strings
   "Candidate 1 is a no-op" and "Sarabun Thai floor is 1.6".
2. **`engine.rs`, `place.rs`, `measure.rs` and two `registry.rs` comments were not on the
   forbidden list** and cite ADR-0087 by number, ship the finished `ink_top`/`ink_bottom`/
   `ink_seams` API, and quote the 8.45 px figure in a doc comment.
3. **ADR-0024's amendment banner** names ADR-0087 and `R-LINE-INK-COLLISION`.
4. **`ls docs/adr`** puts the verdict in a filename.

Opus and Fable each mitigated by writing their own probe from scratch and treating every
leaked number as a claim to falsify — which is how the 1.6→1.7 correction and the
over-reporting defect were found at all. But **"confirmed 3/3" is worth less than it looks**,
and the two rejections should be read as *reproduced* rather than as *independently
discovered*.

One further process failure: a juror edited the shared probe (`probe/src/main.rs`) on disk
during another juror's session. Fable noticed, worked from its own scratchpad copy with
SHA-256-verified fonts, and said so. Jurors must be told to work in their scratchpad and
modify nothing in the repo.

**If this format is run again**: the prototype's assertion script, the ADR filenames, every
source file that cites the ADR, and the amendment banners all belong on the forbidden list —
or, better, jurors should be given a worktree at the commit *before* the work landed.

## The ballots

- [`opus.md`](opus.md) — the cancellation proof, the cluster sweep, the leak diagnosis
- [`sonnet.md`](sonnet.md) — the `error` dissent, and the single-line finding
- [`fable.md`](fable.md) — the rasterised measurement that found the over-report
