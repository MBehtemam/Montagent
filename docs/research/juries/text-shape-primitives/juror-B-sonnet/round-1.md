# Answers: text and shape primitives (Q1–Q6)

Written as the agent who has to open this file with an exact-string replace, add an
element, and re-check it with `validate`/`frame`/`measure`/`query`.

## Q1 — One `type:"shape"` with a `shape` discriminator, or sibling types?

**Answer: sibling types — `type:"rect"`, `type:"ellipse"`, etc. No `type:"shape"` wrapper.**

**Confidence: high.**

**Reason.** Every other element type in this format already discriminates on `type`
directly — `image`, `video`, `audio`, `text` — and ADR-0012 is explicit that "the
schema varies by type" and that a mismatched field (`x` on audio) is a schema error
naming the *type's* replacement. A `type:"shape"` + `shape:"rect"` double-discriminator
introduces the one thing ADR-0012 just finished rejecting for `scale`: two ways to
learn what an element is, so every consumer — `validate`, `query`, an agent reading a
line to decide what fields are legal — has to branch on a second field instead of one.
Concretely: I'm asked to add a circular badge behind `handle-logo`. Under sibling
types I search `"type":"ellipse"` and get exact hits or nothing. Under the nested form
I'd have to search `"shape":"ellipse"`, which also matches a `type:"shape"` polygon
that happens to carry a variant field named the same by accident, or I'd have to know
to search for the pair. It also breaks the file's one-line-per-element convention
functionally, not visually: a discriminator that takes two keys to resolve is a
two-step lookup pretending to be flat. The existing `type` field already does this job
for four other element kinds; a fifth kind reusing the same axis is consistent, not a
new decision.

## Q2 — Which shapes exist in v1?

**Answer:**
- **In:** `rect`, `ellipse`.
- **Out (named, not deferred):** `path` — arbitrary bezier path data is exactly the
  "expression the renderer must evaluate" ADR-0003 excludes when it defers After
  Effects; a `d`-string primitive is unreadable by the inert-data test the whole
  format is built on, and no fixture evidence or CapCut/Premiere comparison calls for
  free-form vector authoring by an LLM typing coordinates.
- **Unevidenced, not rejected:** `line`, `polygon`. Both are ordinary CapCut/Premiere
  shape-tool primitives — squarely inside ADR-0003's reference class — but nothing in
  the fixture uses one and nothing in the ADRs blocks one. `polygon` is a genuine
  open question (regular polygon by side count? explicit point list? — point lists
  reopen exactly the unlabelled-array trap ADR-0012 called out for keyframes). `line`
  is closer to trivial (two points, or a rect with height forced to stroke width) but
  still unscoped.

**Confidence: high on rect/ellipse in, path out; medium on where line/polygon land.**

**Reason.** The fixture gives only `rect` (fact 3, 10 elements, `fill` + flat
`#RRGGBB`, nothing else) — but per the brief's standing rule that's evidence `rect`
is *needed*, not that `ellipse` is unneeded. I'd reach for `ellipse` on my very next
edit of this file: the logo badge (`brand/logo-en.png`) is described as circular and
the corner-alpha finding (fact 2) means the *visual* circle currently lives only
inside a baked PNG — if I ever need a drawn (non-image) circular badge, a `mask` or
an `ellipse` element is the only way to spell it, and `mask:"circle"` is explicitly
called out elsewhere as dead weight on this asset. `ellipse` costs one more element
type and buys "circular badge without a source image," which is squarely
CapCut/Premiere-class, not After-Effects-class. `path` is the one I'd actually reach
for out of over-generalization and shouldn't get — it's the After Effects side of
the line ADR-0003 drew (cf. skew being pushed to #22's closed vocabulary for the
same reason).

## Q3 — How is a colour spelled?

**Answer: `#RRGGBB` only. No `#RRGGBBAA`, no 3-digit shorthand, no CSS names.**

**Confidence: high.**

