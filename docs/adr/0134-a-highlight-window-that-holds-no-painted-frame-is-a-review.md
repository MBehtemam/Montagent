---
status: accepted
amends: 0006 (`validate` gains a fact-only `review` check, `R-HIGHLIGHT-UNPAINTED`, one finding per document), 0051 (a third check on `highlight` windows, beside its two `error` checks, and the first that reads the frame grid)
---

# A highlight window that holds no painted frame is a `review`, under its own code

[#542](https://github.com/MBehtemam/Montagent/issues/542). The pack's
`presenter/take-1.words.json` gives "Now" as 2420–2440 ms. All three no-skills runs of
brief B copied that window into a `highlight` ([#447](https://github.com/MBehtemam/Montagent/issues/447),
BASELINE.md pattern 4 and product gap 6), and `validate` raised nothing. At 30 fps frame 73
is painted at ⌊73 × 1000 / 30⌋ = 2433 ms, so that window lights **one** frame. A 20 ms window
a few ms later, 2434–2454, lights **none**: the painted instants either side are 2433 and
2466. The document says the word is lit, and the video never shows it.

[ADR-0051](0051-word-alignment-is-external-validate-and-compare-catch-drift.md)'s two checks
catch a window outside its element and two windows that overlap. Neither reads the frame
grid, so neither can see this.

Decided by two rounds of `/court` (Opus, Sonnet, Haiku in round one; Opus, Sonnet, Fable in
round two; three jurors each, blind to one another), with the human ruling on each split.

## The decision

### 1. `validate` reports a window that holds no painted frame

A run's `highlight` window `[start, end)` that contains no instant ⌊n × 1000 / fps⌋ is
reported. The threshold is **zero frames**, and it comes from the format's own rendering
semantics ([ADR-0077](0077-the-nine-render-readings-are-ratified.md)'s painted instant). It
borrows nothing, so it needs no citation under
[ADR-0061](0061-validate-judgment-boundary-threshold-provenance-and-a-fenced-exception.md).
This is [ADR-0006](0006-validate-reports-facts-and-render-enforces.md)'s *"rounds out of
existence"*, one level down from an element: what quantization changes, never what is
unaligned. A window that lights one frame is not reported here (see §6).

For an integer window the test is the same whether it uses the exact instant n × 1000 / fps
or the floored one: with integer bounds `a` and `b`, ⌊x⌋ ≥ a ⟺ x ≥ a and ⌊x⌋ < b ⟺ x < b.

### 2. Class `review`. Unanimous in both rounds

The render is not refused and is not guaranteed wrong: the text is on screen, and an author
can mean it, for example by trimming a window to drop a word's emphasis. That is `review`,
the class every *"declared but never painted"* fact already takes: `N-QUANTIZATION`
(ADR-0006, [ADR-0105](0105-the-sheets-refusals-are-invocation-errors-its-blind-spots-are-not-findings-and-an-unpainted-state-is-quantization.md))
and `R-KEYFRAME-UNREACHED` ([ADR-0035](0035-keyframe-grid-alignment-is-a-review-check-not-a-schema-rule.md)).
`note` was rejected because it means *"you will not act on it today"*, and brief B shows that
an unreported invisible word ships. No `repair` field, since that belongs to `error` only.

### 3. Its own code, `R-HIGHLIGHT-UNPAINTED`. Split 2–1

Folding it into `N-QUANTIZATION` as a fourth condition was the dissent, and its argument is
real: a check is defined by its question, and *"what does quantization change?"* covers this.
It was rejected because every condition `N-QUANTIZATION` has is about **presence**: an
element, a gap, or a visual state, which [CONTEXT.md](../../CONTEXT.md) defines over *the set
of visual elements*. A highlight changes how a run is painted, not what is present, so it is
**not** a visual state and ADR-0105 does not cover it. A consumer that routes or silences
`N-QUANTIZATION` today would silently start receiving a finding about a different subject.
The new code sits beside `E-HIGHLIGHT-RANGE` and `E-HIGHLIGHT-OVERLAP`. The `R-` prefix is
the convention for a `review`, not a rule.

Like ADR-0118's state finding, it fires even where `N-QUANTIZATION` already reported the
window's element as vanished: they are findings about different subjects, and a check does
not triage against another check's matches.

### 4. One finding per document. Unanimous

A talking-head caption track has hundreds of runs, aligners give short function words tiny
windows, and the footage skill's captions script writes one text element per caption page.
So one bad words file can leave many windows unpainted across many elements. One per window
is the alarm fatigue ADR-0006 rejected. One per element still multiplies with the page
count. The usual cause is one upstream file, and one finding with a list says *"look at the
words file"*. The finding carries `fps`, the count, and a `detail` listing each window: its
element, run text, `start..end`, and the painted instants either side. That is
`N-QUANTIZATION`'s shape for conditions 1 and 2.

### 5. `validate` only, not `frame`. Unanimous

ADR-0105 §5 has `frame`'s range mode raise `N-QUANTIZATION` because the contact sheet skips
that state's tile, and the skip and the finding must be one identity. A highlight is not a
visual state, so the sheet neither draws nor skips anything for it. There is no `frame`
output for the finding to explain.

### 6. There is no fewer-than-N-frames check: no source gives an N

A window that lights one frame (33 ms at 30 fps), which is brief B's own case, may look as
invisible as one that lights none. But any N ≥ 2 is a perceptual claim, and ADR-0061 binds
it: allowed only at `review` or `note`, with the raw measurement as its substance and a
cited source. The glossary's **Check** keeps it out of this check, which is fact-only.

[#553](https://github.com/MBehtemam/Montagent/issues/553) asked whether any citable source
gives such an N, and the answer is **no**
([the findings](../research/highlight-visibility-floor.md)). The vision research measures
other things:

- **Chromatic critical durations** (about 100–200 ms) apply to detecting faint pulses near
  threshold. A full-contrast highlight is far above that, so they give no floor for it.
- **Change blindness** shows that attention, not duration, decides whether a change is
  noticed.
- **Rapid presentation** shows a single 13 ms frame can register, which argues against a
  frame floor at all.

The caption guides give reading and display times, which measure something else, as does
montagent-craft's 330 ms per word. So the borrowed check is **dropped**, and it comes back
only if a source that measures seeing a highlight change turns up.

The research does support one point as guidance, not as a threshold: a highlight is most at
risk when another change masks it at the same moment, such as a cut, a whole-frame change or
many words changing at once. Being short matters less.

## What this does not catch

**Not brief B's root cause.** The words file is about 110 ms late: the audio shows "Now"
starting at about 2310 ms (`silencedetect`), not 2420. A window can light many frames and
still be wrong against the audio, and no frame count can see that. Checking a window against
the media on disk is the gap ADR-0051 named and left open, and it stays open. On brief B's
own numbers this check is silent: the window lights one frame. What it adds is the guarantee
that a word the document lights is never painted on no frame at all. For agents with the
footage skill, its captions script stays the remedy for short and misaligned words: it
lengthens any word under its `min_word`, checks timings against detected pauses, and says
what it changed.

## Consequences

- `validate` gains `R-HIGHLIGHT-UNPAINTED`: `review`, threshold internal, document set, one
  finding per document, nothing emitted on a project where every window holds a frame.
- `frame`, `render`'s refusals, and `N-QUANTIZATION`'s meaning are unchanged. `render` runs
  the identical check as a `review`, which withholds nothing.
- The build is its own ticket. Its tests must include: 2434–2454 at 30 fps fires; 2420–2440
  at 30 fps does **not** fire (one frame); 2401–2433 at 30 fps fires (half-open: a window
  ending on a painted instant does not light it); several unpainted windows across two
  elements produce **one** finding listing all of them.
- There is no fewer-than-N-frames check. #553 found no citable source for an N, and one
  that measures seeing a highlight change is the only thing that would reopen it.
- The audio-drift gap from ADR-0051 is unchanged and still open.
