# Juror C2 — verdict on #48 (the `fit` vocabulary)

Evidence run before deciding: `fit_rounding_scan.py` passes (exit 0, all assertions);
all four photo sources are 1536x2720 on disk, `brand/logo-en.png` is 800x800; the 8
image elements were listed from the committed fixture directly.

## Q1

**Decision:** `fit` is a **derivation claim**, not a render instruction. At render time it
does nothing: ADR-0013's "The declared rect is authoritative at render" means the source is
resampled to exactly `width`x`height` whatever `fit` says — a renderer that consulted `fit`
to pick a sampling factor would be re-deriving a number not in the file, the exact defect
that section kills. Its actual consumers are (a) the authoring agent, at write time, as the
published procedure for computing `width`/`height`, and (b) `validate`/`render`'s
fit-deviation check, which uses it to know *which* inequality the declared rect claims to
satisfy against the media on disk. `fit` is an assertion about how the author computed the
extents — a checkable provenance stamp, structurally like the text `height` ADR-0014 kept:
frozen at authoring, armed to fire when the world (the source file) changes underneath it.

**Why:** The two readings are mutually exclusive and ADR-0013 already executed one of them.
If `fit` were a layout mode, 1912 vs 1913 would be invisible (the renderer would compute
cover itself) and the fit-deviation note could not exist. The note exists, names "declared"
vs "rule" values, and fires on a stale source — that is a checker consuming a claim. The
stale-source case is the proof of value: swap the source for a 1536x2200 export and every
number in the file still looks right; only the claim `fit:"cover"` lets any tool notice the
23.6% stretch.

**Strongest counter:** ADR-0013 leaves one render-time job standing — `render` *prints* the
fit-deviation notes, so `fit` does alter render behaviour (its output stream, not its
pixels). And calling it "only a claim" undersells that agents will read `fit:"cover"` as
CSS `object-fit` and expect the renderer to do the fitting; a field that looks like an
instruction and acts like an assertion is a standing misread. The answer to that counter is
documentation plus the check itself: the misread produces a rect the check immediately
contradicts, so it is loud, not silent.

## Q2

**Decision:** The box `bw x bh` is the **`clip` aperture's width and height**. When `clip`
is absent, the box is the **project frame**, which is what an absent `clip` already means
(the element is drawn through the whole frame). So a declared `fit` with no `clip` claims
the rect was derived by fitting the source to the frame; no third case exists.

**Why:** Computed from the fixture, not inferred. Photos: source 1536x2720, `clip`
`[0,0,1080,1300]`. Cover into the clip: `1080*2720 >= 1300*1536` so width drives, width
1080 verbatim, slack `2720*1080 // 1536 = 1912` — exactly the declared rect. Cover into
the 1080x1920 frame instead: `1920*1536 > 1080*2720` so *height* would drive, giving
1084x1920 — the declared 1080x1912 fails on the driving axis. The clip hypothesis is the
only one the numbers allow. `handle-logo` is instructive the other way: its clip
`[478,96,68,68]` is offset in frame coordinates and its declared rect *equals* the clip
size, so on that element the two hypotheses coincide — and its 800x800 source is an exact
aspect match needing no rounding. Only `photo-06`'s rect-larger-than-clip geometry
disambiguates; `handle-logo` shows the box is the clip's *size* (the offset is placement,
not fitting), and shows the exact-ratio degenerate case.

**Strongest counter:** ADR-0013 never says "clip"; it says "box", and the aperture-coverage
check ("the declared rect must contain `clip`") treats clip and rect as independent facts,
which my reading makes partially redundant — under `fit`-into-clip the rect *always*
contains a clip anchored to it. But it is not redundant: `x`/`y`/`origin` place the rect
relative to the clip freely (that freedom is what replaces `gravity`, see Q5), so coverage
can still fail. The deeper counter is that grounding the box in `clip` couples `fit`'s
meaning to an optional field, so editing `clip` silently changes what `fit` asserts — true,
and exactly what the deviation check exists to catch.

## Q3

