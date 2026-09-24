# Juror B scratch work

## Exercise 1 — lower third + Ken Burns photo

Chosen model while authoring (this is the point of the exercise, not a pre-decision):
- one positioning model for all visual elements: x, y, origin (nine-way), width, height
- `box` as "id of another element to fit inside" renamed `fit_box` to stop colliding
  with the literal-rect `box` ADR-0007 also uses
- transform properties (x, y, scale, rotation, opacity) are flat fields, keyframed with
  the existing positional-pair shape, absolute pixel units, absolute keyframe times (first draft)

```json
{
  "id": "photo-tips-bg",
  "type": "image",
  "start": 0, "end": 10000,
  "source": "images/pumpkin.png",
  "x": 540, "y": 960, "origin": "center",
  "width": 1080, "height": 1920,
  "fit": "cover",
  "scale": [[0, 1.0], [10000, 1.12]]
}
```

```json
{
  "id": "lt-bg",
  "type": "shape", "shape": "rect",
  "start": 2000, "end": 8000,
  "x": [[2000, -370], [2400, 60]],
  "y": 1600, "origin": "center-left",
  "width": 700, "height": 140,
  "fill": "#00000099",
  "opacity": [[2000, 0], [2400, 1]]
}
```

```json
{
  "id": "lt-text",
  "type": "text",
  "start": 2000, "end": 8000,
  "x": [[2000, -370], [2400, 60]],
  "y": 1600, "origin": "center-left",
  "align": "start",
  "fit_box": "lt-bg",
  "font": "brand", "size": 48, "color": "#FFFFFF",
  "opacity": [[2000, 0], [2400, 1]],
  "runs": [{"text": "Carving Tips"}]
}
```

FINDING: to make the bg card and its text slide+fade together I had to duplicate the
identical x-keyframes and opacity-keyframes onto two separate elements. There is no
grouped-transform / parenting mechanism (correctly rejected, ADR-0001/0003 — no
nesting). Cost is real and specific: editing the slide duration on one sibling and not
the other desyncs them silently, and nothing checks it. `group` (CONTEXT.md) only
labels, it does not synchronize.

FINDING: writing the rect's motion forced a choice between keyframing four numbers
(a `box` array) or giving shapes the same x/y/origin+w/h fields text and (my redesign
of) images use. Positional-pair-of-arrays (`box: [[t,[x,y,w,h]], ...]`) is unreadable
by eye — confirms one shared model, not per-type box shapes.

FINDING: `align` collides across types exactly like `box` does. Image `align` (with
`fit`) means "which part of the source survives the crop" (`top`/`center`/`bottom`
against a cover box). Text `align` (ADR-0007) means "how do lines align to each other"
(`start`/`center`/`end`). Same field name, disjoint vocabularies, disjoint meaning.
Not asked directly by the brief but found while authoring photo-tips-bg next to
lt-text — flagging separately, not folding into the box answer.

## Exercise 2 — rotating badge

```json
{
  "id": "badge-new",
  "type": "image",
  "start": 0, "end": 5000,
  "source": "badges/new.png",
  "x": 980, "y": 180, "origin": "center",
  "width": 160, "height": 160,
  "rotation": [[0, 0], [3000, 1080]]
}
```

Used 1080 (3×360) rather than 360 so the badge visibly completes three full turns
in 3s rather than ending indistinguishable from having never moved — rotation must
NOT be modulo-wrapped by the renderer or "continuously… then hold" is inexpressible
as a single ramp. From 3000 to 5000 there is no keyframe, so the value must hold at
whatever the last keyframe said (1080°, visually identical to 0°, badge motionless).

FINDING: nothing in CONTEXT.md or the ADRs states the before-first/after-last
keyframe rule. I assumed "hold at nearest keyframe's value," which is the universal
convention in every animation tool I know of, but it is an assumption, not something
the repo commits to. ADR-0006 talks about keyframes legally existing *outside an
element's own start/end* (the trimmed-move case) but never states what value the
property takes there or past the array's ends. Two different questions, both unsettled.

## Exercise 3 — shift +2000ms at t=20000, photo-06

Source data (from the actual sample project, `git show
prototype/sample-project-file:.../en-halloween-decorating.montagent.json`):
`photo-06`, start 17472, end 30603, `"scale": [[17472,1.0],[32472,1.08]]`.

t=20000 is strictly inside photo-06's range (17472–30603) → photo-06 is a
time-invariant straddler → ADR-0005: `shift` stretches its `end` by delta.
New end = 30603 + 2000 = 32603. Not in dispute.

The dispute is the keyframes. Two mechanical readings:

**A — keyframes untouched (held):**
`"scale": [[17472,1.0],[32472,1.08]]` (unchanged).
At new end (32603): fraction = (32603-17472)/(32472-17472) = 15131/15000 = 1.0087,
clamped to 1 → scale = 1.08. The ramp finishes at 32472 and holds for the trailing
131ms.

**B — keyframes shifted mechanically along with every other time ≥ at=20000:**
Second keyframe's time, 32472, is ≥ 20000, so a literal reading of "shift moves
every time at or after `at`" moves it too → 32472+2000 = 34472.
`"scale": [[17472,1.0],[34472,1.08]]`.
At the new end (32603): fraction = (32603-17472)/(34472-17472) = 15131/17000 = 0.8901
→ scale = 1.0 + 0.8901×0.08 = 1.0712, still ascending.

