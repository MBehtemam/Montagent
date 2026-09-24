# Juror E2 — round 2 verdict on #48, the `fit` vocabulary

All arithmetic below was executed, not estimated. `fit_rounding_scan.py` reproduces
(31,402,800 combinations, 4.466% float disagreement, 0 integer undercoverage).

The closed set I am recommending is exactly three words:

```
fit : "cover" | "contain" | "declared"
```

---

## R2-Q1 — the escape value's name

**Decision:** `declared`. `fit:"declared"` means *no derivation rule was applied; these
integers are the author's and nothing is derived from the source*. `none`, `exact` and
`stretch` are all rejected, each for a different reason. Runner-up if `declared` is
rejected: `manual`. **Not** `none`.

**Why:**

*The name must describe the cause, not the effect, because every other member of the set
does.* Round 1 settled that `fit` is a derivation claim, not a render instruction — at
render time all three values produce byte-identically the declared rect, so **the effect
is constant across the vocabulary and cannot name anything.** `cover` and `contain` name
derivation rules. A member named for its effect (`stretch`) would be the one word in a
closed three-word set sitting on a different axis, and a reader who correctly infers "the
values name rules" would look for the rule `stretch` names and find none. `declared`
names what the other two name: where the integers came from. Here, from the declaration.

*`none` is the one candidate that is actively dangerous, and it is dangerous in the
format's own preferred failure mode — quietly.* CSS `object-fit: none` means **use the
source's intrinsic pixel size and crop to the box — do not resample**. Montagent's escape
means the exact opposite: **do resample, to my integers, anisotropically if that is what
the numbers say.** Same word, inverted behaviour, and an agent arriving from CSS — which
is the population this format keeps designing against (ADR-0013 reached for
`transform-origin` and `background-position` to settle `center` vs `middle`) — gets a
plausible-looking centre-cropped picture and no error. That is ADR-0012's *"it fails
quietly because a centre-cropped photo looks plausible"* re-entering through a value
name. `none` has a second, independent defect: `fit:"none"` and an omitted `fit` read as
synonyms in every agent's head, and round 1 settled that omission is a **schema error**.
A value whose spelling is a synonym for the illegal state is a trap.

*`exact` is spent inside this repo's own prose.* ADR-0013 uses "exact" throughout as a
term of art for *the unrounded real value*: "exact cover here is **1912.5** px", "exact
integer arithmetic", "it is exact by construction". `fit:"exact"` therefore reads to a
Montagent-native reader as *"do the cover computation exactly, don't floor it"* — which is
not an escape, it is a fourth rounding mode, and it is unrepresentable (extents are
integers). The word is unavailable.

*`stretch` is wrong on the majority of the cases it would be written for.* It names an
anisotropic resample, but the escape is needed whenever the author declines derivation
for any reason, including when the result is isotropic or exact. `handle-logo` is
800x800 → 68x68: an exact aspect match, zero stretch. If that element were written with
the escape it would be spelled `fit:"stretch"` while stretching nothing. A value name
that is counterfactual on real elements is the same defect as a field the renderer cannot
honour.

*There is no CSS-native name available anyway, which is itself the argument against
reaching for one.* CSS's word for this concept is `object-fit: fill`, and `fill` is
**doubly spent** in Montagent: ADR-0005 spends it on the time axis (`fill: hold|loop`) and
ADR-0014 spends it on shape paint (all 10 `rect` elements carry `fill`). So any
CSS-shaped name we choose will necessarily be the *wrong* CSS word. Better to choose a
word CSS does not have at all: an agent from CSS meets an unfamiliar token, must look it
up, and the failure mode is loud.

*`declared` has no collision in `CONTEXT.md`* — the recorded collisions are `anchor`,
`align`, `box`, `gravity` and `path`, and the word is already this format's own normative
vocabulary for exactly this property: ADR-0013's section is titled *"The declared rect is
authoritative at render."* The escape value is the case where that sentence is the whole
story.

