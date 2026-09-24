# Round 2 verdict — juror E1 — issue #48, the `fit` vocabulary

Premises from round 1 taken as settled: `fit` is a derivation claim consumed by `validate`;
the box is `clip`'s `width`/`height`; `fit` is required on raster elements; `gravity` is
retired; the fit-deviation check is an `error` with the zero-grace/`{floor,ceil}` predicate.

Everything below was computed, not recalled. Working:
`cover(1536,2720 → 1080,1300) = (1080,1912)`; `contain(...) = (734,1300)`;
`cover/contain(800,800 → 68,68) = (68,68)`; `cover(1536,2720 → 1080,1920) = (1084,1920)`.

---

## R2-Q1. The escape value's name

**Decision: `declared`.** The closed set is exactly `{"cover", "contain", "declared"}`.
`fit:"declared"` means *these integers are authored, not derived; no box was consulted and
no disk-agreement claim is made.* Corollary that ships with it: `validate` puts the count of
`fit:"declared"` elements on the summary line, beside the `UNCHECKED` count.

**Why:**

*Cause, not effect — and this is forced, not stylistic.* Under declared-rect-authoritative
every member of this vocabulary has the **same** effect: the source is resampled to exactly
the declared `width`/`height`. `cover` and `contain` do not name effects, they name
derivations. A member named for an effect would be the only entry in a closed set that is a
different kind of thing from its siblings — and an agent reading `{cover, contain, stretch}`
would reasonably infer that `cover` and `contain` are also render instructions, which is the
exact misreading round 1 had to settle. `declared` is a derivation claim (the null one) and
sits in the set as a peer.

*An effect name is also simply false a lot of the time.* `stretch` claims anisotropy. An
author who declines derivation may write the exact cover integer, or a uniform shrink, or a
0.03% squash. On the fixture's `handle-logo` geometry (800×800 → 68×68) the rule value and
any hand value coincide with zero distortion. A name that is false on the zero-deviation case
is a name that will be argued about forever.

