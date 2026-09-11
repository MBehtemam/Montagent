---
status: accepted
amends: 0012 (the rounding paragraph over-generalised; `photo-06` was right), 0006 (gains the aperture-coverage error, the fit-deviation note, and the UNCHECKED category)
---

# Fitted extents floor, and the nine `origin` keywords are spelled

> **Amended by [ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md)**, in three
> places. **Tiebreak (2) below is false**: `contain` is now defined, and ceil preserves
> containment just as trivially as floor preserves coverage (zero violations under either over
> 3,286,969 cases). Floor still stands, on tiebreaks (1) and (3) only. The **fit-deviation
> `note` is now an `error` at strict equality** with the rule value — the `{floor, ceil}`
> membership preferred under *Not settled here* was measured to produce two legal files for one
> input on three of six authoring tasks. And the **aperture-coverage error is parameterised by
> `fit`**, since a correct `contain` element is smaller than its own clip.

> **Extended by [ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md)**: as
> `fmt` may never rewrite a declared extent, it may never rewrite a **colour** — including
> converting between `#RRGGBB` and `#RRGGBBAA`. Under declared-authoritative the spelling
> is content. ADR-0014 also declines `gravity` to [#48](https://github.com/MBehtemam/Montaget/issues/48)
> rather than settling it, on this ADR's own refusal to create a schema value by implication.

Two numbers an author must know before writing a single element, that no document
stated. [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md) made
`width`/`height` **required on the element**, so a fitted extent is not a sampling
detail the renderer settles privately after the fact — it is typed by a human or an
agent before any renderer runs. And the nine `origin` keywords were given by ellipsis,
so six of them had no spelling at all.

```json
{"id":"photo-06","type":"image","x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover","gravity":"top","clip":[0,0,1080,1300]}
```

That element is **unchanged from ADR-0012**. This ADR does not correct it — it corrects
a *sentence* in ADR-0012 that over-generalised its own scope, and supplies the rule
ADR-0012 explicitly asked a successor for: *"The renderer must publish a sampling rule,
because the rectangle is not integral: exact cover here is 1912.5 px."*

## The decision is that exactly one integer is published

[#44](https://github.com/MBehtemam/Montaget/issues/44)'s complaint is *"Two authors who
round differently write different files for the same picture, and nothing in either file
says which is right."* **Determinism is the decision.** Floor-versus-ceil is a tiebreak
underneath it, and this ADR is explicit about that ordering because the alternative —
presenting floor as geometrically forced — is precisely the failure that produced #44.
ADR-0012's *"ties away from zero"* read as a general principle because it was written as
one.

## Decisions

**A fitted extent is computed in exact integer arithmetic, and floors.** Under
`fit:"cover"` with source `sw x sh` into a box `bw x bh`:

- The **driving axis** is chosen by integer cross-multiplication — width drives when
  `bw*sh >= bh*sw` — never by comparing `bw/sw` against `bh/sh` as floats.
- The driving axis **takes the box dimension verbatim**. It is exact by construction and
  must never survive a round trip through a float.
- The **slack axis** is `(s_slack * b_driving) // s_driving`, floor, in integer arithmetic.

For the fixture: `2720*1080 // 1536 = 1912`. Exact cover is 1912.5; the published element
was right and its stated rule was not.

**The integer clauses are not an implementation note.** `math.floor(sw * f)` with
`f = bw/sw` loses the driving axis to a ULP on a large fraction of inputs — `sw=103,
bw=1920` gives `1919.9999999999998`, floors to **1919**, and the cover fails by a visible
pixel on the axis that is exact by construction. Measured over **31,402,800** (source, box)
combinations: the float method disagrees with the integer method on **4.466%** of them,
and the integer method produces **zero** coverage violations. The fixture escapes the bug
entirely because `1080/1536 = 45/64` is dyadic and exact in binary — it is a single
lucky data point that structurally cannot demonstrate the defect it avoids. This is the
hole ADR-0005 left on `speed`, where `3368/0.645` sent four agents to 5222 and one to 5220,
in a new costume.

**The invariant is rationale, and it is normative for `cover` only.** *A derived bound
rounds in the direction that preserves the bound.* Any future `fit` value must name the
inequality that defines it, and its extent rounds to preserve that inequality. But
`contain` is defined in no ADR and appears nowhere in `CONTEXT.md`, and legislating a
rounding rule for it would **create a schema value by implication**. `cover` is 8 of 8
elements in the only real project file; the `fit` vocabulary is undefined and is
[#48](https://github.com/MBehtemam/Montaget/issues/48).

### Why floor, honestly

**Both floor and ceil are geometrically safe, and this ADR says so rather than pretending
otherwise.** Under cover, whichever term wins the max gives that axis the box dimension
exactly — an integer, on which floor is the identity — and the slack axis satisfies
`s*f >= b`, so `floor(s*f) >= b` holds for integer `b`. **Ceil preserves the bound
trivially.** Direction-preservation is therefore *indifferent* for `cover` and selects
floor uniquely only for `contain`. Three tiebreaks decide it, in descending strength, and
none of them is geometric:

1. **Ceil is not additive — it would reopen a settled decision.** Ceil changes **7 of the
   8** image elements in the committed fixture (all seven photos, 1912 → 1913;
   `handle-logo` is an exact aspect match at 800x800 → 68x68 and is unaffected), and
   additionally requires correcting ADR-0012's *published* element, rewriting `migrate.py`,
   and inverting the migration README's D1. Floor is a supplement; ceil is a correction.
2. **Forward-consistency across a vocabulary that does not yet exist.** If `contain` is
   ever defined with the obvious semantics, only floor is safe there. Floor for `cover`
   keeps **one rounding operation across all fit values**; ceil forks the rule into a
   per-value rounding table the day a second value lands. This is conditional, and is
   recorded as conditional.
3. **Implementation entropy.** `(s_slack * b_driving) // s_driving` *is* floor in every
   language's default integer division for positive operands. Ceil requires a deliberate
   second construct — `-((-a)//b)` or the known-buggy `(a+b-1)//b`. For a rule whose whole
   purpose is that independent implementers land on the same integer, take the operation
   you get for free.

**Two arguments for floor were raised and are withdrawn**, recorded here so they are not
re-raised:

- *"Bound-preservation derives floor."* It does not. Both roundings cover; the invariant
  is indifferent for `cover`. Stated as a bare elegant one-liner it would license ceil.
- *"Ceil manufactures a rect the source cannot paint."* Void under this ADR's own
  Q3 reading: with the declared rect authoritative and the source resampled to it, 1912 is
  a **0.0261% squash** and 1913 a **0.0261% stretch** — same magnitude, opposite sign.
  Note the coupling: this argument is void under *declared-rect-authoritative* and **live**
  under isotropic sampling. If that decision is ever reopened, floor becomes geometrically
  forced again and ceil does not.

**And the corpus is thinner than it looks.** The fixture contains exactly **two** distinct
`(source, box)` geometries — `(1536, 2720) -> (1080, 1300)` on seven elements, and
`(800, 800) -> (68, 68)`, which is exact and needs no rounding. So *"all seven elements
say 1912"* is **one geometry counted seven times**, produced by a script whose own comment
says it follows ADR-0012's published element. The evidential base is one hand-authored
integer of unrecorded provenance, and tiebreak (1) is what actually carries the weight.

### The declared rect is authoritative at render

The source is resampled to exactly the declared rect. The renderer never silently samples
at the exact isotropic factor and treats the declared integers as an approximate bound —
that would make what is actually drawn a number **not in the file**, derivable only from
source dimensions **also not in the file**, which is the defect ADR-0012 spent three
paragraphs killing, re-entering through the renderer after being shut out of the schema.
It would also leave half a pixel of unspecified freedom (top? bottom? split?), which is
ADR-0011's *"two agents implementing `shift` from ADR prose shipped two different videos"*
in miniature.

The cost is bounded and already unavoidable: under floor the residual is `< 1 px` always,
giving 0.0261% anisotropy on the fixture — orders of magnitude below the 8.9 px framing
error ADR-0012 used to *disqualify* an option, and every frame after the first is already
a resample at an arbitrary non-integral factor because seven of eight photos carry a
`scale` ramp to 1.08.

### The nine `origin` keywords

```
top-left      top-center      top-right
center-left   center          center-right
bottom-left   bottom-center   bottom-right
```

**Vertical component first**, which is forced rather than chosen: `CONTEXT.md`'s two
published endpoints are `top-left` and `bottom-right`, and 18 elements in the committed
fixture already say `top-left`. **The middle is `center` alone.** The grammar is
`{top|center|bottom}-{left|center|right}` with exactly one elision.

**`center-center` is a schema error naming `center`, not a legal alias.** Two spellings of
one value is the union ADR-0012 rejected for `scale`, by the same mechanism: ADR-0011
requires `fmt` to normalise on write, so an accepted alias would be silently rewritten and
the agent's next exact-string replace on the string it just wrote gets **zero hits**.

**`center-left`, not `middle-left`.** `top-center` and `bottom-center` need a
horizontal-middle word regardless; if the vertical-middle word were `middle`, the set would
spell the same concept two ways depending on axis. One word for one concept, one grammar
generating all nine. CSS `transform-origin` and `background-position` use `center` on both
axes; `middle` is the HTML `valign`/ASS lineage. The fixture already writes `center-left`
on `chip-text` and `handle-text`.

**`anchor` carrying a string is a schema error naming `origin`.** `CONTEXT.md` records the
collision as *"near-certain to be rediscovered"* — every comparable tool calls the nine-way
point an anchor, and Montaget spends that word on layer-relative stacking. The error fires
on **shape, not presence**: `anchor` carrying a below/above object is the existing feature
and stays legal.

### What `validate` checks

**Aperture coverage is an `error`.** The declared rect must contain `clip`. This is
computable from `x`, `y`, `origin`, `width`, `height` and `clip` — **all in the document,
no probe required** — so it passes 8 of 8 image elements today without opening a source
file. The consequence of failing it is background showing through where the author
declared full cover. Note the structural property: **the error-severity check needs no
media, so an unprobeable source can never suppress an error.**

**Fit deviation is a `note`, and never an `error`.** When the declared rect disagrees with
the value this ADR's rule computes from the source on disk, `validate` names the declared
value, the rule value, the source's actual dimensions, and **the distortion as a ratio**:

```
photo-06: declared 1912; cover from images/06.png (1536x2200) is 1546; 23.6% anisotropic stretch
```

The case that justifies this check is not a hand-typed 1913. It is a **replaced source**:
swap the file for a differently-sized image and the declared rect silently means a
different crop forever, with every number in the file still looking right — ADR-0012's own
*"it fails quietly because a centre-cropped photo looks plausible."* Nothing else in the
tool surface catches it. On a correct file the note fires zero times, so it costs nothing
per run, and unlike ADR-0006's union-of-visual-coverage rule that zero is verifiable
against committed data rather than asserted.

**`render` prints the fit-deviation notes** rather than only refusing on errors. ADR-0006
already has render run the identical checks; printing this one announces a stale source at
the moment the video is made. Non-blocking, non-ignorable. A `--strict` flag promoting
notes to errors is **rejected**: ADR-0006 killed opt-in checks by name, two of eight agents
ranking one below doing nothing.

**`fmt` must never rewrite a declared extent.** ADR-0011 defines `fmt` as rewriting the
file in the canonical *convention*. Under declared-rect-authoritative the integer is
**content**, so a `fmt` that rewrote 1913 → 1912 would be a formatting tool that changes
the output video. That is disqualifying on its own; the exact-string-replace trap is the
second reason, not the first.

**An unprobeable source is `UNCHECKED`.** Missing file, permission denied, a URL that
cannot be fetched — the disk-agreement half of validate's question is *unanswerable*, not
*failed*. Reuse ADR-0012's frame-change-census vocabulary rather than coining a word, and
**put the unchecked count in the summary line**, because `0 errors` over a file where
nothing could be probed is exactly the false-confidence failure ADR-0006 was written
against. The asymmetry that makes this clean: `render` must decode the source to draw it,
so at render time the check is always answerable — UNCHECKED is a `validate`-only category.

## Consequences

- **`query`'s crop rectangle is now computable and exact.** ADR-0012 unblocked it in shape;
  the integer rule makes the number reproducible across implementations.
- **ADR-0006 gains three findings**: the aperture-coverage error, the fit-deviation note,
  and the UNCHECKED category with its summary-line count.
- **`migrate.py` is corrected in this change** and its docstring now cites this ADR instead
  of describing the contradiction as open. Fixing it changes **zero bytes** of the committed
  fixture — verified by regenerating and byte-diffing — because the fixture's only
  non-trivial ratio is dyadic. `verify.py` gains a check that recomputes every image
  element's `width`/`height` by this rule, which is what turns the fixture from *consistent
  with* the rounding rule into *evidence for* it.
- **A fixture that passes proves the fix is safe here; only the scan proves it is
  necessary.** Both are committed, per `docs/agents/domain.md`'s *"commit the evidence an
  ADR rests on."*
- **`gravity` gets a measured fact, not a decision.** ADR-0012 left *"whether `gravity`
  survives at all"* open on a split jury with no measurement. Under the settled drawn-rect
  reading, `gravity` is **inert on 8 of 8 image elements** in the only real project file —
  the declared rect and `clip` together already determine which part of the source
  survives. That number is handed to [#13](https://github.com/MBehtemam/Montaget/issues/13)
  and [#21](https://github.com/MBehtemam/Montaget/issues/21). It is not decided here.
- **An author who wants deliberate anisotropy** writes the rect at the rule value and puts
  the distortion in `scale`, which is `[sx, sy]` precisely so anisotropy can be stated —
  `1546` with `scale:[1.0, 1.236740]`. See *Not settled here* for why this is not yet good
  enough to hang an error on.

## Not settled here

- **The severity of a deviation larger than one integer step.** The check above reports
  every still-covering deviation as a `note`, including the reproduced stale-source case at
  23.6% stretch. Making large deviations an `error` is defensible — a rect contradicting
  its own declared `fit` given the media on disk is the document contradicting itself, which
  is ADR-0006's charter — but it is **blocked on the `fit` vocabulary**
  ([#48](https://github.com/MBehtemam/Montaget/issues/48)). `cover` is the only value that
  exists, omission is undefined, and so there is no legal way to spell *"do not fit this,
  use my rect."* The one available escape — expressing the stretch in `scale` — **collides
  with the animation channel on 7 of 8 image elements**, which already animate `scale`:
  spelling static geometry there bakes the stretch into every Ken Burns keyframe
  (`[1.0,1.236740]` → `[1.08,1.335679]`), so base geometry and animation share one channel
  and neither stays readable. An error whose named replacement is wrong for 87.5% of the
  corpus fails ADR-0012's own *"a schema error naming the replacement"* pattern.

  `note` is correct under both futures and tightening later breaks **0 of 8** committed
  elements, so it ships now. Two shapes were considered for the eventual error and are
  recorded: *deviation greater than one integer step*, and *membership in
  `{floor(exact), ceil(exact)}` on the slack axis with zero grace on the driving axis*. The
  second is better — a step count is not scale-free either, since the same 2-step deviation
  is **0.105%** on `photo-06`'s 1912 and **2.941%** on `handle-logo`'s 68, a **28x** spread
  in one file — but neither can ship until #48 provides the escape. The stale-source case is
  committed to the scan script so #48 inherits a re-executable artifact rather than a
  paragraph.
- **Whether `fit` may be omitted, and what omission means.** ADR-0012's defaults list does
  not include `fit`, nothing requires it, and `verify.py` asserts `gravity` and `clip` on
  images but not `fit`. #48.
- **`contain`, and the rest of the `fit` vocabulary.** #48.
