# Juror G1 — round 2 verdict, Montagent #48 (`fit` vocabulary)

Premises taken as settled from round 1: `fit` is a derivation claim consumed by `validate`;
the box is `clip`'s width/height; `fit` is required on raster-source elements; `gravity` is
retired; the deviation check is an `error` with zero grace on the driving axis and
`{floor(exact), ceil(exact)}` membership on the slack axis.

## R2-Q1

**Decision:** `declared`. The escape value is spelled `fit:"declared"`.

**Why:** Round 1 settled that `fit` is a derivation claim, not a render instruction — the
renderer resamples to the declared rect regardless of the value. So the value's meaning is
about the *cause* (which rule, if any, produced these integers), and the effect
(anisotropic resampling) is contingent: an author who declares her own integers may well
declare an isotropic rect. A name describing the effect can therefore be false on a legal
element. That eliminates `stretch` — and `stretch` additionally imports CSS/Android
"always distort to fill" semantics that this value does not promise.

Eliminating the rest by collision:

- `exact` is already spent, fatally, in this repo's own normative prose: ADR-0013 uses
  "exact" throughout to mean the *un-rounded rule value* ("exact cover here is 1912.5")
  and "exact integer arithmetic" for the derivation itself. `fit:"exact"` would read as
  "the exact rule value" — the precise opposite of declining the rule.
- `none` collides with CSS `object-fit: none`, which means "do not resize; draw at
  intrinsic size and crop". An agent arriving from CSS — the population `CONTEXT.md`'s
  collision log exists for — would read `fit:"none"` as "no resampling", which directly
  contradicts declared-rect-authoritative (the source is *always* resampled to the rect).
  This is the `align`-on-images misreading pattern again: a familiar word meaning
  something else here.
- `declared` has no CSS `object-fit` counterpart to mislead from, and it is the one word
  this repo already uses with exactly the intended meaning: "declared rect",
  "declared-rect-authoritative", `CONTEXT.md`'s "size is declared, never derived".
  `fit:"declared"` reads as the sentence it means: *these extents are declared, not
  derived; there is no rule for validate to check them against.* Cause-framing also makes
  the validator's behaviour self-explaining — no derivation claim, so the fit-deviation
  check has no predicate to evaluate and does not run.

**Strongest counter:** `declared` under-discriminates — under ADR-0012 *every* extent is
declared (typed before any renderer runs), including `cover` ones, so the word names a
property all elements share rather than what distinguishes this one. And as a pure
cause-name it hides the one thing a reader skimming the file might want flagged: this
element may be anisotropically distorted. The response is that the distortion is
contingent (a `declared` rect can be isotropic) so an effect-name would lie on some legal
elements, while `declared` is merely mildly redundant on the others — but the redundancy
charge is real.

## R2-Q2