**Strongest counter:** `declared` is arguably tautological, and the tautology is not
small. Under declared-rect-authoritative **every** extent in the file is declared and
authoritative, including under `cover` — the `1912` on `photo-06` is as declared as any
other integer. So the word names a property the entire schema has, and does not
distinguish the member from its siblings. `manual` (a human chose these), or even `none`
(no rule), at least name something the other two lack. The rebuttal is that the
distinguishing content is *sole-sourcing* — under `cover` the declaration is a
transcription of a derivation and `validate` will check it against the source; under
`declared` the declaration is the only source and there is nothing to check it against —
and the schema's one-line gloss carries that. But the counter is real: `declared` is the
weakest of the three names on self-evidence, and it wins on the elimination of the other
candidates more than on its own merits. If a fourth candidate were offered I would not
defend `declared` hard against `manual`.

---

## R2-Q2 — does `contain` ship in v1?

**Decision: yes, `contain` ships in v1, and the aperture check is parameterised by `fit`
rather than scoped away.** In the same breath I report that **ADR-0013's stated reason for
preferring floor is factually wrong**, and supply a replacement (see also *Anything these
five missed*).

**The inequality.** `contain` derives the largest rect of the source's aspect that fits
**inside** the box: `w <= bw` **and** `h <= bh`. The bound is containment, and the
derivation is the mirror of `cover`:

- The **driving axis** is chosen by integer cross-multiplication: **width drives when
  `bw*sh <= bh*sw`.** Note the inequality is `<=` where `cover`'s is `>=`; both put
  **width on the tie**, so the tie convention is one sentence for the whole vocabulary.
- The driving axis **takes the box dimension verbatim**.
- The slack axis is `(s_slack * b_driving) // s_driving`, **floor**, integer arithmetic.

Worked on the fixture's one non-trivial geometry: source 1536x2720 into clip 1080x1300.
`1080*2720 = 2937600` is **not** `<= 1300*1536 = 1996800`, so **height drives**:
`h = 1300`, `w = 1536*1300 // 2720 = 734`. The element is **734x1300** — the mirror image
of `cover`'s 1080x1912, and 346 px narrower than its aperture (173 px of background per
side).

**Why floor — and why ADR-0013's reason for it does not survive contact.** ADR-0013's
tiebreak (2) says: *"If `contain` is ever defined with the obvious semantics, only floor
is safe there."* **That is false.** Under `contain` the slack axis's exact value is by
construction `<= b_slack`, and `b_slack` is an integer, so `ceil(exact) <= b_slack`
holds — ceil preserves containment exactly as trivially as floor preserves coverage.
Measured: **3,000,000 random `(source, box)` pairs up to 8000x8000 → 0 containment
violations under floor and 0 under ceil**, plus an exhaustive edge scan over
`sw,sh in [1,400)` against seven pathological box dimensions, also 0 and 0. Floor and ceil
disagree on 99.85% of those pairs and **both are safe on all of them**.

So direction-preservation is indifferent for `contain` too, and the tiebreak survives on
ADR-0013's *other* two legs, which do hold:

1. **Uniformity.** Floor is already fixed for `cover` by ADR-0013's tiebreak (1) —
   additivity — which that ADR itself says "is what actually carries the weight". Given
   that, floor for `contain` keeps **one rounding operation across the whole vocabulary**;
   ceil forks the rule into a per-value rounding table for no geometric gain.
2. **Implementation entropy.** `//` is floor in every language's integer division for
   positive operands. Ceil needs `-((-a)//b)` or the known-buggy `(a+b-1)//b`. For a rule
   whose purpose is inter-implementer agreement, take the free operation.

Floor has one real cost that `cover` does not have and that nothing in the repo has
noticed: **`contain` + floor can derive a zero extent.** `src 300x7` into box `10x10`
gives `(10, 0)` — an element that is not drawn. In a small-box sweep this occurs on
**14,380,626 of 135,978,921** combinations. `cover` + floor can never do this (the slack
axis is `>= b_slack >= 1`). I do **not** add a clamp — a clamp is a second rounding rule
and re-forks what floor was chosen to keep unforked. It needs no special casing: a
declared extent must be a positive integer, so a rule value of 0 can never equal any legal
declared rect, and round 1's promoted fit-deviation **error** fires automatically. The
author's recourse is `fit:"declared"`. That is the escape value doing precisely the job it
was created for, and it closes the loop between Q1 and Q2.

**The aperture collision, resolved by parameterisation.** ADR-0013 ships *"the declared
rect must contain `clip`"* as an `error`, hard-coded to the cover direction. A correct
`contain` element is smaller than its clip and trips it — `734x1300` vs `clip
[0,0,1080,1300]` fails on the width axis by 346 px.

