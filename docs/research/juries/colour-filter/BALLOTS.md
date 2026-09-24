# Colour-filter vocabulary: court ballots

Evidence for [ADR-0049](../../../adr/0049-v1-colour-filter-vocabulary-four-scalar-members.md),
resolving [#107](https://github.com/MBehtemam/Montagent/issues/107). Two court rounds, run
via `/court`, jurors on Opus, Haiku and Fable, independent and blind to each other's
ballots and to the author's recommendation.

## Round 1

Three questions put simultaneously to three separate three-juror panels.

### Q1 — shape: `color{mode}` vs several flat named effects

🗳️ **Juror (Opus)** — **VOTE: B** (several named effects)

**Reasoning:** The effects list is already a composition mechanism with a flat
vocabulary: `[blur, shadow, mask]` gets ordering, repetition, and independent params for
free because each operation is its own name. Colour operations are not a different kind
of thing — grayscale, sepia, and tint are pixel transforms exactly like blur is, and they
compose with each other and with blur in a defined order. Introducing a `mode`
discriminator would make `color` the only effect in v1 whose identity is two levels
deep, meaning an agent must learn two lookup rules instead of one: "what effects exist"
and, for this one member, "what modes exist inside it". That is a worse schema for a
machine-authored format, where flat enumerable vocabularies are cheap to validate, cheap
to document, and cheap to complete. It is also worse for error messages — `unknown
effect "sepai"` with a list of valid names beats a nested union mismatch.

The stacking consequence settles it. Grayscale-then-tint is a real, common grade
(desaturate, then push a colour cast), and under A it either becomes inexpressible
within one element or requires two `color` entries that disagree with the singular
framing of a `mode` field, or a bespoke internal ordering concept duplicating what the
list already provides. Any of those is a defect the list-of-names shape simply does not
have. The one genuine argument for A — that colour ops share a family and A keeps the
top-level vocabulary small — is a documentation concern, solvable by grouping the names
under a "colour effects" heading without paying for it in schema shape.

**Trade-offs:** B grows the top-level effect vocabulary, so each new colour operation is
a new name and a new schema entry rather than an added enum arm, and there is no single
place that says "these three are the colour family" unless the docs say so explicitly.
It also permits nonsense stacks like two tints or grayscale twice — legal,
order-defined, and harmless, but the validator will not reject them, whereas A's
single-entry framing would have implied at most one colour operation. A would have
bought a tidier surface and a natural home for shared params, at the cost of being the
only nested effect in the vocabulary and of making the most useful colour combination
awkward or impossible; that price is too high for v1, and B remains extensible —
nothing stops a later ADR from adding a genuinely multi-mode effect if one ever appears.

🗳️ **Juror (Haiku)** — **VOTE: B**

**Reasoning:** The existing effects vocabulary (`blur`, `shadow`, `mask`) establishes a
clear precedent: effects are flat, independently-named list members with no internal
discriminator. Extending that pattern to color operations maintains schema uniformity,
making the effects list predictable for agents and developers. Stacking multiple color
operations (`[grayscale, tint]`) works out-of-the-box the same way stacking blur and
shadow does today — no special ordering concept needed. Option A's single `color{mode}`
effect breaks the established pattern and forces either awkward workarounds (multiple
`color` entries when mode exists, or forbidding combinations) or schema-level complexity
(internal stacking rules).

**Trade-offs:** Option B trades a slightly larger effects vocabulary (more entries to
document) for API consistency and compositional clarity. Option A saves vocabulary size
but costs clarity, pattern-matching, and compositional ergonomics — agents would face
ambiguity about whether to reuse one `color` effect or add another, and the schema
introduces nesting that none of the existing effects use.

🗳️ **Juror (Fable)** — **VOTE: B**

**Reasoning:** The `effects` list is already the project's composition primitive: order
is the list, each entry is one named operation, and the three shipped effects prove the
pattern. Colour operations are not a special case of that pattern — `grayscale`,
`sepia`, and `tint` are exactly as independent from each other as `blur` is from
`shadow`, and stacking them (`[grayscale, tint]`, a very common "desaturate then wash
with brand colour" recipe) is expressible with zero new concepts. A `mode` discriminator
would introduce a second-level union that exists nowhere else in the vocabulary, forcing
every consumer (schema, validator, `fmt`, agents reading the spec) to learn one extra
shape for one family of effects. For an agent-first spec, a flat vocabulary is also the
cheaper thing to emit and validate: the effect name alone determines the parameter set,
with no cross-field conditional (`if mode == "tint" then color is required`) that JSON
Schema handles awkwardly and agents get wrong. Grouping by "these are all colour-ish" is
a taxonomy for docs, not a reason to change the wire shape; the docs can still list them
under a "Colour" heading.

**Trade-offs:** Option B grows the top-level effect vocabulary faster — every new colour
operation is a new name rather than a new enum value — so a future twenty-mode colour
toolbox becomes twenty entries in the effects registry, and there is no single place
that says "these are the colour operations" other than documentation and naming
discipline. It also gives up the ability to declare mode-exclusivity (one colour
treatment per element) at the schema level, if that were ever wanted; with B,
`[grayscale, sepia]` is legal and the renderer just does it in order, which is arguably
correct but is a choice being made implicitly. Option A's real appeal is the namespace
tidiness and a single shared parameter block (e.g. a common `amount` or `mix` field) —
but that can be achieved in B by convention, whereas A's cost (a nested union, an
ordering story inside one entry, or the odd double-`color`-entry workaround) lands on
every consumer.

**Verdict: unanimous 3/3 for B.**

### Q2 — exact member list

🗳️ **Juror (Opus)** — **VOTE: D** (`tint`, `saturation`, `brightness`, `contrast`)

**Reasoning:** The stated stopping rule — "each member is a single irreducible numeric
adjustment along one perceptual axis, not reproducible by composing two other admitted
members" — does most of the work if applied honestly, and only D survives it cleanly.
`grayscale` is not an axis; it is a *point* on the saturation axis (its zero endpoint).
Admitting both `grayscale` and `saturation` means the list contains an axis and a
distinguished value on that same axis, which is exactly the kind of redundancy the rule
exists to forbid, and it creates a semantic question with no good answer (what does
`grayscale` + `saturation{1.5}` mean? order-dependent nonsense). Option C dodges this by
deleting `saturation` and keeping the endpoint — but that is the worse amputation: it
makes the most commonly wanted colour adjustment in real editing (desaturate to 40%, or
push saturation up) unexpressible, while keeping a member that is merely a preset.
`sepia` fails the rule outright and both C and D correctly drop it. What remains in D is
four genuine, mutually irreducible axes: hue-ward pull (`tint`), chroma (`saturation`),
luminance offset (`brightness`), luminance spread (`contrast`). No two compose to make a
third. That is a real stopping rule, not a vibe — and it is the only list in the four
where I can state, for each excluded candidate, the member or composition that already
covers it.

The family-creep worry is answered by the same rule rather than by shrinking the list.
Duotone = tint composed with saturation{0} — excluded. Curves and LUTs are not single
numeric adjustments along one axis — excluded categorically, not by taste. Vignette,
sharpen, temperature/tint-as-white-balance would need their own argument against the
rule. A rule that only holds because the list was cut small (A) is not a rule; a rule
that survives a list of four is one.

D also matches the existing v1 precedent better than A does. `blur`, `shadow`, `mask`
are closed purely-numeric surfaces, and `saturation{amount}` with 0 = grey, 1 =
unchanged, >1 = boosted is precisely that shape, with the natural identity at 1 matching
how `amount` behaves elsewhere. The one cost D pays — losing the zero-parameter
`grayscale` spelling that an agent might reach for first — is a documentation and prompt
problem, not a schema problem: `saturation{amount: 0}` is one token longer and is
discoverable from the field's own range description.

**Trade-offs:** D's real cost is ergonomic and migration-shaped. "Make it black and
white" is the single most common colour request an agent will receive, and `grayscale`
is the word for it; forcing `saturation{amount: 0}` puts a small inference step between
intent and schema, and any prior draft, example, or user muscle memory that wrote
`grayscale` now breaks. It also introduces the first unbounded-above numeric in the
effect surface (>1 oversaturation), which needs an explicit clamp/validation decision
the other options don't force. I accept both because the alternative is permanent
ambiguity in the schema itself.

A is too small to be a stopping rule, only a stopping *point*. Brightness and contrast
are not reproducible from grayscale and tint by any composition, so excluding them is
not justified by the stated rule — it is justified only by fear of the slope, which
means the very first real request for brightness reopens the closed list and the rule
has no defence to offer. A closed list that cannot explain its own boundary will not
stay closed.

B fails on its own terms twice over: `sepia` is grayscale-plus-warm-tint (a preset, not
a primitive) and `grayscale` duplicates `saturation`'s endpoint. Shipping it is shipping
the redundancy the ticket was convened to eliminate, and it is the option most likely to
invite duotone next, since it has already admitted one preset.

C is the closest rival and I'd accept it over A or B. But it keeps the derived member
and drops the irreducible one — an inversion of the stated rule — and the practical
consequence is worse than D's: partial desaturation and oversaturation become
permanently inexpressible in v1, which is a capability gap, whereas D's gap is only a
spelling gap.

🗳️ **Juror (Haiku)** — **VOTE: C** (`grayscale`, `tint`, `brightness`, `contrast`)

**Reasoning:** Option C respects the irreducibility constraint (grayscale, tint,
brightness, contrast are each adjustments on independent perceptual axes—none
reproducible from the others) while establishing a defensible v1 boundary. Grayscale
remains an explicit, simple effect, consistent with the design of existing v1 effects
(blur, shadow, mask—all small, closed, purely numeric). The stopping rule is clear and
statable: "v1 includes fundamental tonal and color adjustments (brightness, contrast,
desaturation-via-grayscale, and color-via-tint); saturation as a continuous control is
deferred."

