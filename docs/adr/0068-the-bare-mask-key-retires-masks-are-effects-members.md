---
status: accepted
amends: 0040 (retires its "no migration needed" Consequences bullet, and states the param-less `mask` member's geometry it left unwritten), 0041 (states where `effects` sits in canonical key order, which it deferred to whichever ADR introduces the field), 0043 (classifies a second retired spelling)
---

# The bare `mask` key retires: masks are `effects` members, and the param-less form is the inscribed shape

> **Amended by [ADR-0075](0075-the-badges-mask-changes-its-own-rim-and-nothing-else.md)** —
> three sentences below no longer hold: *"The change is pixel-inert"*, *"Nothing about the
> rendered frame changes"* in the same paragraph, and the Consequences bullet *"The
> rendered frame is unchanged"*. The renderer minifies the 800×800 badge into a 68×68 slot,
> so the drawn badge reaches past the circle the stored one does not, and the mask trims
> that rim: 112 pixels of 540×960 change, mean channel delta 0.0041. Everything else below
> — the retirement, the param-less geometry, the key order, the advise-class
> classification, the migration — stands.

[ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md) contains two accepted
sentences that cannot both be implemented. Found while breaking
[#168](https://github.com/MBehtemam/Montaget/issues/168) into tickets, where the very
first ticket's demo — *the committed fixture round-trips* — is the thing that fails.

**Its schema clause** puts masks inside the effect list:

> The schema gains a discriminated union `effects: [{name, ...params}]` with exactly three
> members in v1: `blur{radius}`, `shadow{dx, dy, radius, color, opacity}`,
> `mask{shape: "circle"|"rect"|"ellipse", ...shape params}`.

**Its Consequences clause** leaves the fixture's bare key where it is:

> `mask:"circle"` on `handle-logo` in the committed fixture becomes a valid declaration
> under this ADR rather than an inert stray field — no migration needed, since the value
> was already legal shape-vocabulary syntax.

Under [ADR-0017](0017-closed-schema-no-escape-hatch.md)'s closed schema, a bare `mask` key
on an `image` element is an unknown key the moment `effects` exists.

## The error is one clause, not the model

**The Consequences bullet argues from the legality of the *value* to the legality of the
*key*.** `"circle"` is indeed a member of the shape enum — and the schema clause puts that
enum somewhere else. The two halves of the sentence are about different things, and the
word *"since"* joining them is the whole defect.

The effect model itself is not in doubt. ADR-0040's Decision had already closed the
attachment question in favour of `effects: [...]`, and three later documents read it that
way and depend on it: `CONTEXT.md`'s **Mask** entry calls it *"an `effects` vocabulary
member"*, [ADR-0049](0049-v1-colour-filter-vocabulary-four-scalar-members.md) adds four
members *"alongside the existing `blur`/`shadow`/`mask`"*, and
[ADR-0055](0055-audio-mixing-model-volume-fades-ducking-deferred.md) clarifies `effects`'
scope without reopening where it lives. **Decision governs; one Consequences bullet is
retired.**

**Admitting both spellings was considered and refused.** It is the failure `center-center`
(ADR-0013), the `scale` union (ADR-0012) and `#RRGGBBFF` (`CONTEXT.md`) were each retired
for, and the mechanical reason is this project's authoring model: `fmt` normalises on
write, so the agent's next exact-string replace gets zero hits. It also breaks ADR-0040's
own ordering argument — a bare `mask` has no position in the `effects` list, so
`blur`-then-`mask` versus `mask`-then-`blur` becomes inexpressible in the sugar form — and
it would be the first field in this format that is sugar for another field. Nothing else
is.

## The param-less form means the inscribed shape

**ADR-0040 never fixed what a `mask` member's parameters are.** Its union member is written
`mask{shape: "circle"|"rect"|"ellipse", ...shape params}`, and that ellipsis is an unmade
decision rather than shorthand: no accepted document names a single parameter, a default,
or what a param-less circle means. Contrast ADR-0049, which spelled out every colour
member's fields and gave an example literal.

This matters because it is load-bearing for everything else here. A retirement that cannot
name its replacement is not a retirement, and a fixture cannot be migrated to a shape that
does not exist.

**So this ADR states the minimum the migration forces, and no more:**

> `{"name": "mask", "shape": "circle"}` with no geometry parameters means the largest
> circle inscribed in the element's own rect — diameter `min(width, height)`, centred on
> that rect. The `rect` and `ellipse` shapes take the element's rect itself under the same
> rule.

The evidence is the only real instance. `brand/logo-en.png` is 800×800 RGBA; decoding it
finds all four corner alphas at 0, the centre at 255, and on a sampled grid **zero opaque
pixels outside the inscribed circle and zero transparent pixels inside it**. The badge is
already exactly the inscribed circle, drawn into the asset. Its element is a square 68×68
slot with `clip` equal to the element rect. The default this ADR writes down is the one the
committed project already means.

**The full parameter surface is not decided here** — explicit centre and radius, `rect`'s
corner radii, `ellipse`'s two axes, and what any of them mean under a keyframed `scale` or
a non-`top-left` `origin`. Those are graduated to their own ticket. Naming a default that
covers every element deterministically is what the migration needs; inventing a geometry
vocabulary is not, and ADR-0049's four-clause discipline is the standard the eventual
parameter set should be held to rather than something to guess at now.

## Where `effects` sits in key order

[ADR-0041](0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md) fixed
canonical key order as schema property order and explicitly deferred any field it did not
enumerate — *"any transform, paint or effect property this ADR doesn't enumerate … gets its
position from the ADR that introduces it into a given type's schema"*. ADR-0040 introduced
`effects` and did not state the position.

**`effects` appends after the type's existing fields.** For `image` that makes the order
`source, x, y, origin, width, height, fit, clip, scale, effects`. This is not a new
convention: ADR-0041 already described the fixture's bare `mask` as *"appending after their
type's core fields, still in a stable relative position"*, and the migration below
preserves exactly that position. Stating it prevents ADR-0041's table going stale the same
way ADR-0040's Consequences did.

## The retired spelling is advise-class

Under [ADR-0043](0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md),
a check refuses when *any* instance it can match could be load-bearing in a way the document
cannot resolve. **A bare `mask` key cannot be.** It never had accepted semantics in any
document — it was undeclared from the day it was committed — so there is no prior meaning a
repair could misread, and with the param-less form now defined the repair is a pure spelling
transposition of a value that is already a member of the target enum.

This is the distinction from `gravity`, which is refuse-class because its repair needed
source pixel dimensions the document does not carry. Here the only input is the string
itself.

**A note on precedent, because it is easy to overstate.** ADR-0043 classifies exactly one
retired spelling — `gravity`. `center-center` is a pure spelling substitution (ADR-0013),
and `box` was migrated by arithmetic script. *"A retired spelling that names its
replacement"* is a message-text property under
[ADR-0016](0016-no-format-version-the-unknown-key-error-is-the-mechanism.md); it is not a
repair class, and the two should not be conflated. This ADR classifies the second one.

## The fixture migrates in this change

`handle-logo`'s `"mask": "circle"` becomes
`"effects": [{"name": "mask", "shape": "circle"}]`, in the same position.

This follows [ADR-0015](0015-fit-is-a-derivation-claim-and-gravity-retires.md)'s precedent
and its stated reason: agents author by copying the shipped fixture *before* reading the
spec, so the migration lands with the decision rather than after it. A fixture carrying a
spelling the schema rejects teaches the wrong thing to whoever copies it next, and #168's
own rule — *a check that fires on the fixture is wrong unless an ADR says otherwise* — is
satisfied here by construction, because the ADR saying otherwise is the one landing the
bytes.

Two tooling changes land with it, because they are coupled and would otherwise break
silently: `migrate.py` line 108 carries the key forward marked `# still #22, unchanged`, and
`verify.py` asserts that `migrate.py` still reproduces the committed fixture. Both are
updated, and `verify.py` gains an assertion that the bare key is absent and the effect
present.

**The change is pixel-inert.** The asset's own alpha already produces the circle, `clip`
equals the element rect, and the slot is square, so the mask selects every pixel the asset
already shows. Nothing about the rendered frame changes — which is the point: this converts
an inert stray field into a live, spellable v1 effect with regression coverage, without
altering the artifact that #168 calls the only test capable of falsifying the format.

Deleting the key instead was considered and rejected narrowly. It is pixel-identical and
the smaller edit, but it would leave v1's `mask` effect with no coverage in the only real
project this repository has, and ADR-0040 leaned on this very key as *"the strongest
evidenced candidate of any considered"*. Deleting the evidence at the moment it becomes
expressible is perverse.

## How this happened

**It did not survive a court; it was never put to one.** ADR-0040 was settled by a
three-juror court whose packet asked four questions — attachment, vocabulary, text effects,
and the absent-list. **None asked what happens to the fixture's key**, and the packet
itself told the jurors it was *"a no-op there, since the source image already has
alpha-transparent rounded corners."* The "no migration needed" sentence was authored after
the court, on a question the court was never asked. Nothing survived scrutiny, because
nothing was scrutinised.

The deeper shape is a **deferral passed forward that the last holder dropped**. The bare
`mask` key was deliberately carried, byte-unchanged, through every migration since the
sample landed: ADR-0012 sent shape masks to #22, ADR-0014 confirmed *"`mask` … stays
#22's"*, ADR-0041 recorded its key position, and `migrate.py` preserved it under a comment
naming the same ticket. Three ADRs handed it forward correctly. ADR-0040 is #22's own ADR —
the end of the chain — and it is where the hand-off was supposed to be caught.

That is a class, not an instance, and the next places to look are ADR-0040's own deferrals
(colour filter → ADR-0049, highlighting → ADR-0048, transitions → ADR-0059). None of those
touch fixture bytes, so they are probably clean — but that is a check, not an assumption.

## Consequences

- A bare `mask` key on any element is a **schema error naming `effects` as its
  replacement**, advise-class under ADR-0043, carrying the transposed repair.
- `{"name": "mask", "shape": "circle"}` with no geometry parameters is the inscribed
  circle of the element's rect; `rect` and `ellipse` take the rect itself.
- The `mask` member's **full parameter set is graduated to its own ticket**, not decided
  here. Until it lands, the param-less form is the only spelling.
- `effects` appends after a type's existing fields in canonical key order. For `image`:
  `source, x, y, origin, width, height, fit, clip, scale, effects`.
- ADR-0040's *"no migration needed"* Consequences bullet is retired. Its Decision, its
  ordering argument, its v1 vocabulary and its deferrals all stand.
- The committed fixture, `migrate.py` and `verify.py` are updated in this change. The
  rendered frame is unchanged.
- #168's ticket for the document model inherits a fixture that actually round-trips, and
  does not get to decide format bytes.

## Evidence

- Three independent jurors (Opus, Haiku, Fable), each given the same brief cold, blind to
  each other, defaulting to "refuted":
  [`docs/research/juries/mask-spelling/`](../research/juries/mask-spelling/README.md).
  Unanimous on retiring the bare spelling; the parameter-set gap was found independently by
  two of the three and is why this ADR states the param-less form rather than assuming it.
- `docs/research/juries/mask-spelling/decode_logo_alpha.py` — re-executable, re-derives the
  badge's alpha geometry from the committed PNG and exits non-zero if it stops holding.
- `docs/research/sample-project-migration/verify.py` — asserts the bare key is absent, the
  effect present, and that `migrate.py` still reproduces the committed fixture.