**Reason.** ADR-0012 already gives every element an `opacity` field, `0..1`, as part
of the flat transform. An `#RRGGBBAA` alpha nibble on `fill` would be a second,
competing alpha channel on the same element — and this format has *already* been
burned by exactly this shape of duplication once: ADR-0007 names "the ASS
BGR-with-alpha-nibble trap" as a known hazard to avoid repeating. Two alpha channels
means a real authoring question with no good answer: if I write `fill:"#1E344C80"`
and the element's own `opacity` is `0.6`, what composites? Nothing in the format
says, and whichever renderer convention wins, `query` now has to expose two numbers
that both mean transparency and an agent has to know which one to edit for "fade
the whole element" versus "make the fill translucent while the stroke stays solid" —
a distinction only worth the cost once `stroke` exists as a separate paint field (Q5),
and even then it argues for opacity living on `opacity`, not proliferating alpha
nibbles per paint field. Shorthand (`#RGB`) and CSS names fail the same test ADR-0007
applied to keyframe positional pairs and to font family names: they're conveniences
that only *look* free. A CSS name table is exactly the "table you cannot read, cannot
commit, and that differs per machine" argument CONTEXT.md uses to kill system font
families — `rebeccapurple` requires the reader to have memorized or look up a spec
table, where `#663399` is legible standalone. 3-digit shorthand additionally breaks
`fmt`'s job: ADR-0011 requires `fmt` to normalise on write, so `#333` would either get
silently expanded to `#333333` (defeating my next exact-string-replace on the shorthand
I just typed — the identical failure mode ADR-0012 used to kill scale unions and
`center-center`) or stay unexpanded and be a second legal spelling of one colour
forever. One canonical spelling, `#RRGGBB`, six hex digits, no ambiguity, no
normalisation round-trip to fall into.

## Q4 — Does a text element still declare `height`?

**Answer: (a) — `width` and `height` both stay required; the vertical overflow check
is legitimately a no-op on 15 of 22 elements.**

**Confidence: medium-high.**

**Reason.** The brief's own framing of (b) already kills it: "declared, never
defaulted" is CONTEXT.md's and ADR-0012's load-bearing rule for *every* visual
element's size, stated as a hard constraint (the argument that killed natural-source-size
defaults for images applies identically to text — an absent `height` "fails quietly
because [it] looks plausible"). An optional `height` reintroduces exactly the failure
ADR-0012 spent three paragraphs closing: an element whose rendered box is not fully
in the document. (c) sounds attractive — flag redundant `height` as a finding — but
it's a trap for the 15/22 case ADR-0007 already litigated: those elements have *no
element behind them to name*, so their `height` is not redundant with anything else in
the file, it's simply derived from the element's own already-declared `size` /
`line_height` / line count. Flagging "your height equals the value we could have
computed" as a finding trains me to stop writing it accurately, which is the same
shape of harm ADR-0007's `box`-retirement text warns about (a field that's sometimes
present and sometimes computed cannot produce a good error message). Concretely: if I
edit a run's `text` to add a line and forget to also bump `height`, under (a) `validate`
correctly reports vertical overflow even though 15/22 of the time the "container" is
notional — that's fine, it's still catching a real editing mistake (I changed content,
the declared box no longer matches). Under (c) the same edit might get suppressed as
"height matches formula, no finding" only for the moment before I add the new line,
and the finding I actually want (overflow) has to compete for attention with a
finding I don't (redundancy). Keep both required; let the no-op be a no-op.

## Q5 — Is `stroke` a primitive field or an effect?

**Answer: `stroke` is an *effect*, not a paint field on the primitive — deferred to
#22's closed vocabulary, same as skew.**

**Confidence: medium.**

