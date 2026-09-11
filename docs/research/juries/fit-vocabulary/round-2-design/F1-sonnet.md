# Round 2 verdict — juror F1

## R2-Q1

**Decision:** `declared`.

**Why:** The name must describe the CAUSE — "no derivation rule was applied, these
integers are mine" — not the effect, because the effect (anisotropic resampling) is
usually *absent*: an author who writes `fit:"declared"` on a rect that happens to match
the source's aspect ratio gets no distortion at all, so an effect-based name would be
false on the common case and only true on the deviant one. `declared` is also the word
this format has already spent on the exact concept: ADR-0013's own heading is *"The
declared rect is authoritative at render"*, and ADR-0012 already uses "declared, never
defaulted" for the sibling rule that `width`/`height` must be authored, not derived.
Reusing it costs nothing new to learn and is self-consistent with the rest of the
vocabulary. It also collides with nothing in CSS `object-fit` (`fill`, `contain`,
`cover`, `none`, `scale-down`), so an agent arriving from CSS habit gets no false
cognate to misapply.

Of the four given candidates: `none` is disqualified outright — CSS `object-fit:none`
means "ignore the box, show the source at its own natural size," which is a completely
different operation (no distortion, but arbitrary crop/overflow) from "these are my
final integers, resample to exactly them." An agent who knows CSS and reads `fit:"none"`
in Montaget will import the wrong mental model. `stretch` names the effect, is
frequently inaccurate (silent when aspect already matches), and isn't even the CSS term
for anisotropic fill (`fill` is). `exact` collides *internally* — ADR-0013 uses "exact
integer arithmetic" and "exact cover" pervasively for the rounding rule itself, which
is an orthogonal concept (how a *derived* number is computed) from this value (that
*no* derivation happened). Using `exact` for both risks a reader conflating "the escape
value" with "the precision the cover rule promises."

**Strongest counter:** `declared` is also, in a loose sense, true of *every* `fit`
value — the final `width`/`height` are always declared and always authoritative at
render, per the settled premise that `fit` is a derivation *claim*, not a render
instruction. So the name doesn't cleanly discriminate "I derived these and wrote down
the result" (`cover`) from "I made these up with no rule at all" (the escape value) —
both produce a declared rect. A name that leans harder into "no rule" specifically,
like `unfit` or `manual`, would discriminate the CAUSE more sharply than `declared`
does, at the cost of coining a word nothing else in the format uses yet.

## R2-Q2

**Decision:** `contain` ships in v1, with its inequality: the declared rect fits
**inside** the box — `width <= box_width` and `height <= box_height`, with equality on
the driving axis. The aperture-coverage error is re-scoped from "unconditional" to
"conditional on `fit:"cover"`" (and, if a future value also guarantees `rect ⊇ clip`,
extends to it structurally rather than by name).

**Why:** `contain`'s inequality is the mirror of `cover`'s, and its rounding is not a
tiebreak the way `cover`'s was — it is **geometrically forced**, which ADR-0013 already
says explicitly: *"direction-preservation is indifferent for `cover` and selects floor
uniquely only for `contain`."* Concretely: the driving axis is chosen by the opposite
comparator (`bw*sh <= bh*sw` picks width, else height) and takes the box dimension
verbatim; the slack axis is `(s_slack * b_driving) // s_driving`, and it must **floor**,
because `s*f <= b` is the invariant contain must preserve, and only floor guarantees
`floor(s*f) <= b` for all inputs — ceiling can push the slack axis one integer *past*
the box, which is not "contained," it's a second flavor of cover. Worked on the
fixture's one real geometry: source 1536x2720 into box 1080x1300, contain drives on
height (1300 exact) and floors width to `1536*1300 // 2720 = 734`, giving a declared
rect of 734x1300 — narrower than the 1080-wide clip, correctly pillarboxed.

That number is exactly the collision the ticket names: 734 < 1080, so a correct
`contain` element on this box would fail ADR-0013's aperture-coverage error as
currently worded ("the declared rect must contain `clip`"), because that check
implicitly assumes `cover`'s guarantee. The fix is not to weaken the check generally —
it stays a hard error for `cover` and for the escape value's *incidental* coverage — it
is to scope the predicate to the `fit` actually declared: `error` fires only when
`fit == "cover"` (the case the check was written for and the only case where failing to
cover is a document-contradicts-itself defect); under `contain`, the rect being smaller
than `clip` is the entire point of choosing that value, so exposing background inside
the aperture is not a contradiction, it is the requested letterbox. Any resulting gap
is then governed by ADR-0006's ordinary gap rules (never an `error`, `review` only if
the union of visual coverage is empty at that instant), not by the aperture check.

I ship it rather than defer because ADR-0003's asymmetry rule cuts specifically against
using "0 of 8 fixture elements use `contain`" as a reason to withhold it: letterboxing
a portrait source into a landscape box (or vice versa) is a baseline CapCut/Premiere
operation, and Montaget is committed to video clips, which make aspect mismatches
routine rather than exceptional. Deferring buys nothing here — the rounding rule and
the check re-scope are both small, principled, and fully specified now; there is no
unresolved research question the way there was for `gravity` or `skew`.

