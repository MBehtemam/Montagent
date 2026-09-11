# The text and shape primitives, answered by the agent who has to write the file

I am the consumer here. Everything below is argued from an edit I would actually type
into `en-halloween-decorating.montaget.json` and what would go wrong when I did.

Two things I checked myself before answering, because they turned out to carry weight:

- **The old pipeline had a full path language and never used it.** Every one of the
  fixture's ten rectangles was drawn in ASS as a `\p1` drawing — `m 48 88 l 420 88 420 172
  48 172`. ASS drawings support cubic beziers (`b`), arbitrary point counts and
  non-axis-aligned polygons. Across all seven `.ass` files there are **six distinct
  drawings and every one is four points, `m`/`l` only, axis-aligned**. Not "the fixture
  doesn't use polygons" — *an author with the capability in hand, drawing a Union Jack,
  reached for three rectangles and left the saltire off.*
- **Fact 4 reproduces exactly**: 15 of 22 text elements have `height ==
  ceil(size * line_height * lines)`; the other 7 are 169 (the card) or 84 (a panel).
  `width` is 984 on 20 of 22 — which is `1080 - 48 - 48`, the frame's content column, not
  the card — and 238/472 on the two header texts.

---

## Q1 — Is a shape one element type or many?

**Answer: sibling element types. `type:"rect"`, `type:"ellipse"`. No `type:"shape"` with a
`shape` discriminator.** Confidence: **high**.

The reason is not taxonomy, it is that ADR-0012 already chose the mechanism and a
discriminator would be a second one doing the same job worse.

ADR-0012 says, in one sentence, *"Flat fields on the element, and the schema varies by
type"*, and enforces it with *"`x` on an audio element is a schema error naming the
replacement."* That is a working, already-shipped rule: **the field set is a function of
`type`.** A `shape` discriminator makes the field set a function of *a field's value*
instead, for exactly one branch of the type tree. Now there are two ways a schema narrows,
and I have to know which one I am under before I can know what fields are legal.

The concrete edit that shows the cost. Today I change what the navy card is by replacing
`"type":"rect"` with `"type":"ellipse"` — one replace, and the schema for the new element
is reachable from the string I just typed. Under the discriminator I replace
`"shape":"rect"` with `"shape":"ellipse"` while `"type":"shape"` sits unchanged three
fields earlier, so the element's `type` no longer tells me what it is. Every `query`
output, every `validate` message and every grep I write (`grep '"type":"rect"'`) gains a
level for no return.

The failure I actually fear, though, is the **default**. `type:"shape"` with `shape`
omitted is a question the format can be asked; `type` omitted is not, because nothing
would dispatch at all. This project has been burned once by a plausible default
(ADR-0012: *"a default is only safe where its wrong answer is loud"*, and the frame-centre
`x`/`y` chosen specifically so a bare element is conspicuous). A discriminator that
defaults to `rect` — which is what a schema author will write, because rect is 10 of 10 —
means a malformed ellipse renders as a rectangle and nothing says so.

The one real argument the other way is that `fill`, `stroke` and `stroke_width` would be
declared once under `shape` instead of repeated per type. That is a **schema authoring**
convenience, not an authoring-agent one: I read the published schema, and a schema that
composes a shared `paint` fragment into two sibling types reads identically to me. It buys
the schema author nothing I can see and costs me a path segment in every tool output.

Note also that `CONTEXT.md`'s Element entry — *"an image, a video, an audio file, a piece
of text or a shape"* — is prose enumerating categories, not a schema statement. It says "a
piece of text" in the same breath, and text is `type:"text"`.

## Q2 — Which shapes exist in v1?

**In: `rect`, `ellipse`. Out with a reason: `line`, `path`. Unevidenced but not rejected:
`polygon`, and `radius` on `rect`.** Confidence: **high** on rect/path, **medium** on
ellipse and line.

**`rect` — in.** 10 of 10 shape elements. Nothing to argue.

**`ellipse` — in, and the argument is `handle-logo`, not the reference class.** The
circular badge in the header is not in the document at all: `brand/logo-en.png` is 800x800
RGBA with corner alpha 0, so the circle is *baked into the asset*, and the `mask:"circle"`
written on the element is inert. That is the format's own named disease — geometry that
determines what is on screen, living somewhere the file cannot state and `validate` cannot
check (ADR-0012 killed implicit `fill` for precisely this, ADR-0013 killed
isotropic-sampling for precisely this). Without an ellipse, an author who wants a coloured
dot, a circular badge plate, a ring behind a countdown digit or a progress bubble has
exactly one recourse: open an image editor and bake it into a PNG. Shipping rect-only
*forces* the out-of-document answer for the second most common shape in the reference
class. The rasterizer is skia-safe (ADR-0010); an ellipse is free there.

I want to be explicit that I am *not* arguing "ellipse would mask the logo" — a shape
element is not a mask, and ADR-0012 drew that seam to #22. I am arguing that the one real
project already needed a circle, and solved it outside the format.

**`line` — out of v1, and out on a reason, not on evidence.** A line does not fit the
placement grammar every other visual element shares. ADR-0012 made `x`, `y`, `origin`,
`width`, `height` universal and ADR-0013 spelled the nine origin keywords over it. A line
is either `(x1,y1,x2,y2)` — a second, incompatible placement grammar for one element type,
and the unlabelled 4-array this repo has already measured the cost of — or it is
`x`/`y`/`width`/`height` where `height` is a lie and `origin:"bottom-right"` means
something no one can predict. Meanwhile a straight segment of any angle is already
expressible: a rect of the segment's length and the stroke's thickness, with `rotation`,
which every element has. That costs me one `atan2` and one hypotenuse at authoring time —
arithmetic whose inputs are all numbers I already hold, which is a different and much
cheaper class than the `620`-from-a-2720px-source arithmetic ADR-0012 rejected. **`line`
is the first thing I would add in v2** if the rotated-rect workaround shows up in real
files, and it should be recorded that way.

**`path` — out, and rejected rather than deferred.** A path is `"m 48 88 l 420 88 420 172
48 172"` — a mini-language inside a JSON string. Reading the file stops being enough to
know what is on screen, which is the exact test ADR-0003 used to defer After Effects'
expression language and the exact test ADR-0007 used to kill fit-to-box ("a property whose
value is HarfBuzz's opinion"). It is also unreachable by this format's editing model:
exact-string replace on a point inside a 40-token path string matches nothing meaningful,
and `validate` can say nothing about it beyond "parses". And the corpus argues against it
from the strongest possible direction — the author *had* this exact language, in ASS, and
drew six axis-aligned quadrilaterals with it.

**`polygon` — unevidenced, not rejected.** It has the same placement problem as `line` (a
points array is a second grammar) but unlike `path` it is readable by reading, so the
objection is cost rather than principle. No file has needed it. Record it as open.

**`radius` on `rect` — unevidenced, and cheap to add later.** FINDINGS §H established the
panels are sharp-cornered against the published frame at 8x, correcting the README. Unlike
every other item here, `radius` is purely additive: it adds a field to an existing type,
changes no existing file, and needs no new grammar. Leaving it out of v1 is reversible at
zero cost, which is exactly the test ADR-0003 sets (*"no primitive may be shaped such that
a general need becomes hard to add later"*). It is not hard to add later, so it waits.

## Q3 — How is a colour spelled?

**Answer: `#RRGGBB` and `#RRGGBBAA`, uppercase, and nothing else. No 3-digit shorthand, no
CSS names. `#RRGGBBFF` is a schema error naming the 6-digit form, and lowercase hex is a
schema error naming the uppercase one.** Confidence: **high** on the exclusions, **medium**
on admitting `AA`.

**The exclusions are already decided, twice, by a rule that has nothing to do with
colour.** ADR-0011 requires `fmt` to normalise on write. ADR-0012 used that to kill scalar
`scale` — *"an agent writes `"v":1.08`, `fmt` re-emits it as `"v":[1.08,1.08]`, and the
agent's next exact-string replace on the string it just wrote gets zero hits."* ADR-0013
used it again to make `center-center` an error rather than an alias. `#fff` and `cornsilk`
are the same trap with a different costume: I write `#FFF`, `fmt` writes `#FFFFFF`, and my
next edit — the one where I change the four cream fills to a warmer cream — finds three of
four. **Every extra spelling of one colour is a silent under-match on replace-all, which is
the single most common edit I perform on this file** (the FINDINGS §F trap is exactly this
failure in the `y` field, and it is recorded as a live defect).

That argument reaches **case** too, and nobody has written it down. The fixture is
uppercase throughout (`#FBF3E3`, `#1E344C`, `#245C8C`). `#1e344c` and `#1E344C` are two
spellings of one value; a `fmt` that silently upper-cases them reopens the trap. So:
uppercase is the spelling, and lowercase is an error naming it — not a normalisation.

**`#RRGGBBAA` is not a second spelling of the same value, so it does not trip that rule** —
provided `#RRGGBBFF` is an error. Without that clause it does trip it, and hard: `#1E344C`
and `#1E344CFF` paint identical pixels, `fmt` would have to pick one, and we are back at
`center-center`. This is the same move ADR-0013 made for extents (*"`fmt` must never
rewrite a declared extent"*) and it must be stated the same way: **`fmt` never expands 6 to
8 and never contracts 8 to 6.**

**Why admit `AA` at all, given ADR-0012 gives every element `opacity`?** For most of what I
do, `opacity` is enough and is better — a 40% scrim over the photo is `fill:"#000000"`,
`opacity:0.4`, and a fade-in is a keyframe on that same channel. The case `opacity` cannot
express is **two paints on one element with different alphas**, and that case arrives the
moment `stroke` lands (Q5): a translucent fill under an opaque outline is one element, and
element opacity multiplies both. It is also the case ADR-0013 already named in a different
guise — *"base geometry and animation share one channel and neither stays readable"* — when
static geometry was pushed into the `scale` channel that seven of eight images already
animate. `opacity` is the animation channel on shapes too, and spelling static translucency
there collides with it identically.

So Q3's answer is **coupled to Q5's**: `#RRGGBB` only is coherent if stroke goes to #22,
and incoherent if stroke is a paint field. I am recommending stroke as a paint field, so I
take `AA` with it.

One trap to write into the schema prose because it already bit this corpus: **ASS alpha is
inverted** — the fixture's `&H00E3F3FB` is *opaque* cream, alpha nibble `00`, channels BGR.
Anything transcribing from ASS will read alpha backwards. `AA` must be CSS-direction
(`FF` opaque), and the ADR should say so in the same sentence that admits it.

## Q4 — Does a text element still declare a `height`?

**Answer: (b). `width` required; `height` optional, absent meaning "no vertical container
is claimed". `validate` prints a census of which text elements claim one.** Confidence:
**medium-high**.

**First, the question's premise about the collision is wrong, and that matters more than
the answer.** Q4 says (b) collides with ADR-0012 and `CONTEXT.md` — *"an element's size is
declared, never defaulted."* Read ADR-0012's actual argument: *"The tempting default —
natural source size — is forbidden … the source's dimensions are not in the document, so
the element's rendered rect would be unreadable, and it fails quietly because a
centre-cropped photo looks plausible."* The prohibition is on defaults **whose inputs are
not in the document**. ADR-0008 then says what text's `width`/`height` actually are:
*"`box` remains a check input, not behaviour."* A text element's `height` does not size
anything — the element draws at `size` x `line_height` x lines whatever the field says, and
the renderer *"never clips, shrinks or wraps it away."* Calling it a "size" and importing
the no-default rule is a category error: it is a claim about a container, and the 15
derived values are all computable from `size`, `line_height` and the `\n`s **in the same
element, on the same line of the file**.

**Second, the real finding in fact 4 is structural, not incidental.** Of the 22 text
elements, `width` is externally sourced in **22 of 22** — 984 is the frame's content column
(`1080-48-48`), 238 and 472 are a header panel's inner extent. `height` is externally
sourced in **7 of 22**, and the other 15 are copies of the element's own arithmetic. That
asymmetry is not this fixture's accident. **Width always has an external source — at
minimum the frame minus its margins, a number that exists before any text is written.
Height usually does not: most captions simply grow downward into whatever is behind them,
and there is nothing to copy.** Faced with a required field and no external number, the
only thing an author can write is the derived value.

And that is the killer: **a check input the author computes from the thing being checked
checks nothing.** ADR-0007 admits `box` on the strength of four agents shipping overflow
three ways and one of them naming it — *"the one property that determines whether the
output is usable is the one property the file cannot state."* On 15 of 22 elements the file
now states it by copying the answer. `validate` confirms my arithmetic and tells me nothing
about the screen, and — worse — it reports a pass, which is the false-confidence failure
ADR-0006 was written against (*"`0 errors, 47 notes` reads as a pass"*).

**The edit that makes (a) actively harmful.** `word-05` is `"cobweb  -  cobweb"` at size
88, `width` 984, `height` 97. I localise it and the string gets longer; the width check
fires (correctly — this is the check earning its keep); I fix it with a `\n`, per ADR-0008,
because that is the only break mechanism. Now `lines` is 2 and the derived height is 194
against a declared 97. Under (a) I must now hand-compute `ceil(88 * 1.1 * 2)` and type
`194` — arithmetic that changes **nothing on screen**, that I will get wrong in the `ceil`
(`96.8`→`97`; `547.8`→`548`) exactly as five agents got `3368/0.645` wrong in ADR-0005's
record, and whose only consequence is silencing a finding. Every `\n` edit and every size
edit on 15 of 22 elements becomes a two-field edit with an invisible arithmetic step.
That is the format paying a tax to check itself.

And the tax buys negative value, because the field is self-fulfilling in both directions:
write the height too small and I get a false positive on text that is fine; write it too
large — which I will, because there is no penalty — and the check is silently dead.

**Why (c) is backwards.** (c) fires a finding when `height` equals the derived value: 15
findings on the only correct file that exists, and **zero** on the 7 elements that carry a
real container claim. ADR-0006's noise budget is explicitly a safety property — *"errors
and near-errors print in full; informational classes collapse"* — and its severity rule is
*computed from the consequence*, with the fixture's ideal validator reporting **one** visual
gap out of eleven. A check that fires 15 times on a clean file and never on a defect is the
opposite of that.

**What (b) costs, and the mitigation.** The honest risk is that I forget `height` on a
caption that genuinely sits inside a card, and lose a check I wanted. Absence is otherwise
safe here — there is no `fmt` round-trip trap in an absent field, because there is no
string to fail to match. The mitigation is the one this project already uses for exactly
this shape of risk: **a census, not a default.** ADR-0007 gained a font census
(*"23 elements use `brand`; 1 uses `brand-old`"*) precisely so that a silent majority
becomes visible. `validate` prints *"7 of 22 text elements claim a vertical container; 15
do not"*, and if my five card sentences read as 4-of-5 I see it in one line.

**What I would additionally want, and it is not one of the three options.** The seven
externally-sourced numbers are externally sourced from elements that *are in the file* —
`card-05` is 984x169 and `sentence-05` says 984x169. ADR-0012 retired `box:"card-05"` and
recorded the loss honestly (*"Placement drift is not fixed here and needs its own
ticket"*). Under (b) that ticket gets sharper, because the 7 that would name a container
are now exactly the 7 that write a `height`. It is the same set. Whoever picks up placement
drift should be told that.

## Q5 — Is `stroke` a field on the primitive, or an effect?

**Answer: a field on the primitive — `stroke` (colour) and `stroke_width` (integer px) —
and on text it is additionally a *run* style delta, not an element field only.**
Confidence: **high**.

The line, in one sentence, as asked:

> **A paint field is consumed while the element is rasterized, in the element's own
> geometry, and therefore scales and rotates with it; an effect takes the element's
> already-rasterized pixels as input.**

Under that line: `fill` and `stroke` are paint; blur, drop shadow and colour filter are
effects; and **#22's candidate "text background box" should be struck from its list
outright**, because the fixture already answers it — the navy card is `card-05`, a `rect`
element with a `fill`, sitting in its own track at layer 20 under the text at 21. It is not
an effect and never needed to be.

Three reasons, in descending force.

**1. Stroke on text must vary per run, and an effect cannot.** ADR-0007's whole case for
`runs` is the ordinary reference-class edit — *"emphasise one word mid-line"* — and it
already filed the run style-delta set as *"colour, outline"*, together, in one clause. #22's
model attaches an effect to an **element**. A run is *inside* an element. So if text stroke
lives in #22, "outline just this word" becomes inexpressible, and the only workaround is
the one ADR-0007 was written to abolish: split the element in two and hand-compute both
baselines — *"which is ADR-0004's worst finding and ADR-0006's invisible defect class."*
This is not a preference; it is a capability that disappears.

**2. A stroke participates in the transform, and effects in #22's sense do not have to.**
The fixture's own ASS says `ScaledBorderAndShadow: yes` in all seven files — the outline
width is in the glyph's coordinate system and scales with it. Seven of eight image elements
carry a `scale` ramp to 1.08, and a stroked shape under a Ken Burns has to answer the same
question ADR-0012 had to answer for `clip` (*"does not rotate and does not scale with the
element"*). Paint scales; the aperture does not. If stroke is deferred to #22, then #22 has
to re-derive the transform interaction that ADR-0012 settled — and the two ADRs can
disagree, silently, in a file where every number still looks right.

**3. `stroke` without `fill` is the only way to spell an outlined shape, and the outline is
the shape.** An effect vocabulary that can be *absent* cannot be the sole carrier of a
primitive's appearance. `{"type":"rect", "stroke":"#245C8C", "stroke_width":4}` with no
`fill` is a frame — a real, ordinary CapCut object. Under the effect reading that element's
`fill` is required-and-meaningless and its visible form lives in a separate optional
structure.

Two clauses the schema must carry, both of which are the "half a pixel of unspecified
freedom" ADR-0013 refused to leave:

- **`stroke_width` is required whenever `stroke` is present**, never defaulted. Same rule
  as ADR-0012's *"a size is required, not defaulted"*, same reason: a defaulted `1` is a
  plausible wrong answer.
- **Where the stroke sits relative to the declared rect must be pinned in the ADR**, not
  left to the rasterizer. See below — this is my answer to "what bites first".

## Q6 — Does `gravity` belong here or with `fit`?

**Answer: with `fit` (#48). This ticket owns exactly one thing about `gravity` — that text
and shape elements may never carry it.** Confidence: **high**.

`gravity` means *which part of the source survives the crop*. Text and shapes **have no
source** — `CONTEXT.md` says so directly under Source range: *"images, text and shapes have
no insides."* There is no question about `gravity` that the text or shape primitive is in a
position to answer; it is in this ticket only because tickets get bundled.

More than that: **the `gravity` question is not answerable without `fit`, by construction.**
ADR-0013 found gravity inert on 8 of 8 and explained why — *"the declared rect and `clip`
together already determine which part of the source survives."* Gravity has a job only when
the fit rule leaves **slack**, i.e. when the drawn rect differs from the declared rect in a
way that is not fully pinned. Under `cover` with ADR-0013's declared-rect-authoritative
reading, there is no slack, which is exactly why it is inert. Whether any slack exists is a
property of the `fit` vocabulary, and `cover` is currently the only value that exists.
Deciding `gravity` here would be deciding it against a one-value vocabulary.

ADR-0013 already made this exact refusal about a different field, and gave the reason:
legislating rounding for `contain` *"would create a schema value by implication"*, so it
declined and handed #48 a re-executable artifact instead. Same move applies. ADR-0012 and
ADR-0013 both handed the measurement to #13 **and** #21/#48 and explicitly declined to
decide — that split routing is the thing to resolve, and it resolves toward #48.

**The one thing this ticket should say.** `origin`, `align` and `gravity` are three
positioning-adjacent words with overlapping value sets, and `CONTEXT.md` records that the
`anchor`/`origin` collision was *"near-certain to be rediscovered"*. I will write
`gravity:"top"` on a `rect` meaning `origin`, from image muscle memory, within a week.
So: **`gravity` on a text or shape element is a schema error naming `origin`** — the same
shape-of-error pattern as `anchor`-carrying-a-string. That exclusion belongs here, because
this ticket defines the types it is excluded from. The vocabulary belongs to #48.

---

## What would bite me first?

**Stroke geometry — whether a stroke grows the element beyond its declared rect.**

Here is the edit, and it is an edit I would make on a Tuesday without thinking. The navy
sentence card is `{"id":"card-05","type":"rect","x":48,"y":1453,"origin":"top-left",
"width":984,"height":169,"fill":"#1E344C"}`. I add `"stroke":"#245C8C","stroke_width":8` to
give it a rim.

If the stroke is centred on the path — which is the default in skia, in SVG, in Canvas, in
every library whose habits I have — the card on screen is now **992 x 177 at (44, 1449)**.
It has eaten 4 px of the 48 px margin that the entire design's chrome is built on. And:

- Every number in the file still says 48, 984, 169. `validate` passes.
- The aperture-coverage error is about `clip` against the declared rect, and doesn't look
  at paint. `fmt` doesn't touch it. `query` reports 984.
- `sentence-05` declares `width: 984` as its container claim, measured against a rect that
  is no longer 984 — so the one check that *is* load-bearing is now checking against a
  stale number, and nothing connects the two.
- Under a keyframed `scale`, the 8 px rim is 8.64 px at 1.08 and the drift animates.

This is ADR-0013's own disqualifying failure re-entering through a door it did not guard:
*"what is actually drawn is a number not in the file, derivable only from source dimensions
also not in the file."* Geometry was shut out of the schema and shut out of the renderer's
sampling rule, and paint would let it back in — silently, plausibly, on the one design axis
(the 48 px margin) that this project's chrome survives a frame change on at all.

I would ship that video. It would look fine at thumbnail size. I would notice at 8x
magnification, next to the reference frame, the way FINDINGS §H noticed the corners.

The fix is one sentence, and it has to be in the ADR rather than in the rasterizer:
**on a shape, the stroke is drawn inside the declared rect, so the declared rect stays
authoritative; on text, the outline grows outward from the glyph, because that is what
every shaper and every reference tool does and a text element's declared `width` is a check
input, not a boundary.** The asymmetry is real and must be written down *as* an asymmetry,
because it is exactly the kind of half-pixel freedom that ADR-0011 records two agents
resolving differently and shipping two different videos.

The runner-up, only because Q4 above already diagnoses it: believing the overflow check
passed on `word-05` when the height it checked was a copy of `word-05`'s own arithmetic.

## Where I disagree with the questions

**Q4 presupposes a collision that does not exist.** It says (b) collides with *"an
element's size is declared, never defaulted"*. A text element's `height` is not a size —
ADR-0008 says it plainly: *"`box` remains a check input, not behaviour."* Nothing about
the rendered pixels changes when it is absent. ADR-0012's rule bans defaults whose **inputs
are not in the document**; the derived height's inputs are `size`, `line_height` and the
`\n`s, all on the same line of the same element. Q4 imports a principle by its slogan
rather than its argument, and the imported version is the one that would have forced (a).
Relatedly, **Q4's option (c) inverts the severity it is trying to create**: it fires 15
times on the only correct file in existence and zero times on the 7 elements that carry a
real claim.

**Q5's two options are not exhaustive.** "A field on the primitive" and "an entry in #22's
effect vocabulary" leave out the location ADR-0007 already assigned for text: **a run style
delta**. That is a third place, it is inside the element rather than on it, and it is the
only one of the three that can outline a single word. A question that offers only
element-level homes for a property whose whole point is sub-element variation would get the
text half wrong whichever of the two you picked.

**Q2 lists its candidates on the wrong axis.** `rect`/`ellipse`/`line`/`polygon`/`path`
reads as one axis of increasing expressiveness. The cost is not expressiveness, it is
**placement grammar**: `rect` and `ellipse` both fit `x`/`y`/`origin`/`width`/`height`
exactly as ADR-0012 and ADR-0013 settled it; `line`, `polygon` and `path` each need a second
grammar, and the second grammar is what has to be justified, not the shape. Ordered that
way the answer nearly falls out.

**Q6 is a routing question wearing a design question's clothes.** There is no sense in which
`gravity` could be "decided together with the text and shape primitives" — it has no
relationship to either; it is a field on `image` only. The live question underneath it is
better stated as: *ADR-0012 and ADR-0013 each handed the `gravity` measurement to two
tickets at once, and nobody owns it.* The answer is #48, and the reason is that `gravity`
has a job only if `fit` leaves slack, so the vocabulary that defines slack has to exist
first.

**And one thing none of the six asks.** Q3 settles how a colour is *spelled* but nothing
here settles how two of them **combine** — element `opacity`, paint alpha, and (if stroke
lands) two paints on one element. `card-05` at `opacity:0.5` with `fill:"#1E344C80"` and an
opaque stroke has an answer, and I do not know what it is from any document in this repo.
That is a smaller version of the same hole as the stroke-geometry one above: a visible
result determined by a rule that is in no file.
