---
status: accepted
amends: 0005 (`shift`'s defining sentence is wrong for keyframes), 0006 (the overflow check gains an aperture term), 0007 (`box` becomes literal `width`/`height`; `align` splits), 0011 (the crop rectangle becomes computable, and `shift` is unblocked)
---

# An element carries a flat transform, and keyframes are carried by their element

> **Amended by [ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md)**: the
> `photo-06` element published below carries `"gravity":"top"`, and **`gravity` no longer
> exists** — the declared rect plus `clip` already determine which part of the source survives.
> Read that element without the `gravity` key. Nothing else in it changed.

> **Amended by [ADR-0013](./0013-fitted-extents-floor-and-the-nine-origin-keywords.md)** on
> the scope of one sentence; every decision below stands, and the `photo-06` element
> published here is **correct as written**. **"Ties away from zero" does not reach fitted
> extents** — it is scoped to `x`/`y` interpolation residuals under SPLIT, where "nearest"
> is right for a value with no directional constraint. A fitted extent is a derived bound:
> it floors, in exact integer arithmetic, with the driving axis assigned the box dimension
> verbatim and chosen by integer cross-multiplication. ADR-0013 also supplies the sampling
> rule this ADR asked a successor for (*"the rectangle is not integral: exact cover here is
> 1912.5 px"*), spells the nine `origin` keywords this ADR used but never enumerated, and
> contributes a measured fact to the open `gravity` question below without closing it.

Every visual element is placed the same way — `x`, `y`, `origin`, plus a declared size —
in **absolute integer pixels** on the project's frame. Any transform property may be a
value or a list of **`{"t","v","ease"}`** records on the project's one absolute clock.
A keyframe's `t` is written in timeline coordinates but **is not a timeline time**:
`shift` moves *elements*, and their keyframes are carried.

```json
{"id":"photo-06","type":"image","group":"item-06","start":17472,"end":30603,"source":"images/06.png","x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover","gravity":"top","clip":[0,0,1080,1300],"scale":[{"t":17472,"v":[1.0,1.0]},{"t":32472,"v":[1.08,1.08],"ease":"ease-in-out"}]}
```

Nothing in `CONTEXT.md` or ADRs 0001–0011 could say where an element is, how big, how
rotated or how opaque. This ADR says it, and it settles the keyframe rule that
[ADR-0011](./0011-tool-surface-reads-checks-renders.md) declined to settle and that
`shift` has been blocked on.

## Decisions

**The property set is `x`, `y`, `origin`, `scale`, `rotation`, `opacity`.** `rotation` is
in degrees clockwise and is **never normalised into `[0,360)`** — `1080` is three turns,
and a writer that wraps it silently renders one third of the motion. `opacity` is `0..1`.
**Skew is out**: CapCut has none and Premiere's Motion panel has none — it lives in Corner
Pin, which is the After Effects side of the line [ADR-0003](./0003-general-video-editor-not-channel-tooling.md)
drew. It arrives later as an entry in #22's closed vocabulary if it arrives at all.

**`scale` is always `[sx, sy]`, never a bare number.** The pair costs +26 % on the seven
Ken Burns lists in the fixture, all fourteen keyframes of which are isotropic, and that
cost is paid deliberately. A union was rejected because `shift` must now *interpolate*
this value, so two possible shapes put a shape test in every consumer that touches a
keyframe — and one of them will get the rarer branch wrong. **`fmt` normalising a scalar
on write is not the escape hatch it looks like**, and this is the mechanism: an agent
writes `"v":1.08`, `fmt` re-emits it as `"v":[1.08,1.08]` — which
[ADR-0011](./0011-tool-surface-reads-checks-renders.md) *requires* of every write tool —
and the agent's next exact-string replace on the string it just wrote gets **zero hits**.
Normalisation does not remove the union's cost; it moves it from the reader, who can see
it, to the write-read round trip, which cannot.

**Absolute integer pixels, in the project's frame space.** `x`, `y`, `width`, `height` and
`clip` are integers; `scale`, `rotation` and `opacity` are ratios and are floats. Fractions
were rejected on a census, not a preference: retargeting the fixture from 1080x1920 to
1920x1080 leaves **32 of 60 elements outside the frame**, and the 28 survivors are exactly
the 20 audio elements plus the 8 header-chrome elements. Chrome survives *because* pixels —
a 48 px corner margin is a 48 px corner margin in both frames. Under `0..1` fractions the
same `chip-panel` resolves 1.8x wider and 0.56x shorter, with a Union Jack inside it that
is no longer the shape of a flag, **silently, because the numbers stay in range**. Content
is a re-layout under any unit system. Fractions buy a solved problem — the same-aspect
resolution bump, which a render-time output size already handles — and sell a broken one.

**Flat fields on the element, and the schema varies by type.** There is exactly one
transform per element, so a nested `transform` object groups nothing and adds a path
segment to every edit, every `jq` and every `query` output. It would also give position two
homes, since [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) already shipped
`x`, `y`, `origin` flat on text. **`"x"` on an audio element is a schema error naming the
replacement**, never a silently-ignored field — ADR-0007 has already ruled twice that *"a
field the renderer cannot honour is worse than no field."* The specific trap is `opacity`
on audio, which an agent will write meaning volume and which would fade nothing, forever.

**A size is required, not defaulted.** An element's own `width`/`height` must be in the
document. The tempting default — natural source size — is forbidden by the same argument
that killed implicit `fill` in [ADR-0005](./0005-absolute-integer-milliseconds.md): the
source's dimensions are not in the document, so the element's rendered rect would be
unreadable, and it fails *quietly* because a centre-cropped photo looks plausible. A
default is only safe where its wrong answer is loud. Everything else defaults: `x` and `y`
to the frame centre, `origin` to `center`, `scale` to `[1,1]`, `rotation` to `0`, `opacity`
to `1`. Centre is chosen over top-left for the same reason: an element that lands at `(0,0)`
under the header chrome looks *intentional*, and a plausible default camouflages a missing
field.

### Keyframes

**Times are absolute** milliseconds on the one clock, and **records are objects** —
`{"t":17472,"v":1.0,"ease":"ease-in-out"}`. Positional pairs were rejected because an
unlabelled 2-array has no room for `ease` and becomes genuinely ambiguous once `v` is
itself a list; this repo has already measured what unlabelled arrays cost, when two agents
had to get `[x,y,w,h]` from a README and *"the guess was 50/50 with no way to check."*

**Clamp at both ends.** Before the first keyframe the value is the first value; after the
last it is the last. So "and then it holds" costs no syntax, and every one of the fixture's
seven trimmed moves stays well-defined.

**`ease` describes the segment *entering* its keyframe** — how the value travels from the
previous keyframe to this one. **`ease` on the first record of a list is a schema error**
naming the convention, not an ignored field.

This is the one decision here that contradicts the industry's centre of gravity, and it was
taken against a measured prior: 4 of 7 jurors, writing blind, used the *leaving* convention.
It was taken anyway on the **write-set invariant**. Under *entering*, `shift`'s write set is
exactly `{records with t >= at} ∪ {two inserts}` — a one-line postcondition a validator can
assert. Under *leaving* it is that set **plus the last record before `at`**, on every
animated property of every straddler, and that record's sole mutation is in the one field
with no numeric signature: on the fixture, the rewritten record is `{"t":17472, …}`, whose
`t` equals the element's own `start` — the single number an agent cross-checks hardest and
will therefore confirm as correct while the ease beneath it has silently become a raw
bezier. The SPLIT rule below is *complete* as worded under `entering` and literally
incomplete under `leaving`, which needs an extra clause reaching a record the rule excludes.
[ADR-0011](./0011-tool-surface-reads-checks-renders.md) records two agents implementing
`shift` from ADR prose and shipping two different videos; an exception clause is exactly
what gets dropped.

Two arguments were raised for this decision and then withdrawn, and are recorded so they
are not re-raised: the schema sentence is **not** self-teaching (both conventions
structurally force one record to omit `ease`, and 5 of 7 jurors wrote an `ease` on the
record their own convention gave no segment to, so the check catches "wrote it everywhere"
under either); and diff-visibility is **not** the argument, because ADR-0007 puts the whole
element on one line, so any keyframe change reflows it regardless.

The CSS prior is weaker than it appears. The construct that matches this shape is
per-keyframe `animation-timing-function` inside `@keyframes`, an obscure corner; the CSS
prior that is genuinely strong is `transition-timing-function`, which belongs to the whole
transition and **attaches to neither endpoint**, so it discriminates nothing. Lottie's
per-keyframe `o`/`i` is real evidence the other way. The reference class is split.

**Easing is a closed set of names, each published in the schema as its cubic bezier, plus
a raw `[x1,y1,x2,y2]` form.** `linear` `[0,0,1,1]`, `ease` `[0.25,0.1,0.25,1]`, `ease-in`
`[0.42,0,1,1]`, `ease-out` `[0,0,0.58,1]`, `ease-in-out` `[0.42,0,0.58,1]`, and `step`.
A name is sugar over one evaluator, never a second mechanism. `x1`/`x2` outside `[0,1]` is
a schema error; `y` outside is legal, because overshoot is a real need beziers give away
free.

**The raw form is forced, not chosen.** Splitting `ease-in-out` at `photo-06`'s own insert
point yields `cubic-bezier(0.362866, 0, 0.693025, 0.369169)` and
`cubic-bezier(0.365106, 0.225535, 0.568419, 1)`. Neither half is any named ease and neither
is in the family — the named set all have `y1 = 0` or `y2 = 1`. Four jurors computed this
independently and three agree to the digit. **Snapping to the nearest name costs 10.3–10.9 px
of framing**, larger than the 8.9 px error that disqualifies MOVE below, so snapping is not
a fallback but the same defect. Only `linear` and `step` are closed under subdivision. Names
survive in files nobody has shifted; a shifted file grows two bezier arrays at the one
segment that was cut.

`step` is named `step`, not `hold`, because "hold" reads as a claim about what happens
*next* — a `leaving` intuition sitting in an `entering` slot.

### What `shift` does — the complete rule

[ADR-0005](./0005-absolute-integer-milliseconds.md) defines `shift` as moving *"every time
at or after `at`"*. **That sentence is wrong for keyframes**, and this ADR amends it. The
axis is not the timestamp, it is the element:

| element vs `at` | `start` / `end` | keyframes |
| --- | --- | --- |
| `end <= at` — entirely before | untouched | **untouched, including any past `end`** |
| `start >= at` — entirely after | both `+= delta` | **every keyframe `+= delta`, including any with `t < at`** |
| `start < at < end`, time-invariant | `end += delta` | **SPLIT**, below |
| `start < at < end`, time-based | **refused**, per ADR-0005 | n/a |
| `at == start` | treat as entirely after | all `+= delta` |
| `at == end` | treat as entirely before | untouched |

Rows 1 and 2 each contradict ADR-0005's literal wording — row 1 refuses to move keyframes
after `at`, row 2 moves keyframes before it — and the proof is the trimmed move. All seven
of the fixture's photo elements carry a keyframe past their own `end`; `photo-05-loop`'s
sits **13.8 s past the end of the project**. `shift(at=31000)` does not touch `photo-06`,
which ended at 30603 — yet a timestamp-global rule drags its keyframe from 32472 to 34472
and **changes a shot that finished 397 ms before the edit point by 8.90 px of framing**.
`shift` changes the past. [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)
protects out-of-range keyframes from `validate`; nothing protected them from `shift`.

**SPLIT**, for a time-invariant straddler, per animated property:

1. `v :=` the property evaluated at `at`, honouring easing.
2. If `at` is outside the list's span, **insert nothing** — clamping already holds the value.
3. If a keyframe already sits exactly at `at`, leave a copy at `at` and move the original.
   **Never two records at one `t`.**
4. Otherwise insert `{"t": at, "v": v}` and `{"t": at+delta, "v": v}`.
5. Shift every keyframe with `t > at` by `delta`.
6. Subdivide the cut segment's easing: the record at `at` takes the **left** half-bezier,
   the record at `at+delta` takes `step`, and the following record keeps the **right** half.
7. **Collapse** any run of three or more consecutive records with identical `v` and identical
   `ease` to its two endpoints. Without this, four successive shifts through one segment grow
   a 2-record list to 10.

**Rounding**, which nothing published before. `x` and `y` round to the nearest integer, ties
away from zero — so SPLIT is lossy on position by at most 0.5 px, at the split instant only.
`scale`, `rotation` and `opacity` round to **6 decimal places**, which is the first precision
that is sub-0.01 px at 8K and short enough to stay greppable. `shift` reports the residual as
a fact in ADR-0006's form. This is the hole ADR-0005 left on `speed`, where `3368/0.645` sent
four agents to 5222 and one to 5220.

**Negative delta is destructive and must say so.** Removing time drops every keyframe in
`[at, at+|delta|)`. Nothing in ADR-0005 or ADR-0011 permits `shift` to destroy data; under
this rule it can, and each destroyed keyframe is named individually in the report.

Measured, over every millisecond `photo-06` is on screen, against the picture before the edit:

| reading | max scale error | px of width @1080 |
| --- | --- | --- |
| **SPLIT** | 3.3e-07 | **0.000** |
| move every keyframe `>= at` | 0.0078 | 8.452 |
| leave keyframes untouched | 0.0107 | **11.520** |

Leaving them untouched is the **worst** of the three, not the cautious one: the freeze is
only its tail, and the whole post-`at` shot plays 2000 ms early against a ramp that did not
move with it. On the whole fixture, SPLIT adds **two keyframes to the entire project**.

### The `box` retirement, and the aperture

**`box` is retired in both of its meanings**, which is the `anchor` collision recurring at
higher frequency — placement is on every visual element.

- The literal `[x,y,w,h]` rect becomes `x`, `y`, `origin`, `width`, `height`. Pure gain: the
  pivot that the 4-array left implicit becomes explicit, `x` and `y` become keyframeable, and
  the unlabelled array that two agents had to decode from a README is gone.
- **ADR-0007's `"box": "card-05"` — the id of an element you must fit inside — becomes literal
  `width`/`height`.** This is a correction to that ADR's *spelling*, not its decision: its
  argument is that the box must be in the document so the overflow check has an input, and
  literal dimensions satisfy that better than an id does. The fact that settles it is that
  **15 of the fixture's 22 text elements have no rect element behind them at all**, so under a
  required id they would name something that does not exist. The anti-drift binding is lost,
  and that loss is real — but it was half a binding: `sentence-05`'s `x`/`y` were already
  literal, so moving `card-05` already required a second edit. It looked like a binding and
  was not one. **Placement drift is not fixed here and needs its own ticket.**
- **`align` splits.** On text it keeps ADR-0007's meaning exactly — `start`/`center`/`end`,
  how lines align to each other, RTL-safe. On images it meant *which part of the source
  survives the crop*, which is not alignment, and becomes **`gravity`**. One word for one
  concept; the fixture's `align:"left"` on text was transcribing ASS `\an4`, *left-middle*,
  so `y` was a centre and nothing in the file said so.

**`clip` is a static frame-space rect that does not rotate or scale with the element.** It is
the aperture; `scale` moves the picture behind it, which is what a Ken Burns *is*. A
source-space `crop` was rejected because it scales with the element and spills.

The aperture is decided here rather than deferred to #22 because **`box` was silently doing
this job and the fixture renders wrong without it**. `photo-06` is a 1536x2720 source in a
1080x1300 box under `fit:"cover"`: cover factor `max(1080/1536, 1300/2720) = 0.703125`, drawn
1080x**1912.5**, so **612.5 px** of photo paints over the cream band the design rests on —
growing to **765.5 px vertical and 43.2 px per side** at the Ken Burns peak of 1.08. The
horizontal spill at 1.08 is what settles `clip` over `crop`. This is 7 of 7 photo elements
plus `handle-logo`; **zero text and zero rect elements need it**, and a rect is its own box.

Three things make this not deferrable:

1. **The workaround violates a settled principle.** The aperture *is* expressible with settled
   fields — an opaque background-coloured rect over the spill — and that is the argument
   against deferring, not for it. Computing its `620` height requires knowing the source is
   2720 px tall, **which is not in the document**: the objection that killed implicit `fill`
   in ADR-0005 and that ADR-0011 restates for this exact image. It also only works because the
   ground is a flat opaque colour, it needs four rects for any box not flush to the frame, it
   cannot clip a rotated element, and the interim rects are **byte-indistinguishable** from
   intentional design rects — so no migration tool could ever tell them apart.
2. **`scale` alone forces the question**, without `photo-06` at all. This ADR cannot ship
   `scale` and stay silent about what bounds the drawn rect. Adding the aperture later
   **changes what every existing file's `scale` means** — same bytes, different pixels.
3. **ADR-0011 already assigned it here, twice by name** — *"Not settled here: how `fit`,
   `align` and `scale` interact (#21)"* — and records `query`'s expensive half as blocked on
   it. Closing #21 without the aperture closes it with its own deliverable unmet.

The seam, which `FINDINGS.md` §G had already drawn: **rectangular frame-space `clip` here;
shape and soft masks — the fixture's `mask:"circle"` on `handle-logo` — to #22.**

## Consequences

- **`shift` is unblocked** and ADR-0011's adoption-in-shape becomes adoption in full.
  ADR-0011's *"no check anyone has proposed catches the difference"* becomes a check: after a
  correct `shift`, rendered values must be identical outside `[at, at+delta)`.
- **`query`'s expensive half is unblocked.** With `width`/`height` on the element the crop
  rectangle is computable by reading. The renderer must publish a sampling rule, because the
  rectangle is not integral: exact cover here is 1912.5 px.
- **ADR-0005 is amended.** Its `shift` sentence is wrong for keyframes and the table above
  replaces it.
- **ADR-0007 is amended**: `box` becomes literal `width`/`height`, `align` becomes text-only.
  Its literal-`size` and runs decisions stand untouched.
- **ADR-0006 gains three checks**: text overflow now has an aperture term as well as a box and
  a width term; elements sharing a `group` whose transform keyframe *times* disagree (the
  lower-third failure — two elements that must move as one, with nothing in the file saying
  so); and a **frame-change census**, listing every element whose placement now falls outside
  the frame and every text element whose measured layout is thereby unverified.
- **`validate` prints the resolved segment table** for any keyframed property — `0–1000
  ease-in / 1000–2000 ease-in-out` — so a one-word-wide convention is readable back. This is
  worth shipping whichever direction `ease` had gone.
- **An animated multi-element entrance is the format's most expensive routine operation.** A
  lower third is always two or more elements, `group` is render-inert, and ADR-0001 and
  ADR-0003 forbid the fix. Two jurors wrote the coupled-delta bug live: elements that must
  slide together must move by the same *delta*, not from the same start value, and nothing in
  the file says the two ramps are one motion. The `group` check above is the mitigation, not a
  cure.
- **"Slide in from the left" needs `measure`.** Off-screen means `x <= -width`, and a text
  element's width is the shaper's opinion. ADR-0007 accepted that bill for sizing; it extends
  to animation.
- **An odd-sized element cannot be written at its own centre** under integer pixels —
  `card-05` is 169 tall, so its centre is `y=1537.5` and the fixture's text sits at 1537, half
  a pixel off. The nine-keyword `origin` is the escape (write odd chrome as `top-left`), but
  the default origin cannot be relied on for pixel-exact layout.

## Not settled here

- **Whether `gravity` survives at all.** With explicit `width`/`height` and a `clip`, the
  aperture's position may already determine which part of the source survives, making
  `gravity` derivable rather than declared. The jury split on this and it was not measured.
  **Now measured, and still open:**
  [ADR-0013](./0013-fitted-extents-floor-and-the-nine-origin-keywords.md) finds `gravity`
  **inert on 8 of 8 image elements** in the only real project file — the declared rect and
  `clip` together already determine which part of the source survives. That is the number
  the split jury did not have. It is a fact handed to
  [#13](https://github.com/MBehtemam/Montaget/issues/13) and
  [#21](https://github.com/MBehtemam/Montaget/issues/21), not a decision.
- **Whether `clip` is keyframable.** Nothing in the fixture animates it — all seven photos use
  one static rect — but a wipe or reveal is exactly a keyframed aperture. If it is, it joins
  the properties SPLIT must handle and the 0.000 px result must be re-run over it.
- **Placement drift**, above: retiring `box:"<id>"` removes a binding that was already only
  half a binding, and nothing now keeps a caption with the card it sits on.
- **Whether the `ease`-on-first-record schema error actually corrects an agent's next attempt.**
  The decision assumes it teaches; that was reasoned, not measured, and two jurors said the
  measurement is what would move them back.
