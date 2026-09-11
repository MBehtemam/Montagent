# Answer — R2, juror A

Everything below was checked against the repo before it was argued. Verifications I ran:

- Fixture census reproduces: 22 `text`, 10 `rect`, 8 `image`, 20 `audio`.
- **Finding 4 reproduces exactly.** 15 of 22 text elements have `height ==
  ceil(size * line_height * line_count)`. The 7 that do not are the 5 `sentence-*`
  (declared 169, derived 61/126/126/61/61 — the navy card's height) and `chip-text` /
  `handle-text` (declared 84, derived 58/38 — the header panel's height). Widths are 984
  (×20), 238 and 472 — none is a typographic measurement.
- **Finding 5 reproduces.** All 10 rects carry `fill` and a flat `#RRGGBB` and nothing else.
  Note the existing asymmetry: text's paint field is `color`, a shape's is `fill`.
- **Finding 3 reproduces.** Six distinct drawings across the seven ASS files, every one
  `m … l l l` with four points, all axis-aligned, not one `b`. The two rectangles at
  (48,88) and (48,1453) are literally `m 48 88 l 420 88 420 172 48 172` and
  `m 48 1453 l 1032 1453 1032 1622 48 1622` — **square corners confirmed from the drawing
  commands themselves**, so finding 1 does not depend on reading the MP4.
- **Finding 6 reproduces**: `ScaledBorderAndShadow: yes` in all seven.
- One fact nobody listed that matters for Q7: **the old pipeline used no stroke at all.**
  Every ASS style has `Outline: 0, Shadow: 0`, and across all seven files there is not one
  `\bord`, `\shad`, `\3c`, `\4c`, `\alpha`, `\1a`, `\blur` or `\be` override. The only
  inline overrides in the entire corpus are `\pos`, `\an`, `\c`, `\p0/\p1` and `\fs`.
  Under ADR-0003's asymmetry this is *not* evidence against stroke — but it does mean
  `ScaledBorderAndShadow: yes` in these files is an inert default, not a decision the
  author made, and Q7 should not lean on it as evidence of intent. It is still the right
  *question* to answer.

---

## Q7 — the paint package

**What I first found attractive: Position A**, and specifically its run argument. Not
because of the "paint is consumed during rasterization" principle, which I think is the
weakest thing in the position, but because of a document fact the brief points at and
neither position quotes: ADR-0007's own Consequences list already parks *"the full run
style-delta set (colour, **outline** — with the ASS BGR-with-alpha-nibble trap and
`ScaledBorderAndShadow` written down)"*. An accepted ADR has already located outline in
the **run style-delta set**. Position B is not choosing between two open options; it is
relocating something a settled ADR scoped, and it does not say so.

### The strongest attack I could mount on A

Three, in ascending order of force.

**Attack 1 — A's stated principle proves far too much.** "A paint property is consumed
while the element rasterizes in its own geometry, so it is part of the primitive" is
equally true of drop shadow, glow and blur. ASS pairs Outline with Shadow under the *same*
`ScaledBorderAndShadow` flag, and both are per-event overridable, i.e. both are run-scoped
in exactly the way A says makes outline a primitive. So A's principle, taken at face
value, drags shadow and blur into the primitive and dissolves #22's boundary before #22
is written. ADR-0010 explicitly names blur as "an obvious member" of #22 and bought
`skia-safe` partly to have it. If A's reasoning admits blur, A is wrong.

**Attack 2 — B's `scale` objection is not as cheap as A treats it.** ADR-0012 spent real
argument establishing that `clip` does *not* scale, because the aperture is frame-space.
A stroke must do the opposite. So the format would carry two geometry-adjacent fields with
opposite scale behaviour, and nothing in the field names distinguishes them. That is a
genuine cost.

**Attack 3 — the one the brief's stroke-geometry finding hands you, and it is nearly
fatal to A as written.** If stroke centres on the path, `stroke_width: 8` on `card-05`
draws 992×177 at (44,1449) while the file still says 48/984/169 and `validate` passes.
ADR-0013 names that exact defect class in terms that read as a direct hit:

> that would make what is actually drawn a number **not in the file**, derivable only from
> source dimensions **also not in the file**, which is the defect ADR-0012 spent three
> paragraphs killing, re-entering through the renderer after being shut out of the schema.

And the margin is not decorative: the ASS drawings show the entire header and card layout
is built on exactly 48 and exactly 1032, and ADR-0012's retarget census says chrome
survives a reframe *because* those pixels are literal. A centred stroke eats 4 of the 48.
Worse, a centred stroke of odd width straddles a half pixel, which is the "half a pixel
of unspecified freedom (top? bottom? split?)" ADR-0013 disqualifies by name.

### Did it survive

Attack 3 kills **centred and outside stroke on shapes**. It does not kill stroke. Attacks
1 and 2 kill **A's stated rationale**, not A's conclusion. So A survives, amended, and the
amendments are what make it survive.

Replace A's "it rasterizes in its own geometry" with a sharper line, which also answers
Attack 1:

> A stroke is a **second paint on the same outline**, addressable at run granularity, that
> **never enlarges the element's declared rect**. Shadow, glow and blur displace or spread
> energy away from the outline; they are compositing over the frame, and they are #22.

That line excludes blur and includes outline, non-arbitrarily, and it is falsifiable: the
day someone wants an outline that grows the box, this ADR is what they have to reopen.

Attack 2 survives as a cost, paid: the format must *publish* both behaviours in one
sentence rather than leave either implicit. `clip` is frame-space and does not scale;
`stroke_width` is element-space and does scale and rotate with the element. That is
`ScaledBorderAndShadow: yes`, which is what all seven fixture files say, so the migration
never has to think about it.

### The alpha half — which I insist is a separate question

The brief asserts the two "resolve together" because alpha is "largely motivated by
wanting two paints with different transparency." **I think that framing is the weakest
link in Q7 and I reject it.** The strongest case for alpha in colour does not mention
stroke at all, and it is an in-repo parity argument from ADR-0013:

> spelling static geometry there bakes the stretch into every Ken Burns keyframe … so base
> geometry and animation share one channel and neither stays readable

ADR-0013 refused to make `scale` carry static anisotropy *and* the animation, and called
the result unreadable. `opacity` is in exactly that position. A 50%-translucent scrim rect
that also fades in must, under B, bake 0.5 into every keyframe value, so "fully present"
is spelled `opacity: 0.5` and the animation channel no longer reads as an animation. That
is ADR-0013's own objection, one field over. Two paints is a second, smaller reason.

B's counter — the ASS trap — I checked and it does not transfer. The fixture's
`&H008C5C24` is BGR (`8C5C24` → `#245C8C`, which is what `word-05` carries), with alpha
**leading** and **inverted** (`00` = opaque). `#RRGGBBAA` is RGB, alpha **trailing**,
**FF = opaque** — it is the negation of the trap on all three axes. ADR-0007 asks for the
trap to be "written down"; that is a migration-note request, not a prohibition.

The real objection to `#RRGGBBAA` is one neither position raised: **is 6-or-8 digits a
union of two spellings**, the thing this repo has killed three times (`center-center`,
scalar `scale`, the bare-string run)? Each of those was killed by the same mechanism —
`fmt` normalises on write, the agent's next exact-string replace gets zero hits. So the
rule has to be written to disarm that mechanism:

- A colour is **exactly 6 or exactly 8 uppercase hex digits**. 6 means alpha `FF`.
- **`fmt` never rewrites a colour** — not 6→8, not case. This is the same clause ADR-0013
  already wrote for declared extents ("`fmt` must never rewrite a declared extent"), for
  the same two reasons in the same order: the value is content, and the round trip.
- Lowercase is a **schema error naming the uppercase form**, not a silent rewrite.

Unlike `scale`, nothing interpolates a colour — ADR-0012's keyframable set is the
transform properties, and colour is not one — so the "every consumer that touches a
keyframe needs a shape test" argument that killed scalar `scale` has no analogue here.

I considered and rejected the obvious alternative, `fill_opacity` / `stroke_opacity`
floats: it avoids the string union but puts a field named `…opacity` next to the animation
channel named `opacity`, which is a name collision this repo would (rightly) refuse, and it
triples the field count once runs carry it too.

### Verdict — Q7

**Position A, amended on its rationale and on stroke geometry.** Confidence **high** for
text stroke (ADR-0007 already scoped it; "outline one word" is genuinely inexpressible
under an element-attached effect vocabulary), **high** for shape stroke, **medium-high**
for `#RRGGBBAA` — the animation-channel parity argument carries it, not the two-paint
argument the brief offered.

### The sub-question: inside, outside, or centred

**One principle, which is the same for shapes and for text: a stroke never enlarges an
element's declared rect.** Its *consequence* differs by type, because `width`/`height`
already mean different things on the two types, and that difference is already in the
document rather than invented here.

**Shapes: the stroke is INSIDE.** The declared rect is the outer bound of every pixel the
element paints; `stroke_width` eats inward from it. `card-05` with `stroke_width: 8` still
occupies exactly 984×169 at exactly (48,1453), the 48 px margin is untouched, and the
drawn rect is still the four numbers in the file. Justified against finding 1 directly:
the ASS drawings prove the layout is built on literal 48/420/1032/1453 edges with square
corners, and a stroke rule that perturbs those by `w/2` makes every one of those integers
approximately true. Three secondary reasons: odd widths need no half-pixel tie-break
(inner edge is `declared − w`, an integer); ADR-0013's aperture-coverage error and every
margin the layout rests on stay computable from the document with no new term; and it is
one line in Skia (stroke the path inset by `w/2` at width `w`, whose outer edge is the
declared rect exactly).

**Text: the stroke is OUTSIDE the glyph contour — and that does not contradict the
principle**, because a text element's `width`/`height` is not its painted geometry. It is
a declared container that typography is checked against; glyph ink already overhangs it
(side bearings, accents, descenders) and ADR-0007 places the *line block*, not the box, by
`origin`. An inside stroke on a glyph is not a thing any reference-class tool draws — it
eats the letterform's thin strokes and at `stroke_width: 8` on a 35 px run it erases the
glyph. So the outline grows outward from the contour, **into** the declared box, and the
principle is honoured by a different clause: **the overflow check's extent gains
`2 * stroke_width` on both axes** (`ceil(size*line_height*lines) + 2*sw` vertical,
measured advance `+ 2*sw` horizontal). That is what stops a text stroke from being a
number that silently escapes every check — which is the same failure mode Attack 3 found
on shapes, closed by a different clause because the two types mean different things by
`height`.

Stated as one sentence for the schema: *where the declared rect is the painted geometry
(shapes) the stroke is drawn inside it; where the declared rect is a container (text) the
stroke grows the typography into it and counts against the overflow check. In neither case
does the declared rect grow.*

One `validate` note falls out, firing zero times on the fixture: `stroke_width` ≥ half the
shape's smaller dimension draws solid, which is a fact from the document.

---

## Q8 — does a text element still declare a `height`

**(c) is dead on arrival** and I will not spend a paragraph on it: 15 findings on the only
correct project file that exists is ADR-0006's noise budget ("`0 errors, 47 notes` reads
as a pass") colliding with ADR-0013's "on a correct file the note fires zero times."

**What I first found attractive: (b).** Its demolition of the "declared, never defaulted"
objection is correct and I want to record that, because it is the part of (a) that looks
strongest and is not. ADR-0012 banned the natural-size default with a specific reason —
*"the source's dimensions are not in the document"* — and gave its own criterion, *"a
default is only safe where its wrong answer is loud."* A derived text height's inputs
(`size`, `line_height`, the `\n` count) are all on the same serialised line, and it has no
wrong answer, because the derived number **is** the typographic truth. So (b) passes
ADR-0012's actual test, and (a)'s uniformity argument is aesthetic, not principled.

I also confirmed (b)'s empirical claim rather than taking it: the externally-sourced
widths really are 984/238/472 (container-derived, 22/22), and for the 15 self-derived
heights there is in fact nothing behind them — they sit over the video, not over a card.

### The strongest attack I could mount on (b)

Not "vacuous inputs" — (a)'s rebuttal to that is right, and (b)'s slogan ("a check input
the author computes from the very thing being checked checks nothing") is **false**. It
confuses a tautology with a snapshot. ADR-0007 already established the pattern it
misreads: literal `size` is computed from the very thing it constrains, and the ADR's
whole defence is *"that loop does not disappear — it moves to authoring time and freezes
its result in the file, where a shaper upgrade cannot move it."* A frozen derived height
is the same object.

The attack that actually lands is different, and it is ADR-0006's, by name:

> an opt-in check they might not tag, on a command they might not run, would replace real
> vigilance with false confidence

Two of eight agents ranked that pattern **below doing nothing**. Under (b), the vertical
overflow check exists only on elements whose author chose to write a `height`. An omitted
height is indistinguishable from a decision not to check, which is *exactly* the `sequence`
label's defect — and ADR-0006 identified the one structural thing that saves `validate`
from `sequence`'s fate: it "checks uniformly, so a clean run is meaningful." Under (b),
`0 errors` no longer means "no text element has outgrown its declared extent"; it means
"none of the ones that opted in did." (b) spends validate's only structural defence.

### Which edits each option catches — the brief's question, answered directly

| edit | (a) both required | (b) height optional |
| --- | --- | --- |
| add a `\n` to an existing run | catches on **22/22**, free arithmetic, no font needed | catches on **7/22** |
| raise `size` on a run | catches 22/22 | 7/22 |
| raise `line_height` | catches 22/22 | 7/22 |
| **lengthen an existing line** (no new `\n`) | **misses** — width term only, needs `measure` + the current font binary | **identically misses** |
| swap the font | misses; ADR-0007's font-swap finding covers it | identically misses |
| delete a line / shrink `size` | **misses**, and leaves a now-false number in the file | omitted: nothing to go stale |

The two axes really are not alike, as the brief's third unvoted finding says, and the table
shows the asymmetry cleanly: **every edit that only the height term can catch is caught by
(a) on 22 elements and by (b) on 7, and every edit the height term cannot catch is missed
identically by both.** (b) has no compensating catch anywhere. Its only advantage is that
it removes 15 numbers from the file.

### Did (b) survive

**No.** The opt-in argument is decisive and it is the repo's own most-cited rule, applied
to a case it was written for. (b)'s best remaining reply — "an author who wants the
tripwire can still write a height" — is precisely the opt-in shape ADR-0006 rejected.

### Verdict — Q8

**Option (a): `width` and `height` both stay required.** Confidence **medium-high**. Two
conditions I would attach, because (a) as argued is not quite right either:

1. **The finding's wording cannot say "overflows its box."** For 15 of 22 there is no box.
   Per ADR-0006's "the level is never the finding," it must state the fact:
   *"`word-05`: declared height 97; the typography (size 88 × line_height 1.1 over 2
   lines) is 194."* That sentence is true and useful whether or not a container exists.
2. **`measure` must return the derived extent**, so the 15 numbers are copied from a tool
   rather than computed by hand. ADR-0005's `speed` precedent (`3368/0.645` → four agents
   at 5222, one at 5220) is what hand arithmetic costs, and ADR-0007 already measured 4/4
   agents inventing `line_height` values.

I record (a)'s residual weakness rather than hiding it: **an over-large height is
indistinguishable from a container claim.** An author who writes 200 instead of 194 "to be
safe" disables that element's tripwire permanently, and by construction nothing can tell
that apart from the 5 `sentence-*` elements legitimately claiming the card's 169. That is
inherent to the design and is not fixable inside this ticket.

The brief's second unvoted finding — the container-copied heights are not bindings, so
resizing `card-05` leaves 5 stale caption heights with `validate` green — is true and is
**out of scope here**. ADR-0012 already saw it and already assigned it elsewhere:
*"It looked like a binding and was not one. Placement drift is not fixed here and needs
its own ticket."* Neither (a) nor (b) nor (c) touches it, and no option should claim to.

---

## Q9 — `line` and `polygon`

**What I first found attractive: Position A (rejected, on placement grammar)**, on the
meta-argument rather than the geometric one. ADR-0003 diagnosed exactly this failure:
*"correcting a session fixes one session; correcting the document fixes every session
after it."* "Unevidenced" is a status that gets re-litigated by the next jury from
scratch; a rejection with a stated, falsifiable reason is the thing that stops recurrence,
and this repo already has the house form for it — ADR-0012 rejected skew with a reason
*and* a named re-entry path ("it arrives later as an entry in #22's closed vocabulary if
it arrives at all"). That is not "never."

### The strongest attack I could mount on A

**A's stated reason is false as written.** "A point list needs a second placement grammar
that no transform, origin or clip rule covers" — it does not, and the counter-design is
five minutes old and obvious: define `points` in a **local** space normalised to the
declared `width`/`height`, exactly as SVG's `viewBox` does. Then `x`, `y`, `origin`,
`scale`, `rotation` and `clip` all apply unchanged, with no new rule anywhere. A polygon
becomes an ordinary element with one extra field. If that is true, A's whole premise
collapses and B wins by default.

Secondary attack: A's throwaway consolation — "a thin rotated `rect` already draws any
segment" — is weak. Drawing a connector from (100,200) to (700,900) requires the author to
hand-compute `sqrt(360000+490000)` and `atan2`, neither integral, and the file that results
states neither endpoint. That is ADR-0005's `speed` divergence in a new costume, and it
means A's "already expressible" leg does not hold weight.

### Did it survive

**The conclusion survived; the reason had to be rebuilt, and the attack is what rebuilt
it.** The local-coordinate design does not escape — it collides with two settled decisions
rather than one:

1. **It introduces a second unit system.** ADR-0012 chose absolute integer pixels over
   fractions on a measured census (retarget the fixture and 32 of 60 elements leave the
   frame; the 28 survivors are the audio plus the chrome, which survives *because* pixels),
   and named the silent failure fractions produce — "a Union Jack inside it that is no
   longer the shape of a flag, **silently, because the numbers stay in range**." Normalised
   point lists reintroduce precisely that class, inside one element.
2. **The alternative — absolute pixel points — makes the extent derived from content**,
   and then either the declared rect and the points disagree (two truths for one geometry,
   with nothing saying which the renderer obeys) or `width`/`height` are computed from the
   point list, which is a **size defaulted from the content** and is banned by ADR-0012
   and re-affirmed by ADR-0013. It also reopens Q7's rule above: a point at (−5, −5) would
   have to grow the declared rect.

So the rejection stands, with the reason restated more strongly and more falsifiably:

> **A point list has no declared extent.** Every visual element in this format is placed
> by `x`, `y`, `origin`, `width`, `height`, and ADR-0012/0013 require that extent to be
> *in the document* rather than derived from what the element contains. A `line` or
> `polygon` must either derive its extent from its points (banned) or carry a second,
> normalised coordinate space (a second unit system, against ADR-0012's census). Either is
> a new decision about **placement**, not a new entry in a shape list.

One more thing I have to be honest about, because the brief warns about exactly this
juror. **Fact 3 must not carry the rejection.** "A real authoring system with an
unrestricted path language shipped only rectangles" is six drawings by one author on one
channel, and ADR-0003's asymmetry says channel output is *never* evidence a capability is
unneeded. Fact 3 is corroboration — it shows the rejection is unlikely to hurt anyone
soon — and it is not a premise. The extent argument above is the premise, and it stands
with the fixture deleted.

### Verdict — Q9

**Position A — rejected, with the reason restated as the extent argument**, fact 3 demoted
to corroboration, and the "a thin rotated rect already covers it" line **dropped** rather
than published, since it is false in the way that matters (the endpoints are not in the
file). Confidence **medium-high**. The re-entry path is explicit and is the payoff of
recording the reason: the day a genuine point-list shape is needed, the ADR to write is
*"how is an element whose extent comes from its content placed"*, and it will settle
`line`, `polygon` and any future `path` at once.

---

## What would bite me first

**1. `measure` not knowing about stroke.** This is my honest first casualty and it is
created by my own Q7 answer. I made a text element's overflow extent
`typography + 2 * stroke_width`. If `measure` returns stroke-naive extents, every author
adds `2*sw` by hand on both axes, and ADR-0005's `speed` precedent says they will not all
get the same number. `measure`'s signature must take `stroke_width` and return the inked
extent, or the check I just specified is the second place the truth lives.

**2. Adding `stroke` to an existing correct file is a churn event.** The moment
`stroke_width` becomes legal, an author who adds it to captions may push 15 previously
clean elements over their declared heights at once — a wall of findings that are all
*correct* and all arrive together, which is exactly the shape ADR-0006 says gets ignored.
The fixture is safe (it has no stroke, verified: `Outline: 0` everywhere, no `\bord`), so
this will bite the first real user rather than the regression guard, which is worse.

**3. Inside-stroke on shapes will read as a renderer bug.** Agent priors are split — SVG
strokes centre, CSS `border` sits inside — so half of all agents will write
`stroke_width: 8` on a rect, see the fill shrink instead of the box grow, and file it as
broken. Mitigation is one schema sentence and the solid-shape note above; it is mitigation,
not a cure.

**4. The over-large height that silently disables its own tripwire** (Q8, recorded above).
Unfixable inside this ticket by construction.

## Questions I think are malformed or presuppose something false

**Q7's bundling is the malformed part.** "These two resolve together" is asserted and then
used to frame both positions as packages. They do not resolve together: the strongest
argument for alpha-in-colour (ADR-0013's static-value-poisoning-the-animation-channel
parity) makes no reference to stroke, and a coherent position exists at every corner of
the 2×2. I answered A on both halves, but on **two independent arguments**, and I think
the bundle should be dissolved in the ADR rather than carried into it. A jury that splits
on stroke should not thereby be split on colour.

**Q8 option (b)'s headline argument is false**, as argued above: "a check input the author
computes from the very thing being checked checks nothing" conflates a tautology with a
frozen snapshot, and the format has already shipped the same object (literal `size`) with
the opposite reasoning. (b) is a defensible option resting on an indefensible slogan.

**One presupposition in Q8 that I could not resolve from the ADRs, and that the ticket
should settle explicitly.** Both options assume declared `width`/`height` on text is
*purely a check input* and affects nothing drawn. ADR-0007 says the line block "is placed
according to `origin`" and that `align` aligns lines *to each other* — which reads as the
declared box framing nothing. But that is inference, not a published sentence, and if the
box does frame `align` or `origin`, then making `height` optional changes **rendering**,
not just checking, and (b) is a far larger change than it presents itself as. Whichever
option lands, the ADR should state in one sentence that a text element's declared
`width`/`height` participates in no placement and no alignment. It is currently true by
silence, which is the condition this repo keeps finding expensive.

**Q9 is well posed**, and I would only note that its Position A ships one false supporting
claim (the rotated rect) which should not survive into the ADR.