**Strongest counter:** ADR-0013 declined to touch `contain` *deliberately*, "so a second
value cannot be created by implication," and the ticket itself frames `contain` as
conditional ("if `contain` is defined"). The safer reading of that caution is: don't
let a jury manufacture a second vocabulary member merely because the shape of the rule
generalizes cleanly — wait for a real element that needs it. Shipping it now means the
aperture-coverage check ships more complex (branching on `fit`) for a value with zero
corpus evidence and zero render-tested behavior (per ADR-0014's own "none of it has
been rendered" caveat), on the strength of a hypothetical. A conservative jury could
defer `contain` to the ticket that first needs it and ship only `cover` + the escape
value now, keeping the aperture check unconditional and simple.

## R2-Q3

**Decision:** A declared `fit` with no `clip` is `UNCHECKED` — reusing ADR-0013's
existing UNCHECKED category rather than any of the other three options — with a reason
string that names the omission (e.g. "no `clip` declared") distinctly from the I/O
reason ("source unprobeable"), and the count surfaces in `validate`'s summary line the
same way.

**Why:** Ruled out first: falling back to the element's own declared rect. The premise
block already names this as an **algebraic fixed point** — it reproduces the same
number the check is supposed to verify, on every one of the 8 committed elements, so it
cannot ever fail. ADR-0006 is explicit that this shape — a check that always passes — is
worse than no check: *"an omitted field is indistinguishable from a decision not to
check,"* which is precisely the `sequence`-label failure mode ADR-0004 was written to
kill. Ruled out second: falling back to the project frame. Unlike source dimensions,
the frame size genuinely is in the document, so it's not an inert-data violation on
that axis — but it manufactures a comparison the author never asked for. An element
with no `clip` has no declared aperture at all; comparing its rect against the *whole
frame's* aspect ratio is a check invented by the tool, not one the author's `fit`
choice implies, and it will fire spurious notes/errors on elements whose rect has
nothing to do with the frame's shape (e.g. a small logo far from matching 1080x1920).
Ruled out third: a schema error requiring `clip` whenever `fit` is present. This
over-constrains a case ADR-0012 deliberately left open — `clip` is optional because not
every raster element needs an aperture narrower than its own rect (an element that
fully occupies its own declared rect, like `handle-logo` conceptually could without its
mask) — and would force boilerplate `clip` values that just restate the element's own
box, purely to satisfy the checker.

That leaves UNCHECKED, and ADR-0013 already tells the next author to reach for it
first: *"Reuse ADR-0012's frame-change-census vocabulary rather than coining a word."*
The structural fit is right even though the *cause* differs from the I/O case (missing
file vs. missing field): in both cases the disk-agreement half of `validate`'s question
is genuinely unanswerable, not failed, and ADR-0006's charter — *"`0 errors` over a file
where nothing could be probed is exactly the false-confidence failure"* — applies
identically to "nothing to check against" as to "nothing to probe." Keeping the count
in the summary line is what prevents this from silently reading as a pass.

**Strongest counter:** Collapsing two structurally different causes — "the disk
disagrees and we can't find out" vs. "the author never gave the rule an input" — into
one status bucket risks exactly the confusion ADR-0006 warns against elsewhere:
a reader who sees "3 UNCHECKED" doesn't know whether three sources are unreachable
(an environment problem, transient) or three elements are missing `clip` (an authoring
problem, permanent, fixable by editing the file). Those two problems call for different
actions and arguably deserve different codes even if they share the same severity tier
and the same summary-line visibility — the reuse-the-vocabulary instinct optimizes for
"don't coin words" over "keep root causes distinguishable," and this ADR has elsewhere
(the `note`/`review`/`error` triad) insisted that a status communicate what the reader
should *do*.

## R2-Q4

**Decision:** "The source's dimensions" means **the pixel grid a compliant decoder
presents after applying orientation/rotation metadata and normalizing to square
pixels** — for images, the dimensions after EXIF `Orientation` auto-rotation is
applied (what `ffprobe -show_streams -autorotate 1` or a browser's
`naturalWidth`/`naturalHeight` reports, not the raw stored raster); for video, the
**display dimensions** — coded width/height corrected for a non-1:1 sample/pixel
aspect ratio (PAR/SAR) and for any rotation side-data, i.e. what plays back upright and
square-pixeled, not the encoded frame buffer. `validate` **must** print the dimensions
it used, unconditionally, every time the fit-deviation check fires (already implied by
ADR-0013's own worked example — `"cover from images/06.png (1536x2200) is 1546"` — this
decision just makes that inclusion normative rather than incidental) and it must also
be legible enough to audit against a second tool — i.e. if orientation/PAR correction
was applied, the finding should say so, not just print bare integers.

**Why:** This is exactly the shape of ADR-0005's `speed` divergence — *"3368/0.645 sent
four agents to 5222 and one to 5220"* — an unstated convention that independent
compliant implementations resolve differently while each believes it did the obvious
thing. "Read the width and height field" is not one operation across tools: `ffprobe`
without `-autorotate` reports storage dimensions; most browsers and OS image viewers
auto-rotate by default; some codecs carry PAR that a naive width/height read ignores
entirely. Defining "source dimensions" as the auto-rotated, PAR-corrected *display*
grid is the option that agrees with what a human looking at the rendered frame sees,
and it agrees with `declared-rect-authoritative`'s own logic — the renderer draws what
a viewer perceives as the picture, not what bytes happen to sit first in the file — so
using the same convention for the *derivation rule*'s input keeps the rule's number
consistent with what actually gets resampled onto the declared rect. Requiring
`validate` to print the dimensions used converts an invisible convention into a
falsifiable fact per ADR-0006's own rule — *"every number inline — especially the
numbers that are not in the file."*

**Strongest counter:** Both failure modes named in the question are genuinely rare in
this corpus's practical range — professionally exported web/social imagery almost
always bakes EXIF rotation into pixels at export time (Photoshop, most CMSs, and every
image the fixture actually ships do this), and delivery video is overwhelmingly
square-pixel (PAR anomalies are an analog-video/broadcast legacy, not something a
modern MP4/H.264 pipeline typically carries). Specifying a normative orientation/PAR
convention now adds real implementation surface — a probing library must correctly
parse EXIF IFDs and video rotation side-data, which is meaningfully more work than
reading `width`/`height` off a container header — for a defect class this project may
never observe. That is a legitimate "solve it when it's measured" argument, structurally
parallel to why `contain`'s rounding was left undefined until now — except ADR-0003's
asymmetry cuts against using corpus absence as the reason to skip it, so the counter is
weaker than it would be under a channel-scoped reading of the tool.

## R2-Q5

**Decision:** Written type-generically now, in terms of "an element's raster source"
rather than `image` specifically, and it already covers `video` cleanly given Q4's
answer — with the named wrinkle being **PAR/rotation metadata** (Q4), which is a video-
skewed instance of the same "source dimensions" ambiguity images have via EXIF, not a
new axis the rule needs restructuring for.

**Why:** ADR-0013's rule is defined purely in terms of two integer pairs — "source
`sw x sh`" and "box `bw x bh`" — with no reference to anything image-specific (no
mention of frames, codecs, or stills); the settled premises already speak of "a raster
source" rather than "an image." ADR-0003's scope rule instructs against narrowing a
capability to what the fixture happens to exercise: video clips are committed to this
format, and there is no structural reason a moving-picture source needs a different
cover/contain geometry than a still one — "cover a box with a source of `sw x sh`" is
identical arithmetic whether the pixel grid is redrawn once or 30 times a second. The
one place video genuinely differs from a still is exactly where "the source's
dimensions" itself becomes ambiguous — PAR and rotation side-data are far more common
in video pipelines than EXIF is in delivered images — but Q4 already resolves that by
defining "source dimensions" as the post-correction display grid for both media types
uniformly, so no video-specific carve-out of the *fit rule itself* is needed. A
secondary, minor wrinkle worth naming: variable-resolution video streams (a codec/
container technically permitting frame-to-frame resolution changes) would make "the
source's dimensions" a per-frame quantity rather than a fixed pair — this is exotic
enough (not a shape any mainstream delivery codec exposes for standard playback) that
it doesn't warrant a rule change, only a note that the rule assumes a constant-
resolution stream, which is true of essentially all video Montaget will ever ingest.

