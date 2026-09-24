# Prototype: the project file for `en-halloween-decorating`

> **Historical record.** This describes the sample project **as it was written for #9**,
> before ADR-0007 and ADR-0012 landed. Its field names are the pre-migration ones — `box`,
> `align` on images, `"text"` instead of `runs`, positional `scale` pairs. The current file
> lives at
> [`fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json`](../../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json);
> what changed and why is in
> [`docs/research/sample-project-migration/`](../sample-project-migration/README.md), which
> also records three findings that land on ADR-0012 itself. Read this document for the
> *evidence and the juries*, not for the schema.


Answers [#9](https://github.com/MBehtemam/Montagent/issues/9) — *"write, by hand, the
complete project file that would produce the reference short, and see whether it is
something an agent could plausibly author and edit."*

The artifact is [`en-halloween-decorating.montagent.json`](./pre-migration.montagent.json):
**14 tracks, 60 elements, 154 lines, 12.9 KB**, expressing all 65.216 s of
`fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4`.

It is **throwaway**. It is not a schema, not a spec, and no renderer exists to run it.
Everything it claims about what would appear on screen was read off the fixture and the
published MP4, never rendered. Fields marked **INVENTED** below are guesses at decisions
the map has not made yet — [#21](https://github.com/MBehtemam/Montagent/issues/21) transform,
[#22](https://github.com/MBehtemam/Montagent/issues/22) effects, the shape primitive, the
text model. Do not read them as proposals; read them as *the shape of the hole*.

The ticket named the pumpkin short and "ten image/audio/subtitle triples". Both are stale
— [#3](https://github.com/MBehtemam/Montagent/issues/3) substituted the decorating short,
which has four items. The exercise is unchanged.

---

## Verdict against the four questions

| Question | Answer |
| --- | --- |
| Can a reader answer *"what is on screen at 6.2 s"* by reading only? | **Yes** — verified against the real frame, and replicated 5/5 by independent judges. But it is a scan of **14 arrays**, and those same judges counted the tracks five different ways (§A, cross-examination). |
| Could an agent write this file in one pass, without a bespoke API? | **No.** Three classes of number are unknowable from the inputs (§C). |
| Is *"move the subtitle down on every card"* a small edit? | **One replace-all, 5 hits.** The formatting convention earns its keep — but the caption slot beside it is unsafe in *both* directions, over-matching on `"y":1470` and under-matching on `"y":1373` (§F, cross-examination). |
| Does anything want a concept the model does not have? | **Yes, five.** None want a concept the model *forbids* — confirmed 5/5 by independent judges, with one partial dissent (§G, cross-examination). |

---

## A. The read test passes, and the cost is now nameable

At 6200 ms the file says: `photo-05`, `word-05` (*cobweb - cobweb*), the eight header
elements, `vo-word-05-a` playing — and no sentence card, because `card-05` starts at
10468. Checked against the published frame at 6.2 s: correct, including the absence.

No arithmetic anywhere. ADR-0001's driving requirement survives contact with the real
short.

But **ADR-0004's estimate of the cost is low**. It said the read becomes *"a filter over
two nested arrays instead of one"*. In practice it is a filter over **fourteen**, because
of §B — and the answer is assembled from fourteen separate places rather than read off
one. Still a read. Still inside a turn. But "two nested arrays" is not what this looks
like.

## B. The non-overlap rule is purely temporal, so static chrome explodes into tracks

The header — cream chip panel, three flag rectangles, the chip text, the handle panel, the
logo badge, the handle text — is **eight objects that never move, never change, and never
overlap each other in space**. Every one is on screen for all 65.216 s, so every one
overlaps the others *in time*, so each needs its own track.

**Nine of the fourteen tracks hold exactly one element.** `chip-panel`, `flag-field`,
`flag-bar-h`, `flag-bar-v`, `chip-text`, `handle-panel`, `handle-logo`, `handle-text` and
`caption-overflow` are tracks in the way an empty box is a container.

The only thing marking these eight as one thing is `group: "header"`, which the renderer
ignores. This is [#23](https://github.com/MBehtemam/Montagent/issues/23) arriving from a
second direction: ADR-0004 left *"the interaction between `group` and `track` is now an
open question"*, and the fixture answers *"and here is what it costs when you don't"*.

Note what is **not** wrong here: Premiere and CapCut would also put these on eight video
tracks. The complaint is not that tracks are the wrong container — it is that a track
carries *stacking* and the thing the author actually needs to name is *co-timed chrome*,
and those are different groupings that the format currently conflates into one.

## C. An agent could write this file in one pass — except for the numbers it cannot know

Three classes, in descending severity.

### C1. Fitted font sizes — the format's first real evaluation problem

The word slot is 88 px for `cobweb`, 88 for `spider`, **80** for `skeleton`, and **49 + 35**
for `string of lights`. The sentence is 55, 57, 57, 55. The intro is 58, the hook 88, the
quiz question 73, the countdown 249. Not one of these is a style decision. The old pipeline
**measured the rendered string and shrank until it fit**.

An agent writing `"size": 88` for `skeleton  -  skeleton` produces overflow, and **nothing
in the file says so**. The file still reads correctly; it just renders wrong. This is a
failure mode the inert-data principle does not protect against, because the document is
internally consistent and the defect lives in the glyph metrics.

This is the **first legitimate candidate for a computed property in the format**, and it is
categorically different from the two ADR-0005 rejected. ADR-0005's argument against implicit
fill and implicit speed was that they need *the source file's duration, which is not in the
document at all*. Fit-to-box needs the font, the weight, the string and the box — **all four
are in the document**. It ships its own inputs. The only external dependency is the font
file, which the renderer must load anyway.

So the choice is real, and it is not settled by any existing principle:

- **literal `size`** — inert, and the agent must run `frame` or `query` to discover overflow.
  That makes self-verification **mandatory in the authoring loop**, not the optional
  belt-and-braces the map currently describes it as.
- **a fit rule** (`max_width` plus shrink-to-fit) — one field, one line, deterministic, and
  the reader can no longer tell the rendered size by reading. `"what is on screen at 6.2s"`
  is unaffected; `"how big is it"` becomes unanswerable without the renderer.

Belongs to the text model. Nothing on the map currently owns it.

### C2. Hand-computed line positions

Item 08's word is **one** subtitle event with two font sizes inside it
(`{\fs49}string of lights\N{\fs35}string of lights`). A text element with a single `size`
cannot express that, so it becomes two elements — and then **I** had to compute
`y: 1352` and `y: 1398` from a block centre of 1373 and two line heights. The renderer
used to do that arithmetic; splitting the element moved it to the author.

And because the two halves are simultaneous, they need **two tracks** (`caption` and
`caption-overflow`). A single visual slot, split for a typographic reason, spans two tracks
— which is §B again, from a third direction.

### C3. Every `source_end` is a probe result

The 20 narration elements carry `source_end` values that exist nowhere but in the mp3
headers. Writing this file required **11 `ffprobe` calls before the first element was
typed**.

This is not a contradiction of ADR-0005, which rejected *probe-before-every-write* — 11
probes for 11 distinct sources is not eleven speculative probes per line. But it does
sharpen the shape: **probe is per-source, not per-element or per-write**, and it is
front-loaded, not incidental. Had I written the file first and let `validate` catch it, the
repair would have been 20 wrong `end` values, not 11 lookups.

## D. `speed` is not deferred vocabulary — the fixture requires it

**Every sentence is spoken twice: once at normal speed, once at ≈0.645×.** Every word is
spoken twice, both at normal speed, 800 ms apart.

Nothing in `beats.json`, `transcript.json` or the fixture README records this. `beats.json`
names one `sentence_*` beat per item. It took a silence analysis of the published MP4 to
find it, and the ratio is stable across all four items (1.5535 / 1.5442 / 1.5474 / 1.5534 →
mean **1.5496**, so speed ≈ **0.645**; the intended value was plausibly 0.65 and the 0.7 %
gap is within the measurement's error — the file must state whichever it is explicitly
either way).

Consequences:

- **Four of the 20 narration elements carry `speed`.** Without it the file is four elements
  short and eight seconds of the published video are unaccounted for.
- The map's fog entry *"the `speed`, `hold` and `loop` vocabulary"* is **promoted from
  nice-to-settle to required by the fixture.** It also confirms that entry's own worry:
  `speed` does reopen the source-range invariant.
- **ADR-0005 states the invariant in its speed = 1 special case.** It says internal
  consistency means `end - start == source_end - source_start`. With speed the invariant is
  `end - start == (source_end - source_start) / speed`. ADR-0005's *claim* survives intact —
  the check is still decidable by reading the element alone with no I/O — but the *formula*
  as written is wrong for any element that declares a speed, which is exactly the case
  ADR-0005 anticipated and did not fold back into the rule.

## E. Ken Burns: absolute keyframes work, with two surprises

The move is a slow centre-anchored zoom, ≈1.0 → ≈1.08 over a fixed 15 s, from a 1536 × 2720
still into a 1080 × 1300 box, top-aligned. Written as **INVENTED**
`box` / `fit` / `align` / `scale` keyframes, since [#21](https://github.com/MBehtemam/Montagent/issues/21)
is open.

**Surprise 1 — the move restarts at every segment boundary.** Confirmed by comparing frames
at 2.90 s / 3.20 s and 63.90 s / 64.20 s: the zoom snaps back. Image `05.png` is on screen
for the intro, its own segment, the quiz and the loop tail, and **restarts all four times**.
So one still becomes four elements with four independent moves — not one element spanning
four beats, which is how the fixture README describes it.

**Surprise 2 — keyframes legally fall outside the element, and outside the project.**
Absolute keyframe times (the only kind ADR-0004 and ADR-0005 permit, since there is no local
clock) make the trimmed move *visible*: `photo-05` carries `[[3018,1.0],[18018,1.08]]` and
ends at 17472. That reads well — you can see the move was cut short. But two elements carry
keyframes at 68856 and 79016 against a project duration of 65216, and a naive validator
would flag both. **`validate` needs an explicit rule here, and the rule is "don't"** —
a keyframe past the end is how a trimmed move is spelled.

**The templating cost, made concrete:** the same 15 s move is retyped seven times with seven
different absolute time pairs. This is the accepted v1 cost, and it is exactly as bad as
advertised — no worse, and it does not obstruct a later template, since seven uniform
elements are a clean expansion target.

## F. The subtitle edit is one replace-all — and the near-identical edit is a trap

*"Move the subtitle down on every card"*: the five sentence texts all carry `"y":1537`, and
that substring appears **exactly five times in the file**. One replace-all, done. The five
navy cards likewise share one identical `"box":[48,1453,984,169]`.

This works **only** because ADR-0005's writing convention holds — one element per line,
stable key order — so a unique matchable substring exists. It is the strongest evidence yet
that the convention is load-bearing rather than cosmetic, and it should be normative on
every writer, not advisory.

**The trap:** `"y":1470` appears **nine** times — the intro title, the hook, the quiz
question, five countdown digits and the loop-tail hook. They are not one visual concept;
they merely share a slot. A reader who succeeded with the sentence edit and repeats the move
on the caption will move all nine and be wrong about at least the countdown. **`group` does
not separate them** — they span `intro`, `item-05`, `quiz` and `loop-tail`. Nothing in the
file distinguishes "these five are the same thing" from "these nine are in the same place",
and the file gives the reader no signal which case they are in.

## G. Concepts the model does not have

Five, all INVENTED here:

1. **`box` / `fit` / `align`** on an image — the entire spatial model. [#21](https://github.com/MBehtemam/Montagent/issues/21).
2. **Transform keyframes** (`scale`) — [#21](https://github.com/MBehtemam/Montagent/issues/21).
3. **A `rect` shape with a `fill`** — the map's *"shape primitive"* fog. Nine of the 60
   elements are rectangles. Note they are **sharp-cornered**, so a plain rect suffices; see
   §H.
4. **`mask: "circle"`** on the logo badge. Probably [#22](https://github.com/MBehtemam/Montagent/issues/22).
5. **Text style within one element.** See C2.

Point 5 has a consequence worth stating on its own. [#19](https://github.com/MBehtemam/Montagent/issues/19)
settled that *"bilingual subtitle" is not in the format — two text elements sharing a group*.
The fixture renders the word slot as **one string**, `"cobweb  -  cobweb"`. Under #19 that
should be two elements — and then they are simultaneous, so they need two tracks, and the
`  -  ` separator has no owner at all. **The separator is a rendering of a pair, not a
pair.** The fixture's degenerate case (target == bridge) hid this for items 05–07; item 08,
where the pipeline broke the pair onto two differently-sized lines, exposes it.

I wrote 05–07 as the fixture rendered them — single strings — rather than silently
re-modelling them, because the two readings disagree about how many elements the short has
and that is a decision, not a transcription choice. **#19's ruling and the fixture's output
are in direct tension and something must give.**

Nothing needed a concept the model **forbids**. No scene, no clip, no nesting, no
expression, no local clock. The rejected terms stayed rejected without effort, which is the
quietest good news here.

## H. Corrections to the fixture README

- **The header panels have sharp corners, not rounded.** The README says *"a rounded cream
  panel"*; the ASS draws four-point paths and the published frame at 8× magnification agrees
  with the ASS. Recorded because it changes whether the shape primitive needs a corner
  radius on day one — it does not.
- **"Total narration: 22.4 s of speech inside a 65.2 s video" undercounts.** That is each
  file played once. The video plays every word twice and every sentence twice, so ~35 s is
  delivered narration and the silence the README calls *"load-bearing content"* is
  substantially smaller than stated. The claim's *point* stands; its number does not.
- **`beats.json`'s 15 beats do not describe the timeline.** The file needs 60 elements. The
  repeats, the 800 ms word pause, the 520 ms sentence pause and the 0.645× slow pass are all
  implicit in the old pipeline's code — which is precisely the disease Montagent exists to
  cure, and a good illustration that the old config is a *template's parameters*, not a
  project.
- **Timing sources disagree slightly and are not smoothed here.** `beats.json` rounds to
  10 ms and drifts up to 26 ms from the segment-duration sum by the loop tail (63.99 vs
  64.016). The segment sums are used throughout. The published MP4 is 65.259 s against the
  segments' 65.216 s.

---

## What this hands back to the map

**Sharpened enough to ticket:**

- **The text model** — `align`/anchor, multi-line, line height, and above all the fitted-size
  question of §C1. Nothing on the map owns text as a primitive, and it is the single largest
  source of unknowable numbers in the file.
- **`speed`** — promoted from deferred vocabulary to a fixture requirement (§D), carrying the
  ADR-0005 invariant correction with it.

**Evidence for tickets that already exist:**

- [#23](https://github.com/MBehtemam/Montagent/issues/23) (`group` × `track`) — §B gives it a
  second, sharper instance than the one it was opened with: eight co-timed static objects
  across eight tracks, tied only by a label the renderer discards.
- [#21](https://github.com/MBehtemam/Montagent/issues/21) (transform + keyframes) — §E hands
  it the absolute-keyframe reading, the out-of-range rule, and the restart-per-appearance
  fact.
- `validate`'s report format (map fog) — inherits the out-of-range-keyframe rule from §E and
  the fit-overflow class from §C1, neither of which is a gap, an overlap, or a probe error.

**Unresolved tension flagged, not resolved:** #19's two-elements-sharing-a-group ruling
against the fixture's single-string word slot (§G).

---

# Cross-examination

The draft above was written by one agent. It was then handed to **five independent judges**
— Opus, two Sonnets (one briefed as a hostile reviewer), Haiku and Fable — each given the
same four questions, the domain docs, the fixture and ffmpeg, and each **blocked from
reading this file or issue #9**. What follows is what they found, including where they
falsified the draft's own conclusions.

**A contamination to declare first.** The fixture README corrections in §H were committed
*before* the judges ran, on the same branch. Four of the five read the 0.645× speed fact,
the repeat structure and the Ken Burns restart straight off that appendix rather than
discovering them. Only the hostile Sonnet rederived the doubled narration independently,
via `ffmpeg silencedetect` against the shipped MP4, and confirmed it matches the draft's
20 narration elements. Treat §D as **replicated once**, not five times — and the setup
error is the author's, not the judges'.

## What the judges agreed on

**The read test is unanimous.** 5/5 answered "what is on screen at 6.2 s" correctly —
`photo-05`, `word-05`, the eight header elements, `vo-word-05-a`, and *no sentence card* —
and every one that checked verified it against the extracted frame. Zero arithmetic. §A
stands.

**Nothing forbidden crept in.** 5/5 walked the rejected list — scene, clip, asset, nesting,
local clock, expression language — and found none. §G's closing claim stands, with one
partial dissent (below).

**Every arithmetic invariant holds.** Independently re-derived by four judges by script:
60 elements, no per-track overlap, tracks sorted by `start`, segment durations summing
exactly to 65216, all ASS `&HBBGGRR` → `#RRGGBB` conversions correct, every box matching
its ASS draw path, every font size matching its ASS override.

## The finding the draft missed entirely

**90 % of the file's cut points do not land on a frame boundary.** At `fps: 25` a frame is
exactly 40 ms. Of the file's 120 start/end values, **109 are not multiples of 40** — and
the only aligned value in the whole file is `0`. The declared `duration: 65216` is
**1630.4 frames**, not a whole number of them.

This is not the 30 fps rounding problem
[ADR-0005](../../adr/0005-absolute-integer-milliseconds.md) excuses. 25 fps divides
milliseconds cleanly; the draft simply inherited the old pipeline's marks (`beats.json`
rounds to 10 ms) and never applied the ADR's own rule to its own output. ADR-0005 requires
`validate` to note non-aligned boundaries and requires the renderer to publish its rounding
rule — **the first real project file is a 90 % failure of that check**, written by an author
who had read the ADR.

It promotes the map's *"Frame alignment and the rounding rule"* fog entry from a renderer
detail to an authoring problem: if every hand-written time is wrong by up to 39 ms, the
rounding rule is not a footnote, it is the thing that decides where every cut lands.

## Where the judges falsified the draft

**1. `fit: "cover"` is the defect ADR-0005 rejected, wearing a different hat.** *(Opus.)*
To know which pixels of `05.png` are on screen you need its dimensions — 1536 × 2720 —
**which are not in the document**. ADR-0005 killed implicit `fill` "in its strongest form
yet" on exactly that ground: *worse than an expression language, which at least ships its
own inputs.* §E invented `fit` and did not notice it reproduces the sin verbatim. The
draft's §C1 argument that fit-to-box is *legitimate* because its inputs are all in the
document is correct **for text and wrong for images** — and the draft applied it to both.

**2. §E's "absolute keyframes are a win" is the weaker reading.** *(Opus, with Fable
concurring.)* Every keyframe pair is `[[start, 1.0], [start + 15000, 1.08]]`. That second
number is a **relative duration materialised into an absolute one** — the dual
representation ADR-0005 "rejects outright", whose stale-half failure it predicted. Hand-edit
a photo's `start` and both keyframes silently desync from it. §E read the out-of-range
keyframe as expressive; it is at least as good a reading that it is a leak.

**3. `speed` is over-determined, and has already gone stale.** *(Opus.)* `2568 / 0.645 =
3981.40` against a declared `3981`; `1992 / 0.645 = 3088.37` against `3088`. Three numbers,
two of which disagree at integer milliseconds, and **nothing says which the renderer
obeys**. This is precisely the failure ADR-0005 used to kill dual representations, occurring
inside the prototype meant to test it. [#25](https://github.com/MBehtemam/Montagent/issues/25)
must pick two of {timeline range, source range, speed} and derive the third.

**4. Layer collides across tracks, and no anchor was used.** *(All five.)* `chip-panel` and
`handle-panel` both declare `layer: 30`; `flag-field`, `handle-logo` and `handle-text` all
declare `31`. ADR-0004 gives no tie-break between two tracks claiming one integer. It renders
correctly only because those boxes happen not to intersect — safety that is invisible without
cross-referencing every box on the layer. Worse: **the file uses zero anchors and zero
per-element layer overrides**, and hand-maintains a 30/31/32/33/34 ladder instead. That
ladder is the exact arithmetic all eight agents in ADR-0004 demanded anchors *to prevent*.
Belongs to [#24](https://github.com/MBehtemam/Montagent/issues/24).

**5. `shift` cannot extend the tail, and does not mention keyframes.** *(Opus; Fable.)*
`65216` appears as a literal **11 times**. Lengthening the video is 11 exact-string edits,
and `shift(at, delta)` cannot help — there is no instant at-or-after which anything needs
moving. Separately, keyframe times are absolute, so `shift` must rewrite them, and
ADR-0005's spec never says so. Two holes in the shift/absolute bargain.

**6. The replace-all trap is worse than §F said.** §F found `"y":1470` matching nine
elements that are not one concept. The hostile Sonnet found a third variant the draft
missed: **`"y":1373` matches only 4 of the 5 word slots**, because item 08 is split across
two elements at 1352/1398 on another track. An agent doing the obvious edit silently misses
item 08 and ships an inconsistent card with no error. So the caption slot is unsafe in *both*
directions — over-matching on 1470, under-matching on 1373.

**7. The card↔text coupling is sharper than §F stated.** *(Opus, Sonnet, Fable.)* Moving the
subtitle is **two** replace-alls on **two different coordinate systems** — text `y: 1537`
(a centre) and card `box: [48,1453,984,169]` (top-left + size) — kept in sync only by the
author knowing `1537 ≈ 1453 + 169/2`. Miss the second and the text slides off its card, and
**nothing validates it**. `group: "item-05"` is far too coarse to select the pair: it also
contains the photo, the hook, the word and six audio elements. `anchor` handles z-order, not
position. Direct evidence for [#23](https://github.com/MBehtemam/Montagent/issues/23).

**8. Text `y` has no declared vertical anchor.** *(Opus.)* `chip-text` at `y: 130` with
`align: "left"` transcribes ASS `\an4` — left-*middle* — so `y` is a centre. Nothing in the
file says so, and `align` names only the horizontal axis.

## New facts, not corrections

- **The countdown is genuinely silent.** *(Fable.)* Measured −91 dB under the 5-4-3-2-1
  digits, so no audio element was owed there. The draft's omission was right.
- **The fixture came one `null` from needing a second audio track.** *(Opus.)*
  `beats.json` carries `"hook_bed": null` — a music bed under the hook. Had it been set, the
  narration track could not hold it.
- **The Ken Burns move is zoom only, no pan.** *(Sonnet A, Fable.)* Both compared first and
  last frames of `kenburns/05.mp4`. The fixture README calls it "pan/zoom"; scale-only is
  correct, though the draft asserted rather than established it, and with no stated transform
  origin there is no way to tell from the file.
- **Boundary reconciliation is unmarked.** *(Fable.)* The draft systematically prefers
  segment-sum arithmetic to the ASS — `17472` vs the ASS's 17.468, `64016` vs 63.996 — each
  defensible, none disclosed.

## Judge errors, recorded

Not everything the judges reported survived checking, and this matters for how much weight
their agreement carries:

- **Track counts: 13, 14, 15, 8-of-13, 12-of-15.** The true figures are **14 tracks, 9 of
  them single-element**. Opus said 13, Fable said 15, Sonnet A and the hostile Sonnet said
  14. The draft itself said "six of the fourteen" and was also wrong. **Five careful readers
  produced five different structural counts of a 154-line file, while all five read
  "what is on screen at 6.2 s" identically and correctly.** That is the cost of §B stated as
  a measurement rather than an opinion: *presence reads reliably; structure does not.*
- Opus reported the word→sentence pause drifting (798/804 ms) and a 4 ms "hole" at the quiz.
  Both are wrong — the pauses are uniformly 800 ms and 520 ms across all four items, and the
  4 ms is the audio file ending before its text does, which is expected.
- The hostile Sonnet said "the two `speed:0.645` elements". There are four.

The strongest report contained the most confident wrong details. Worth remembering before
any of this is promoted into an ADR.