**Trade-offs:**
- vs. A: Brightness and contrast are orthogonal to grayscale/tint and represent
  essential perceptual axes; dropping them feels incomplete.
- vs. B: Saturation is the crack that leaks into oversaturation, curves, LUTs—once you
  admit it as a first-class control (rather than implied by grayscale's existence), the
  "why not?" chain resumes.
- vs. D: Folding grayscale into saturation=0 is economical but trades clarity for
  compactness; it makes grayscale less discoverable (a special case of a more complex
  parameter) and complicates the saturation API (0–1–∞ range). Keeping grayscale
  explicit preserves the precedent of simple, single-purpose effects.

🗳️ **Juror (Fable)** — **VOTE: D**

**Reasoning:** The stated stopping rule is "each member is a single irreducible numeric
adjustment along one perceptual axis, not reproducible by composing two other admitted
members." Applied honestly, that rule kills `grayscale` as a separate member: grayscale
is the 0-endpoint of a saturation axis, so keeping both violates the rule's own second
clause. It also kills `sepia`: sepia is grayscale plus a warm tint, i.e. `saturation{0}`
composed with `tint{color, amount}`. What survives are four genuinely orthogonal axes:
chroma (`saturation`), luminance offset (`brightness`), luminance slope (`contrast`),
and colour cast (`tint`). Each is one scalar (plus a colour for tint), each is a
well-known, closed-form pixel operation, and none is derivable from the others. That is
option D exactly, and it is the only option that is a *consequence* of the stopping rule
rather than a list the rule has to be bent to justify. Consistency with
`blur`/`shadow`/`mask` is also best: D is all-numeric, every member has an `amount`, and
there is no zero-parameter oddball.

**Trade-offs:** D costs the convenience of writing `grayscale` as a bare word — an agent
must write `saturation{amount: 0}`, which is slightly less self-documenting and less
greppable in a `.montagent.json`, and the >1 oversaturation range is a small
interpretive burden (needs a documented clamp/reference behaviour). A is too thin: it
cannot express the two most-requested corrections (exposure and contrast) and forces an
immediate v1.1 reopening, which is the "family that doesn't stop" outcome by another
route. B admits two members (`sepia`, `grayscale`) that the rule says are compositions,
and once you've admitted one composite preset the argument against duotone/vintage/
"cinematic" is gone. C is close but keeps `grayscale` and drops `saturation`, which is
backwards: it discards the axis and keeps the endpoint, so the moment anyone wants 50%
desaturation the list reopens. D's four members are the minimal spanning set of the axes
that every existing colour-adjust tool (CSS `filter`, FFmpeg `eq`/`hue`, SVG
`feColorMatrix`) also treats as primitive, which is the strongest available evidence
that this list actually stops.

**Verdict: 2-1 for D** (Opus, Fable) over C (Haiku). No juror chose A or B.

### Q3 — stopping-rule formulation

🗳️ **Juror (Haiku)** — **VOTE: ENDORSE**

**Reasoning:** The rule correctly identifies the foundational principle: admissibility
depends on a fixed, bounded numeric parameter set that represents a single irreducible
perceptual adjustment, not an arbitrary function or lookup mechanism. This cleanly
excludes all four test cases. Sepia fails (multi-axis composition); duotone fails
(arbitrary color mapping); curves fail (unbounded function space); LUTs fail (external
dependency + arbitrary function). The reasoning "it's an arbitrary function, not a fixed
parameter set" is the load-bearing insight that protects the closed-vocabulary
invariant. The criterion "not reproducible by composing two other admitted members"
elegantly prevents scope creep. A future reader proposing, e.g., "temperature shift" or
"per-channel contrast" can apply this rule: count the axes, count the parameters, check
for composability, check for external dependencies. All four yes-or-no tests are
actionable and derivable from the principle.

**Trade-offs:** "Near-linear transform" introduces mild interpretive slack (how close to
linear counts?), and the axes list (hue/saturation/lightness/tint) should ideally be
declared exhaustive to prevent incremental creep. These are minor hygiene issues; the
core rule and its reasoning are durable and will not require re-litigation.

🗳️ **Juror (Opus)** — **VOTE: TIGHTEN** — replace the "≤2 parameters / linear-ish
transform" implementation test with a *representability* test — a colour op is
admissible only if its full effect is recoverable from a fixed, named, finite parameter
tuple declared in the schema, where every parameter is a bounded scalar and the
parameter count is fixed by the effect's name, not by the user's input.

**Reasoning:** The rule's core insight is right and durable: the exclusion is not about
difficulty, it's that a curve or LUT is an *arbitrary function*, and a vocabulary that
admits arbitrary functions is no longer closed. That sentence alone would carry the ADR.
But three clauses in the candidate are doing work they can't sustain under a future
reader's pressure.

First, "linear or near-linear transform of pixel colour" is an implementation-side
criterion, and "near-linear" is exactly the soft edge the ADR exists to eliminate.
Saturation in a perceptual space is not linear; gamma-correct lightness isn't either;
sepia *is* a linear matrix and is meant to be excluded. So the linearity test both
admits what the rule wants out and threatens what it wants in — it has the sign
backwards in at least two places. The property the ADR actually cares about is about the
*schema*, not the maths: can I read this effect's entire behaviour off a fixed, finite,
named tuple of bounded scalars? A curve fails because its parameter count is set by the
author's chosen control points. A LUT fails for the same reason plus the external file.
Sepia fails because it is compositionally reachable (tint + saturation), which the
candidate's own non-composability clause already catches — correctly, and without
needing linearity.

Second, "≤2" is an arbitrary number masquerading as a principle, and arbitrary numbers
are what future readers re-litigate. The number that matters is *fixed*, not *small*: an
effect whose arity is determined by its name is closed; one whose arity is determined by
its input is not. Under the tightened form, a three-parameter HSL nudge would be judged
on composability, not on losing a coin flip at two.

Third, "perceptual axis (hue/saturation/lightness/tint)" is load-bearing and should
stay, because it is what keeps the family from creeping into per-channel and
spatially-varying territory — but note it already subsumes the per-channel-matrix
exclusion, so the categorical list should be framed as *worked examples of the rule
applied*, not as an independent list. That framing is what makes the rule durable: a
future reader confronted with "bloom" or "film grain" or "temperature" doesn't check
membership in a blacklist, they run the three tests — fixed named arity, bounded
scalars, one perceptual axis, not composable from admitted members — and gets an answer
with a reason attached.

With that modification, the rule does the thing the ticket asks for: it records that
curves and duotone were *considered and rejected*, and it hands the next reader a
decision procedure rather than a precedent to argue with.

**Trade-offs:** The tightened form is slightly more abstract than "≤2 parameters", which
is cheap to check mechanically; a reviewer now has to reason about whether an arity is
name-fixed or input-fixed, which is a judgement call in rare cases (a fixed 3-stop
gradient map is name-fixed by the letter of the rule, and must be excluded by the
perceptual-axis clause instead — the rule works, but only if both clauses are read
together). Dropping the linearity clause also gives up a cheap renderer-side sanity
check that implementers might have found useful for reasoning about compositing order
and GPU cost; that concern is real but belongs in an implementation note, not in the
admissibility rule, since letting performance intuitions leak into vocabulary
definitions is how closed sets get argued open. Finally, the non-composability clause
remains the rule's sharpest edge and its most expensive one: it means every future
proposal must be checked against the *current* member set, so admitting any new member
can retroactively make a previously-admissible proposal inadmissible. That is a feature
— it is what forces the set to stay minimal — but it does mean the ADR must state the
member list explicitly so the composability check has something to run against.

🗳️ **Juror (Fable)** — **VOTE: TIGHTEN** — replace the "linear or near-linear transform
along one perceptual axis" clause with a mechanical admission test — an effect is
admissible iff (a) it is fully specified by at most two bounded scalars, each with a
documented identity value at which the effect is a no-op; (b) no parameter is a colour,
array, path, or table; (c) it is not a composition of already-admitted members — and
make the axis list (hue, saturation, lightness) part of the rule text rather than a
parenthetical, dropping "tint" unless it is redefined as a scalar temperature.

**Reasoning:** The candidate gets the *why* right — a curve or LUT is an arbitrary
function, not a parameter set, and admitting one converts a closed vocabulary into an
open one — and its composition clause correctly disposes of sepia (tint + desaturate)
and duotone (two colour endpoints mapped over luminance). But the operative language is
not sharp enough to survive a new proposal without re-litigation, for three reasons.
First, "linear or near-linear transform of pixel colour" is self-contradicting with the
exclusion of "per-channel matrices": hue rotation *is* a 3×3 RGB matrix, and
brightness/contrast are affine transforms — so a reader applying the letter of the rule
either rejects hue or admits contrast, and the ADR intends neither reading
unambiguously. "Near-linear" has no test; gamma is not linear and the rule cannot say
whether it's in. Second, "≤2 closed numeric parameter set" is ambiguous about a colour
parameter: is a tint colour one parameter or three? That ambiguity is exactly the door
duotone walks through ("it's just two colours — two parameters"). Requiring parameters
to be *scalars* with an *identity value* closes it cleanly and is also the sharpest
available discriminator between an adjustment and a lookup: saturation has identity 1.0,
hue has identity 0°, a curve has no scalar identity, a LUT has none. This ties directly
back to ADR-0040's "independently bounded" property — a bounded scalar with a no-op
point is what a validator can range-check. Third, the axis list is doing real closing
work (it's what keeps vibrance, temperature, exposure, levels from arriving one at a
time as "just another axis") and so must be stated as the rule, not as an example inside
parentheses; a future reader must be able to say "your proposal is not on the axis list,
and adding an axis is a new ADR" rather than argue perceptual-axis philosophy. The
composition clause should also state that it tests *candidates against the enumerated
set*, so admission order can't be gamed.

**Trade-offs:** Scalar-only permanently excludes tint-by-colour and sepia-as-preset from
v1, both of which are common user asks; any effect taking a colour parameter needs a v2
ADR rather than an incremental addition. Enumerating axes in the rule concedes that the
rule is partly a list after all — durability comes from the "new axis = new ADR" clause,
not from a purely intensional definition. The identity-value requirement admits gamma if
someone proposes it (one scalar, identity 1.0), so the axis list must consciously
include or exclude it rather than leaving it to the transform-shape language. Net: the
tightened rule is more mechanical and less elegant, which is the right trade for a
document whose job is to stop arguments.

**Verdict: 2-1 TIGHTEN** (Opus, Fable) over ENDORSE (Haiku), with Opus and Fable
converging independently on the same underlying fix (fixed arity, scalar parameters,
identity value) while diverging on whether the axis list belongs in the rule text and
whether `tint.color` needs an explicit exception clause.

## Round 2

The two splits from round 1 restated as direct two-way questions, with the dissenting
reasoning surfaced back to a fresh panel.

### Question A — member-list packaging: D vs C

🗳️ **Juror (Opus)** — **VOTE: D**

**Reasoning:** The stopping rule is the panel's own admission criterion, and it is a
criterion about the *structure of the member set*, not about how a member is spelled.
Under it, `grayscale` and `saturation` cannot both be admitted, so the real choice is
which of the two axes v1 loses. `saturation{amount}` strictly dominates: it is the
actual perceptual axis, it covers the desaturate-slightly, fully-desaturate, and
oversaturate requests as one continuous control, and grayscale is exactly one value on
it. Option C keeps only the endpoint and throws the axis away, which means the entire
"wash the colour out a bit" and "make it pop" families have no v1 member at all — a far
worse hole than a discoverability cost. The dissent concedes the structural point and
rests on ergonomics, but ergonomics is not what a member list is for: an agent-first,
declarative spec is read by agents that load the whole effect schema, so "an agent has
to know saturation exists" is nearly a non-problem when the schema is the thing it
reads. And discoverability is fixable outside the member list — a doc line, a parameter
description ("0 = full grayscale"), or a later alias layer — whereas a missing axis is
only fixable by admitting a new member, i.e. re-opening the decision. Fix ergonomics in
documentation; do not buy ergonomics with structure. The `blur`/`shadow`/`mask`
precedent doesn't transfer either: those are not zero-endpoints of an admitted
continuous member, so they never faced this rule.

**Trade-offs:** D costs exactly what the dissent says: "make it black and white" does
not map to a literally named effect, and an agent that only skims member names may miss
it. That cost is real but bounded and repairable in the schema's prose. Choosing C
instead would spend the most valuable colour member in the set to buy a name, leaving v1
unable to express any partial or increased saturation — and would leave the door open to
re-admitting `saturation` later, at which point the pair violates the stopping rule
anyway and `grayscale` has to be deprecated from a shipped surface. Taking that removal
now, before v1 ships, is cheaper than taking it after.

🗳️ **Juror (Fable)** — **VOTE: D**

**Reasoning:** The stopping rule is the only admission criterion the panel adopted, and
the dissent concedes the structural fact that decides the case under it: grayscale is
`saturation{amount: 0}`. Once that is conceded, keeping `grayscale` as a separate member
is a rule violation by the rule's own text, and the dissent's argument becomes "the rule
is right but we should break it for ergonomics." A stopping rule that yields to the
first ergonomic appeal is not a stopping rule; the next petitioner will ask for `sepia`
(tint at a fixed colour), `invert`, `desaturate-slightly`, and each will cite
`grayscale` as precedent. Option C is also strictly less expressive: it gives up
continuous saturation entirely, which means "make it a bit less punchy" or "oversaturate
this" has no v1 answer at all, while D covers every request C covers. The
discoverability cost is real but belongs in the docs and the schema, not the member
list — a `saturation` description that says "0 = black and white" and an example under
the "black and white" heading solves it without adding a redundant member. The
`blur`/`shadow`/`mask` precedent does not help C: those are irreducible axes with small
param sets, not fixed points on another member's axis.

**Trade-offs:** D costs an agent one extra inference step ("black and white" →
saturation → 0), and an alias-free surface means a literal grep for `grayscale` in the
spec fails. If agent evaluations later show this step is a recurring failure, the right
repair is a documented alias or a schema `examples` entry, not a member — and that
repair remains available under D, whereas C forecloses continuous saturation and sets a
precedent of admitting reproducible members whenever they are common enough.

🗳️ **Juror (Haiku)** — **VOTE: D** (flipped from round 1's C)

**Reasoning:** The stopping rule exists to prevent logical redundancy in the API
surface. Option D applies this principle consistently: saturation from 0 to >1 is a
continuous perceptual axis, and grayscale at saturation{0} is a point on that axis, not
a separate irreducible adjustment. Admitting both members violates the rule's own stated
purpose. The functionality remains available; grayscale is not lost, merely accessed via
saturation{0}.

**Trade-offs:** Grayscale loses first-class discoverability — users must know or
discover that saturation{0} equals grayscale rather than finding a member literally
named `grayscale`. This is real friction compared to the existing API precedent of named
zero-param effects. However, the alternative (removing saturation entirely to keep
grayscale explicit, as C requires) discards continuous saturation control—a more
valuable loss. Special-casing high-frequency use cases to bypass design rules creates
precedent for future exceptions. The rule should hold.

**Verdict: unanimous 3/3 for D.**

### Question B — stopping-rule packaging: A (axes as commentary) vs B (axes in rule text, `tint` exception flagged)

🗳️ **Juror (Opus)** — **VOTE: A**

**Reasoning:** The mechanical test is the rule because it is checkable without a
taxonomy debate: count the parameters, check each is a bounded scalar with a documented
identity, check composability. The axis list has no such test behind it — "is this on a
perceptual axis?" is itself an unbounded argument, and enumerating hue/saturation/
lightness in the rule text converts a stopping rule into a closed vocabulary. That's a
different decision (freeze the v1 member set) smuggled in as a criterion. If the panel
wants a frozen list, say so directly — "v1 admits exactly these four, additions need a
new ADR" — rather than dressing it as an admissibility predicate that any future
proposal can argue its way onto by relabelling its parameter as an axis. Framing B is
also self-refuting on its own terms: it has to immediately carve out an exception for
`tint`, an adopted member, which is evidence the promoted conditions don't actually
describe what the project decided. A rule whose first application requires an exception
is a rule mis-stated.

That said, B's substantive catch is real and should survive into A's packaging:
`tint{color, amount}` genuinely does not satisfy "bounded scalar parameters" if `color`
is an arbitrary colour value. The right home for that is the illustrative commentary —
show `tint` as the worked case where the scalar requirement bites, and either state that
`color` is a bounded enumerated/parsed value the identity rule applies to via
`amount: 0`, or note that `tint` is pending a scalar redefinition. Under A, `tint`'s
identity value is `amount: 0`, which is well-defined regardless of `color`'s type; that
is the honest justification, and it belongs in prose next to the examples rather than as
a rule-level exception clause.

**Trade-offs:** A is weaker against a future proposal that passes the mechanical test
while being perceptually incoherent — some two-scalar colour op nobody wants, admitted
because it technically qualifies. The panel accepts that risk: an ADR is a decision
record, not an automated gate, and a human or agent reviewing a proposal can still say
"this passes the test and we still don't want it" without needing the axis clause to
justify the refusal. Choosing A also means the `tint` scalar tension is documented as
commentary rather than flagged as a formal exception, which is slightly easier for a
future reader to skim past — mitigated by making `tint` the explicitly worked example
rather than one name in a list.

🗳️ **Juror (Haiku)** — **VOTE: B**

**Reasoning:** The core mechanical test (fixed arity, bounded scalar parameters,
identity value, non-composability) contains an internal inconsistency: `tint{color,
amount}` violates the "bounded scalar parameters" constraint by including a non-scalar
`color` parameter. Framing A would hide this violation by treating the axes only as
illustrative commentary, leaving the exception implicit and undocumented in the rule
itself. Framing B forces an explicit design decision: either justify `tint`'s non-scalar
parameter as a sanctioned exception to the scalar rule, or redefine `tint` to use a
scalar parameter (hue-angle or temperature). This clarity strengthens the rule by
surfacing a real inconsistency rather than burying it.

**Trade-offs:** Framing B is more restrictive and less extensible—new perceptual
dimensions require updating the rule text itself, which raises friction for future
proposals. If color operations outside the enumerated axes become legitimate (e.g., new
perceptual color models), the rule would need revision. Framing A is more extensible and
principle-based, but at the cost of silently tolerating a violation of its own stated
scalar-parameter constraint and leaving future designers without clear guidance on
whether non-scalar parameters are acceptable.

🗳️ **Juror (Fable)** — **VOTE: A**

**Reasoning:** The panel's actual convergence was on a *mechanical* test — fixed arity,
bounded scalars, documented identity value, non-composability — precisely because the
prior "≤2 params / near-linear" rule was a taste judgment dressed as a threshold.
Framing B re-introduces a taste judgment through the back door: "sits on one of the
enumerated axes" is not something an agent or reviewer can check without arguing about
what counts as an axis, and it freezes today's list (hue/saturation/lightness) into
rule text that will need an ADR amendment the moment a legitimate scalar op like
`contrast{amount}` or `gamma{value}` shows up — neither of which is cleanly "on" hue,
saturation, or lightness, yet both trivially pass the mechanical test and are exactly
what a v1 colour vocabulary should be able to admit. The mechanical test already does
the gatekeeping work the axis list was meant to do: an op with fixed arity, bounded
scalars, and an identity value that can't be built from admitted members *is* a
perceptual axis by construction. Listing the current yield as commentary keeps the ADR
honest about what the rule produces without making the examples load-bearing. On
`tint`: B's observation that `color` is non-scalar is correct and worth recording, but
it's a note about an already-adopted member's fit, not a reason to change the rule's
packaging — and the fix B proposes (redefine as scalar hue-angle/temperature) is a
separate design change that should be its own decision, not smuggled into a
stopping-rule ADR. The ADR can carry a one-line consequence: "`tint.color` is the sole
non-scalar parameter among adopted members; it predates this rule and is grandfathered;
new proposals get no such allowance."

**Trade-offs:** A gives up the extra guard B wanted against a proliferation of odd-but-
mechanically-valid scalar ops (e.g. a `sepia{amount}` that is arguably composable from
saturation+tint but hard to prove so). The non-composability clause carries that load
imperfectly, and reviewers will occasionally have to argue composability by hand rather
than pointing at an axis list. A also leaves `tint`'s non-scalar `color` as a documented
grandfathered inconsistency rather than resolving it — which is a real wart, and B is
right that it deserves eventual attention. But B's cost is worse: an enumerated-axis
clause is either redundant (when the mechanical test already rejects) or wrong (when it
rejects a legitimate op like contrast/gamma), and it conflates "what the rule is" with
"what the rule has yielded so far," which is exactly the ambiguity the tightening was
meant to eliminate.

**Verdict: 2-1 for A** (Opus, Fable) over B (Haiku), with Fable's ballot proposing the
synthesis this ADR adopts: A's framing plus one explicit line grandfathering
`tint.color`.

## Judge's read

D is unanimous across both rounds (4 votes total, one flip); no further debate needed.
On packaging, the 2-1 plus Fable's own proposed synthesis converges cleanly: adopt A's
framing (mechanical test is the whole rule, axes are commentary) with Fable's one added
sentence explicitly grandfathering `tint.color`, which also answers Haiku's objection
without adopting B's frozen-axis cost. The human agreed with this read; see
[ADR-0049](../../../adr/0049-v1-colour-filter-vocabulary-four-scalar-members.md).
