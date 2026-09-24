---
status: accepted
amends: 0040 (overturns its chroma-key out-of-scope entry, replaces that entry's stated reason with one that survives, and adds an eighth member to the effects vocabulary), 0049 (states that its stopping rule and its closed `tint.color` exception are scoped to *colour operations* and do not reach a matte operation; `shadow.color` is named as the precedent that was already outside them; `spill` is admitted under the rule's own clauses rather than under the exception), 0006 (adds `R-CHROMA-AFTER-COLOUR`, `R-CHROMA-ON-ALPHA-SOURCE`, `R-CHROMA-ON-AUTHORED-ELEMENT` and `N-CHROMA-INERT` to the check list), 0024 (`measure` gains a keyed-alpha coverage derivation, sampled per frame, still with no verdict)
---

# Chroma key is a matte operation, not a colour one: it is admitted, and `color` stays literal

[ADR-0040](./0040-effect-model-attachment-and-v1-vocabulary.md) put chroma key on its
*"Out of scope for a read-not-executed, agent-first format"* list, beside LUTs, with a
one-sentence reason. [#340](https://github.com/MBehtemam/Montagent/issues/340) reopened it.
This ADR overturns that entry, replaces the reason it was refused on — which is false —
with the one that is true and narrower, and admits the keyer.

Two things are **not** reopened. Effect parameters stay static: ADR-0012 keeps keyframes
transform-only, and this ADR states the consequence as a boundary rather than working
around it. And the closed-vocabulary property stays: this is an eighth named member with a
fixed parameter set, not an escape hatch.

## The evidence

`docs/research/alpha-decode/` (on `main`) measures the decode path. The keyer's own
measurements come from a real forcing case — a 1920x1080 h264 greenscreen clip, 24 fps,
140 frames — **committed at `docs/research/chroma-key/green-screen-trex.mp4`**, with
`chroma_key_scan.sh` beside it re-deriving every number below and exiting non-zero the
moment one stops reproducing. It asserts **both directions** throughout: the tolerances that
must key and the tolerances that must not, so a change making the keyer more permissive
fails as loudly as one breaking it. Runs in ~4 s.

Both courts' ballots are committed verbatim at `docs/research/chroma-key/BALLOTS.md`, since
this ADR decides against the first court's unanimous verdict and characterises both.

| | measured |
| --- | --- |
| screen colour, 5 timestamps x 5 points | **`(0, 205, 0)` at every one** |
| `chromakey` tolerance keying the subject correctly | **0.05 – 0.30, all identical** |
| tolerance keying *nothing* | 0.01 (100% opaque) |
| border-band contamination, all 140 frames, tolerance `0.10` | **min 0, max 0 px** |

The hue-spelling measurements are in *Why `color` and not a hue angle*, below.

## ADR-0040's stated reason does not survive, and should not be left standing

> **Chroma key / green-screen** — its result depends on source pixels and tolerance/spill
> parameters no document can predict; not readable from the schema alone.

Both halves fail.

*"Depends on source pixels"* is true of every `video` element the format already carries —
[ADR-0023](./0023-video-source-dimensions-par-and-rotation.md) exists because it is.

*"Not readable from the schema alone"* would equally exclude `blur{radius: 12}`. No agent
knows what a 12 px blur does to a given frame by reading the schema; it knows by calling
`frame`. **This is not a case of the tool surface having moved on**: `frame` was already in
[ADR-0011](./0011-tool-surface-reads-checks-renders.md)'s verb table and `preview` in
[ADR-0021](./0021-preview-budget-and-graceful-degradation.md), both before ADR-0040 was
written. The refusal was made *with* those verbs in hand, which makes the reason harder to
explain rather than easier — the criterion excludes `blur`, `shadow` and all four colour
scalars by the same logic, and the project shipped every one of them.

**Leaving a falsified reason in place is the live hazard, not a tidiness question.** The
same sentence would refuse the next candidate on grounds the vocabulary already
contradicts, and nobody re-reading ADR-0040 would notice.

### The reason that does survive, restated

Chroma key is the one effect whose correct parameter is a function of the *source's
lighting over time*, and ADR-0012 makes effect parameters static. For a screen that drifts
mid-take, one `tolerance` cannot be right for the whole element, and the only remedy the
format offers is cutting the clip into elements at drift boundaries — with a step in matte
quality at each cut.

That is a real, structural, falsifiable objection. It is also **narrower than a refusal**:
it says the keyer serves screens that are uniform in time, not that it serves nothing. This
ADR adopts it as a stated boundary (below), which is what ADR-0040 should have carried
instead of the unpredictability sentence.

## ADR-0049 does not govern the keying, and `spill` does not need it to

[ADR-0049](./0049-v1-colour-filter-vocabulary-four-scalar-members.md)'s stopping rule opens
*"A **colour operation** is admissible in v1 only if…"*, and its exception closes with
*"a future **colour-op** proposal does not get the same allowance by citing `tint` as
precedent."* Both clauses are scoped, in their own words, to operations that change pixel
colour. `CONTEXT.md`'s glossary agrees: it defines *Colour filter* as *"the four scalar
`effects` members that change pixel colour rather than geometry"* — a named sub-family, not
a synonym for `effects`.

**Keying computes an alpha matte.** Its output is transparency; the decision of which
pixels to keep is not a colour decision. It sits in the same category as `mask` — which
ADR-0040 admitted with no colour-rule argument anywhere near it.

The corroboration is already in the vocabulary: **`shadow{dx, dy, radius, color, opacity}`
carries a non-scalar `color` and predates ADR-0049 entirely.** ADR-0049 calls `tint.color`
*"the **sole** grandfathered exception"* and never mentions `shadow`, which is only
coherent if the rule was scoped to the colour-filter family from the start. `shadow.color`
is a *paint* rather than a *key*, so it does not prove a keyer is shadow-like; it proves
the rule was never global, which is the only thing it is cited for.

### `spill` is a colour operation, and it is admitted on the rule's own terms

**The keying is not a colour operation. `spill` is**, and this ADR says so rather than
letting the matte framing cover it. Despill suppresses screen colour reflected onto the
subject — it rewrites the RGB of pixels the matte *retains*, fully opaque ones included.
A claim that the member leaves RGB untouched would be false for any non-zero `spill`, and
that claim is not made here.

`spill` needs no exception, because it satisfies ADR-0049's stopping rule directly:
**(a)** its arity is fixed by the effect's name; **(b)** it is a bounded scalar, `0.0`–`1.0`;
**(c)** its identity value is `0`, at which the member is a pure matte and RGB is provably
untouched; **(d)** it is not reproducible by composing `tint`/`saturation`/`brightness`/
`contrast`, because those operate on every pixel while despill is conditioned on the
matte's own key distance. `tint.color`'s closed exception is not cited and is not needed.

ADR-0049's final clause asks that *"a proposal wanting an arbitrary-colour parameter needs
its own stated justification"* — unscoped, and so it reaches `color` here. The next section
is that justification, and it is a measurement rather than an argument.

## Decision

### The member: `chroma{color, tolerance, softness, spill}`

```json
{"name": "chroma", "color": "#00CD00", "tolerance": 0.10, "softness": 0.08, "spill": 0.50}
```

- **`color`** — the screen colour, `#RRGGBB`, uppercase, the spelling
  [ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md) fixed for every colour in
  the format. Not `#RRGGBBAA`: an alpha on the key colour is meaningless and would be a
  second way to say nothing.
- **`tolerance`** — `0.0`–`1.0`. Normalised distance in the chroma plane within which a
  pixel is keyed out. **Identity `0`: keys nothing.**
- **`softness`** — `0.0`–`1.0`. The width of the partial-alpha band at the edge of the key.
  **Identity `0`: a hard, binary matte.** This is the parameter that makes hair, motion blur
  and semi-transparent edges expressible at all, and an earlier draft of this ADR lost it by
  mistaking it for `spill`; a keyer without it can only produce the binary matte that the
  forcing case happened to want.
- **`spill`** — `0.0`–`1.0`, suppression of the screen colour reflected onto the subject.
  **Identity `0`.** The one colour-changing parameter, admitted above on ADR-0049's clauses.

Four parameters, fixed arity, three bounded scalars and one `#RRGGBB`, every one with a
documented identity value, and the whole member reducing to a no-op at `tolerance: 0`.

### Why `color` and not a hue angle — measured, against a real hue keyer

#340 carried a proposal to key on `hue` in degrees: a bounded scalar, no colour-typed
parameter, no jurisdictional argument needed. ADR-0049 itself suggests exactly this route
(*"a scalar redefinition (e.g. a hue-angle…)"*). It was tested against `ffmpeg`'s `hsvkey`,
a genuine angular-hue keyer, and it fails in two independent ways.

**A hue angle alone does not determine a key.** `hsvkey` takes `hue`, `sat` *and* `val` —
three coordinates — and that is not an implementation quirk: a hue names a direction in
colour space and nothing else, so the other two must come from somewhere. Left at their
identity values the target is an unsaturated, valueless colour, and on the forcing case the
result inverts catastrophically:

| `hsvkey=hue=120:similarity=…` | transparent |
| --- | --- |
| 0.05 | 0.00% |
| 0.20 | 0.28% |
| 0.50 | **10.28% — and it is the subject, not the screen** |

At `similarity=0.50` the background corner is alpha **255** and the subject's centre is
alpha **0**. The hue-only spelling keys the T-rex and keeps the green.

**Supplying the missing coordinates is the colour, restated in three fields.** With the
screen's actual saturation and value given (`hue=120:sat=1:val=0.804`), the key works — from
`similarity=0.70`, against `0.05` for the literal `#00CD00`:

| similarity | `hsvkey` h=120 s=1 v=0.804 | `chromakey` `#00CD00` |
| --- | --- | --- |
| 0.05 | 0.00% | **89.25%** |
| 0.30 | 0.00% | 90.15% |
| 0.50 | 0.00% | — |
| 0.70 | **89.48%** | — |

So the hue spelling buys no economy — three numbers instead of one — loses the `#RRGGBB`
form ADR-0014 fixed for every other colour in the format, and pushes the working tolerance
an order of magnitude away from where the literal spelling puts it. The saturation and
value it forces the author to restate are **information already sitting in the source**,
which `#00CD00` states exactly and in the format's own spelling.

An earlier draft argued this from a different measurement — `chromakey` on the
fully-saturated `#00FF00` — which was a strawman: that is a Euclidean key on a saturated
literal, not a hue-angle key, and it tested a premise about hue that hue does not have. The
conclusion was right; the reasoning was not, and it is replaced rather than patched.

### The static-parameter boundary, stated rather than discovered

**A `chroma` effect keys correctly for a screen that is uniform in time.** One `tolerance`
covers the whole element, because ADR-0012 makes effect parameters static and this ADR does
not reopen that. ADR-0040 already carries *"Animated effect parameters — v1 effects are
static"* as a boundary marker for every effect, so this asks for no exception.

On the forcing case this costs nothing — though that is close to tautological, the clip
being CGI and uniform by construction. The measurement that is *not* tautological is the
width of the plateau: 0.05 through 0.30 all key identically, six times wider than the
precision anyone would tune to, and one value produced zero border-band contamination
across all 140 frames.

Footage shot on a real cyc under real lights will drift, and there the boundary bites: the
author must cut the element at drift boundaries and tune each piece, accepting a step in
matte quality at each cut. **That is a limitation of this decision, written here so it is
found by reading rather than by shipping.** The `measure` reading below is the instrument
for finding those boundaries; without it the author would be cutting blind.

### `validate` gains three review findings and one note, none requiring a render

ADR-0006 keeps `validate` to *"is this internally legal, and does it agree with the media on
disk"*. A check that had to key frames to form an opinion would be the *"does it say what
you meant"* verb ADR-0006 refuses to be, so none of these do.

- **`R-CHROMA-AFTER-COLOUR`** — a `chroma` member listed after any of `tint`, `saturation`,
  `brightness` or `contrast` in the same `effects` list. `effects` is ordered and order is
  semantically real (ADR-0040), so a colour scalar ahead of the key changes the pixels the
  key is measured against, and the author's `color` no longer names what is in the frame.
  Structural, document-only, zero cost. Review rather than error: a deliberate pre-grade
  before keying is a real if unusual technique.
- **`R-CHROMA-ON-AUTHORED-ELEMENT`** — a `chroma` member on a `text`, `rect` or `ellipse`
  element. The format authored those pixels; keying a colour out of them is the author
  asking for a shape they could have declared. Structural, zero cost.
- **`R-CHROMA-ON-ALPHA-SOURCE`** — a `chroma` member on a source the probe reports as
  already carrying alpha (`Probe::alpha`, `media/probe.rs:91`). Keying an already-keyed
  asset is almost always a mistake, and it costs one `ffprobe` the pipeline runs anyway.
  **The #339 relationship is coverage, not trust**: `pix_fmt`-derived detection yields false
  *negatives* for VP9, so this check is correct whenever it fires and merely silent when it
  should have fired. It ships with that gap stated, and #339 closes it.
- **`N-CHROMA-INERT`** — `tolerance: 0`, the identity value, which keys nothing. A declared
  effect that does nothing is the shape
  [ADR-0052](./0052-review-check-for-inert-ease-on-held-keyframes.md) already made a finding
  for with inert ease. Note-class.

All state facts and none propose a repair, per
[ADR-0043](./0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md).

### `measure` gains a per-frame keyed-alpha coverage derivation

For an element carrying a `chroma` effect, `measure` returns the resulting alpha coverage —
opaque fraction, transparent fraction, and the partial-alpha fraction between them — **as a
bare derivation with no verdict**, the shape
[ADR-0024](./0024-measure-writes-the-fit-repair-not-the-verdict.md) fixed for this verb.

**Sampled per frame across the element's own time range**, not at a single unstated instant.
The temporal sample is the whole point: a single frame answers "did this key at all", while
the series answers "did this key *stay*", which is the drift question the static-parameter
boundary above leaves open. A coverage series that steps mid-element is exactly the signal
telling an author where to cut.

This reading is load-bearing rather than decorative. ADR-0040 refused the keyer because its
result was *"not readable"*; this is the mechanism that makes it readable. An agent that
writes `tolerance: 0.01` and asks `measure` is told `opaque 100%`, which is the whole
diagnosis, without looking at a picture.

### `render` reading

`chroma` is applied at its position in the ordered `effects` list, to the element's pixels
as they stand at that point, before compositing the element against what is beneath it.
Two `chroma` members in one list are legal and apply in order, consistent with ADR-0040's
rule that two effects of the same name are ordinary.

## Consequences

- The `effects` vocabulary goes from seven members to eight. The schema's
  `/$defs/Effect/oneOf` gains one branch; `model/effects.rs`'s single enumeration gains one
  variant.
- ADR-0040's out-of-scope list loses its chroma-key entry and keeps LUTs, whose refusal
  rests on semantics living in an external file — untouched by anything here.
- #342 is unblocked and becomes the implementation of this ADR.
- **The pre-keyed alpha path is not deprecated.** A source that arrives already keyed still
  composites, and for drifting footage it remains the better route, since the upstream tool
  could animate what this format cannot. #338 and #339 stand on their own.

## The court that opposed this, and what it got right

Three independent models (Opus 5, Haiku 4.5, Fable 5.1), given the same brief cold and
blocked from each other's ballots, **unanimously declined the in-format keyer.** This ADR
decides against all three.

- **The drift argument is theirs and it stands.** It is adopted above as the stated
  boundary and is the sentence replacing ADR-0040's falsified one. They were right about
  the mechanism and argued the hard case as if it were the whole case; the forcing case that
  would have shown them otherwise did not exist when they voted.
- **They were not briefed on the document-authority premise.** `CONTEXT.md`'s opening —
  *"A project states, by being read, what is on screen at any given moment"* — is what the
  pre-keyed path violates: the key colour and tolerance live in a shell command, the project
  references an opaque artifact, and changing the key leaves the document byte-identical.
  The omission was the brief author's, not the jurors'.
- **Two of their claims were over-read and are corrected above**: ADR-0049's stopping rule
  does not reach the keying, and `shadow.color` was always outside it.
- **One juror's alternative was measured rather than dismissed** — the `hue` scalar, which
  lost to `hsvkey` on the numbers above.

A second court of four (Opus 5, Sonnet 5, Haiku 4.5, Fable 5.1) reviewed this ADR in draft
and returned three REVISE against one ACCEPT, agreeing on the decision and rejecting the
document. Every defect they named is repaired above: the `spill`/RGB contradiction (found
independently by two jurors), a false claim that `frame` and `preview` postdated ADR-0040,
the strawman hue measurement, `softness` lost from the parameter surface, `measure`'s
unstated temporal sample, the inverted #339 dependency, and two missing findings.

Recorded because unanimous opposition from independent models is evidence, and an ADR that
overrides it owes the record a reason rather than a note that the author disagreed.

## Implementation, which this ADR does not do

[#342](https://github.com/MBehtemam/Montagent/issues/342) is the code: the `Effect` variant
in `model/effects.rs`'s single enumeration, the `/$defs/Effect/oneOf` branch, the pixel
operation in `render/canvas.rs`, the four findings, and the `measure` reading. This ADR is a
decision and a specification; nothing in the binary keys anything until #342 lands.

## What is still not measured

- **Real shot footage.** Every measurement is from CGI green, uniform by construction. The
  drift boundary is reasoned, not observed.
- **Edge quality.** `softness` is specified but untested; the forcing case produced a binary
  matte and contained no hair, motion blur or semi-transparent edges — the conditions where
  keyers are actually judged.
- **Interaction with `scale`.** A keyed element scaled up resamples a matte computed at
  source resolution. This ADR takes the position that `effects` apply in element space, per
  ADR-0084's rule for `mask`, but does not measure it.