The fix is to recognise what the check's content actually is. It is **not** a universal
geometric law about apertures. Its stated justification is *"the consequence of failing it
is background showing through **where the author declared full cover**"* — it is a
document-self-consistency check, and the clause it is consistent **with** is the declared
`fit`. It reads as universal only because `cover` was the only value in existence when it
was written. Rename it **aperture agreement** and give it three branches:

| declared `fit` | the check | level |
| --- | --- | --- |
| `cover` | declared rect must **contain** `clip` on both axes | `error` |
| `contain` | declared rect must be **contained by** `clip` on both axes | `error` |
| `declared` | neither relation is implied; if the rect does not contain `clip`, report the uncovered area in px | `review` |

Every branch keeps the structural property ADR-0013 specifically prized: **all three are
computed from `x`, `y`, `origin`, `width`, `height` and `clip` alone — no media, no probe
— so an unprobeable source can never suppress an error.** The `cover` branch is
byte-identical to what ADR-0013 shipped and still passes 8 of 8 image elements. The
`contain` branch is its exact mirror and is equally media-free. And the `declared` branch
is the important one: **the escape value does not switch checking off, it demotes one
check from `error` to `review` and prints the fact.** Background inside the aperture is
the textbook ADR-0006 `review` — *"legal, renders, and you must look at a frame to know if
it was meant"* — and the `review` count is in the summary line, so `declared` can never
become a silent hole.

**On ADR-0003 and the fixture's non-use of `contain`.** The asymmetry rule forbids me from
counting zero fixture instances as evidence against, and I do not. But my positive
argument is not *"a general editor needs letterboxing"* either, because that argument
would equally license shipping ten more values. It is structural and it is about this
schema: **with only `cover` + `declared`, the sole spelling for "my picture sits inside
the aperture" is `declared`, which is the value that turns the `error` off.** That is
ADR-0006's disease exactly — the only way to express a legitimate intent is to stop being
checked — and it would make `declared` the routine value rather than the last resort,
degrading the whole vocabulary. Shipping `contain` gives the second-commonest real intent
a **checked** spelling and keeps `declared` rare. That is an argument from the format, not
from the channel, which is what ADR-0003 demands.

**Strongest counter:** `contain`'s rounding rule is unfalsifiable by committed data.
Floor-versus-ceil for `contain` breaks **0 of 0** elements, so unlike `cover` — where
ADR-0013 could point at seven elements ceil would rewrite and at ADR-0012's published
element — there is no regression guard and no corpus at all. ADR-0013 was scrupulous that
its own evidential base was *"one hand-authored integer of unrecorded provenance"*; mine is
zero integers. Worse, I have just falsified the one *geometric* argument for floor on
`contain`, which means the choice now rests entirely on uniformity with a decision taken
for a different value on additivity grounds — a chain of two contingent tiebreaks with no
measurement under either. A disciplined alternative is genuinely available: ship `cover` +
`declared` in v1, and let the first real `contain` element bring its own rounding decision
and its own evidence. I reject it because the cost of waiting is not neutral — it is
`declared` becoming load-bearing, which is worse than a thinly-evidenced rounding
direction — but I hold this counter to be the strongest in this verdict, and I would
mitigate it rather than dismiss it: **add the `contain` derivation, the zero-extent case,
and the ceil-is-also-safe sweep to `fit_rounding_scan.py`**, per the repo's own *"commit
the evidence an ADR rests on."* Committed evidence is what turns this from an assertion
into a fact that can rot loudly.

---

## R2-Q3 — a declared `fit` when `clip` is absent

**Decision: the box falls back to the element's own declared `width`/`height`.** Not
`UNCHECKED`, not the project frame, and not a schema error.

**Why:** the premise that makes the other three options look attractive — that self-box
makes the check vacuous — **is false, and I measured it.**

It is true that self-box is a fixed point on all 8 committed elements. It is **not** a
universal fixed point:

```
src 1536x2720, rect 1080x1912  -> self-box cover (1080, 1912)   reproduces
src 1536x2720, rect 1080x1913  -> self-box cover (1080, 1913)   reproduces
src 1536x2720, rect 1080x1911  -> self-box cover (1080, 1912)   FIRES
src 1536x2720, rect 1080x100   -> self-box cover (1080, 1912)   FIRES
```

