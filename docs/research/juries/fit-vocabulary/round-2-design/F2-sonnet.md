# Juror F2 — round 2 verdict on #48, the `fit` vocabulary

## R2-Q1

**Decision:** `literal`.

**Why:** All four listed candidates have a real defect. `declared` collides with the
single most load-bearing word in ADR-0013 — "the declared rect is authoritative at
render" applies to *every* element regardless of `fit`; every fitted extent is also
declared once written. Naming one specific `fit` value `declared` implies the other
values (`cover`, `contain`) are somehow not, which is false and actively confusing.
`exact` collides with the ADR's own arithmetic prose ("exact cover here is 1912.5
px", "exact isotropic factor") in the opposite sense from the one intended — it
would read as "the precisely correct fit," when the escape value means the opposite:
no rule was applied at all. `none` is the dangerous one: CSS `object-fit: none`
means *draw the source at its own intrinsic size, cropped or letterboxed inside the
box* — it still leaves the box's declared dimensions in force and never stretches.
Montagent's declared-rect-authoritative render always resamples to the declared
`width`/`height` regardless of `fit`, so an agent arriving from CSS and reading
`fit:"none"` would form exactly the wrong mental model: that the renderer falls back
to 1:1 sampling. That is a false-friend collision, not a coincidental one, because
`fit` is deliberately CSS-shaped vocabulary already (`cover`/`contain` are CSS
`object-fit` values). `stretch` names the *effect*, not the *cause*, and the effect
is contingent: an author can write this escape value with numbers that happen to be
isotropic, in which case nothing stretches. A name that's sometimes false describes
a bug in the *name*.

The task's own framing (cause vs. effect) settles it: the cause is always true — no
derivation rule was applied, these are the author's integers — while the effect is
only sometimes true. `literal` says the cause, and it is not a fresh coinage: it is
this codebase's existing word for exactly this relationship. ADR-0007's own title is
"Text Runs, Literal Size, Declared Fonts" — `size` on text is "literal," meaning
stated directly rather than derived, and ADR-0012 uses "literal `[x,y,w,h]`" the
same way when retiring `box`. Reusing `literal` here extends an established sense
rather than adding a fourth word for one concept.

**Strongest counter:** `literal` is currently used only as descriptive prose
(`literal size`, `literal width/height`), never as a closed-vocabulary token inside
an enum. An agent skimming the schema might read `"fit":"literal"` as underspecified
adjective-flavored English rather than immediately parsing it as a peer of `cover`/
`contain` in a three-member closed set. That's a minor readability cost, not a
correctness collision, but it's real, and a genuinely new coinage (e.g. `manual`)
would avoid even that ambiguity at the cost of losing the cross-document
reinforcement.

## R2-Q2

**Decision:** `contain` ships in v1, defined as: driving axis chosen by integer
cross-multiplication with the **opposite** comparison from `cover` (width drives
when `bw*sh <= bh*sw`), takes the box dimension verbatim; slack axis is
`(s_slack * b_driving) // s_driving`, **floor**, in integer arithmetic — same
formula as `cover`, but floor here is *forced*, not a tiebreak: the invariant for
`contain` is `s*f <= b` (fit *inside*, never exceed), and only `floor(exact) <= b`
holds unconditionally for a non-integer exact value; `ceil(exact)` can push the
slack axis over the box and violate the very inequality that defines `contain`.
This is the one case ADR-0013 already names as where direction-preservation
"selects floor uniquely."

The aperture-coverage error is **parameterized by the declared `fit` value**, not
scoped away or dropped. Its content stays "the declared rect disagrees with what
this element's own `fit` claims about its relationship to `clip`" — under `cover`
that's `rect ⊇ clip` (unchanged, still an error); under `contain` the relationship
is legitimately inverted (`rect ⊆ clip`, or in the general placed case, no forced
containment relationship at all, since letterbox bars are the entire point of
choosing `contain`) — so the *cover*-direction aperture check simply does not run
against a `contain` element. It is not weakened to nothing: it still exists, still
fires on cover elements exactly as before (0 of 8 today), and a `contain` element
that fails to actually satisfy `s*f <= b` (a genuine `contain` bug) is still
catchable by the ordinary fit-deviation check, unaffected by this change. Under
`literal` (R2-Q1), the check does not apply at all, since no `clip`-relative
inequality is claimed.