**Reason, in one sentence for the line:** a field belongs on the primitive when the
renderer draws it as part of resolving the shape's own geometry and fill in one pass
(what fact 3 already shows — `fill` is a flat paint field, nothing else, on every
rect in the only real file); it belongs to effects when it composites *around* or
*on top of* the already-resolved shape (an outline with its own width, colour, join
style, and inside/outside/center placement, plus — per ADR-0007's own named
`ScaledBorderAndShadow` trap — behavior under `scale` that has to be decided once for
every stroked thing rather than once per element type). Concretely: the moment I
write `stroke` on `card-05`, I immediately need to know whether the 4px border scales
with the Ken Burns `scale` keyframe the way `clip` explicitly does *not* (ADR-0012:
"clip... does not rotate and does not scale with the element") — that's an animation-
interaction question identical in shape to the one ADR-0012 refused to settle for
skew and effects generally, and ADR-0007 flags the ASS border/shadow scaling trap by
name as exactly this unresolved territory. Confidence is only medium because `rect`'s
paint set (fact 3: `fill` and nothing else, on 10/10) is thin evidence either way — it
proves nothing about stroke was *needed* yet, not that it's excluded, per the
standing rule. But `stroke` on *text* makes the "it's an effect" case stronger, not
weaker: text already has a base-style/run-delta split (ADR-0007) and an outline is a
classic ASS/CapCut caption treatment that composites over already-shaped glyphs — it's
naturally styled the same way as a drop shadow or glow, i.e., alongside other text
effects in the closed vocabulary, not as a shape-only paint primitive that text would
then have to duplicate.

## Q6 — Does `gravity` get decided with text/shape, or with `fit`?

**Answer: with `fit`, not here.**

**Confidence: high.**

**Reason.** This question is entirely about images (fact 5, ADR-0013's finding that
`gravity` is inert on 8/8 image elements once the declared rect and `clip` are
authoritative) and has zero surface area on text or shape primitives — no text or
shape element in the fixture or the ADRs carries `gravity`, and nothing about `rect`,
`ellipse`, `stroke`, or text height touches "which part of the source survives a
crop" (shapes and text have no source to crop). ADR-0012 and ADR-0013 both explicitly
hand this fact to #13 *and* #21, not to whatever ticket owns text/shape — deciding it
here would mean re-deriving the fitted-extent floor-vs-ceil reasoning and the
aperture-coverage check from ADR-0013 inside a document about an unrelated primitive,
duplicating settled machinery instead of citing it. Concretely, if I'm asked to add a
`gravity` decision to this ticket's output, the edit I'd have to make is on `image`
elements' schema — a type this ticket doesn't otherwise touch at all — which is a
strong signal the question is misfiled here rather than genuinely open on shapes/text.

---

## What would bite me first?

**Q4/height, in the 7-of-22 "container" cases.** The design correctly keeps `height`
required everywhere, but it does not — and the brief doesn't ask it to — settle what
happens when the *container* changes size out from under a caption. `sentence-05`'s
`height:169` is copied from `card-05`'s `height:169`; nothing ties them together (the
brief's own fact 4 says this plainly, and ADR-0012 names "placement drift" as a
known, explicitly *not settled* problem — "nothing now keeps a caption with the card
it sits on"). If I resize the navy card by 20px for a longer sentence on a later
edit, `validate` will not tell me the caption's declared `height` is now stale,
because both numbers are independently "declared, never defaulted" and both pass
every check on their own terms — the card's `height` is a valid rect height, the
text's `height` is a valid (if now wrong) box. I would ship a video where the card
grew but the text-overflow check that's supposed to protect me was computed against
the old, now-incorrect number, and nothing in `validate`'s output would distinguish
that from a healthy file. This is the same failure class ADR-0012 already named for
`box:"card-05"` retirement ("it looked like a binding and was not one") recurring
under a different name, and it's the thing most likely to make me trust a green
`validate` run that is quietly lying about one specific element.

## Where I disagree with the question itself

**Q1 presupposes shapes need their own decision separate from the type-discrimination
pattern already set by `image`/`video`/`audio`/`text`.** It isn't a live design
question so much as "does this format's existing, already-settled convention apply to
a fifth type" — the answer falls out of ADR-0012's type-varies-schema rule with no new
reasoning required, and treating it as open invites re-litigating something the
project already decided under a different name.

**Q5 asks for "one sentence" on where the primitive/effect line falls, but the fixture
gives no evidence either way** — 10/10 rects have `fill` and nothing else, which is
consistent with "stroke doesn't exist yet" and with "stroke exists but nobody used it,"
and can't distinguish them. The honest answer leans on ADR-0003's After-Effects
boundary and ADR-0007's named ASS border trap, not on this project's own fixture — worth
flagging because every other question here is at least partly fixture-grounded and
this one structurally can't be.