What self-box computes is the **box-independent residue of the cover claim**: under
`cover` into *any* box, the derived rect carries the source's aspect ratio with the driving
axis exact and the slack axis floored. That property does not mention the box. So when the
box is gone, the surviving check is *"the declared rect is the source's aspect ratio to
within one floor step"* — and it accepts exactly `{floor(exact), ceil(exact)}` on the
slack axis, which is **verbatim the predicate round 1 settled.** The fallback does not
weaken the predicate; it evaluates the same predicate with the only box the document still
offers.

Mechanically, under self-box the driving axis is the box dimension by identity, so round
1's zero-grace driving-axis clause is trivially satisfied and the slack clause carries the
whole check. That is not degeneracy — it is the claim minus the part the document stopped
asserting.

**And it catches the case ADR-0013 handed to this ticket.** The stale-source case — source
re-exported to 1536x2200, declared rect still 1080x1912 — with `clip` removed:

```
self-box: 1080*2200 = 2376000  <  1912*1536 = 2936832   -> HEIGHT drives
          h = 1912 (declared, exact)
          w = 1536*1912 // 2200 = 1334      declared 1080     -> ERROR, 254 px / 23.5% off
```

A fallback that fires on the flagship failure case is not a check that has silently
weakened to nothing. Compare the alternatives, each of which fails on ADR-0006 grounds:

- **`UNCHECKED`.** ADR-0013 defines that category narrowly and structurally: the question
  is *unanswerable* — missing file, permission denied, unfetchable URL. Here it is
  answerable; we have the source and the rect. Spending `UNCHECKED` on an answerable
  question inflates the exact count that exists to prevent false confidence, which is
  self-defeating. And it would make **omitting an optional field the way to stop being
  checked** — ADR-0014's *"An omitted `height` is indistinguishable from a decision not to
  check"* and the `sequence` label re-entering for the third time.
- **The project frame.** Falsified by round 1 on the fixture (1080x1920 gives 1084x1920,
  not the published 1912) and wrong in principle: it would declare that every image is
  meant to fill the frame, so any deliberately small image — a logo, an inset — fires a
  false error. `handle-logo` at 68x68 would report a rule value of 1920x1920.
- **A schema error.** This forces `clip` onto every raster element, since round 1 made
  `fit` **required**. Required-`fit` + error-without-`clip` = required-`clip`, which
  contradicts `clip` being optional and contradicts ADR-0012's own finding that *"zero text
  and zero rect elements need it"* — the aperture exists for spill, and a full-frame image
  with no spill needs none. The only alternative left to such an author would be writing
  `declared`, which would push the escape value onto ordinary elements and re-run Q2's
  degradation.

The honest statement for the schema is one sentence: **when `clip` is absent the box is
the element's own declared rect, and the check that survives is aspect agreement with the
source within one integer step.** Publish the weakening explicitly rather than letting a
reader discover it, which is ADR-0006's *"the report's own boundary is printed in the
report"* applied to a schema clause.

**Strongest counter:** the fallback is a **semantic** fiction even though it is an
arithmetic success. "Cover" is a relation between a picture and an aperture; with no
aperture, saying the element covers *itself* is a sentence with no content, and I am
keeping it only because a useful check falls out. That is reasoning backwards from the
check to the semantics — and this format has been strict about not doing that (ADR-0013
refused to create a value by implication for exactly this reason). A cleaner design would
name the aspect check as its own thing — say `fit:"aspect"`, a fourth value meaning *my
rect is the source's aspect, no box involved* — and make `cover`-without-`clip` an error,
so each word means one thing. I reject that because it adds a fourth word to a closed
vocabulary to express what `cover` already implies, and because it would force a rewrite of
all 8 committed elements the moment anyone deleted a `clip`. But the counter correctly
identifies that I am overloading `cover` with two readings distinguished by the presence of
another field, and that is a real cost this design pays.

---

## R2-Q4 — "the source's dimensions", normatively

**Decision: the source's dimensions are its _presentation_ dimensions — what a conforming
decoder hands the compositor after orientation metadata, clean-aperture/cropping metadata,
and pixel aspect ratio have been applied — never its coded dimensions. They are carried as
exact rationals through the whole derivation, and only the final extent floors. `validate`
must print the dimensions it used, and must mark them when they differ from the coded
dimensions.**

**Why:**