I ship it now rather than defer, weighing ADR-0003 explicitly: the fixture's zero
uses of `contain` is not evidence against it, and the reference class this ADR sets
(CapCut, Premiere) universally exposes exactly two basic scale-to-box modes — "fill"
(cover, crop to fill, the fixture's choice for full-bleed Ken Burns photos) and
"fit"/"fit to frame" (contain, letterbox) — as the two headline options in the
crop/scale tool of essentially every consumer editor. Deferring the second half of
a two-member pair that's this basic in the reference class reads as exactly the
narrow-channel drift ADR-0003 was written to kill, applied one level down: "the
fixture only needed cover" is not an argument that `contain` is unneeded, any more
than "the fixture is 9:16 Latin text" was an argument against RTL or 16:9.

**Strongest counter:** shipping `contain` forces the aperture-coverage error to
become fit-conditional the same release it was promoted to `error` severity, adding
a branch to a check ADR-0006 wants to stay uniform ("no fast mode... no scoping of
what is checked" — though that clause is about *invocation*, not per-value
predicate branching, so this isn't a direct violation, but it is more moving parts
landing simultaneously). And unlike `cover`, `contain` has genuinely zero evidence
in the only real project file — not "one geometry counted seven times" thin, but
*zero* geometries. Deferring it costs nothing today (no element needs it) and keeps
the aperture-coverage error simple for one more release; shipping it now is buying
correctness for a case nothing in this repository has yet exercised, which is
precisely the position ADR-0014 found the original jury in on stroke evidence
("the fixture has no stroke evidence of any kind") — that ADR still shipped stroke,
but on CapCut/Premiere reference-class grounds identical to the ones I'm using here,
so the counter mostly just says "the same argument could still be wrong twice."

## R2-Q3

**Decision:** A declared `fit` that names a real derivation rule (`cover` or
`contain`) without a `clip` present is a **schema error naming `clip`**. When `fit`
is the escape value (`literal`, per R2-Q1), `clip` stays independently optional,
exactly as ADR-0012 already has it — the escape value has no box to fit into, so
there is nothing for `clip`'s absence to leave undetermined.

**Why:** The own-rect fallback is disqualified by the fixed-point property named in
the brief: using the element's own declared rect as the box makes the box and the
fitted extent the same quantity by construction, so the check can never fail —
`cover`/`contain` against your own rect is trivially satisfied for any rect. That is
exactly ADR-0006's "indistinguishable from a decision not to check," the same defect
ADR-0014 used to nearly retire the text-height requirement before catching itself:
a check that structurally cannot fire is not a lenient check, it is an absent one
wearing a check's clothes. `UNCHECKED` is disqualified on a category error: ADR-0013
defines `UNCHECKED` specifically for the disk-agreement half of validation — a
source that cannot be *probed* (missing file, permission denied, unreachable URL) —
and its whole justification is the asymmetry that render must always resolve it
because decoding is mandatory. A missing `clip` is not an unprobeable fact; it's an
absent document field, fully knowable by reading, and forcing it into `UNCHECKED`
blurs two structurally different failure classes the same way `gravity`-as-`align`
blurred two unrelated concepts under one word.

That leaves frame-fallback and schema-error as the live options, and both are
legitimate in the narrow sense that neither is tautological and neither needs a
probe (the project frame's width/height are in the document root). I pick
schema-error because inventing "no `clip` means fit against the frame" is itself
creating meaning by implication — the exact move ADR-0013 refused for `contain`'s
rounding rule and the exact move this whole ADR chain treats as disqualifying
("legislating a rounding rule for it would create a schema value by implication").
Nothing published anywhere says omitted `clip` means "the frame is the box"; that
reading has to be invented in this brief to even be considered. The schema-error
reading needs no invention: `cover` and `contain` are meaningless without a box, so
requiring the box whenever the rule that consumes it is invoked is just requiring
the rule's own input, the same discipline that makes `width`/`height` required
rather than defaulted from a source the document doesn't cite.

**Strongest counter:** This makes `clip`'s requiredness conditional on the value of
a sibling field — a cross-field constraint the schema hasn't needed at this
granularity before (it's had type-conditional field sets, per ADR-0012's "`x` on
audio is a schema error," but not value-conditional-on-a-peer-field constraints).
It also imposes real boilerplate on the extremely common case of a full-bleed
element with no separate aperture — the author must write out `clip` equal to the
full frame rect just to keep using `cover`/`contain`, where frame-fallback would
have let that case go unstated for free, and every one of the fixture's 7 photo
elements today *does* carry an explicit `clip` smaller than its own rect, so the
corpus offers no evidence either way on how common the bare-frame case will be
going forward.

## R2-Q4

**Decision:** "The source's dimensions" means the pixel grid a compliant decoder
would hand a renderer for *display* — for images, the raster dimensions **after**
applying any orientation transform carried in the file's metadata (EXIF
`Orientation` values 5–8 transpose width and height; 2, 3, 4, 6, 7 mirror/rotate
without transposing); for video, the coded frame dimensions **after** correction by
the stream's pixel/sample aspect ratio and any container-level rotation/display
matrix. In both cases: the dimensions as they would appear on screen, never the raw
encoded buffer size. And yes — `validate` must print the dimensions it used,
unconditionally, whenever a fit-deviation finding fires. This isn't a new
obligation invented for this question: ADR-0013's own worked finding already does
it — `"cover from images/06.png (1536x2200) is 1546"` names the source dimensions
inline — so this decision is naming the existing precedent as normative rather than
incidental, and extending it with an explicit note of *which* correction (if any)
was applied, e.g. "EXIF orientation 6 applied: raw 2200x1536 → displayed 1536x2200,"
so a human or a second implementation debugging a divergence sees the intermediate
value, not just the final integer.

**Why:** The rule's entire purpose, stated by the ADR itself, is that independent
implementers land on the same integer. A dimension convention that's silent about
orientation fails that purpose exactly the way `3368/0.645` did for `speed` —
except worse, because unlike a division convention that's at least discoverable by
reading the one line of code that computes it, EXIF orientation is invisible in the
number itself: two implementations can each print a clean, confident, internally
consistent integer and disagree, with nothing in either output flagging that they
disagree about which axis is which. Defining the input normatively, plus printing
which dimensions and which correction were used, converts a silent divergence into
a loud, comparable one — the same fix ADR-0005 needed and didn't get.

**Strongest counter:** Mandating EXIF-orientation-corrected dimensions requires
every implementation to correctly parse EXIF, and EXIF orientation support is
notoriously inconsistent across decoders and libraries — some strip it, some apply
it and rewrite pixels, some report it but leave pixels raw, some ignore it outright.
This risks re-manufacturing the exact cross-implementation divergence the rule
exists to prevent, just relocated one layer down: "did you round the same way" was
replaced with "did you parse the orientation flag the same way," and the latter has
a worse track record in the wild than integer floor division does. A more radical
and arguably safer answer would sidestep the ambiguity entirely by treating any
source carrying orientation metadata as a validate-time hygiene error — require
pre-normalized, orientation-baked assets, and define "the source's dimensions" as
simply the raw decoded buffer, full stop, no correction clause at all. That trades
a harder normative definition for an operational constraint on the asset pipeline,
and it's the kind of trade this project has taken before (declared-rect-authoritative
itself is exactly this move: push a hard rendering question onto the document
instead of solving it at render time).

## R2-Q5

**Decision:** The rule is written **type-generically** now — the cover/contain
arithmetic, the floor tiebreak, and the aperture-coverage error apply to any visual
element carrying a raster or frame source and a declared `width`/`height`/`clip`,
which today means `image` and, once it exists, `video`. What is explicitly **not**
settled by extension is the Q4 input-determination question for video specifically:
video's normative "source dimensions" carries a wrinkle images mostly don't —
pixel/sample aspect ratio and, especially for this project's own 9:16-short use
case, a container-level rotation/display matrix on phone-shot vertical footage,
which is a strict superset of EXIF's four-discrete-state problem (some containers
carry an arbitrary affine display matrix, not a closed enum of eight states). That
sub-question is parked, the way `gravity` was parked by ADR-0013 — handed forward
as a measured wrinkle rather than settled by silence.

