---
status: accepted
amends: 0013 (tiebreak (2) is false; the fit-deviation note becomes an error at strict equality), 0014 (`gravity` is decided here, as that ADR deferred it), 0012 (its published `photo-06` element carries a `gravity` that no longer exists)
---

# `fit` is a derivation claim, not a layout mode — and `gravity` retires

> **Amended by [ADR-0023](./0023-video-source-dimensions-par-and-rotation.md)**, which
> discharges the "PAR and video source dimensions" deferral below. "Source dimensions"
> is generalised from *decoded, orientation-applied* to *decoded, rotation-resolved,
> PAR-applied* — the image case is unchanged, since PAR defaults to `1:1` and EXIF
> orientation is the only rotation signal a raster image carries.
>
> **Also amended by these, not summarised above** —
> `docs/adr/README.md` carries the full *Amended by* view:
>
> - [ADR-0017](0017-closed-schema-no-escape-hatch.md) — confirms "`gravity` is a schema 
>   error on every element type" is implementable as written
> - [ADR-0024](0024-measure-writes-the-fit-repair-not-the-verdict.md)
> - [ADR-0026](0026-exact-aspect-fit-both-spellings-stand.md)
> - [ADR-0027](0027-vector-sources-out-of-scope.md) — discharges its "sources with no 
>   intrinsic pixel dimensions" deferral

