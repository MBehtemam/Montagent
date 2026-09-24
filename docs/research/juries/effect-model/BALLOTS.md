# Effect model court — ballots

Three jurors (Claude Opus, Claude Haiku, Claude Fable), empanelled via `/court` to
resolve [#22](https://github.com/MBehtemam/Montagent/issues/22)'s four open questions.
Each received the identical question packet (background context plus Q1–Q4, reproduced
below) cold, isolated, with no visibility into the other jurors' ballots and no
visibility into the author's recommended answers. Feeds
[ADR-0040](../../adr/0040-effect-model-attachment-and-v1-vocabulary.md).

## The question packet sent to every juror

> BACKGROUND (settled, do not reopen):
> - Reference class is CapCut/Premiere; After Effects (precomps, expression language,
>   open plugin architecture) is out of scope.
> - Effects must be a CLOSED, NAMED, PARAMETERISED vocabulary published in the schema —
>   never an open plugin system. Reason: agent discoverability via reading the schema.
> - The transform model (#21/ADR-0012) already owns: x, y, origin, scale [sx,sy],
>   rotation, opacity — as flat fields on the element, animated by keyframes
>   `{"t","v","ease"}` on absolute project-clock milliseconds, restricted to transform
>   properties only.
> - Ken Burns is NOT an effect — it's scale keyframes + a static `clip` (a rectangular
>   frame-space rect, non-rotating/non-scaling). `clip` belongs to the transform model,
>   not the effect model.
> - `stroke` (text/shape outline) is NOT an effect — it's a paint field on the primitive
>   (run-addressable on text, a shape field on rect/ellipse). The boundary reached: "a
>   stroke is a second paint on the same outline, run-addressable, that never enlarges
>   the declared rect." This excludes it because effects attach to whole elements, and a
>   run lives inside one.
> - "Text background box" is struck from any effect candidate list — it's already
>   expressible as a plain `rect` element with a `radius` field.
> - The renderer is skia-safe (chosen over tiny-skia specifically because tiny-skia
>   lacks image filters/blur/GPU) — so blur, drop shadow, and colour filters are all
>   natively available; renderer capability does NOT constrain the vocabulary.
> - A discriminated-union JSON Schema (keyed on effect name, each with its own
>   parameters) is confirmed expressible and enforced (Rust host, `jsonschema` crate
>   against JSON Schema 2020-12).
> - The one real fixture project has ZERO effects of any kind to reason from (all 35 ASS
>   subtitle styles set Outline:0, Shadow:0, no inline overrides) — except one
>   undeclared, currently-inert `mask:"circle"` on a logo element (a no-op there, since
>   the source image already has alpha-transparent rounded corners). The project's own
>   standing rule: the fixture is evidence a capability is NEEDED, never evidence one is
>   UNNEEDED — its absence of effects proves nothing about what belongs in v1.
>
> **Q1 — Attachment mechanism.** How does an effect attach to an element?
>   (a) A `"effects": [...]` list field on the element — items are `{name, params}`,
>       discriminated union by name.
>   (b) An effect is itself an element that targets another element by id (needed for
>       two-element concepts like transitions).
>   (c) A property baked into the element's type.
> Note: transitions are explicitly OUT of scope for this question — they're a separate
> future ticket. This question is about element-local effects (blur, shadow, mask, skew,
> colour filter).
>
> **Q2 — v1 vocabulary.** Which of these belong in the closed v1 effect list: mask
> (shape-only closed vocabulary like circle/rect, OR also soft/alpha masks from an image
> source — pick), blur (gaussian), drop shadow, colour filter (tint/grayscale/duotone/
> etc.), skew (rejected from the transform model, so it's an effect-vocabulary entry or
> nowhere)? State which you'd include and which you'd defer, and for mask specifically
> state shape-only vs also-soft-mask.
>
> **Q3 — Text effects.** Are text effects (what's left after stroke and background-box
> are already excluded above: essentially shadow, and per-word highlighting) the same
> `effects` mechanism applied to text elements, or a distinct concept? Per-word
> highlighting needs sub-element (run) addressing and animated highlighting doesn't fit
> the transform-keyframe shape (keyframes are restricted to transform properties only) —
> should per-word highlighting be resolved in this ticket, or explicitly deferred as its
> own future ticket?
>
> **Q4 — Deliberately absent from v1.** What should be explicitly recorded as excluded
> from v1 (so the list reads as a decision, not an oversight)? Consider: per-word
> highlighting (if deferred in Q3), whichever of colour-filter/skew you deferred in Q2,
> chroma key/green-screen, LUTs, glow, vignette, 3D/perspective distortion, anything else
> you think belongs on this list or should NOT be listed as absent.

## Juror 1 — Claude Opus

### Q1 — VOTE: (a)

**Reasoning:** The settled constraint that decides this is agent discoverability by
reading the schema. With (a), an agent reading an element's schema sees `effects` as a
field of that element, and the discriminated union enumerates every legal name and its
parameters in one place — the whole vocabulary is reachable from the element it applies
to. (c) is the worst on this axis: baking blur into `image` and shadow into `text`
scatters the vocabulary across every element type, forces N×M schema duplication, and
makes "what can I do to this element?" a question with no single answer. It also makes
adding an effect a breaking change to element types rather than an additive union
member. (b) — effect-as-element-targeting-by-id — buys nothing here because the
question explicitly excludes transitions, the only concept that genuinely needs two
operands. For element-local effects, id-targeting introduces referential integrity the
validator must now police (dangling ids, cycles, an effect element whose target is on
another track with a different time range, ordering ambiguity when two effect elements
target the same id), and it splits an element's visual definition across two places in
the document, which is precisely the read-cost an agent pays every time. A list, not a
map or a single object, because order is semantically load-bearing (blur-then-shadow ≠
shadow-then-blur) and because two instances of the same effect (two shadows) are
ordinary in the reference class.

**Trade-offs:** The cost is that a future transitions ticket cannot reuse this
mechanism — it will need its own shape, probably (b)-like, and the format will then
carry two ways to express "a visual operation." I regard that as correct rather than as
debt: transitions are a genuinely different concept (they belong to the timeline,
between two clips, not to one element), and forcing element-local effects into an
id-targeting shape today to pre-pay for it would tax every blur in the document for a
feature that isn't scoped. A second cost: `effects` is a fourth place where visual state
lives, alongside flat transform fields, `clip`, and primitive paint fields — an agent
must learn which knob lives where. That boundary is already drawn by the settled
decisions (transform owns geometry of the whole element; paint fields own the
primitive's own ink; effects own filtering of the rendered result), and it is a
defensible, statable line. Finally, effects being unanimated in v1 (no keyframes
reaching into `params`) is implied by this shape and the keyframe restriction; I treat
that as correct for v1 and note it as a known future pressure point, not as a reason to
choose differently now.

### Q2 — VOTE: Include `blur` (gaussian), `shadow` (drop shadow), and `mask` — mask
SHAPE-ONLY (closed shape vocabulary, no image-source/soft/alpha masks). Defer `colour
filter` and `skew`.

**Reasoning:** Three tests decide inclusion: does it have a closed, small, obviously-
parameterised surface; is it a thing CapCut/Premiere users actually reach for
constantly; and is there evidence of pressure from the format itself.

`blur` is the canonical closed effect: one parameter (radius), unambiguous semantics,
native to skia, and universally present in the reference class. `shadow` is the same
shape (offset x/y, blur radius, colour, opacity) and is the one text-adjacent effect the
fixture's own subtitle format has a slot for — the 35 styles set `Shadow:0`, which tells
us the concept exists and is addressable in the source material even though this
project didn't use it. `mask` has the strongest evidence of any candidate: the fixture
contains an actual, undeclared `mask:"circle"` on a logo element. It is inert there, but
it is a real authoring gesture that some producer made, and it is currently
unrepresentable in the schema — that is exactly the fixture's admissible use as evidence
a capability is NEEDED. Shipping `mask` in v1 also converts that inert field from an
undeclared stray into either a valid declaration or a validation error, which is
strictly better than the status quo where the format has no opinion about it at all.

Shape-only for mask, firmly. A shape mask (circle/rect, with radius/inset parameters) is
a closed, fully-enumerable, purely-numeric vocabulary — an agent can read it and know
every legal value. An image-source soft/alpha mask is a different animal: it introduces
a second asset reference inside an effect, raises immediate unanswered questions (how is
the mask asset fitted to the element — stretched, cover, contain, in what coordinate
space? does it use luminance or alpha? what if it's a video?), and is not discoverable
in the way the closed-vocabulary rule demands, because the semantics live in a file
rather than in the schema. Every one of those questions is a small ADR of its own.
Shape-only answers the evidenced need; soft masks answer a need with no evidence and
considerable cost.

Defer `skew`: it was already rejected from the transform model, and "rejected from its
natural home, therefore put it in the other bucket" is a bad reason to ship something.
Skew has no evidence behind it, is rare in the CapCut/Premiere workflows this format
targets, and semantically it is a geometric transform — placing it in `effects` while
`rotation` and `scale` are flat transform fields puts two members of the same conceptual
family on opposite sides of the format's cleanest boundary, which is a discoverability
cost paid by every agent forever, for a feature nobody asked for. If skew is ever
genuinely needed, the right conversation is whether to reopen the transform model, not
whether to smuggle it in here.

Defer `colour filter`: not because it's hard, but because "tint/grayscale/duotone/etc."
is the exact shape of a category that never closes. Grayscale invites sepia invites
duotone invites tritone invites brightness/contrast/saturation/temperature/tint invites
curves invites LUTs. A closed vocabulary that admits one open-ended family has quietly
stopped being closed. If v1 wants any colour operation it should be a single, tightly-
defined one with a named parameter set and an explicit ADR about where the family stops
— and that is a ticket, not a line item. v1 with blur, shadow, and shape mask is a
coherent, defensible, fully-closed three.

**Trade-offs:** The obvious cost is that v1 ships no colour operation at all, and colour
grading is table-stakes in the reference class — users will hit that wall. I accept it:
a three-effect v1 that is genuinely closed is more valuable than a five-effect v1 whose
fifth member is a door left ajar, and additive union members are cheap to add later,
whereas removing a shipped effect name is a breaking change. Shape-only mask means any
genuine feathered-edge or image-matte need is unmet, and there is a real risk someone
forces a soft-mask result by stacking a shape mask under a blur, which will not do what
they want — worth an explicit note. And deferring skew means the transform model's
rejection of it leaves a hole that stays a hole; if that rejection turns out to have
been wrong, the fix is a transform-model revision, which is the more honest correction
anyway.

### Q3 — VOTE: Same `effects` mechanism on text elements for shadow (element-scoped, no
new concept). Per-word highlighting is EXPLICITLY DEFERRED to its own ticket — do not
resolve it here.

**Reasoning:** Once `stroke` and the background box are excluded, what remains splits
cleanly along a line the format has already drawn. Shadow on text attaches to the whole
element and is expressible with the identical `{name, params}` entry used on an image or
a shape — same parameters, same semantics, same renderer path. Inventing a text-specific
shadow concept would duplicate the vocabulary for no gain and violate the
discoverability principle: an agent that learns `shadow` once should be able to apply it
anywhere, and the schema should say so by using one union member, not two. So text
effects are not a distinct concept; text is just an element.

Per-word highlighting is a different question wearing the same costume, and it fails on
two independent axes at once. First, addressing: `effects` attaches to elements, and a
word is a run inside an element. This is precisely the boundary that already excluded
`stroke` from the effect model — "effects attach to whole elements, and a run lives
inside one." Applying that reasoning consistently, per-word highlighting cannot be an
effect. Reversing the stroke decision to let it in would mean effects sometimes address
runs and sometimes elements, which is a much larger change than the feature warrants and
would make every effect's scope ambiguous from the schema alone. Second, time:
highlighting that follows spoken words is inherently animated, and keyframes are
restricted to transform properties on absolute project-clock milliseconds. A karaoke
highlight is not a transform — it's a per-run paint change scheduled against word
timings. Expressing it needs either a widened keyframe model or a run-level timing
construct, and both are format-level decisions with consequences well beyond effects.

Those two problems together mean per-word highlighting is not a member of the effect
vocabulary that happens to be hard — it is a separate feature that needs a
run-addressing model and a non-transform timing model, neither of which exists.
Resolving it inside an effects ticket would force both of those decisions as side
effects of a smaller question, decided by people reasoning about blur radii. That is how
formats acquire their worst joints. Defer it, name it, and let it have a ticket where
run addressing and word timing are the subject rather than the collateral.

**Trade-offs:** The cost is real and should be stated plainly: per-word highlighting is
arguably the single most characteristic feature of the short-form video the reference
class serves, and a v1 without it will feel conspicuously incomplete to anyone coming
from CapCut. Deferring it also means the eventual solution cannot assume it will slot
into `effects`, so there is some chance the future design would have preferred a
different effects shape — I judge that chance low, because whatever shape run-addressed
animated paint takes, it will need constructs `effects` does not have regardless of how
`effects` is designed today. The counter-argument I considered and rejected: define a
minimal static per-word highlight now (no animation) to at least reserve the name. That
is worse than deferring, because a static highlight that can't follow the audio serves
almost no real use case while permanently committing the name and an addressing
precedent to a design that was never thought through.

### Q4 — VOTE: Record as explicitly absent from v1: per-word / karaoke highlighting;
colour filters (tint/grayscale/duotone and family); skew; chroma key / green-screen;
LUTs; glow; vignette; 3D / perspective distortion; soft / image-source / alpha masks;
particle and generator effects; audio effects; animated effect parameters. Do NOT list
transitions among these — they are already out of scope by a separate ticket, not an
effects-vocabulary exclusion.

**Reasoning:** The list's job is to prove the vocabulary is closed by decision rather
than by accident, so it should cover every candidate a reader would plausibly expect to
find and say why it isn't there. Grouped by the reason each is absent:

*Deferred with a known successor ticket* — per-word highlighting (needs run addressing
plus non-transform timing, per Q3); colour filters (a family that doesn't close without
its own bounding decision, per Q2); soft/image-source masks (need mask-asset fitting,
coordinate space, and luminance-vs-alpha semantics defined first). These are "not yet,"
and the record should say what each is waiting on.

*Rejected on model grounds, not deferred* — skew, because it is a geometric transform
and was already rejected from the transform model; its absence from `effects` is the
consistent continuation of that decision, not a second, softer rejection. 3D /
perspective distortion belongs with it: the format's spatial model is explicitly 2D
(flat x, y, scale, rotation, a non-rotating rectangular `clip`), so perspective isn't a
missing effect, it's outside the coordinate model entirely. Saying this in the record
prevents someone later reading the absence as an oversight and bolting on a z-rotation.

*Out of scope for a read-not-executed, agent-first format* — chroma key and LUTs. Both
are effects whose behaviour is not readable from the document: a key's result depends on
the source pixels and a pile of tolerance/spill parameters nobody can predict from the
schema, and a LUT's entire semantics live in an external `.cube` file, which is a plugin
in all but name and squarely against the closed-vocabulary rule. Glow and vignette are
the cheaper case — both are compositions of things that exist or nearly exist (glow ≈
blur plus additive blend; vignette ≈ a radial darkening), so they are absent as "not yet
justified, and possibly never as primitives," which is a different and weaker reason
than chroma key's.

*Boundary markers worth stating even though nobody proposed them* — particle/generator
effects (an effect must filter an existing element, never author new content; this is
the line that keeps the model from becoming a rendering language) and audio effects (the
effect model is visual; audio operations, if they come, are their own vocabulary on
their own elements). Both are cheap sentences that foreclose a whole class of future
misreading.

*The one that is easiest to forget* — animated effect parameters. v1 effects are static,
because keyframes are restricted to transform properties. This is the most likely thing
for a future reader to assume is supported and quietly wrong about, so the absence of
animated effect params deserves to be written down as an explicit v1 boundary rather
than inferred from the keyframe rule.

Two things I'd keep OFF the absent-list: transitions, because listing them here implies
they were considered as effect-vocabulary candidates and rejected, when in fact they're
a scoped-out separate concept and the framing already says so — listing them muddies the
record. And stroke and the text background box, for the same reason: both were resolved
as belonging elsewhere in the format (a paint field, a `rect` element), so recording
them as "absent from v1" would be actively misleading, since both are expressible in v1.
The absent-list must mean "you cannot do this in v1," not "this isn't an effect."

**Trade-offs:** A long exclusion list carries two risks. It reads as a roadmap and
invites "when is glow coming?" pressure on items that may never come — mitigated by
grouping the list by reason, so "outside the coordinate model" and "not yet justified"
are visibly different commitments. And it costs maintenance: every future effects ticket
must also update this list or it silently rots into a lie. I judge both costs worth
paying, because the alternative — a five-line effect list with no stated boundary —
leaves every future contributor to re-derive the closed-vocabulary rationale from
scratch, and some of them will get it wrong in the direction of "surely one LUT is
fine."

## Juror 2 — Claude Haiku

### Q1 — VOTE: (a) `"effects": [...]` list field on the element

**Reasoning:** An `effects` list as an element property is the natural extension of the
declarative model. Each effect is a discriminated-union object `{name, params}` attached
to its target element. This keeps concerns aligned: all element mutations (including
effects) live as properties on the element itself, not as external pointers. The schema
is straightforward to express and validate (discriminated union in JSON Schema 2020-12).
Composition is trivial — multiple effects layer in order. Aligns with the existing
pattern: transforms are properties on elements, not separate entities.

**Trade-offs:** Option (b) is overengineered for element-local effects (transitions are
out of scope); it introduces a separate entity type, indirection, and schema complexity.
Option (c) embeds effects into the type system, which calcifies the schema and makes
adding new effects harder. Option (a) defers multi-element effects (transitions) to a
future ticket where option (b) can be revisited if needed then.

### Q2 — VOTE: Include mask (shape-only), blur, drop shadow, colour filter. Defer skew
and alpha masks.

**Reasoning:**
- **Mask (shape-only):** Essential for composition (circular logos, corner rounding,
  framing). Shape-only (circle, rect) is purely parametric — no resource references, no
  complexity. Defer alpha/image-based masks to v2 (they introduce resource binding and
  blending logic).
- **Blur (Gaussian):** Fundamental, skia-native, tiny parameter surface (radius). Belongs
  in v1.
- **Drop shadow:** Universal effect, skia-native, low parameter complexity (offset,
  radius, color, opacity). Belongs in v1.
- **Colour filter (tint/grayscale/duotone):** Covers common tone/mood adjustments,
  skia-native, simple parametrization. Belongs in v1.
- **Skew:** Was deliberately rejected from the transform model (where scale/rotation
  live). If it didn't make the fundamental transform layer, it's not urgent for effects.
  Defer to v2.

**Trade-offs:** Excluding skew and alpha masks makes v1 lean and proven. Skew is
specialist; alpha masks require image resource resolution. Both can be added later
without schema breakage. Colour filter should start narrow (grayscale, tint, basic
adjustments) and expand later if needed.

### Q3 — VOTE: Text shadow uses the same `effects` mechanism. Per-word highlighting
deferred as a separate future ticket.

**Reasoning:** Text shadow is element-level (like shadow on any other element) and
belongs in the same `effects` list — no special case needed. Per-word highlighting is
fundamentally different: it requires run-level (sub-element) addressing, keyed animation
(which breaks the transform-only keyframe restriction), and a separate run-property or
inline-style system. It's a distinct feature that needs its own design (how runs declare
styling, how highlighting animates, whether it's a property or a decoration layer).
Trying to shoehorn it into the element-effects mechanism would complicate that design and
leave per-word highlighting unfinished. Defer it explicitly.

**Trade-offs:** Deferring per-word highlighting means v1 text effects are limited to
element-level shadow. That's acceptable — it covers 80% of simple use cases. The
deferred feature can then be designed cleanly without constraining the effects
vocabulary.

### Q4 — VOTE: Explicitly exclude: per-word highlighting, skew, alpha/soft masks from
image sources, chroma key, LUTs, 3D/perspective distortion, glow, vignette.

**Reasoning:** Record these as deliberate exclusions so the v1 vocabulary reads as a
closed decision, not an oversight. Per-word highlighting and skew are deferred (per
Q2–Q3). Alpha masks need image resource binding (shape-only included instead). Chroma
key is specialist (niche use case, complex alpha logic). LUTs are advanced color grading
(out of scope for v1's parametric closed vocabulary). 3D/perspective is out of scope
(belongs to a camera system, if one exists; currently undesigned). Glow can be
approximated by layering shadow+blur later; not essential. Vignette is decorative and
rare; deferrable.

**Trade-offs:** An explicit exclusion list makes the schema's boundaries clear and
prevents future confusion ("why isn't chroma key in v1?"). It also signals to users what's
designed out, not missing. The list can be revisited in v2 without breaking v1 (new
effects are backwards-compatible additions).

## Juror 3 — Claude Fable

### Q1 — VOTE: (a) — `"effects": [...]` list on the element, items a name-keyed
discriminated union, order significant

**Reasoning:** The element already owns its own transform as flat fields; an effect is a
per-element post-process on that element's rendered pixels, so it lives on the element
too. A list (not a map) is the right container because effect order is semantically
real — blur-then-shadow and shadow-then-blur produce different frames, and an agent
reading the file must be able to read the order rather than infer a hidden fixed
pipeline. A name-keyed discriminated union gives the schema exactly the discoverability
the closed-vocabulary rule is for: an agent reads `oneOf` and sees every legal effect
with its full parameter surface. (b) is only motivated by two-element concepts, and
transitions are ruled out of scope here; adopting an id-targeting mechanism for
single-element effects would pay the cost of indirection (dangling ids, a validate check
for target existence, a second place to look when reading an element) for no benefit in
this ticket. (c) collapses as soon as two effects need to coexist and it smears effect
fields across every element type's schema, which is the opposite of a closed vocabulary.

**Trade-offs:** (a) does not scale to transitions, so the future transition ticket will
need its own mechanism (likely a timeline/track-level object referencing two element
ids). That is the right split — a transition is not an element-local concept — and
choosing (a) now does not foreclose it. Cost: the list allows nonsense duplicates (two
`blur` entries); accept it, since order-sensitive stacking is legitimate and a duplicate
is a compare/validate concern, not a schema one.

### Q2 — VOTE: Include `mask` (shape-only: `rect`, `rounded_rect`, `ellipse`), `blur`
(gaussian), `shadow` (drop shadow), `color` (a single named filter with a small closed
list: `grayscale`, `sepia`, `tint{color, amount}`, `brightness`, `contrast`,
`saturation`). Defer soft/alpha image masks and `skew`.

**Reasoning:** Mask, blur and shadow are the three things every CapCut/Premiere user
reaches for on an overlay, and the fixture already contains a `mask:"circle"` an agent
wrote unprompted — that is exactly the "evidence a capability is needed" the standing
rule admits. Shape-only mask is the right v1 line: it stays inside the closed-vocabulary
rule, its parameters are pure numbers, and it needs no second asset reference. A
soft/alpha mask from an image source is a different kind of thing — it introduces a
second media dependency into an effect, raises resolution/alignment questions (mask
pixels vs. element pixels vs. frame), and its parameter shape is not a number but a
path; that belongs in a later ticket once the asset-reference model is settled for it.
Colour filter goes in because it is cheap on skia-safe, agents use it constantly for
"make the b-roll grayscale/tinted under the caption", and a closed short list keeps it
discoverable. Skew is deferred: the transform model rejected it, and promoting a
rejected transform into an effect just to give it a home is not a use case, it is a
leftover. It is also not a pixel post-process — it is a geometric matrix — so it would
be the one effect that changes an element's bounds, which complicates every other
effect's bounds reasoning.

**Trade-offs:** Deferring skew means an agent cannot do a "leaning italic card" in v1;
that is a rare CapCut request and worth the cleaner invariant "effects never change the
element's declared geometry". Including a colour filter adds four to six parameter
shapes to the schema; keep each to one or two numeric fields to bound that. Shape-only
mask means no gradient/feathered edges in v1 except via `feather` on the shape mask,
which I'd allow as a single optional number — it is not an alpha image, it is a blur of
the shape's coverage.

### Q3 — VOTE: Text shadow is the same `effects` mechanism (a text element takes
`shadow` like any element). Per-word highlighting is a distinct concept and is
explicitly deferred to its own ticket.

**Reasoning:** A drop shadow on a text element is a whole-element pixel post-process —
it does not care about runs, glyphs or words — so it is precisely what `effects` is for,
and inventing a text-only shadow field would violate the one-mechanism rule and split
the vocabulary. Per-word highlighting is different on both axes the question names: it
addresses a run, not the element, and its interesting form is animated (the "karaoke"
highlight moving word by word with the audio), which the keyframe model — transform
properties only, absolute clock ms — cannot express without either extending keyframes
beyond transform or introducing a timed-run concept. Both are real design decisions with
their own trade space (is a highlighted word a paint change on a run with its own time
window? is it a sequence of run-level styles keyed to word timings from the caption
source?). Deciding that here, inside an effects-vocabulary ticket, would settle the
keyframe scope and the run model as a side effect, which is exactly how those models get
an accidental exception baked in.

**Trade-offs:** Deferring leaves v1 captions without the most popular short-form caption
look (word-by-word highlight). That is a real gap, but the fixture's 35 ASS styles have
zero inline overrides, so there is no concrete case to design against yet, and a static
"highlight this run" is already expressible today as a run-addressable paint (the same
mechanism stroke lives on) — only the animated form is missing. The cost of resolving it
here would be either an ad-hoc `highlight` effect with per-word timing parameters (an
effect that secretly contains a timeline) or a keyframe extension for non-transform
properties; both are worse than a dedicated ticket.

### Q4 — VOTE: Record as deliberately absent from v1: per-word / karaoke highlighting;
skew; soft/alpha image masks; chroma key; LUTs; glow; vignette; 3D / perspective
distort; effect-parameter animation (keyframed blur radius etc.); transitions
(cross-ref the separate ticket). Do NOT list: text background box, stroke, Ken Burns,
`clip`.

**Reasoning:** The absent list should hold things a CapCut/Premiere user would plausibly
look for and not find, each with a one-line reason so the reader sees a decision. Per-
word highlighting and skew are the Q2/Q3 deferrals. Soft/alpha masks: deferred pending
an asset-reference model for effect inputs. Chroma key: real and in the reference class,
but it is a pixel-classification effect with tolerance/spill parameters that need tuning
against real footage — no fixture footage exists, and a keyer shipped blind is a keyer
that gets rewritten. LUTs: an external file dependency (.cube) whose format and colour-
space semantics are a project of their own; out until an asset model for non-media files
exists. Glow: a composite of blur + additive blend + colour that is better expressed
once blend modes exist, so it waits on that. Vignette: expressible today as a
`rect`/`ellipse` element with a gradient or feathered mask over the frame; record that it
is absent as an effect because it is already composable, not because it is unwanted.
3D/perspective: rejected with skew on the same grounds, plus it is After Effects
territory per the settled reference class. Effect-parameter animation must be named
explicitly: keyframes are transform-only, so an agent cannot animate a blur in v1 — that
will be the most-asked "why can't I" and it deserves a recorded answer. Transitions
belong on the list only as a pointer, so a reader does not conclude the effects list
forgot them.

**Trade-offs:** Listing composable things (vignette) risks looking like the list
contradicts itself; the fix is to word those entries as "absent as a named effect;
express as X", which is more useful to an agent than silence. I would not list stroke,
background box, Ken Burns or `clip` because they are settled as belonging to other
models — repeating them here would suggest they were rejected rather than relocated, and
an agent reading the effects vocabulary could wrongly conclude they are unavailable.

## Judge's read

Q1 unanimous, closed without further debate: (a), order significant. Q2's split
(colour filter: Haiku/Fable include a small closed list, Opus defers) resolved in
Opus's favour — this project's own track record is that "closed" categories erode once
a first small member is admitted, and the fixture gives no evidence for colour filter
the way it does for mask. Q3 unanimous and load-bearing: the addressing argument
(effects attach to elements, a run lives inside one) is the same boundary ADR-0014
already drew for `stroke`, so excluding per-word highlighting here is consistency, not
evasion. Q4's core (per-word highlighting, skew, soft/alpha masks, chroma key, LUTs,
glow, vignette, 3D/perspective) is convergent 3/3; adopted Opus's grouped write-up
(deferred-with-successor / rejected-on-model-grounds / out-of-scope-for-file-as-truth /
boundary markers) and his position that transitions, stroke, background-box, Ken Burns
and `clip` must be kept OFF the absent-list, since an "absent from v1" list means "you
cannot do this," and all four can be done, just elsewhere. Folded in Fable's "absent as
a named effect, express as X" phrasing for vignette specifically, since flatly excluding
it would be misleading when it's already composable today.