**Decision:** **Required** on every element type that has a source to fit — image (and
video when it lands). Omission is a schema error naming the value set, including the
escape value. `gravity`-style presence on text/shape stays a schema error (no source, per
ADR-0014's clause pattern).

**Why:** ADR-0006's rule, as ADR-0014 applied it to the text box, decides this directly:
"an omitted `height` is indistinguishable from a decision not to check" — an omitted `fit`
is indistinguishable from a decision not to check the rect against the media, which spends
`validate`'s one structural defence, that it checks uniformly. The stale-source case is
the fit-deviation check's whole justification, and an optional `fit` makes that check
opt-in per element — the `sequence` label again. Unlike the text-box debate, requiring
`fit` costs the author nothing they don't have: the escape value (Q4) is the honest
spelling for "I chose these numbers myself", so required-with-escape never forces a false
claim. 8 of 8 committed elements already carry it; required breaks zero bytes.

**Strongest counter:** ADR-0012's defaults list deliberately made most fields optional
with loud-wrong defaults, and one could default `fit` to the escape value — omission then
means "no claim", which is at least honest. But a defaulted no-claim is exactly the silent
opt-out ADR-0014 rejected: an element whose author *forgot* to think about fit and one who
*declined* it must not look alike, which is the same argument that makes
paint-less shapes an error naming `fill` and `stroke` rather than an invisible render.

## Q4

**Decision:** The closed v1 set is **`cover`, `contain`, `exact`** — each published in the
schema with its inequality and rounding:

- **`cover`** — the scaled source covers the box: extent `>=` box on both axes. Driving
  axis by integer cross-multiplication (width drives when `bw*sh >= bh*sw`), box dimension
  verbatim; slack axis `(s_slack * b_driving) // s_driving`, **floor** (per ADR-0013;
  floor preserves `>=` because exact slack is already `>=` an integer bound).
- **`contain`** — the scaled source fits inside the box: extent `<=` box on both axes.
  Width drives when `bw*sh <= bh*sw`, box dimension verbatim; slack axis same integer
  floor. Here floor is not a tiebreak but **forced**: it is the only direction that
  preserves `<=`, which is exactly ADR-0013's tiebreak (2) cashing out — one rounding
  operation across the whole vocabulary.
- **`exact`** — the escape: "do not fit this; the rect is authored, not derived." Its
  defining property is stated as the *absence* of an inequality, explicitly, so it cannot
  be read as a fourth geometry: the fit-deviation check is skipped, the aperture-coverage
  error still applies. No rounding, because nothing is derived.

`contain` belongs in v1. The argument is ADR-0014's `radius` precedent plus ADR-0003:
letterboxing is unremarkable in the CapCut/Premiere reference class, "the fixture never
contains" is inadmissible as evidence against, and ADR-0013 already worked out that floor
is safe for it — admitting it now costs one clause; the vocabulary is closed either way.
The one honest cost is that a second value doubles the checker's branch surface before any
renderer exists.

Spent and misleading names for the escape, recorded: **`fill`** is spent (a shape's paint
field, ADR-0014) *and* is CSS `object-fit: fill`, which means "stretch to the box" — here
the box is the aperture, so it names the wrong rect; doubly disqualified. **`none`** is
CSS `object-fit: none` = "natural source size, cropped" — precisely the
source-dims-not-in-the-document default ADR-0012 banned, so an agent's CSS prior would
read it as the one thing this format forbids. **`scale`** is spent (the transform
channel). **`stretch`** presumes distortion, but the escape also covers undistorted
hand-chosen rects. `exact` collides with nothing in the format or in CSS and says the
true thing: the extents are exact as written.

**Strongest counter:** Against `contain` in v1 — ADR-0013 refused to create it by
implication and nothing has evidenced it since; a jury adding it is doing exactly what
0013 declined, one ticket later, and every unrendered value is speculative surface. That
is real, but 0013's refusal was about creating a value *by implication of a rounding
rule*; #48 is the ticket whose job is to create values explicitly. Against `exact` as a
name — it could be read as "exactly the rule value" (the opposite claim); `declared` or
`manual` dodge that reading but `declared` restates a property every field has under
declared-authoritative, and `manual` implies a human. The ambiguity of `exact` dies in
the schema's one-line definition; the others' problems don't.

## Q5

**Decision:** **Retire it.** `gravity` on an image becomes a schema error naming the
replacement: position the rect relative to `clip` with `x`, `y`, `origin`. The fixture
migration removes `gravity` from all 8 image elements (and `verify.py`'s assertion moves
from "gravity present" to "gravity absent"), changing zero rendered pixels because the
field is inert on 8 of 8.

**Why:** ADR-0013 measured it: under drawn-rect semantics, the declared rect, its
placement and `clip` already fully determine which part of the source survives. The
fixture demonstrates the replacement working today — `gravity:"top"` sits beside
`origin:"top-left", x:0, y:0`, rect 1080x1912 behind a top-anchored 1300-tall clip: the
geometry *is* the gravity, stated twice. A second field carrying a fact derivable from
others is ADR-0006's rejected `kind`-on-track: "a second fact that drifts from the
first." Worse, it is a field the renderer cannot honour, and ADR-0007/0012 have ruled
twice that such a field is worse than no field — an agent will set `gravity:"bottom"`,
see the top of the image, and have no error to tell it why. Giving it a new job (e.g.
letterbox placement under `contain`) fails the same way: `x`/`y`/`origin` already place a
contained rect; there is no residual degree of freedom for `gravity` to own.

The cost is real and is the strongest thing on the other side: this is the first decision
in the sequence that changes bytes of the committed fixture (ADR-0013 tiebreak (1) chose
floor *specifically* to avoid that), touches `migrate.py`, `verify.py`, and ADR-0012's
published `photo-06` element. I take the cost because the alternatives are worse: keeping
it required keeps 8 lies-in-waiting; keeping it optional-and-ignored is the silently
ignored field this format's ADRs have rejected by name three times. Removal of an inert
field is also the cheapest possible fixture edit — provably zero output change.

**Strongest counter:** ADR-0003's asymmetry: the fixture *uses* `gravity` on 8 of 8, and
the channel is evidence a capability is needed. But the capability the author reached for
— "keep the top of the photo" — is served, in the same elements, by the geometry; what
was measured inert is the *field*, not the need. The sharper counter: under a future
isotropic-sampling reopening (which ADR-0013 explicitly flags as live), the renderer
would need a sub-pixel placement rule and `gravity` is the natural home. If that decision
reopens, reintroducing `gravity` is one schema entry; carrying it inert for years against
that contingency is the worse trade.

## Q6

**Decision:** Severity **`error`**, now that `exact` provides the legal decline. The
predicate is the second recorded shape, made exact:

For an element with `fit` in `{cover, contain}`, source `sw x sh` probed on disk, box
`bw x bh` (the `clip` size, else the frame):

1. Choose the driving axis by integer cross-multiplication per the value's inequality
   (`>=` for cover, `<=` for contain).
2. **Driving axis: zero grace.** The declared extent must equal the box dimension
   verbatim, or `error`.
3. **Slack axis: membership.** With `exact_slack = s_slack * b_driving / s_driving`
   (rational, never floated), the declared extent must be in
   `{floor(exact_slack), ceil(exact_slack)}` — a singleton when the ratio divides evenly
   — or `error`. Declared `ceil` when the rule says `floor` stays a `note` (it still
   satisfies the inequality; it is one step of authoring slop, not a contradiction).

`fit:"exact"` skips this check entirely; the aperture-coverage `error` and the UNCHECKED
category are untouched. The error message names the two legal repairs: the rule rect, or
`fit:"exact"` — never `scale`, which resolves ADR-0013's blocker (the escape is one word
on the element, not distortion baked into seven Ken Burns keyframe lists).

