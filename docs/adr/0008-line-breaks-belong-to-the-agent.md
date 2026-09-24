# ADR-0008: Line breaks belong to the agent; the renderer only obeys them

**Status:** accepted
**Date:** 2026-09-04
**Resolves:** [#28](https://github.com/MBehtemam/Montagent/issues/28)
**Supersedes:** the "Text and internationalisation" paragraph of
[ADR-0007](./0007-text-runs-literal-size-declared-fonts.md). Everything else in
ADR-0007 stands.

## Context

ADR-0007 settled the text primitive and said two things that cannot both be operative.

Its decision:

> **Literal `size`. No fit-to-box. No automatic wrapping.** … the renderer never
> chooses a size and never chooses a line break.

Its internationalisation section, fifteen lines later:

> **v1 renders bidi reordering, complex-script shaping, per-character font fallback,
> and UAX #14 line breaking including CJK.** Not prioritised — *unavoidable*.

and then files the Thai fork as a renderer question:

> **The candidates fork here and it is a live input to [#7]:** … under `cosmic-text`
> Thai breaks at spaces only — silently and plausibly wrong. `parley` declares a
> `complex-scripts` feature … Neither has been run.

[#27](https://github.com/MBehtemam/Montagent/issues/27) ran it. The fork is real and
large: cosmic-text offers **zero** break opportunities in Thai, Khmer and Lao, parley
with the flag off is byte-for-byte identical, and the flag costs 3.82 MB of binary and
no startup time. That measurement made the contradiction load-bearing, because a
discriminator worth 3.82 MB is about to be weighted into [#7](https://github.com/MBehtemam/Montagent/issues/7).

## Decision

### The renderer never computes a line-break opportunity. The agent always needs one.

**These are two different questions and ADR-0007 collapsed them.** They look like one
question in English, where the break opportunities are the spaces and are free to see.
They come apart completely in Thai, which writes no spaces at all.

- **Render time.** Under "no automatic wrapping" the renderer has no use for an
  *opportunity* — a place text *may* break. Nothing chooses. The line partition is
  fixed by the mandatory breaks the author already wrote.
- **Authoring time.** The agent placing that break in `ฉันกำลังถือแมงมุมตัวใหญ่ไว้ในมือ` must know
  that `แมงมุม|ตัวใหญ่` is a word boundary and `แมงมุ|มตัวใหญ่` is not. Nothing in the string
  marks it. This is UAX #14 class `SA`, which the standard hands to morphological
  analysis "beyond the scope of the Unicode Standard".

ADR-0007 stated the transfer as a cost — *"this makes `frame`/`measure` mandatory in
the text authoring loop … That loop does not disappear — it moves to authoring time"* —
and then specified a tool that cannot pay it:

> `measure` (string + font + size → **advance width, ascent, descent, line count**)

Every term is metric. **`measure` gains a break-opportunity output**, under ADR-0007's
own general rule: *wherever this format refuses a convenience, the convenience belongs
in an authoring-time tool whose output is inert.* Its output is a `\n` frozen into a
committed file, so a later disagreement between segmenter versions cannot move a
shipped video — the same argument that admits `measure` at all.

**The `SA` dictionary fork is therefore an authoring-tool requirement, not a renderer
requirement.** `icu_segmenter` can be called directly by `measure` whichever crate
renders. [#7](https://github.com/MBehtemam/Montagent/issues/7) must not weight #27's
3.82 MB discriminator as a renderer criterion.

### `\n` stays the only break mechanism, and no wrap policy field is ever added

Unchanged from ADR-0007 and reaffirmed under pressure: a Thai authoring exercise run
through three agents produced no case for a `wrap`, `overflow` or `break` field. A
`wrap: "auto"` field would be *worse* in Thai than in English — under a stack with no
Thai dictionary it yields exactly one overflowing line, silently, which is ADR-0007's
own named failure: *"a field the renderer cannot honour is worse than no field."*

`box` remains a **check input**, not behaviour. `validate` states overflow as a fact;
the renderer never clips, shrinks or wraps it away.

### Montagent owns the line partition. It splits on mandatory breaks itself.

**Montagent splits the text into lines and hands each line to the layout stack as its
own paragraph.** It does not delegate the partition.

Three independent reasons, and it is required by ADR-0007 regardless of any defect:

1. **ADR-0007's own line-height rule forces it.** *"A line's height is the largest
   `size` among the runs on that line × `line_height`."* Montagent must know which runs
   are on which line before it can place anything; it cannot learn the partition from a
   shaper it then has to feed per-line metrics to.
2. **It is semantics-preserving.** Under no-auto-wrap, a mandatory break is the only
   thing that can end a line, so splitting there loses nothing.
3. **It removes a third party's mandatory-break handling from the critical path** —
   which measurement shows is not hypothetical. See below.

### Splitting is UAX #14, not `split('\n')`

The mandatory-break set is **BK, CR, LF and NL** — U+000A, U+000D, **CRLF as one break
and not two**, U+0085, U+000B, U+000C, U+2028 and U+2029. ADR-0007 commits to
*"UTF-8, NFC, written as raw characters"* with **"No tidying pass, ever"**, so a CR
introduced by an editor is content the format promises to preserve, and a naive
`split('\n')` leaves a stray `\r` inside the line.

**These characters also delimit bidi paragraphs**, so a mandatory break resets the
embedding level. The splitter is a paragraph splitter, and each line is laid out as an
independent bidi paragraph. This is the *proper subset* of UAX #14 that a strictly
no-wrap renderer still owes, and ADR-0007's blanket "the renderer never chooses a line
break" obscured it.

## Consequences

- **`measure` grows a break-opportunity output**, dictionary-backed for class `SA`,
  reporting the segmenter and data version alongside the offsets. It is the only place
  in Montagent that needs a Thai dictionary.
- **A conformance test is part of the renderer contract**, not a nicety: for every
  script in the fixture matrix, `lines == mandatory breaks + 1`, split at exactly the
  authored offsets. It is cheap and it catches the defect below.
- **A recorded upstream defect.** parley 0.11.1 **silently discards a mandatory `\n`**
  inside Thai, Khmer and Lao runs in its default wrap mode — 6 of 11 cases, with and
  without `complex-scripts`, font-independent and width-independent, while
  `icu_segmenter` v2.3.0 (**the same shared node parley links**) reports the boundary
  correctly. `TextWrapMode::NoWrap` — the mode this format calls for — renders it
  correctly. cosmic-text 0.19 is correct in all 11. Evidence and minimal repro
  (`"กก\nกก"` → 1 line): `docs/research/prototypes/thai-line-breaking/HARD-BREAKS.md`
  on branch `prototype/thai-line-breaking`. **This does not disqualify parley**, and
  under the owned-partition decision above it is unreachable either way — but it is
  exactly why the partition is owned.
- **Where `\n` may go is still a judgement, not a computation.** Two judges placed a
  *legal* break in different places in the same Thai sentence, for good and opposing
  reasons. `measure` reports what is legal; the agent chooses among the legal ones.
  No tool should pretend otherwise.

## Evidence

- **Automatic wrapping could not have produced the fixture's published output.**
  Measured in the fixture's real font at its real size against its real 984 px card:
  `sentence-06` (900 px) and `sentence-07` (880 px) **fit** and were broken anyway;
  a greedy UAX #14 wrapper reproduces **1 of the 5** multi-line elements and gets the
  other four wrong — two by not breaking at all, two by breaking elsewhere. All seven
  `.ass` files carry `WrapStyle: 2`. The breaks are editorial.
- **Three agents on three models, isolated, identical brief**, given a Thai edition of
  the fixture to author with no renderer. **3–0** that the renderer needs no
  opportunity, **3–0** that the agent does, **3–0** that it is not the same question,
  **3–0** against any wrap policy field. All three independently named ADR-0007's
  internationalisation paragraph as the text under most pressure. Two independently
  proposed the break-opportunity output; one independently proposed the owned split.
- **One was briefed adversarially** and made two falsifiable predictions; both had
  already been run, one falsified and one confirmed. The confirmed one is why the
  first draft's "parley cannot render the format" was **withdrawn**.
- **Setup flaws, recorded as flaws.** The brief said three of four fixture sentences
  carry a `\n`; two do. It said four `sentence-*` elements; there are five — all three
  judges found `sentence-quiz` independently. It asked for breaks in Thai strings that
  measurement later showed fit on one line. Judge A further observed the fixture's own
  transcript calls this "a degenerate bilingual case", so a real Thai edition would
  need new audio and would re-cut the video — the exercise was about the text
  primitive and the question stands, but the deliverable was not shippable.
