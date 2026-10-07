---
status: accepted
amends: 0007 (closes its open run style-delta item; the box stays required and gains a rule), 0006 (the overflow finding is reworded and gains a stroke term), 0013 (`fmt` may not rewrite a colour, as it may not rewrite an extent)
---

# Stroke is paint, the text box is required, and a point list has no extent

> **Amended by nine later ADRs.** Read them before relying on anything below.
>
> - [ADR-0015](0015-fit-is-a-derivation-claim-and-gravity-retires.md) — `gravity` is
>   decided here, as that ADR deferred it
> - [ADR-0028](0028-text-block-arithmetic-is-exact-tenths.md)
> - [ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md) — confirms stroke stays a
>   paint field, not an effect member
> - [ADR-0048](0048-per-word-highlighting-is-a-timed-window-on-the-run.md) — extends the
>   run-addressable paint precedent stroke established
> - [ADR-0058](0058-text-box-slack-is-a-note-with-sibling-census.md) — closes its recorded
>   residual: "an over-large `height` disables its own tripwire... nothing in this design
>   catches it"
> - [ADR-0149](0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md) — its
>   "gradients are out of v1" is closed: a gradient enters as a paint, linear or radial, in a
>   closed vocabulary
> - [ADR-0154](0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md) — its
>   rejection of `line`, `polygon` and `path` is lifted: a point list enters as one `path`
>   element in integer pixels from the declared box; the `d`-string refusal stands
> - [ADR-0161](0161-a-text-element-bends-its-one-line-along-its-own-inline-path.md) — the
>   derived-height checks do not apply to a text on a path; its box frames the curve, inset by
>   the largest run size plus the largest stroke width
> - [ADR-0158](0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md) — `rect` and
>   `ellipse` take a dash pattern; their join and cap stay as drawn

> **`gravity` is decided by [ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md)**,
> which this ADR deferred to #48. It is **retired** — a schema error on every element type, not
> only on text and shapes. This ADR's one owed clause stands and is now the general case.

Three decisions the shape primitive and the text primitive were still missing.
[ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md) gave every
visual element its placement, so `card-05`'s rectangle was already expressible; what was
not settled is **how a shape is painted**, **which shapes exist**, and whether a text
element must keep declaring the box
[ADR-0007](./0007-text-runs-literal-size-declared-fonts.md) required of it.

```json
{"id":"card-05","type":"rect","start":10468,"end":17472,"x":48,"y":1453,"origin":"top-left","width":984,"height":169,"fill":"#1E344C"}
```

That element is **unchanged**. Every decision below is additive: this ADR changes
**zero bytes** of the committed project file, asserted by `verify_facts.py` check 8
rather than claimed.

## Decisions

### Shapes are sibling element types: `rect` and `ellipse`

`type:"rect"`, not `type:"shape"` with a `shape:"rect"` discriminator inside it.
Unanimous across eight sessions, on a reason stronger than tidiness: ADR-0012 already
made the field set a **function of `type`** — *"`x` on an audio element is a schema
error"* — so a discriminator adds a second, weaker narrowing mechanism for one branch.
The concrete hazard is that a second field is **omissible**: a malformed ellipse missing
its discriminator renders as a rect, silently and plausibly, which is this format's named
failure class. `type` is also the substring an agent greps under exact-string editing.

`ellipse` is in v1 rather than deferred, and the fixture is the argument *for* it: the
header's circular badge is baked into an 800×800 RGBA asset with corner alpha 0, so the
geometry lives **outside the document** in a PNG no reader can inspect. That is the
disease the format exists to treat, and it is what a missing `ellipse` costs.

An ellipse inscribes its declared rect. It needs no new placement rule, which is exactly
what distinguishes it from the shapes below.

### `line`, `polygon` and `path` are rejected, because a point list has no declared extent

Not "unevidenced". The reason is load-bearing and it is **not** the one first proposed.

The tempting argument — *a thin rotated `rect` already draws any segment* — is false in
the way that matters: the endpoints never appear in the file, so the author maintains by
hand the trigonometry the format was supposed to state. The second tempting argument,
that a point list needs a placement grammar no transform covers, was **falsified during
the review**: a viewBox-style local coordinate space covers a point list with no new
transform rule at all.