`error` is right because with a decline spelled, a rect outside the membership set is the
document contradicting itself — its own `fit` claim against the media it names — which is
ADR-0006's charter for `error` ("guaranteed wrong": either the rect, the claim, or the
source is not what the author believes). The membership shape beats the step-count shape
for the reason ADR-0013 recorded: a step is not scale-free — the scan reproduces the same
2-step deviation as 0.105% on 1912 and 2.941% on 68, a 28x spread in one file.

**Fixture check, computed:** cover into clip from the real media on disk. Photos (7
elements): sources are 1536x2720; `1080*2720 = 2937600 >= 1300*1536 = 1996800`, width
drives, declared 1080 == 1080; slack exact = `2720*1080/1536 = 1912.5`, membership set
`{1912, 1913}`, declared 1912 — pass (and it equals floor, so not even the ceil-note
fires). `handle-logo`: 800x800 into 68x68, exact ratio, membership set `{68}`, declared
68 — pass. **The rule breaks 0 of 8 committed elements**, and the stale-source case
(1536x2200 → exact 1546.875, set `{1546, 1547}`, declared 1912, 366 off) becomes the
`error` it always deserved to be.

**Strongest counter:** Zero grace on the driving axis convicts an author who fitted into a
box one pixel different from the one `validate` infers — and the box inference (clip else
frame) is mine (Q2), not yet in any ADR; if a future ADR gives `fit` an explicit box, every
element authored under the inference risks mass error. Also `error` blocks `render`
unconditionally (no `--strict`, none wanted), so a stale source now stops a ship where
ADR-0013 let it sail with a printed note — a user re-exporting assets mid-edit hits a wall.
That wall is the point, but the counterweight is honest: the check depends on a probe, and
a wrong probe (mislabeled media) manufactures a false `error` with no override other than
editing the document. UNCHECKED-on-unprobeable keeps the worst of that bounded.

## Anything the questions missed

- **The box needs to be normative, not inferred.** Q2's answer is computed from two
  geometries, one of which is degenerate. The ADR that settles #48 must *state* "the fit
  box is `clip`'s extent, else the frame" as a decision, or the whole deviation predicate
  rests on archaeology. It should also state what `fit` means on an element whose `clip`
  is later edited: the claim silently re-targets, and only the deviation check notices.
- **`fit` on video elements.** Video has the same source-with-dimensions shape as image,
  ADR-0003 obliges it, and nothing anywhere says whether this vocabulary reaches it. The
  ticket's questions are all image-scoped by evidence, but the decision should be written
  per "element with a spatial source", or video inherits by implication — the exact move
  ADR-0013 refused.
- **Probing image dimensions joins the cache.** The deviation check as an `error` makes
  image width/height probing load-bearing; ADR-0006's `(path, size, mtime)` cache covers
  duration today. One sentence extends it; silence forks the mechanism.
- **`gravity`'s retirement needs its ADR-0012 erratum.** ADR-0012's published `photo-06`
  element — twice re-published as "correct as written" — carries `gravity`. Retiring the
  field means the third ADR in a row annotates that element; the amendment chain on one
  JSON line is becoming its own readability problem and is worth a consolidated statement
  of the current canonical element.