*Presentation, not coded, because the alternative disagrees with every renderer that
exists.* `render` must decode to draw, and every practical decode path honours EXIF
orientation. If `validate` reads coded dimensions and `render` draws the rotated picture,
then on a transposed source (EXIF orientation 5–8, which swaps width and height) the two
choose **different driving axes** and the cover fails on the axis that is exact by
construction. A rule whose entire purpose is that independent implementers land on the same
integer cannot have its own two tools disagree.

*One sentence, not a list, because the list is open.* Orientation and PAR are the two named
today; container clean-aperture is a third, and there will be more. Defining the input as
*"the dimensions a conforming decoder presents to the compositor"* binds all of them at
once and binds `render` to the identical number by construction, rather than enumerating
transforms that a future format extends past.

*No intermediate rounding, because that is precisely the `3368/0.645` defect.* ADR-0005's
`speed` divergence sent four agents to 5222 and one to 5220 because a real-valued
intermediate was rounded at an unspecified point. Non-square PAR reintroduces exactly that
shape: `1.0926` is a ratio, and `round(sw_coded * PAR)` is a second rounding site whose
direction nobody specified. The rule must therefore be stated so **only one rounding
operation exists in the whole pipeline** — the final floor. Concretely, with PAR
`p_n/p_d` applied to width:

- cross-multiplication (cover): width drives when `bw*sh*p_d >= bh*sw*p_n`
- slack height: `(sh * bw * p_d) // (sw * p_n)`
- slack width: `(sw * p_n * bh) // (sh * p_d)`

All integer, all exact, one floor. This is ADR-0013's own integer discipline extended to
the one input it did not have to think about because its corpus is PNG. If a container
reports PAR only as a decimal, the implementation must recover the rational (`p_n/p_d`) the
container actually stores; an irrecoverable PAR is `UNCHECKED`, not `1.0`. A default of 1.0
there is precisely the *"a default is only safe where its wrong answer is loud"* violation
ADR-0012 wrote down — a mildly-stretched photo is not loud.

*The fixture is **structurally incapable** of testing any of this, and that fact is the
argument, not an excuse.* I checked the committed media: all five sources are PNG
(1536x2720 ×4, 800x800 ×1). PNG carries no pixel aspect ratio at all, and none of the four
photos carries an `eXIf` chunk. `brand/logo-en.png` **does** carry one — the only source in
the corpus with EXIF at all — and it contains a single ExifIFD-pointer entry and **no
Orientation tag**. It is also the **800x800 square**, where any orientation value
whatsoever, including a transposing one, is inert. So a clean fixture run proves exactly
nothing here. This is the same shape as ADR-0013's own most honest paragraph: *"the fixture
escapes the bug entirely because `1080/1536 = 45/64` is dyadic... a single lucky data point
that structurally cannot demonstrate the defect it avoids."* The dyadic ratio hid the ULP
bug; PNG-only sources hide the orientation and PAR bug; a square source hides it twice
over.

**Must `validate` print the dimensions it used? Yes — and this is not a presentation
preference.** ADR-0013's note format already prints them (`images/06.png (1536x2200)`), so
the cost is zero; what I am adding is that it must be **normative** and must be
**annotated**:

```
photo-06: declared 1912; cover from images/06.png (1536x2720; coded 2720x1536, orientation 6)
          is 1912; ...
```

The reason is diagnostic and specific. When two implementations disagree about an extent,
the disagreement surfaces as a fit-deviation finding in exactly one of them. If that
finding prints only the computed extent, localising the cause needs a debugger; if it
prints the dimensions and flags that they differ from the coded ones, a human diffs two
reports and finds it in one read. ADR-0005's `speed` divergence went undetected because
nothing printed the intermediate. And it costs nothing against the noise budget — it rides
on findings that fire **zero times** on a correct file. I specifically do **not** want a
per-element dimension census on clean runs: that is 8 lines of nothing on the fixture and
is exactly the *"`0 errors, 47 notes` reads as a pass"* failure. Findings only, plus the
same fields in `--json`.