*Which names are already spent in this format.* `stretch` is spent twice and both are live:
ADR-0005 uses it for time (*"`shift` may stretch them safely"*, and a jury's "uniform-stretch
policy"), and ADR-0013's own fit-deviation finding text is *"23.6% anisotropic **stretch**"* —
so `fit:"stretch"` would make the one message this vocabulary exists to emit read as if it
were naming the field's value. `exact` is spent harder still, and in this precise
neighbourhood: ADR-0013's normative rule is *"computed in **exact** integer arithmetic"* and
its worked line is *"**exact** cover here is 1912.5 px"*. `fit:"exact"` would be read by half
of all readers as "use the exact, unrounded cover", i.e. the opposite of declining the rule.
Note also what is *not* on the candidate list and why: CSS's actual anisotropic member is
`fill`, which Montagent has spent twice already (a shape's paint in ADR-0014, and `hold`/`loop`
in ADR-0005). That double spend is what pushes a CSS-minded author toward `none` — which is
the trap in the next paragraph.

*The CSS `object-fit` collision, which disqualifies `none` outright.* `object-fit: none`
means **"use the source's intrinsic size"**. An agent arriving from CSS will read
`fit:"none"` as *render this image at its natural pixel dimensions*, which is precisely the
implicit natural-size default that ADR-0012 banned by name (*"the source's dimensions are not
in the document"*) and that ADR-0005 killed as implicit `fill`. This is not a mild mismatch:
it is a name whose CSS meaning is a decision this project already rejected, and whose wrong
reading is quiet — a natural-size render of a 1536×2720 photo looks plausible. `none` also
reads as absence, which fights the round-1 decision that `fit` is **required**: a required
field whose escape value is spelled "none" invites the reading that omitting it is the same
thing. No `object-fit` member is named `declared`, so a CSS-trained reader has nothing to
mis-map onto and falls back to reading the schema — the correct failure mode.

*`declared` is already this format's own word for this concept.* "the **declared** rect is
authoritative", "an element's size is **declared**, never defaulted", "the **declared**
driving axis", "a text element's width/height is a container claim... not **declared**
geometry". The vocabulary needs no new term; it needs the term the ADRs have been using for
this idea all along, promoted to a value.

*The ADR-0006 obligation the name creates, discharged.* An escape value is literally "the
disk-agreement check does not apply to this element", which is a hair from the `sequence`
label ADR-0004 killed. The difference is structural and is the whole reason `fit` had to
become required first: with `fit` required, **silence is not an opt-out** — you must type the
word `declared`, it is visible in a one-line element, and it is greppable. Silence meaning
nothing is what damned `sequence`; here silence is a schema error. The summary-line count
closes the remaining gap, on ADR-0013's own precedent for `UNCHECKED` (*"`0 errors` over a
file where nothing could be probed is exactly the false-confidence failure"*).

**Strongest counter:** `declared` is arguably a tautology, because *every* fit value carries
a declared rect — under declared-rect-authoritative the 1912 on a `cover` element is exactly
as declared and exactly as authoritative as the integer on a `declared` element. So the name
names a property the value does not uniquely have, and a careful reader could infer the
inverse: that a `cover` rect is *not* declared, i.e. that the renderer derives it — resurrecting
the render-instruction misreading from the other end. The honest reply is that the field is
`fit`, and the value answers *"fitted how?"* — "not fitted; declared" — so the tautology
dissolves in context; and that the second-best candidate on the cause axis (`manual`,
`authored`, `verbatim`) buys nothing the repo's existing vocabulary does not already supply.
But the objection is real and the schema's one-line gloss should be written to pre-empt it:
*"`declared` — the extents were authored, not derived from a box; every fit value's extents
are honoured verbatim at render."*

---

## R2-Q2. Does `contain` ship in v1?

**Decision: `contain` ships in v1.** Its inequality, its rounding, and the rescoping of the
aperture error:

- **Inequality.** `contain` is the largest rect with the source's aspect that fits *within*
  the box: `drawn_w <= bw` **and** `drawn_h <= bh`. (`cover`'s is `drawn_w >= bw` and
  `drawn_h >= bh`.)
- **Driving axis, by integer cross-multiplication, no floats.** Width drives when
  `bw*sh <= bh*sw` — the same product, the inequality reversed. Ties (`bw*sh == bh*sw`) give
  width the drive under both values, so one sentence covers the whole vocabulary; on an exact
  aspect match the tie-break is inert anyway (`800×800 → 68×68` yields `68×68` under both).
- **The driving axis takes the box dimension verbatim**; the slack axis is
  `(s_slack * b_driving) // s_driving`, **floor**, in integer arithmetic.
- **Floor is not a tiebreak here, it is forced.** ADR-0013 says so in its own text: direction-
  preservation is *indifferent* for `cover` and *"selects floor uniquely only for `contain`"*.
  Ceil would produce a rect exceeding the box on the slack axis and break the defining
  inequality. This is ADR-0013's conditional tiebreak (2) — *"floor keeps one rounding
  operation across all fit values"* — being cashed in exactly as it was written down.

**The aperture-coverage collision, resolved by scoping the error to the fit value that makes
the claim.** ADR-0013's error is not a geometric axiom; its stated rationale is *"the
consequence of failing it is background showing through **where the author declared full
cover**."* The claim is the input. So:

| `fit` | aperture check | rationale |
| --- | --- | --- |
| `cover` | **error** if the declared rect does not contain `clip` | letterbox where full cover was declared |
| `contain` | **error** if `clip` does not contain the declared rect | crop where letterbox was declared |
| `declared` | **neither fires** | no containment claim was made |

This is one predicate with its direction selected by `fit` — the same shape as the extent rule
itself, and a direct discharge of ADR-0013's standing requirement that *"any future `fit` value
must name the inequality that defines it, and its extent rounds to preserve that inequality."*
The check keeps the property ADR-0013 leaned on: **it needs no media**, so an unprobeable source
still cannot suppress it. Verified against the fixture: all 8 elements declare `cover`, and all
8 still pass unchanged (`1080×1912 ⊇ [0,0,1080,1300]`; `68×68 ⊇ [478,96,68,68]`). The live cost
the brief names is real and is exactly what the table fixes: a correct `contain` element on the
fixture's geometry is `734×1300`, which is 346 px narrower than its 1080-wide clip and trips
today's hard-coded check.

**Why ship rather than defer.** ADR-0003's asymmetry forbids reading the fixture's non-use of
`contain` as evidence against it — *"the channel is evidence that a capability is needed, never
that one is unneeded"*, and this is the exact inversion ADR-0014's method note recorded a juror
committing under a verbatim guard. Positively: the reference class is CapCut/Premiere, where
"fit" and "fill" are both first-class one-click operations, and the single most ordinary
operation in the vertical-shorts class Montagent's fixture belongs to is placing 16:9 footage in
a 9:16 frame. Computed: `contain(1920×1080 → 1080×1920) = 1080×607`; `cover` of the same gives
`3413×1920`, discarding 68% of the frame's width of picture. Without `contain` the only legal
spellings of a letterbox are `cover` (wrong rect, error under the promoted check) or `declared`
(legal, but silently surrenders the disk-agreement check on the single case where a swapped
source most obviously changes the picture). Deferring therefore does not defer the capability —
it routes the most common video-era geometry permanently into the escape hatch, which degrades
the value of the escape hatch as a signal. And ADR-0014 set the governing precedent one ADR ago
with `radius`: *"admitting it now costs one clause where admitting it later is a schema change."*
Here it costs one clause plus one direction flag.

**Strongest counter:** shipping `contain` ships a branch of the format's *only* media-free
error-severity check that **zero committed elements exercise**, into a repo whose entire
epistemic discipline is "commit the evidence an ADR rests on" and whose last two ADRs proudly
changed zero fixture bytes. ADR-0013's own corpus warning applies with more force here than
there: the fixture has two distinct geometries and one of them needs no rounding, so `contain`
would ship with a rounding rule validated against literally nothing, and a reversed aperture
predicate validated against nothing. A defensible alternative is to ship `cover` + `declared`
now, and let the first real video project produce the `contain` element that justifies its
direction. I reject it because the deferral is not free — it is a bet that no one writes a
letterboxed element before the ADR lands, and the escape value makes such an element *legal
and silently uncheckable* in the meantime, so the defect accumulates in files rather than in a
ticket. The mitigation, which should be a condition of shipping: `fit_rounding_scan.py` gains a
`contain` section — the exhaustive sweep asserting zero *over*-coverage violations (the mirror
of its current zero-undercoverage assertion), plus the `1536×2720 → 1080×1300 = 734×1300`
worked case and a synthetic `contain` element asserted against both aperture directions. An
unexercised branch with an exhaustive sweep behind it is not the same object as an unexercised
branch with a paragraph behind it.

---

## R2-Q3. What does a declared `fit` mean when `clip` is absent?

**Decision: `fit:"cover"` or `fit:"contain"` on an element with no `clip` is a schema error,
naming both replacements** — *"a derived fit needs a box: add `clip`, or write
`fit:"declared"`."* No fallback box is invented. `fit:"declared"` is legal with or without
`clip`.

**Why:**

*The own-rect fallback is disqualified by the brief's own fixed-point fact.* If the box is the
element's declared rect, the rule reproduces the declared rect on all 8 elements — and on every
element that will ever exist, because it is an algebraic identity, not a measurement. That is a
check that passes by construction. ADR-0006's charter is aimed at exactly this object: an
opt-in check that *"would replace real vigilance with false confidence"*, two of eight agents
ranking it below doing nothing. This is worse than opt-in, because the opt-out is invisible —
the element reads `fit:"cover"`, the report says no findings, and nothing anywhere discloses
that the check degenerated. It is also worse *now* than it would have been in round 1, because
the check is an `error`: a green `0 errors` over a file of self-certifying elements.

*`UNCHECKED` is the wrong label and would corrupt the one count that exists to prevent false
confidence.* ADR-0013 defines `UNCHECKED` precisely: the disk half is **unanswerable** —
missing file, permission denied, unfetchable URL — and it is a `validate`-only category because
`render` must decode the source anyway. Here the source is perfectly readable; what is missing
is the *box*, which is a document fact, not a media fact. Filing it under `UNCHECKED` puts a
document defect in a media-failure bucket and inflates the number a reader uses to judge how
much of their file was actually examined. And it would be `UNCHECKED` at render time too,
which breaks that category's defining asymmetry.

*The project frame is a real rectangle, in the document, and still wrong.* It is not a fixed
point and it keeps teeth, so it is the serious rival; the full-bleed background image with no
aperture is a genuine, common element and would be a cover-to-frame. Two things kill it.
First, it makes the identity of the box depend on the *presence of an optional field*: adding a
`clip` to an element silently changes what its `fit` claims, and can turn a passing element
into a refused render without any edit to `fit`, `width` or `height`. Second — and decisively
under the promoted severity — it is **noisy-wrong on legal files**. An author who fitted to a
1080×1600 design box and wrote no aperture now gets an `error`, not a note, computed against a
box they never named. ADR-0006 treats manufactured findings on correct files as a safety
problem, not a presentation one, and this variant manufactures render refusals. Falsification
is already on record: the frame gives `1084×1920` on the fixture's geometry against a published
`1912`.

*The repo has already ruled on this exact shape, one ADR ago.* ADR-0014 faced "is this field
optional, and what does absence mean?" and answered: **required**, on the sentence *"an omitted
`height` is indistinguishable from a decision not to check"*, with an edit census showing
required catches 22/22 and optional 7/22. Every fallback proposal here is a way of assigning
a meaning to an absence; the schema error is the option that refuses to. The cost is one field,
and in the motivating full-bleed case that field is `clip:[0,0,1080,1920]` — **render-inert**,
since the frame clips there anyway — which converts an implied box into a literal one. That is
ADR-0012's governing principle (*the inputs must be in the document*) applied to the box term,
and it is the reason this option is not merely strict but positively better: after it, the box
is never inferred, anywhere, by anyone.

**Strongest counter:** `clip` is not an inert annotation — it is a render-affecting aperture —
so "add a `clip`" is not always the harmless documentation act I have described. An author who
genuinely derived extents by covering a 1080×1600 design box, and who genuinely wants the
overflow to spill onto the background rather than be cut, cannot say so: writing the `clip`
changes the picture, and `declared` throws away the very check they were entitled to. That is a
small, real capability hole, and it is the one case where the frame fallback or a separate
inert box field would serve the author better. I accept the hole rather than fix it, because
both fixes are worse: the fallback buys this rare author a check by charging every other author
an inferred box and possible false render refusals, and a dedicated "the box I fitted to" field
would put geometry in a second place in the document — the drift ADR-0005's stale-half
objection and ADR-0012's "two homes for position" argument both refuse. Recorded as a residual,
in ADR-0014's style, rather than papered over.

---

## R2-Q4. "The source's dimensions", normatively

**Decision.** *The source's dimensions* are its **display geometry**, defined as an ordered
triple of integers `(sw, sh, par = pn/pd)` where:

1. `sw`, `sh` are the coded pixel dimensions of the **first frame of the stream**, after any
   **orientation** metadata is applied — EXIF `Orientation` 5–8 and a container display-matrix
   rotation of ±90° **transpose** `sw` and `sh`; 1–4 and 0°/180° do not. Orientation is always
   honoured, never ignored.
2. `par` is the **pixel aspect ratio** as an exact rational, defaulting to `1/1` when absent.
3. Where container-level and codec-level orientation or PAR metadata **disagree**, the
   **container's wins**, and `validate` reports both.
4. Precision is preserved by carrying `par` as a rational into the existing integer rule, never
   by pre-multiplying into a float. With display width `sw*pn/pd` and display height `sh`:
   - `cover` drives width when `bw*sh*pd >= bh*sw*pn`; `contain` when `<=`.
   - width driving → `h = (sh * bw * pd) // (sw * pn)`; height driving →
     `w = (sw * pn * bh) // (sh * pd)`.
   With `pn = pd = 1` this reduces **character for character** to ADR-0013's published rule, so
   nothing already decided is reopened and the fixture's integers are untouched.

**And yes — `validate` must print the dimensions it used, but conditionally, not always.**
The prose report prints them whenever the *ambiguity was live*: when orientation transposed the
axes, when `par != 1/1`, or when container and codec metadata disagreed — plus, as ADR-0013
already requires, inside any fit-deviation finding. Otherwise they are carried in the canonical
JSON finding set and not printed. The summary line carries a count of sources whose display
geometry differs from their coded geometry.

**Why:**

*Display geometry, because that is what the check is comparing against.* The fit rule's consumer
asks "does this declared rect agree with the picture on disk?" — and the picture on disk is the
oriented, square-pixel-corrected one, because that is what any renderer paints. Defining the
input as coded dimensions would make `validate` compute a landscape cover for a portrait photo
and report an error about a file that is correct. That is the same defect as ADR-0013's
disqualifying *"what is actually drawn is a number not in the file"*, arriving through the
probe instead of through the renderer.

*The divergence is not theoretical and it dwarfs the precedent.* Computed:

| case | reader A | reader B | divergence |
| --- | --- | --- | --- |
| ADR-0005 `speed`, `3368/0.645` | 5222 | 5220 | **0.038 %** |
| 720×480 DV, PAR 32:27 → box 1080×1300 | 2311×1300 (PAR honoured) | 1950×1300 (coded) | **18.5 %** |
| 4032×3024 JPEG, EXIF orientation 6 → box 1080×1920 | 1440×1920 (oriented) | 2560×1920 (coded) | **77.8 %** |

ADR-0005's hole sent four agents to 5222 and one to 5220 and that was enough to be cited as a
cautionary precedent in two later ADRs. This hole is three orders of magnitude wider, in a rule
whose *entire stated purpose* is that independent implementers land on the same integer, and
ADR-0013 spent a 31-million-combination sweep eliminating a **4.466 %** float discrepancy on a
rule whose input it never defined. The 0.0261 % anisotropy the floor-vs-ceil debate turned on is
noise beside a transposed axis.

*Why the rational, and why not a float.* Pre-multiplying PAR into a float display width
reintroduces exactly the ULP defect ADR-0013 measured and killed (`sw=103, bw=1920` floors to
1919). The rational form keeps every operation in integers with one extra factor on each side,
so the driving axis stays exact by construction.

*Why disclose conditionally rather than always.* ADR-0006's noise budget is a safety property
and its clean case is **one line**; printing dimensions for every raster element would add 8
lines to a clean fixture run to disclose nothing (all sources are square-pixel PNGs with no
orientation). Triggering disclosure on *the existence of the ambiguity* is scale-free, costs
zero on any file where implementations cannot diverge, and fires on exactly the files where
they can — the same discipline as ADR-0006's "report what quantization changes" rather than
"report misalignment".

**Strongest counter:** clause 3 is the weakest sentence in this verdict, and it is doing the
most work. "Container wins over codec" is a decree, not a derivation — MP4/MOV practice is
genuinely split (a `tkhd` matrix and a `pasp` atom can be authored by different tools at
different times), and FFmpeg, AVFoundation and browser decoders do not agree on all of these
cases. So the rule may pin determinism to a reading that the most-used implementation does not
produce, which would make every conforming Montagent disagree with `ffprobe` on the same file —
the worst possible outcome for a format whose authors will check their work with `ffprobe`.
An alternative worth weighing is to define the dimensions as *whatever the project's declared
reference decoder reports*, which is unambiguous by construction but outsources a normative
question to a version number. I keep the decree because a named, falsifiable rule can be
corrected by a later ADR while a deferred one cannot, and because the disclosure requirement
makes any disagreement visible the first time it bites instead of silently. But this clause
should ship flagged as the one most likely to be amended, and the first real anamorphic or
EXIF-rotated source committed to the repo should be the evidence that settles it.

---

## R2-Q5. `video` as well as `image`?

**Decision: type-generic now.** Every rule in this ADR — the required `fit`, the closed
vocabulary, the extent rule, the display-geometry definition, the direction-scoped aperture
error — applies to **every element carrying a raster source**, which today is `image` and
`video`. It is not scoped to `image` and widened later. Symmetrically, and mirroring ADR-0014's
one-clause `gravity` obligation: **`fit` on a text or shape element is a schema error**, because
they have no source — the message names `width`/`height`.

**Why:** ADR-0003's asymmetry is dispositive and this is its textbook application — *"That the
fixture uses Latin text at 9:16 with still images proves those must work. It proves nothing
whatever about Arabic, 16:9, or video clips."* Video clips are named in that sentence. Beyond
the guard, the sequencing is one-way: scoping to `image` now means video elements ship with
`fit` absent or meaningless, and the later widening is a **breaking** schema change that
retroactively makes existing legal files illegal — whereas writing it type-generically now is
additive and costs one word in the rule ("an element carrying a raster source" instead of "an
image element"). This is ADR-0012's aperture argument in its general form: adding the rule
later *changes what every existing file means*, same bytes, different pixels. The rule is
already type-blind in substance — its inputs are a source's dimensions and a box, neither of
which knows what the element's `type` is.

**Wrinkles video introduces that images do not** — all real, none blocking:

1. **Non-square pixels are a video problem.** Essentially every raster still is square-pixel;
   anamorphic and DV/HDV sources are not. The `par` rational in Q4 is therefore mostly a video
   clause, and is the reason it has to be in this ADR rather than deferred with video.
2. **Container rotation is far more common than EXIF.** Phone-shot vertical video is routinely
   a landscape-coded stream with a ±90° display matrix. Q4's clause 1 covers it; without it,
   every phone clip computes a transposed fit.
3. **Dimensions are a property of a stream, not of a frame.** Resolution can change mid-stream
   (some codecs, and adaptive sources). Normative: the dimensions are the stream's **declared**
   display geometry — clause 1's *first frame of the stream* — and explicitly **not** the first
   frame of the element's `source_range`, so that trimming an element can never change its
   `fit` verdict. A source whose frames deviate from its declared geometry is out of scope here
   and should be reported as a fact, not silently absorbed.
4. **The probe already exists and gains a field, not a cost.** ADR-0006 caches
   `(path, size, mtime) → duration`; it now caches `→ (duration, sw, sh, par, orientation)`
   from the same header read. So the type-generic rule adds **zero** I/O to `validate`, which is
   what makes "always everything, no fast mode" survive the widening.
5. **No interaction with `speed`, `fill`, or `source_range`.** `fit` is a static claim about two
   integers; time fields do not touch it. Worth one sentence in the ADR so an implementer does
   not go looking for one.
6. **`scale` never participates in the fit check**, for video or image. The rule compares the
   *declared rect* against the source; the Ken Burns ramp is downstream. Stating this matters
   more once video lands, because "the clip is zoomed" will tempt an implementer to fold the
   keyframed scale into the comparison and produce a time-varying verdict.

**Strongest counter:** no `video` element exists anywhere in this repository, so writing the
rules type-generically legislates for a type whose *other* fields — how a video's source range,
speed and fill interact with placement — are not designed yet. The precedent against is
ADR-0013's own refusal to legislate `contain` lest it *"create a schema value by implication"*;
by the same logic, saying what `fit` means on a `video` element half-creates the video element.
The reply is that the two cases are not alike: `contain` would have created a **value** that
nothing in the format required, whereas video elements are already committed by ADR-0003's
reference class, and this ADR is not defining them — it is declining to write a type test into
a rule that has no type-dependent term. Declining to narrow is not the same act as inventing.
Still, the ADR should say so explicitly, so that a later video ADR knows it inherited a rule
rather than a decision it may quietly override.

---

## Anything these five missed

**1. Promoting fit-deviation to `error` breaks the structural property ADR-0013 leaned on, and
nobody asked.** ADR-0013 wrote: *"the error-severity check needs no media, so an unprobeable
source can never suppress an error."* That held only while the **sole** error was aperture
coverage, which is document-only. The round-1 promotion creates an error whose input is the
media — so an unreadable source now suppresses an **error**, not a note, and `validate` prints
`0 errors, N unchecked` over a file that may be wrong. This is the highest-value finding I have
and it needs an explicit clause. The resolution is available and cheap, in three parts:
(a) the `UNCHECKED` summary line must name the severity at risk — *"N elements could not be
probed; a fit disagreement in them would be an error"*; (b) `validate`'s **exit code must not
be success** when the unchecked count is non-zero, since a clean exit is what a CI or an agent
loop actually reads; (c) state plainly that the real chokepoint is unchanged — `render` must
decode to draw, so at render time the check is always answerable and ADR-0006's *"the renderer
is the only true chokepoint"* still carries the weight.

**2. This is the first ADR in the chain that changes fixture bytes, and it should say so
deliberately.** ADR-0013 and ADR-0014 both advertised zero-byte fixture changes. Retiring
`gravity` deletes a field from **8 of 8** image elements in the committed file. The `migrate.py`
/ `verify.py` / regenerate-and-byte-diff ritual therefore inverts here: the expected diff is
exactly 8 lines and nothing else, and that should be asserted rather than eyeballed.

**3. `fmt`'s obligations extend one field over.** ADR-0013 forbids `fmt` rewriting a declared
extent; ADR-0014 extends it to colours on the same reasoning. The same clause is needed here and
is currently unwritten: **`fmt` may never rewrite, insert, or normalise a `fit` value** — in
particular it may not "helpfully" change `declared` to `cover` when the integers happen to
match the rule, nor insert a missing `fit`. Under declared-authoritative the value is content,
and a derivation claim the author did not make is a claim `validate` would then enforce forever.

**4. A source with no intrinsic pixel dimensions has no defined `fit`.** ADR-0014 says *"commit
an SVG or a PNG instead"*, so SVG sources are anticipated — and an SVG may carry a `viewBox`
with no pixel size at all, which means `cover` and `contain` have no computable input. One
clause: a source without intrinsic pixel dimensions requires `fit:"declared"`; a derived `fit`
on one is an error naming it. Without it, this rule has an undefined case the format has
already invited.

**5. `gravity`'s retirement needs the same one-clause error `anchor` and `center-center` got.**
Round 1 settled that it goes; what is not settled is what the schema *says* when it meets the 8
committed spellings. On the pattern of ADR-0013 (`anchor`-as-string names `origin`) and ADR-0014
(`gravity` on text/shape names `origin`), the clause is: **`gravity` on a raster element is a
schema error naming `clip` and the element's declared rect** — *"which part of the source
survives is determined by your rect and your aperture; there is nothing left for `gravity` to
say."* An error that only deletes teaches nothing.