Both numbers reproduce the two divergent figures already reported in issue #21's
comment thread (1.0800-frozen vs 1.0712-still-zooming) even though that thread used
a different `at`(30000) — confirms this is the general mechanism, not an artifact of
one instant.

**The right answer is A (held/frozen tail), and here is why, stated as a rule rather
than a preference:** `shift`'s job (ADR-0005) is to move or extend *timeline*
structure — `start`/`end` — arithmetically, without the agent doing anything. A
keyframe timestamp is not timeline structure in that sense; it names an absolute
instant at which a specific visual state was authored, tied to the schedule that was
actually drawn, not to the element's duration. Stretching `end` on a still image is,
by ADR-0005's own language, "the hand-fix... to close a hole," i.e. holding the last
frame in place for the inserted span — the natural read of "stretch" for a static
image is "keep showing what was already going to be shown, for longer," not "replay
the same motion faster to fill the new duration." Reading B silently changes the
ramp's *rate* (15000ms → 17000ms, a ~13% rate change, ~10px of framing per ADR-0011's
own arithmetic on this exact element) with no field anywhere recording that a rate
change happened.

**Where a plausible agent gets it wrong:** implementing `shift(path, at, delta)`
literally off ADR-0005's own sentence — "moves every time at or after some instant by
an offset" — without asking whether "every time" was meant to include keyframe
timestamps belonging to a stretched (not translated) element. That is reading B. It is
not a careless agent; it is the textually correct implementation of the spec as
written, which is exactly ADR-0011's finding: "both files are legal, both validate
clean, and they are different videos." The bug is in the spec's silence, not the
agent.

**A rule that resolves it without inventing new machinery:** a keyframe's absolute
time shifts **by the same delta as its element's `start`**. When the whole element
moves (both `start` and `end` move by delta because it is wholly after `at`), its
keyframes move by delta too — the motion translates intact. When only `end` moves
(the stretched-straddler case, `start` unchanged), keyframes do **not** move — they
were never a function of the extended tail. This produces A automatically and is a
two-line addition to `shift`'s spec.

## Exercise 4 — retarget 1080x1920 → 1920x1080

Concretely, under the pixel-absolute model already in the fixture, retargeting touches:
- every `x`/`y` on every element (portrait composition puts the sentence card at
  y=1453-1622, centered horizontally at x=540 — none of that maps onto a
  1920×1080 canvas without a full re-layout: 540 is off-center-left in a
  1920-wide frame, and 1453 is below the visible frame entirely in a 1080-tall one)
- every `box`/width/height rect (`[0,0,1080,1300]` photo crop area, `[48,1453,984,169]`
  sentence card, all four header chip/flag rects)
- every keyframe value if it targets `x`/`y` (my lower-third's `x: -370 → 60` was tuned
  to a 1080-wide frame; meaningless numbers on a 1920-wide one)
- `size` on nothing directly (font size is legible pixel count independent of frame
  aspect, only position needs redoing) but `fit`/crop choices on images do change,
  because "cover, top-aligned" on a 1080x1300 portrait box crops a very different
  region of the same source than "cover" on a 1920x1080 landscape box

**This is inherently a re-layout, not a coordinate transform, and resolution-independent
units would not have saved it.** The portrait design puts a full-bleed photo in the top
2/3 and a caption band in the bottom 1/6 — a composition decision tied to the *aspect
ratio*, not to the pixel count. Expressing the photo box as `[0, 0, 1.0, 0.677]`
(fractions of frame) survives a same-aspect-ratio resolution bump (1080x1920 →
2160x3840) for free — that case really is solved by fractions. It does not survive a
90° aspect change, because "top 2/3 of a 9:16 frame" and "top 2/3 of a 16:9 frame" are
different shapes holding a different photo, and no unit system encodes "regenerate the
composition." The most the format can do is make each individual number's edit local
and exact-string-replaceable (which ADR-0005/0007's one-line-per-element convention
already buys) so the retarget is 40-some small independent edits instead of a
cross-file computation — it cannot make the retarget itself smaller.

## Exercise 5 — read by eye at t=6000

Using exercise 1's `lt-text`/`lt-bg` (start 2000, end 8000):
- 6000 is inside [2000, 8000) → the title card is on screen.
- x-keyframes `[[2000,-370],[2400,60]]`: 6000 is past the last keyframe (2400) →
  by my assumed hold-last rule, x = 60 (fully in its resting position, origin
  center-left, y=1600).
- opacity-keyframes `[[2000,0],[2400,1]]`: same reasoning → opacity = 1 (fully opaque).

**Answer: the card is at rest at x=60,y=1600 (center-left origin), fully opaque (1.0).**

**How hard that was:** the arithmetic was trivial — two elements, two two-point
keyframe lists, one clearly past the second point. The actual friction was
epistemic, not computational: nothing in the repo states what "past the last
keyframe" *means*. I had to import a convention (hold-last) from every other
animation tool I've used; the file itself is silent, and a differently-primed
reader could just as defensibly assume "undefined" or "linear extrapolation
continues past 1.0/60." That ambiguity is invisible until you try to answer exactly
this kind of question, which is exactly the ADR-0001 test ("what is on screen at
6.2s, by reading, no arithmetic, no evaluation") — the arithmetic-free part held;
the *evaluation-rule-free* part did not, because the rule needed doesn't exist yet.