**Why:** The geometric content of the rule — driving-axis selection by integer
cross-multiplication, box-dimension-verbatim, floor on the slack axis — has nothing
image-specific in its statement; it consumes an abstract `(source width, source
height) -> (box width, box height)` mapping and nothing about that mapping cares
whether the pixels came from a JPEG or a decoded video frame. Scoping the rule to
`image` and "extending later" would repeat, one level down, exactly the drift
ADR-0003 exists to kill: today's fixture happening to be image-only would become
the reason video is left unconstrained, when ADR-0003's own asymmetry says the
fixture's non-use of video-with-fit is not evidence video doesn't need it — and
ADR-0003 has already committed this project to video clips by name. A type-scoped
rule also creates a real gap for a general editor: the moment a video element ships
with `width`/`height`/`clip` (which ADR-0012's flat-transform fields already permit
today, since those fields aren't type-restricted the way text/shape fields are),
there would be no `fit` rule to govern it, which is worse than an imperfect
type-generic rule — it's an unconstrained one.

**Strongest counter:** Video's `fit` has genuinely zero evidence in the only real
project file — not "one geometry seven times" thin, *zero* — and the extra
normative surface it needs (PAR, an arbitrary rotation/display matrix rather than a
closed EXIF enum, and in principle a source whose native resolution requires an
actual video decode rather than a header read) is real, uninspected work being
signed off on the strength of an analogy to images rather than a measurement. That
is close to the shape of mistake this exact ADR chain has caught juries making
before — the false `ScaledBorderAndShadow` inference in ADR-0014, the two agents who
derived a confident, wrong, unmeasured conclusion in ADR-0006's exercise. The
disciplined move might be to publish the rule's *arithmetic* as type-generic (it
costs nothing extra to state it that way) while explicitly declining to certify
that video's *inputs* to that arithmetic are settled, which is close to what I
decided — but a stricter reading of that same discipline would say video's
*rule as a whole* should wait for a fixture element to specify against, per the
literal fixture-driven method every other ADR here has used, rather than
type-generalizing on architecture-cleanliness grounds alone.

## Anything these five missed

**The escape value's interaction with `scale`, left dangling by ADR-0013's "not
settled here."** ADR-0013 records that the only legal way to spell deliberate
anisotropy today is `rect at the rule value + scale`, and that it collides with the
Ken Burns channel on 7 of 8 elements. `literal` (R2-Q1) *is* the fix ADR-0013 was
waiting on — but none of the five questions asks the jury to say so explicitly, and
without that link this ADR risks shipping a `fit` vocabulary that still leaves the
fit-deviation severity question (the actual thing #48 was blocking, per the ticket's
own framing) unresolved in the same document. I'd add a sixth decision explicitly
closing that loop: with `literal` available, the fit-deviation check may now be
promoted from `note` to `error` using the `{floor(exact), ceil(exact)}` shape
already specified and pre-verified against 0 of 8 committed elements, because the
escape now exists and the blocking condition ADR-0013 named is satisfied. Leaving
this unstated means #48 is "settled" on paper while the concrete thing it was
blocking is still one more ticket away in practice.