`fit` was on 8 of 8 image elements in the only real project file and **no document defined
its value set**. [#48](https://github.com/MBehtemam/Montagent/issues/48) asked for the
vocabulary. The vocabulary turned out to be the smaller half of the answer.

## What `fit` actually is

[ADR-0013](./0013-fitted-extents-floor-and-the-nine-origin-keywords.md) settled that **the
declared rect is authoritative at render**: the source is resampled to exactly `width` x
`height`, always. Follow that through and `fit` **never executes**. No renderer reads it.

So `fit` is not `object-fit`. It is a **claim about how the author computed `width`/`height`**,
and its only consumer is `validate` (and `render`'s copy of those checks). It is the format's
first field that is purely an assertion — a provenance tag on two integers.

This is the load-bearing sentence of this ADR, because every other decision in it follows
from it: the value set is a vocabulary of *derivation rules*, so it needs a member meaning
*no rule*; and `gravity` names a quantity that, under an authoritative rect, does not exist.

Expect this to be mis-read. `fit` is a CSS word and every author arrives expecting behaviour.
`CONTEXT.md` records it, and the evidence that the mis-reading is real is in this ADR's own
consumer exercise: two of eight agents read `fit` as a render instruction on first contact.

## The closed value set

**`cover` · `contain` · `literal`.** No other value is legal.

| value | inequality | driving axis | slack axis |
|---|---|---|---|
| `cover` | drawn rect ⊇ box | width drives when `bw*sh >= bh*sw` | `(s_slack * b_driving) // s_driving` |
| `contain` | drawn rect ⊆ box | width drives when `bw*sh <= bh*sw` | `(s_slack * b_driving) // s_driving` |
| `literal` | *none* | — | — |

The driving axis takes the box dimension **verbatim**. All arithmetic is exact integer
arithmetic — ADR-0013's float-ULP finding (4.466% disagreement over 31,402,800 combinations)
governs `contain` identically.

**`fit` is required** on every element carrying a raster source. Omission is a schema error
naming the three values. This is ADR-0006's opt-in rule as
[ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md) applied it to the text box —
*an omitted field is indistinguishable from a decision not to check* — and it costs zero
bytes, since 8 of 8 elements already carry `fit`. `fit` on a text, shape or audio element is
a schema error.

### The box is `clip`

Nothing defined ADR-0013's `bw x bh`. It is **`clip`'s width and height**. Verified against
the fixture: source 1536x2720 into `clip` 1080x1300 gives exactly the published 1080x1912,
where the project frame (1080x1920) gives 1084x1920 and is falsified.

`cover` and `contain` therefore **require `clip`**; declaring one without it is a schema error
naming `clip` and `literal`. Under `literal` there is no box, so `clip` stays optional.

The tempting alternative — fall back to the element's own declared rect — is rejected, and
the reason is subtler than it first appears. It is **not** vacuous: it is an algebraic fixed
point for a *consistent* source but still fires on a stale one (1536x2200 against a declared
1080x1912 yields rule width 1334). It is rejected because it silently substitutes an
**aspect-fidelity** check for the **fit-to-aperture** check the author actually wrote — one
field checked against two different boxes, which is exactly the two-meanings defect that
retired `box` in `CONTEXT.md`.

### Why `literal`

The escape value needed a name and four were live. `literal` is not a coinage: **ADR-0007's
headline is *"Literal `size`. No fit-to-box."*** — the format's own established opposition
between a literal number and a fit rule, which is precisely what this value selects.

`declared` was the design jury's plurality and is **rejected on corpus evidence**: it appears
14 times as "declared rect" and 6 as "declared extent", describing *all three* values, so it
distinguishes nothing. `fill` is spent by ADR-0014's shape paint field. `none` is rejected as
a false friend — CSS `object-fit: none` means *intrinsic size*, a different thing — and is a
schema error naming `literal`, because it is the CSS-habit reach: 3 of 8 consumers reached for
`none` or `fill` before finding the legal value.

### `contain` can derive a zero extent, and there is no clamp

`contain` of a 300x7 source into a 10x10 box derives 10x**0**. This is left to the
fit-deviation error, which names `literal`. A clamp to 1 would fabricate an integer the
published rule did not produce, destroying the one property authors relied on throughout the
consumer exercise: that re-running the arithmetic reproduces the file's numbers.

## `gravity` is retired

**A schema error naming `x`/`y`/`origin` and `clip`.** ADR-0014 deferred it here; ADR-0013
measured it inert on 8 of 8 elements.

The census is corroboration, not the argument — under ADR-0003's asymmetry, *"the fixture
doesn't use it"* is never evidence against a field. The argument is **structural**: with the
declared rect authoritative, nothing is cropped in the sense `gravity` means. The rect's
position (`x`, `y`, `origin`) and the static frame-space `clip` already determine which part
of the source survives, with no degree of freedom left for `gravity` to spend.

**This is the first ADR in the chain to change fixture bytes** — 8 `gravity` deletions —
and the migration lands in the same change as the decision. That is not tidiness. All 8
consumers reached for `gravity` or `align`, and several reported copying `"gravity":"top"`
**straight from the shipped fixture before reading the spec**. Agents author by copying the
nearest example, so a retirement that leaves the field on 8 shipped elements is a retirement
in name only.

`gravity` on a text or shape element remains the schema error ADR-0014 specified.

## The fit-deviation check becomes an error, at strict equality

ADR-0013 shipped fit deviation as a `note` and blocked its promotion on this ticket supplying
an escape. `literal` supplies it, so it promotes:

**`error` — the declared rect must equal the rule value exactly, on both axes.** Not reported
under `literal`.

```
photo-06: declared height 1912; cover from images/06.png (1536x2200) is 1546
          (23.6% anisotropic stretch). Write 1546, or fit:"literal" if deliberate.
```

**This overturns ADR-0013's preferred shape and two unanimous design juries**, and the
evidence is behavioural rather than argued. ADR-0013 recorded *membership in
`{floor(exact), ceil(exact)}` on the slack axis* as the better of two candidates; sixteen
design jurors endorsed it 16–0. Eight agents then authored real elements under it, and on
**three of six tasks the group produced two different, equally legal files** — 1546/1547,
66/67, 1733/1734.

That is #44's founding complaint reproduced verbatim — *"two authors who round differently
write different files for the same picture, and nothing in either file says which is right"* —
under the design commissioned to end it. ADR-0013's headline is that **exactly one integer is
published**; a check accepting two unpublishes it. The agent that diverged on two tasks
independently named floor-versus-ceil the vocabulary's worst flaw, and changed its own
position to strict equality on seeing the data.

Strict equality breaks **0 of 8** committed elements — the same price the loose rule pays. The
tolerance bought ambiguity for no compatibility at all. The decisive authoring cost, named by
four consumers independently: *seeing `1547`, you cannot tell a deliberate choice from a stale
value from the other agent's rounding, so you cannot safely re-derive it* — and re-derivation
after a source changes is the most common edit this format has.

Note the coupling, raised by a consumer: this follows from ADR-0012 making `width`/`height`
**required and author-written**. If a tool ever writes the extent, the question dissolves.
While authors type the integer, there must be exactly one legal integer.

### The aperture-coverage error is parameterised by `fit`

ADR-0013's aperture check is hard-coded to the cover direction. A *correct* `contain` element
is smaller than its clip — 1200x400 into a 200x160 aperture gives 200x66 — and would trip it.
So the check now tests the inequality the element's own `fit` names: `cover` → rect ⊇ clip;
`contain` → clip ⊇ rect; `literal` → no aperture claim. This stays media-free and remains an
`error`.

### Source dimensions, defined

The rule's input was undefined, and it is [ADR-0005](./0005-absolute-integer-milliseconds.md)'s
`speed` divergence in a new costume. **Source dimensions are the decoded, orientation-applied
integer pixel dimensions.** A JPEG carrying EXIF orientation 5–8 transposes; the transposed
dimensions are the input.

The magnitude is why this is normative rather than an implementation note: two conforming
implementations reading the same file diverge enormously. On the worked cases in this ADR's
scan: an EXIF-6 3024x4032 source covering a 1080x1300 aperture gives **1080x1440** read raw
and **1733x1300** read oriented, a **60.5%** divergence on the width; a 1440x1080 source under
a 4:3 PAR gives 1733 against 2311, **33.4%**. ADR-0005's `speed` divergence, considered
serious enough to legislate, was **0.038%**.

**`validate` must print the dimensions it used** whenever the check fires. All 8 consumers
applied the orientation flag correctly, but 4 reported reaching for the stored dimensions
first — the rule is learnable but not reflexive, and a silently wrong 1080x1440 is
well-formed.

**PAR is explicitly deferred**, unanimously (8–0) among consumers. Raster images are
square-pixel; a PAR rule written now would be untestable speculation with no element to
exercise it. The fixture is *structurally incapable* of testing any of this — all five sources
are PNG and the only `eXIf` chunk sits on the 800x800 square with no Orientation tag — so the
scan script carries synthetic cases instead. The ADR states the scope rather than staying
silent, because silence is what made EXIF a near-miss.

### These rules are type-generic

They govern **any element carrying a raster source**, not `image` alone. ADR-0003 commits
Montagent to video clips, which have the identical source-dimensions shape; writing this
image-scoped now buys a schema change later. Video's open wrinkles are named, not solved: PAR,
container rotation metadata, and mid-stream dimension changes.

## ADR-0013's tiebreak (2) is false

ADR-0013 justified floor partly on: *"If `contain` is ever defined with the obvious semantics,
only floor is safe there."* `contain` is now defined and **ceil is equally safe** — `contain`
picks the driving axis so `exact_slack <= b_slack`, and `b_slack` is an integer, so
`ceil(exact_slack) <= b_slack` always. Measured: **zero** containment violations under either
rounding over 3,286,969 cases; they differ on 97.77% of them.

**Floor still stands**, on tiebreaks (1) additivity and (3) implementation entropy. But an ADR
predicted that a future decision would vindicate a reason, and the decision refutes it. This
erratum is the same correction ADR-0013 made to ADR-0012's over-generalised rounding sentence,
one ADR later and against itself. It also extends that ADR's own honesty section, which had
already withdrawn *"bound-preservation derives floor"* as too strong — tiebreak (2) is the
same error one paragraph further down.

## Consequences

- **ADR-0013 gains an erratum** on tiebreak (2), and its fit-deviation `note` becomes an
  `error` at strict equality. Its aperture-coverage error is parameterised by `fit`.
- **ADR-0014's one deferred clause is discharged**: `gravity` is decided.
- **ADR-0012's published `photo-06` element is reprinted** without `gravity`.
- **The fixture changes for the first time**: 8 `gravity` deletions, no other byte. `verify.py`
  inverts its `gravity` assertion and gains strict fitted-extent and `fit`-membership checks;
  `migrate.py` stops emitting `gravity`.
- **`query`'s crop rectangle is unaffected** — it never depended on `gravity`.
- **The `UNCHECKED` count becomes load-bearing.** This is the **first `error` an unprobeable
  source can suppress**, which forfeits ADR-0013's stated structural property that no
  error-severity check needs media. The property survives at `render`, which must decode to
  draw, but not at `validate`. Stated here rather than left to erode.

## Not settled here

- **Where `contain`'s slack goes.** `contain` leaves the drawn rect smaller than its aperture
  — 94 px on the consumer exercise's badge — and no idiom names the placement. It is fully
  expressible with `x`/`y`, and 4 of 8 consumers hand-computed a centre while 4 top-anchored;
  all 8 reached for a centring field first. Neither design jury could have seen this: both
  reasoned about a corpus containing only `cover`, where no slack exists.
- **PAR and video source dimensions**, deferred above.
- **No tool writes the repair.** The new error names an integer no tool produces; `measure`
  returning the fitted extent is the obvious home, mirroring ADR-0014's derived-height clause.
- **A keyframed `clip`** would make the fit box time-varying with no named instant. ADR-0012
  left `clip` keyframability open; this ADR assumes it is static.
- **Exact-aspect elements admit two spellings.** `handle-logo` is 800x800 into 68x68, where
  `cover` and `contain` agree, so two `fit` values describe one element with no canonical
  preference stated.
- **Sources with no intrinsic pixel dimensions** (SVG-like) have no rule input and must use
  `literal`.

## Evidence

`docs/research/juries/fit-vocabulary/` — sixteen design-juror verdicts over two rounds, eight
consumer worklogs from the authoring exercise, and the spec sheets the consumers worked from.
`docs/research/sample-project-migration/fit_vocabulary_scan.py` re-derives every number in this
ADR and exits non-zero if any of it stops reproducing.