**Strongest counter:** presentation dimensions make the rule's input depend on metadata
that is **more** implementation-divergent than raw dimensions are, not less. Decoders
disagree about EXIF in practice (some ignore it in PNG's `eXIf` chunk entirely, which is
comparatively new; some honour orientation but not clean aperture; FFmpeg's rotation
handling has changed across major versions). Coded dimensions are the one number every
decoder on earth reads identically from the header. So a rule saying *"use what your
decoder presents"* may have **imported** the divergence it was written to eliminate,
defining the answer in terms of the very thing that varies. The counter is serious, and the
principled answer is that Montagent must then pin the *behaviour* rather than delegate it —
normatively enumerate the eight EXIF orientation values and their width/height transposition,
require the `eXIf` chunk to be honoured for PNG, and require clean aperture to be applied —
so that "presentation dimensions" is a Montagent definition and not a deference to whatever
library is linked. That is more specification than this ADR may want to carry. But the
alternative — coded dimensions — guarantees that validate and render disagree on any
rotated JPEG, which is a **certainty** rather than a risk, and I take a specified risk over
a certain defect.

---

## R2-Q5 — `image` only, or type-generic?

**Decision: type-generic now.** The rules bind **every element carrying a raster source**,
which today is `image` and `video`. Not scoped to `image` with a later extension.

**Why:**

*Scoping to `image` would leave `video` with a required field and no defined meaning.*
ADR-0012 made `width`/`height` **required on the element** with no type exception, so a
video element already must declare extents. If `fit`'s rules are image-only, then the day
someone writes a video element the extents are mandatory, undefined, and unchecked. That is
strictly worse than either alternative: it is not deferral, it is a hole with a required
field sitting in it.

*The arithmetic is not type-sensitive at any point.* A 1920x1080 video into a 1080x1300
aperture is the identical cross-multiplication, the identical driving-axis choice, the
identical floor. There is no image-specific term anywhere in the rule. A rule with no
type-sensitive term should not carry a type restriction; the restriction would be pure
scope-signalling and ADR-0003 names that drift by its proper name.

*ADR-0003 is dispositive on the evidence question.* The fixture's 8-of-8 images are
evidence that images must work and **"proves nothing whatever about... video clips"** —
ADR-0003 names video clips in that sentence, literally. And its consequences clause binds
the shape, not the feature: *"no primitive may be **shaped** such that a general need
becomes hard to add later."* A type-scoped `fit` would have to be re-derived for video, and
a second derivation is where two vocabularies diverge. Writing it generically costs one
word — "raster source" instead of "image" — and is what ADR-0003 asks for.

*The retirements carry over unchanged.* `gravity`'s retirement is structural (under
declared-rect-authoritative nothing leaves the crop underdetermined) and does not mention
type. `fit` required on any raster-source element covers video by the same argument.

**Wrinkles video introduces that images do not:**

1. **Non-square pixel aspect ratio.** This is the big one, and it is essentially
   video-only — no still format in common use carries PAR. It is exactly why Q4 had to be
   answered generically; an image-scoped ADR could have skipped PAR and then been
   ambiguous the day video arrived. Q4's rational-arithmetic rule handles it.
2. **Dimensions can change mid-stream.** Some containers permit a resolution change at a
   keyframe, and rotation metadata can be per-track. The fit rule's input would then be
   time-varying against a static declared rect. **Ruling: the dimensions are those of the
   first frame at or after the element's source in-point (`source_start`, in _source_ time,
   not timeline time — `speed`, `fill:hold|loop` and `shift` all make timeline time the
   wrong anchor). A mid-stream dimension change is a `note` naming the source time at which
   it changes.** A `note` and not an `error`: it renders, and the author may well intend it.
3. **Rotation metadata lives on the track, not in a still's header.** Same Q4 definition
   resolves it, but the metadata's *location* differs, so an implementation that only knows
   how to read EXIF will silently get video wrong. Worth one sentence in the schema.
4. **Probe cost.** Decoding a video header is heavier than reading `IHDR`. But ADR-0005
   already rejected *"probe before every write"* and already has `validate` read media for
   duration, so the **marginal** cost is zero — the same probe that yields duration yields
   dimensions. No new argument arises.
5. **`UNCHECKED`'s asymmetry still holds, and holds harder.** A partially-corrupt video is a
   realistic unprobeable source in a way a 5 KB PNG is not, so the `UNCHECKED` summary count
   will actually be exercised once video lands.
6. **A wrinkle images share once you look:** animated GIF/APNG/multi-frame TIFF have the
   same "which frame's dimensions" question. Ruling (2) is written to cover them, which is
   another reason not to scope by `type`.