**Strongest counter:** This is generalization from a zero-instance case — the exact
failure pattern the brief's own framing warns about in Q1, just aimed at a different
question. Every number in ADR-0013 (the 1912 for `photo-06`, the 4.466% float-vs-integer
divergence, the 28x deviation spread) was measured against 8 image elements on a
renderer that does not exist yet — per ADR-0014's own caveat, *"none of it has been
rendered."* Declaring the rule type-generic today is an unverified claim that the
integer cross-multiplication and floor/ceil arithmetic behave identically once a real
decoder hands the renderer a video frame with, say, a non-integer framerate's worth of
PAR correction rounding of its own — a rounding step this ADR has never had to compose
with. A more conservative posture: scope ADR-0013 to `image` explicitly now, and open a
dedicated ticket to re-verify (not just re-assert) the rule once a real `video` element
with fit exists, so the extension is a measurement rather than an inference from
symmetry of the prose.

## Anything these five missed

**`fit` on a non-raster element type has no named schema error yet.** ADR-0012 and
ADR-0014 both establish the pattern explicitly — `"x"` on audio, `gravity` on text or
shape — that *"a field the renderer cannot honour is worse than no field"* and each such
collision gets its own named schema-error clause pointing at the replacement. `fit` is
now a real vocabulary member with a required/optional story and an escape value, but
nothing in the settled premises or this brief says what happens if an agent writes
`fit` on a `rect`, `ellipse`, `text`, or `audio` element (plausibly reaching for it out
of CSS `object-fit` habit on *any* box-shaped thing). This should be closed the same way
`gravity` was: a schema error on shape/text/audio elements, naming that `fit` applies
only to elements carrying a raster `source`.