What kills it is that such a space needs one of two things, and both are already closed.
Either a **second unit system** — against ADR-0012's absolute-integer-pixels decision and
its fractions census — or an extent **derived from the content**, which ADR-0012 bans
outright and ADR-0013 re-bans for fitted extents. So:

> A point list has no declared extent. Admitting one is a new ADR about placement, not a
> schema addition.

That is the reopening condition, stated so it is falsifiable. `path` is additionally a
mini-language inside a JSON string — unreadable by reading, unmatchable by exact-string
replace, and the shape half of the After Effects class
[ADR-0003](./0003-general-video-editor-not-channel-tooling.md) ruled out. Commit an SVG
or a PNG instead.

**A corroboration, explicitly demoted from a premise.** The fixture's ASS pipeline had
cubic beziers available and drew six axis-aligned rectangles with no `b` command
anywhere. That is six drawings by one author, and ADR-0003's asymmetry — *the channel is
evidence a capability is needed, never that one is unneeded* — forbids using it as a
reason. It is recorded because it is interesting, not because it decides anything.

### Stroke is a paint field on the primitive, not an effect

`stroke` (a colour) and `stroke_width` (an integer) on shapes; on text, additionally
available as a **run** style delta. The boundary against
[#22](https://github.com/MBehtemam/Montagent/issues/22)'s effect vocabulary is:

> A stroke is a second paint on the same outline, run-addressable, that never enlarges
> the declared rect.

That formulation replaces the one first proposed — *"paint is consumed while the element
rasterizes; an effect operates on pixels already drawn"* — which was **rejected during
review because it equally admits shadow and blur**,
[ADR-0010](./0010-skia-safe-rasterizer-text-beside-it.md)'s obvious members of #22. The
replacement excludes them non-arbitrarily: a blur is not a paint on the outline, and a
drop shadow is not addressable by run.

The decisive positive is a document fact: **ADR-0007's own open item already parks
"outline" inside the run style-delta set**, so treating stroke as an effect relocates
something a settled ADR had scoped. And it must be run-addressable, because #22 attaches
effects to **elements** while a run lives *inside* one — under the alternative,
*"outline one word"* is not expressible at all.

**Geometry — one principle, two consequences, because a text box is not drawn geometry.**

- **Shapes: the stroke falls inside the declared rect.** `card-05` with
  `stroke_width: 8` still occupies exactly 984×169 at (48,1453). Centred or outside, it
  would occupy 992×177 at (44,1449) — eating 4 px of the 48 px margin the layout rests
  on, while every number in the file still reads 48/984/169. That is ADR-0013's
  disqualifying *"what is drawn is a number not in the file"*, re-entering through paint
  after being shut out of geometry. Inside also needs no half-pixel tie-break on odd
  widths.
- **Text: the stroke falls outside the glyph contour**, the ASS `\bord` model, because
  inside or centred erases letterforms by thinning the stems. It does **not** enlarge the
  element: a text element's `width`/`height` is a *container claim*, not painted
  geometry, so the stroke grows into the box and the overflow extent gains
  `2 × stroke_width` on both axes.

  This asymmetry is written down rather than left to be inferred, because two reviewers
  independently rejected the assumption that one rule covers both — and both were right.

**Stroke is in element space and scales with `scale`.** `stroke_width: 8` under the
fixture's Ken Burns ramp is **8.64 px** at the 1.08 peak. The worked number ships with
the rule because a scaling convention nobody wrote down is how ADR-0005's `speed`
became a self-consistent guess that no validator could falsify.

### A colour is `#RRGGBB` or `#RRGGBBAA`, and nothing else

No 3-digit shorthand, no CSS names, no `#RRGGBBFF` — each is a schema error whose message
names the canonical form. Two spellings of one value break the write-read round trip:
`fmt` normalises on write and the agent's next exact-string replace finds nothing, which
is precisely why `center-center` is an error naming `center`. **Case is part of the
spelling** and no prior document covered it: hex digits are uppercase.

**Alpha earns its place independently of stroke.** The initial framing bundled the two;
that was half wrong, and two reviewers said so independently. The strongest argument
never mentions stroke — it is ADR-0013's own objection, one field over: `opacity` is the
**animation channel**, and spending it on a static per-run transparency poisons the
channel exactly as a static value poisons a keyframed one. A translucent fill under an
opaque stroke is the second case; per-run alpha is the first, and neither is expressible
with one whole-element keyframed scalar.

**`fmt` may never rewrite a colour**, including converting between the six- and
eight-digit forms. This is ADR-0013's clause for declared extents, extended: under
declared-authoritative, the spelling is content.

The ASS `&HAABBGGRR` convention — byte-reversed, with alpha *inverted* so `00` is opaque
— is documented here as the trap it is. Montagent's spelling is RGB order with `FF`
opaque. Every colour in the fixture was converted out of the ASS form by hand.

### `radius` is a field on `rect`; `fill` becomes optional once `stroke` exists

`radius`, a single integer, defaulting to 0. The fixture is measurably square — the
README's "rounded cream panel" was wrong and is corrected in this branch — and under
ADR-0003's asymmetry that is **not** evidence against the field. A rounded rectangle is
unremarkable in the CapCut/Premiere reference class, and admitting it now costs one
clause where admitting it later is a schema change.

`fill` may be omitted when `stroke` is present, giving an outlined shape. A shape with
**neither** is a schema error naming both, rather than an invisible element that renders
nothing and reads as deliberate.

**Gradients are out of v1, as unevidenced rather than rejected.** The line is clear: a
flat colour is a value, while a gradient with stops is a paint *description* — a second
notation inside the document, and the same objection that rejects `path`. If it arrives
it will be a named entry in a closed vocabulary, not an open syntax.

### `gravity` is not decided here

It is *which part of the source survives the crop*, meaningless except as a modifier of a
fit rule whose vocabulary [#48](https://github.com/MBehtemam/Montagent/issues/48) owns and
which ADR-0013 deliberately scoped to `cover` alone. Deciding it here would create a
schema value by implication — the refusal ADR-0013 made by name. Unanimous across all
eight sessions.

This ADR owes exactly one clause: **`gravity` on a text or shape element is a schema
error**, and for the positioning sense the message names `origin`.

### The text box stays required, and this is the decision that changed under review

`width` **and** `height` remain required on every text element.

The case against was measured and looked strong. For **15 of 22** text elements the
declared `height` is exactly `ceil(size × line_height × line_count)` — computed from the
very typography it is meant to bound — while `width` is externally sourced in **all 22**.
The reading was that the height term is dead on 68% of the only real project file.

**That reading was falsified twice, in different directions.**

First: the equality holds *at the instant of authoring*. The number is frozen, so the
next edit is what it exists to catch — add a `\n` without updating it and the check fires
correctly. It is not dead, it is armed and unfired. Literal `size` is the same
frozen-snapshot object, and ADR-0007 kept it deliberately.

Second, and decisively — this is [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)'s
own rule applied to a new field:

> An omitted `height` is indistinguishable from a decision not to check.

That is the `sequence` label that ADR-0004 rejected and ADR-0006 spent itself defeating,
re-entering as an optional field. It spends `validate`'s one structural defence — that it
checks uniformly, so a clean run means something. The edit census is one-sided: for the
edits only the height term catches, required catches 22/22 and optional catches 7/22;
every edit neither catches — lengthening a line, swapping a font — both miss identically.

**The argument that made optional look right is real and is answered, not dismissed.**
The apparent collision with *"an element's size is declared, never defaulted"* is
**false** in both directions: ADR-0012 bans defaults whose **inputs are not in the
document**, and a derived height's inputs (`size`, `line_height`, the `\n` count) are all
on the same line. The vertical axis never touches the shaper — ADR-0007 defines a line's
height as the largest `size` on it × `line_height`, and ADR-0008 fixes the line count
from mandatory breaks (BK, CR, LF, NL), which is character classification. ADR-0007's own
worked example computes 55 × 1.1 = 60.5 → 1506.75–1567.25 with no font at all. So
*"declared, never defaulted"* does not decide this question; ADR-0006's opt-in rule does.

A third option — require both, and emit a finding when `height` equals the derived value
— is **dead in all eight sessions**: it fires 15 findings on the only correct project
file in existence, which is the alarm fatigue ADR-0006 exists to prevent.

**The two overflow axes are not alike, and the report must say so.** Height overflow is
arithmetic on numbers already in the document. Width overflow needs `measure` against the
current font binary, and all 22 widths are externally sourced — so it is the axis that
goes stale silently whenever a font or a string changes, and it belongs in ADR-0013's
`UNCHECKED` category unless `measure` has been run.

## Consequences

- `validate`'s overflow finding, parked by ADR-0006 and specced by ADR-0007, **must not
  say "text overflows its box"** — 15 of 22 elements have no box in any meaningful sense.
  It reports the declared extent against the computed one, as facts, per ADR-0006's
  findings-state-facts rule. Its width term counts as `UNCHECKED` until measured.
- **`measure` must return the stroked extent**, not the typographic one. If it returns
  stroke-naive numbers, every author adds `2 × stroke_width` by hand and they diverge —
  ADR-0005's `speed` failure exactly: a convention each party re-derives, where a
  self-consistent wrong guess is unfalsifiable.
- `measure` also returns the derived height, so an author can fill the required field
  without hand-arithmetic. Required-and-computable is only affordable if the tool
  computes it.
- **Recorded residual, unsolved:** an over-large `height` disables its own tripwire, and
  is indistinguishable from a genuine container claim. Nothing in this design catches it.
- **Recorded residual, inherited:** the 7 container-copied heights are *copies, not
  bindings*. Resize `card-05` and `validate` stays green on a now-stale caption height —
  the placement-drift failure ADR-0012 named when it retired `box:"<id>"`, which
  [#24](https://github.com/MBehtemam/Montagent/issues/24) owns.
- `#22` loses a candidate: **"text background box" should be struck from its list**, since
  the fixture's navy card is already a `rect` and needs nothing from the effect
  vocabulary. `mask` — the fixture's undeclared `mask:"circle"` — stays #22's.
- The schema rejects `anchor` as a string naming `origin`, `weight`/`bold` naming the
  font chain, `text` naming `runs`, and now `gravity` on text or shape naming `origin`.

## Evidence

**Eight agent sessions across two rounds — four models (Opus, Sonnet, Haiku, Fable), each
isolated to its own output file, each blocked from the author's reasoning and from the
map and ticket.** Round one took six questions cold; round two put the three that split
back as unattributed opposing positions, with each juror required to attack its own
preferred answer before endorsing it. Briefs and all eight answers are in
`docs/research/juries/text-shape-primitives/`; `verify_facts.py` re-derives every factual
claim above and exits non-zero if one stops reproducing.

**State plainly what this is not: none of it has been rendered.** No Montagent renderer
exists, so not one of these decisions has been tested against a frame. Eight agreeing
sessions are eight arguments from the ADRs, the fixture and the published MP4 — not a
measurement. The first render is where the stroke geometry and the `2 × stroke_width`
extent get their real test.

**The author was wrong three times and the jury caught all three.**

- **A fact supplied in the brief was false.** `ScaledBorderAndShadow: yes` in all seven
  ASS files was cited as evidence that the reference stroke behaviour was known. Two
  jurors independently checked: all **35 style definitions carry `Outline: 0, Shadow: 0`**
  and there is not one inline `\bord`, `\shad`, `\3c`, `\4c` or `\alpha` override. The
  flag is a tool default governing a border and a shadow that are both zero. The fixture
  has **no stroke evidence of any kind**. Asserted as check 6 and 7 so the correction
  cannot rot back.
- **The stated primitive/effect boundary failed.** It admitted blur and shadow, which
  belong to #22. Rebuilt above.
- **The "tautological box" reading was too strong**, and the question put to the jury
  presupposed that one stroke-geometry rule covers shapes and text. Two jurors rejected
  that presupposition independently.

**Method note worth keeping.** Round one's brief carried the anti-drift rule verbatim and
one juror still argued *"all 10 shapes in the fixture are rectangles, so ship rect only"*
— the exact inversion #17 made and #19 ruled against. Round two's brief **quoted that
sentence and named the failure**, and no juror in round two repeated it. A written guard
was not sufficient; a written guard with a worked example of breaking it was. Separately,
one juror demoted the author's favourite fact — the six axis-aligned ASS drawings — to
corroboration, unprompted, on the ADR-0003 asymmetry.

**One presupposition flagged and not settled from the ADRs:** both box options assume a
text element's declared `width`/`height` frames nothing *drawn*. That is true only by
inference from ADR-0007. It is stated as a decision above — the box is a container claim,
not painted geometry — precisely so the next reader does not have to infer it.
