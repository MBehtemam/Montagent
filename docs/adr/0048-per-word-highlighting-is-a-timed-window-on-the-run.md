---
status: accepted
amends: 0007 (a run gains an optional timed style delta, in addition to its unconditional one), 0012 (confirms keyframes stay transform-only; this is a separate, non-transform timing construct), 0014 (extends the run-addressable paint precedent stroke established), 0040 (settles the deferral that ADR excluded from its own scope)
---

# Per-word (karaoke) highlighting is a timed window on the run: no run ids, no event array, no automatic wrapping

> **Amended by
> [ADR-0051](0051-word-alignment-is-external-validate-and-compare-catch-drift.md).** Fulfills
> the deferred authoring-time-tool obligation.

[ADR-0040](./0040-effect-model-attachment-and-v1-vocabulary.md) excluded per-word
highlighting from the effect model on two independent grounds — addressing (a word
lives inside a run, not at the element level effects attach to) and timing (the
interesting, animated form needs audio-synced timing, and keyframes are transform-only)
— and graduated the animated case to this ticket
([#106](https://github.com/MBehtemam/Montaget/issues/106)). A **static** per-word
highlight (fixed paint, no timing) was already expressible with the existing run
style-delta mechanism; only the animated, audio-synced form was open.

No worked example exists to check this against: the project's one real fixture has zero
inline per-word highlighting, and no word-level timing data exists anywhere in its
source materials — only whole-sentence audio and a plain transcript with no word-level
timestamps. This design is built on inference, not evidence from the reference project,
which [ADR-0003](./0003-general-video-editor-not-channel-tooling.md)'s asymmetry
explicitly permits: the fixture's silence is not evidence the capability is unneeded.

## Decision

**A run gains an optional `highlight` object: a start/end time window (within the
element's own range) plus the style delta that applies during it, falling back to the
run's unconditional style outside the window. No run ids. No element-level timed-event
array. Word times are literal integers, authored once and frozen in the document.**

```json
{"text":"cobwebs","color":"#FFFFFF","highlight":{"start":5500,"end":5680,"color":"#FFD34D"}}
```

### The window lives on the run, not in a separate array

A run already owns the word's text and its unconditional style; it also owns when and
how that word lights up. The alternative — a `{t, run, style}` event array at the
element level, addressing runs by a new `id` field — was rejected because it
reintroduces the exact join the project rules out everywhere else: to know what a word
looks like at time *t*, a reader would hold the `runs` array and a second array in mind
at once and reconstruct the join by hand. That is evaluation performed by the reader,
which is the failure this format's "understandable by reading" rule exists to prevent.

It also reverses two settled boundaries rather than extending them. [ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md)
put paint at run level on the grounds that *"a run lives inside"* the element effects
attach to; ADR-0040 excluded karaoke on the same addressing ground. An element-level
event array hoists run-level paint back up to element level, undoing both. Its
resemblance to [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)'s
keyframes is a false analogy: keyframes are element-level because transforms *are*
element-level properties. Copying their shape for a run-level paint property borrows the
syntax while discarding the reason it fit there.

Run ids are also a cost with no current payoff: a new addressing concept in a file whose
editing model is exact-string matching against a one-line serialisation, opening a
failure class that does not exist today — an orphaned id, a duplicate, an event that
outlives the run it names — each needing its own validation and error text this ADR
would otherwise never have to write.

### Every affected word is its own run; a run carries at most one highlight window

Karaoke timing is per-word by definition regardless of encoding, so this fragmentation
happens either way — the only choice is whether it happens along the addressing unit
the format already has (a run) or a new one invented for this feature alone (character
offsets into a shared run). A run is already the unit of style-delta addressing: one
text span, one delta set, findable by unique substring. A `highlight` window is another
style delta, gated by time; keeping it 1:1 with the run keeps that invariant. Letting one
run span several words, or carry several windows, would need sub-run text offsets to say
which characters a given window paints — exactly the sub-run addressing this format's
runs exist to avoid, and a silent-desync hazard on top: a text edit to the run leaves the
JSON structurally valid while invalidating every offset inside it, with nothing to flag
it.

A word that highlights twice (a flash) is not an argument for multi-word runs or
sub-run offsets — it is `"highlight":[{...}, {...}]` on a single-word run, an array of
windows over one still-word-sized text span. This ADR does not adopt that shape
preemptively; it is named so a future ticket extending to multiple windows per word has
a stated, narrow landing spot rather than reopening run/word granularity from scratch.

### Word times are literal integers, frozen at authoring time — never a reference into an external source

The rejected alternative — the element points at an external word-timing source
(an ASR/forced-alignment JSON) and the renderer resolves per-word times at render
time — looks like it avoids duplicated data, but it is a larger surrender of
"understandable by reading" than [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md)'s
fit-to-box rejection, not an equivalent case: fit-to-box was rejected because the
computed value is the *renderer's own opinion*, changing when the renderer's shaper
changes. Word timings are a measurement of a fixed artifact (the audio) by a process
that is not the renderer and has no reason to live in it — deferring to a live reference
would put an ASR model's opinion inside the render path itself, making a video's content
nondeterministic with respect to a tool this project does not ship, and would force the
renderer to version a third-party alignment format it does not own.

More fundamentally, a highlight window is not a derived convenience value at all — it is
content, the same status a clip's `start`/`end` already has. That the number originated
from a forced aligner is no more relevant to its status in the document than that an
in-point originated from someone scrubbing a timeline. Every time in a Montaget document
is a literal someone or something decided; word times are not a special category needing
a special mechanism, and this is the same authoring-time-freeze pattern
[ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) already established for
fitted font sizes: *"the loop does not disappear — it moves to authoring time and
freezes its result in the file."*

## Consequences

- `runs[]` items gain an optional `highlight` object: `{start, end, ...style delta}`,
  where `start`/`end` are absolute milliseconds within the element's own `[start, end)`
  and the style delta shape matches the run's existing unconditional delta fields
  (colour, and any other run-addressable paint field already in the schema).
- No `id` field is added to runs. No new element-level array is added. `validate` needs
  no new join logic to answer "what does this run look like at time *t*" — it is local
  to the run.
- **The project now owes an authoring-time tool to produce word times.** None exists:
  the fixture's own source materials carry no word-level timestamps, only per-sentence
  audio and a plain transcript. Writing per-word `highlight.start`/`end` by hand through
  a generic edit tool is exactly the error-prone, unscalable case the fitted-size
  precedent already flagged for `frame`/`measure`. This ADR does not design that tool —
  it names the obligation so it is not silently assumed solved.
- A karaoke-highlighted sentence fragments into one run per highlighted word (plus
  unhighlighted runs for the words around them), which is more runs and a longer
  single-line serialisation than an un-highlighted sentence needs. Accepted as the cost
  of keeping every highlight locally readable and exact-string-editable.
- Multiple highlight windows on one run (a flashing word) are a stated, narrow future
  extension (`highlight` becomes an array), not adopted here. Spanning several words in
  one run, or addressing a highlight by character offset, remains rejected on the same
  grounds `path`/`polygon` were rejected in [ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md):
  the content the check would need is not in the document as a declared value.

## Evidence

Three courts, one per sub-question, three jurors each (Claude Opus, Claude Sonnet 5,
Claude Haiku 4.5 — independent, blind to each other's ballots). **Unanimous 9/9** across
all three questions and all nine ballots.

- **Mechanism** (timed overlay on the run vs. a separate run-id-keyed event array):
  unanimous 3/3 for the run-level overlay. Independently-named reasoning across all
  three: the event array reintroduces the external-join failure the format is built to
  avoid; it reverses the stroke/effects run-addressing precedent instead of extending
  it; its resemblance to transform keyframes is a false analogy since transforms alone
  are element-level; and run ids are a new addressing concept and failure class paid for
  with no current benefit.
- **Time source** (frozen literals vs. a live reference into an external timing
  source): unanimous 3/3 for frozen literals. Independently-named reasoning: fit-to-box
  was rejected because the computed value was the renderer's own opinion, and a live
  timing reference is a *larger* violation of that rule, not an equivalent one, since it
  would put a third-party model's output inside the render path itself; a highlight
  window is content (an authorial assertion) regardless of what produced the number,
  the same status every other literal time in the format already has; and a changing
  external transcript is not a case this format should silently re-resolve, since
  anything that moves word timings almost always moves the sentence text and shot list
  too, making a silent re-derivation a render whose output moved without a diff.
- **Granularity** (one run per highlighted word vs. a run spanning multiple words or
  carrying multiple windows): unanimous 3/3 for one run per word. Independently-named
  reasoning: a run is already the addressing unit for style deltas, and a highlight
  window is one more style delta at that same grain; multi-word or multi-window runs
  need character-offset addressing, which is exactly the sub-run addressing this
  format's runs exist to avoid and which desyncs silently under a text edit; the "word
  flashes twice" case is answerable as an array of windows on one still-word-sized run,
  without ever introducing offsets.