**Decision:** `contain` ships in v1. The set is `{cover, contain, declared}`, closed.
Inequality: the drawn rect is the largest rect at source aspect satisfying
`width <= bw` **and** `height <= bh` (the mirror of cover's `>=` pair). Driving axis by
integer cross-multiplication — width drives when `bw*sh <= bh*sw` — takes the box
dimension verbatim; slack axis is `(s_slack * b_driving) // s_driving`, **floor**, which
is the direction that preserves the `<=` bound (ADR-0013 already records that only floor
is safe for contain; contain is the case where direction-preservation stops being
indifferent). Worked example: source 1536x2720 into box 1080x1300 — `bw*sh = 2937600 >
bh*sw = 1996800`, height drives, width = `1536*1300 // 2720 = 734`, giving 734x1300.

The aperture-coverage collision is resolved by **parameterising the check on the
inequality the element's own `fit` names**, not by scoping contain out of it:

- `fit:"cover"` — declared rect must contain `clip` (the existing ADR-0013 error,
  unchanged; its rationale, "background shows through where the author declared full
  cover", is specific to cover's inequality).
- `fit:"contain"` — declared rect must be contained **in** `clip`. The symmetric
  self-contradiction: contain's meaning is "the whole source is visible"; a rect
  overhanging its aperture gets cropped by it, so the document contradicts its own
  declared fit. Same severity (`error`), same structural property — computable from
  document fields alone, no probe, so an unprobeable source can never suppress it.
- `fit:"declared"` — no inequality is named, so no aperture-containment claim exists and
  the check does not apply to the element.

This is not an ad-hoc patch: ADR-0013's own rule is that any fit value "must name the
inequality that defines it", and the aperture check simply *tests the named inequality*.
One check, per-value direction, instead of a cover-only special case that calcifies.
Note the interlock is self-consistent: the round-1 deviation predicate tolerates
`ceil(exact)` on the slack axis, but a ceil'd contain that exceeds the box trips the
containment error, so the two checks jointly pin contain to floor exactly where it
matters.

On the fixture's non-use: ADR-0003's asymmetry rule makes it inadmissible as an argument
for deferral — 8 of 8 `cover` elements prove cover is needed and prove nothing about
contain. What argues *for* shipping it is that letterboxed placement is a bread-and-butter
capability of the reference class (CapCut, Premiere), and without `contain` the only legal
spelling of it is `fit:"declared"` — which forfeits the stale-source guard, the single
defect class the deviation check exists for. Deferring contain would leave every
letterboxed element permanently unguarded against ADR-0013's replaced-source failure.

**Strongest counter:** Every contain rule above ships with a corpus of zero — no fixture
element, no published integer, no regression byte to diff against — which is exactly the
evidential posture that produced the #44/`speed` divergences, and the subtle
ceil-vs-containment interlock is the kind of seam two implementations quietly disagree
on. Deferral is cheap: `declared` gives letterboxing a legal spelling today, adding a
member to a closed vocabulary later is additive, and the aperture check could be
parameterised now (future-proofing the shape) while shipping only cover's direction. The
answer to this counter is that the parameterisation *is* the expensive part and it must
be designed against a second concrete value to be designed right — but the counter is the
best argument in this round.

## R2-Q3

**Decision:** When `clip` is absent, the box falls back to the element's own declared
rect. Not UNCHECKED, not the project frame, not a schema error.

**Why:** The fixed-point property, examined rather than feared, shows this fallback
degrades the check gracefully instead of vacuously. With box = declared rect, a correct
element is a fixed point (cover of 1536x2720 into 1080x1912: `1080*2720 = 2937600 >=
1912*1536 = 2936832`, width drives, slack = `2720*1080 // 1536 = 1912` — reproduces
itself), so the check has zero false positives by construction. But it is **not**
tautological: it degenerates into the assertion *"my declared rect is within one integer
step of source aspect"*, and that assertion still catches the defect class the check
exists for. Computed on ADR-0013's committed stale-source case (source re-exported to
1536x2200, declared rect 1080x1912): `1080*2200 = 2376000 < 1912*1536 = 2936832`, height
drives, rule width = `1536*1912 // 2200 = 1334`, declared 1080 — 254 steps off the
driving-axis-adjacent value, and the deviation **error fires**. The fallback keeps the
stale-source guard alive on every clip-less element.

Against the alternatives:

- UNCHECKED is defined by ADR-0013 as *unanswerable*, reserved for unprobeable media.
  Here the source is probeable and the aspect question is answerable; filing it as
  UNCHECKED would be a check silently weakening to nothing on a whole element class —
  ADR-0006's `sequence`-label failure ("running it and reading a pass") arriving through
  a definition. Worse, removing a `clip` would silently switch its element from checked
  to unchecked with every number still looking right.
- The project frame was falsified as the box in round 1 (1084x1920 ≠ 1912), and as a
  fallback it is absurd for small elements — `handle-logo` is a 68x68 chip at x=478; no
  reading of its authoring involves covering a 1080x1920 frame.
- Schema error would make `clip` transitively required on every raster element (since
  round 1 made `fit` required), inverting ADR-0012, which shipped `clip` as the aperture
  seven photo elements need — an image drawn whole needs no aperture and should not carry
  a rect that merely restates its own extents.

One obligation attaches: the finding text must **name the box it used** ("cover into own
rect 1080x1912" vs "cover into clip 1080x1300"), so the weakened claim is visible in the
report rather than implicit.

**Strongest counter:** The fallback verifies aspect, never the box — an author who
cover-fitted into a real box and then deleted `clip` keeps a green check while the claim
being verified has silently changed from "matches the rule against the box" to "matches
source aspect", which is a milder form of the very ADR-0006 weakening I cite against
UNCHECKED. It weakens to *something* rather than nothing, and the named-box line makes
the weakening legible, but a reader who trusts the summary count will not see it.

## R2-Q4

**Decision:** The source's pixel dimensions are, normatively: **the integer sample counts
of the decoded image as presented — after applying container/metadata orientation (EXIF
orientation for images; the rotation/display matrix for video) and after codec-level
display cropping (e.g. H.264's 1920x1088 coded vs 1920x1080 display: use 1080) — and
before any pixel-aspect-ratio correction.** PAR is ignored by the rule. And yes,
`validate` **must** print the dimensions it used, in every fit finding.

**Why:** Each clause is chosen so that two conforming implementations read the same
integers:

- *Orientation applied*: every real decoder pipeline and every reference-class editor
  presents a rotated JPEG rotated; a rule computed on pre-rotation dimensions would
  disagree with what render draws whenever the flag transposes (portrait phone photos —
  a huge input class), and the transpose itself is exact — two integers swapped, no
  arithmetic, no rounding, no divergence surface. (Mirror-only orientation values do not
  change dimensions and are irrelevant to the rule.)
- *Display cropping applied*: 1088 is a storage artifact of 16-pixel macroblocks that no
  player ever shows; using coded dimensions would make the rule disagree with every
  renderer on a large class of ordinary video.
- *PAR ignored*: this is the ADR-0005 lesson applied directly. Correcting for PAR means
  multiplying a sample count by a rational and rounding — a fresh `3368/0.645`, a derived
  number with a rounding rule that no document states, in the one place whose entire
  purpose is that independent implementers land on the same integer. Sample counts are
  exact integers every demuxer agrees on; PAR metadata is exactly the kind of field
  ffprobe and other demuxers report inconsistently. An author with an anamorphic source
  who wants display-correct geometry declares the corrected extents and writes
  `fit:"declared"` — the escape exists for precisely the case where the rule's input is
  not the geometry the author means.

Printing the dimensions is not optional because it is the divergence detector: ADR-0005's
four-agents-at-5222-one-at-5220 failure was invisible precisely because no intermediate
was printed. ADR-0013's note format already carries `(1536x2200)`; this decision makes
that normative — the printed pair **is** the post-orientation, post-crop, pre-PAR pair —
so two implementations that read different dimensions from the same file disagree in one
visible line of output instead of in a silently different integer.

**Strongest counter:** "Ignore PAR" bakes a known geometric wrong into the rule for
anamorphic sources forever — a conforming author who trusts `cover` on anamorphic footage
gets a distorted picture with a clean validate, and changing the definition later changes
what committed files mean, which is unfixable by policy. And "after orientation"
moves EXIF parsing inside the determinism boundary: implementations genuinely disagree on
obscure orientation values and on whether video rotation metadata is honored, so the
definition shrinks the divergence surface rather than eliminating it. The printed-dims
mandate is the mitigation, not a cure.

## R2-Q5

**Decision:** Written type-generically now. Every rule in this ADR is normative for *any
element carrying a raster source* — `image` and `video` alike — not scoped to `image`
and extended later.

**Why:** ADR-0003 is explicit that no primitive may be shaped so a general need becomes
hard to add, and ADR-0003 commits Montagent to video clips by naming the reference class
(CapCut, Premiere — editors that *edit video*). A video frame is a raster; nothing in
the settled machinery is image-specific: the box is `clip`, the derivation is integer
arithmetic on sample counts, `fit` is a derivation claim, the escape declines it.
Scoping to `image` would create the exact debt pattern this repo keeps paying down —
a rule that reads as general, is silently narrower, and gets rediscovered as a ticket
(the #44/#48 shape). The probe cost objection is void: ADR-0006 already probes every
media file unconditionally for duration; width/height ride the same probe for free, and
the same `(path, size, mtime)` cache covers them.

The wrinkles video introduces that images do not, named:

1. **Dimension ambiguity is worse in video** — coded vs display-cropped (1088/1080),
   rotation as a container display matrix rather than EXIF, and PAR being overwhelmingly
   a video phenomenon. Q4's normative definition exists mostly *for* video; for images it
   is nearly vacuous beyond EXIF.
2. **Dimensions are per-stream, and can in principle change mid-stream.** Normative
   input: the container's declared display dimensions for the selected video stream. A
   stream whose decoded frames deviate from its declared dimensions is a render-time
   concern, not a validate divergence — validate's answer must be computable from the
   probe, or the check re-enters the decode-everything cost ADR-0006's budget forbids.
3. **The time fields do not interact** — `speed`, `fill`, `source_start`/`source_end`
   act on the time axis and are orthogonal to fit's geometry; the ADR should say so in
   one sentence so nobody derives an interaction by implication.

**Strongest counter:** Not one video element exists in any committed file, so every
video-facing clause ships exactly as untested as `contain` does — specced against a
corpus of zero — and video's dimension ambiguity (wrinkle 1) is materially thornier than
anything the 8 image elements exercised. A disciplined alternative is to write the rules
generically but mark the video-specific clauses as untested-normative until the first
video fixture lands. The reply is that ADR-0003 already rules the evidential form of this
argument inadmissible, and that scoping to `image` doesn't dodge the ambiguity — it just
schedules its rediscovery.

## Anything these five missed

1. **What the escape forfeits should be stated in the ADR, in one sentence.** Under
   `fit:"declared"` the deviation check does not run, so the stale-source defect class —
   the entire justification for the check — is deliberately unguarded on escape elements.
   A source swap under a `declared` rect silently means a different crop forever, with a
   clean validate. That is the correct trade (the escape's whole meaning is "no claim"),
   but if the ADR does not say it out loud, the first stale-source-under-escape incident
   reads as a validator bug rather than a documented price.
2. **The aperture check's applicability to `declared` is a real amendment to ADR-0013,
   not a footnote.** ADR-0013's aperture-coverage error currently applies to all image
   elements unconditionally; scoping it to the fit-named inequality (Q2) *removes* a
   check from `declared` elements. The ADR should carry the amends-0013 header for it,
   or the two documents will state different predicates for the same error code.
3. **Error messages must name replacements, per the ADR-0012 pattern:** the retired
   `gravity`'s schema error should name what replaced its job (`clip` plus the declared
   rect, or `fit:"declared"` for deliberate reframing), and an unknown `fit` value's
   error should enumerate the closed set — `cover`, `contain`, `declared` — the way the
   easing and origin vocabularies do.
4. **`fmt` and the fit strings:** trivially, `fit` values are a closed vocabulary with no
   aliases, so `fmt` never rewrites one — worth one line so nobody invents `"Cover"`
   normalisation and trips the exact-string-replace trap.