**Strongest counter:** every word above about video is unmeasured, and this repo's own
standard is measurement. ADR-0014 said it flatly — *"State plainly what this is not: none
of it has been rendered."* There are zero video **elements** anywhere (the repo's five MP4s
are reference output and Ken Burns renders, i.e. the tool's *results*, not its inputs), so
ruling (2)'s first-frame anchor is designed against an imagined file, and rule (2) is
precisely the kind of clause that gets a detail wrong when a real container finally turns
up — PAR that changes mid-stream, a rotation that applies to display but not to coded
dimensions, an in-point that lands between keyframes. An image-scoped ADR would publish
only what its corpus supports and let video bring its own evidence, which is the discipline
ADR-0013 modelled when it scoped its normative sentence to `cover` alone. The distinction I
rely on is that ADR-0013 was declining to create a **value**, whereas type-generic wording
creates nothing — it writes the existing rule at its natural scope. But I concede the
asymmetry honestly: wrinkles **1 and 3** are forced (they are facts about containers) and
belong in the ADR; wrinkle **2** is a judgement call with no corpus behind it, and it should
be published as such, or split into its own ticket, rather than asserted at the same
confidence as the rest.

---

## Anything these five missed

**1. ADR-0013's tiebreak (2) is factually false and should be corrected, not inherited.**
It reads *"If `contain` is ever defined with the obvious semantics, only floor is safe
there."* Ceil is safe there too, for the mirror of the reason floor is safe for `cover`:
the slack exact value is `<= b_slack`, an integer, so its ceiling is also `<= b_slack`.
Measured 0 violations under both across 3,000,000 random pairs and an exhaustive edge scan,
with floor and ceil differing on 99.85% of them. ADR-0013 was scrupulous about recording
*"two arguments for floor were raised and are withdrawn"*; this is a third, and it is the
one that is *published as live*. Since #48 is the ticket that cashes the conditional, #48 is
where it must be corrected — otherwise it hardens into settled rationale. The floor decision
survives intact on tiebreaks (1) and (3); only the reason changes. Add the sweep to
`fit_rounding_scan.py` so the correction cannot rot back, exactly as ADR-0014 did with its
three author errors.

**2. `fit` needs a stated relationship to `scale`, and #48 is the last moment to state it.**
ADR-0013's escape hatch — *"an author who wants deliberate anisotropy writes the rect at the
rule value and puts the distortion in `scale`"* — is not merely inconvenient once `declared`
exists; it becomes **wrong**, because there are now two legal spellings of one picture
(`1546` + `scale:[1.0,1.236740]`, versus `declared` with `1912` + `scale:[1.0,1.0]`) that
render identically and validate differently. Two spellings of one value is the union
ADR-0012 rejected for `scale` and ADR-0013 rejected for `center-center`, arriving a third
time by composition rather than by a field. The ADR should say plainly: **static geometry
belongs in the rect (`declared` when it is not derived); `scale` is for animation and for
transform**, and withdraw ADR-0013's escape-hatch sentence now that a real escape exists.
This also finally repairs the 7-of-8 collision that blocked the promotion in the first
place.

**3. Nothing yet says whether `fit` is legal on an element with no source.** Round 1 made
`fit` required on raster-source elements. The symmetric clause is missing and every
comparable case in this format has one: ADR-0012 rules `"x"` on audio a schema error naming
the replacement, ADR-0014 rules `gravity` on text or shape a schema error naming `origin`.
So: **`fit` on a `text`, `rect` or audio element is a schema error**, and the message should
say there is no source to fit. Without it the schema has a required-here/undefined-there
field, which is the shape ADR-0007 ruled on twice (*"a field the renderer cannot honour is
worse than no field"*).

**4. The promoted error needs `UNCHECKED` wired through, and ADR-0013's clean asymmetry
breaks when the check becomes an `error`.** ADR-0013's structural defence was: *"the
error-severity check needs no media, so an unprobeable source can never suppress an error."*
Round 1 promoted the fit-deviation check from `note` to `error` — and that check **does**
need media. So as of this ticket there exists, for the first time, an `error` an unprobeable
source can suppress. That is not a reason to reverse the promotion, but it must be stated,
and the `UNCHECKED` summary count is now load-bearing rather than informational: it is the
only thing standing between a missing file and a clean-looking run. The summary line should
distinguish *"n sources unprobeable — n fit checks not run"* from a bare unchecked count.
