---
status: accepted
amends: 0002 (fonts carve-out), 0006 (overflow check gains a box and a width term)
amended-by: 0029 (baseline placement within the line slot)
---

# Text is styled runs at a literal size, in fonts the project declares

> **Amended by [ADR-0029](./0029-line-baseline-half-leading.md)**, which states the
> baseline-placement rule this ADR's slot definition never gave: half-leading,
> `baseline_y = slot_centre_y + (ascent − descent) / 2`, read from the max ascent
> and max descent across every run on the line rather than the single run that
> sets the slot height. No schema change; `measure` reports the resolved value.
> Every other decision below stands.

> **Amended by [ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md)**, which
> closes this ADR's open item: the run style-delta set is settled, and **outline is a
> `stroke`/`stroke_width` paint field** — on text it falls *outside the glyph contour* and
> grows into the declared box rather than enlarging the element. The box stays **required**,
> on a reason this ADR did not have: an omitted `height` would be indistinguishable from a
> decision not to check. Note also that **`ScaledBorderAndShadow: yes`, cited in this ADR's
> open item, is an inert default** — all 35 fixture styles set `Outline: 0, Shadow: 0`, so
> the fixture carries no stroke evidence at all. Every other decision below stands.

> **Amended by [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)** on
> two spellings; every decision below stands. **`box` is no longer the id of an element to
> fit inside** — it is literal `width`/`height` on the element, because fifteen of the
> fixture's twenty-two text elements have no element behind them to name, and a required id
> would have them naming something that does not exist. The argument for the box (it must be
> *in the document* so the overflow check has an input) is unchanged and better served.
> **`align` is now text-only**: on an image the same word meant which part of the source
> survives the crop, which is `gravity`. **[#72](https://github.com/MBehtemam/Montaget/issues/72)
> updated the worked example below to the post-ADR-0012 shape** — `box` dropped, literal
> `width`/`height`/`align` added, and `start` corrected to the committed fixture's value —
> since the code block still carried the pre-ADR-0012 field a copy-paste would reintroduce.

A `text` element carries a base style and an ordered **`runs`** array. Each run is
its own text plus style deltas over the base. Size is a **literal number**; the
renderer never chooses a size and never chooses a line break. Fonts are **files the
project declares by path** in a top-level table, and the renderer opens nothing
else.

```json
{"id":"word-05","type":"text","start":5316,"end":17472,"x":540,"y":1373,"origin":"center","width":984,"height":97,"font":"brand","size":88,"line_height":1.1,"color":"#245C8C","align":"center","runs":[{"text":"cobweb  -  cobweb"}]}
```

## What the primitive must not preclude

**No assumption of one font, one direction, one script, or one style per element.**
The fourth clause is the one that was missing, and it was not found by reasoning
about internationalisation — it was found in plain English typography. In the one
real project ever authored for this tool, **every** sentence event overrides its own
declared style's size inline (`\fs55`, `\fs57`, `\fs58` against a style default of
76), and one event carries two sizes across two lines inside a single subtitle
event. The author could not express that as one element, split it in two, and hand
computed both baselines — which is [ADR-0004](./0004-tracks-as-constrained-lanes.md)'s
worst finding and [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)'s
invisible defect class, arriving together in a Latin-script short.

## Decisions

**Runs, always an array.** No bare-string shorthand. Six of seven judges reached
this independently, and on one argument: *the shorthand optimises every edit except
the one that actually occurred.* Adding a second style to a flat string is a shape
change — delete the value, synthesise an array, **retype the original string** — at
precisely the moment the author is doing something typographically delicate.
Under runs-only it is a splice into an existing array and the string is never
retyped. Element-level `text` is a schema error with a message naming `runs`.

**A run boundary is style-only; a line break is `\n` inside a run's text.**
Confirmed from the fixture, which refutes the alternative directly: of its four
multi-line text events, **three break with no style change at all**
(`How do you say you hang\Ncobwebs over the door?` under a single `\fs73`). Only one
has the break coincide with a restyle. Reading that coincidence as the rule would
make the ordinary reference-class edit — emphasise one word mid-line — inexpressible.

**The whole element, runs included, serialises on one line.** [ADR-0005](./0005-absolute-integer-milliseconds.md)'s
convention exists so a unique matchable substring exists per element; a serializer
that pretty-prints a nested array repeals it *precisely for the elements where edits
are most delicate*, and repeals it by a library default nobody chose. No length
threshold: a conditional serializer reintroduces the shape-discrimination cost that
killed the shorthand, and makes formatting non-idempotent — add three characters,
cross the threshold, and the next write reflows the element into eight lines with a
diff that has nothing to do with the edit.

**Literal `size`. No fit-to-box. No automatic wrapping.** This overturns [#9](https://github.com/MBehtemam/Montaget/issues/9)'s
conclusion that fit-to-box is "the format's first legitimate computed property", and
the reason is that its premise is false: fit ships *some* of its inputs. The ones it
does not ship — the shaper, the font binary, the UAX #14 line-break data — live in
the renderer. A property whose value is HarfBuzz's opinion in the version you happen
to have installed fails "understandable by reading" for the same reason an expression
language does. With runs it is additionally incoherent (shrink which run?). The
fixture is already this design: all seven subtitle files carry `WrapStyle: 2` — ASS's
*no word wrapping* — with every break placed by hand.

**Stated as a cost, not softened: this makes `frame`/`measure` mandatory in the text
authoring loop**, not the optional self-check this project's map describes. The
fitted sizes in the fixture (88, 80, 73, 55, 49, 35…) are unknowable from the
document; they came from a previous pipeline that measured and wrote literals. That
loop does not disappear — it moves to authoring time and freezes its result in the
file, where a shaper upgrade cannot move it.

**`line_height` is an element-level multiplier.** A line's height is *the largest
`size` among the runs on that line* × `line_height`; the block of lines is the sum,
and the block is placed according to `origin`. Per-run leading was rejected as
incoherent when two runs share a line — CSS's max-of-them rule is a decades-old
confusion generator. This **generalises** ADR-0006's overflow arithmetic rather than
repealing it: the single-size case degenerates to the old formula exactly, and the
worked example there (size 55, `line_height` 1.1, centred on 1537 → 1506.75–1567.25)
reproduces. `line_height` defaults to 1.2 when omitted.

### Fonts

**A project declares its fonts in a top-level table with semantically loaded keys**,
each an ordered fallback chain of files:

```json
"fonts": {"brand": [{"file": "fonts/Inter-SemiBold.ttf"}],
          "brand+fa": [{"file": "fonts/Inter-SemiBold.ttf"}, {"file": "fonts/Vazirmatn-SemiBold.ttf"}]}
```

- **Always a file path, relative to the project. Never a system family name.**
  Verified rather than assumed: the fixture's `SF Pro Rounded` resolves on its
  author's machine only because it was hand-installed into `/Library/Fonts`; macOS
  ships `.SF NS Rounded` under a private name. The one real project ever authored
  names a font a clean machine cannot find.
- **No `weight`, no `bold`.** A different weight is a different file. With one
  declared file and no family to search, `bold: true` can only mean synthetic
  emboldening, which is renderer-specific. The real project file carries
  `"weight":"bold"` on all 22 text elements, so every agent's CSS priors will
  reach for it — the schema must **reject** it naming the replacement, never ignore it.
- **`index`** for `.ttc` collections, defaulting to 0. 49 of the fonts in a stock
  macOS `/System/Library/Fonts` are collections; this is the normal case.
- **No variable-axis fields in v1.** `cosmic-text`'s variable-axis support is an
  open issue, and a field the renderer cannot honour is worse than no field — it
  reads as declarative and is silently ignored, which is the failure this format
  exists to prevent. Recorded as deferred, not forgotten.
- **Fallback walks the chain and stops.** The renderer never consults system fonts.
  This is implementable at zero cost on the leading candidate: `fontdb::Database`
  starts empty, and `load_system_fonts` is a call you simply never make, which
  renders `cosmic-text`'s per-platform fallback lists inert. A character with no
  glyph in any chain entry renders `.notdef` and is a `validate` **error** — under
  ADR-0006's definition it is *guaranteed wrong*, and unlike a gap there is no
  intent it could express.

### Text and internationalisation

**v1 renders bidi reordering, complex-script shaping, per-character font fallback,
and UAX #14 line breaking including CJK.** Not prioritised — *unavoidable*. Verified
from the crates' own manifests: `cosmic-text` requires `unicode-bidi`, `harfrust`,
`unicode-linebreak` and `fontdb` non-optionally; `parley` documents shaping,
line-breaking and bidi-reordering in its `Layout`. Shipping "LTR-only v1" would mean
writing code to disable capability already linked in.

**Deferred but not precluded:** vertical writing modes, and dictionary segmentation
for UAX #14 class `SA` — Thai, Lao, Khmer, Burmese — which the standard itself names
as requiring morphological analysis "beyond the scope of the Unicode Standard".
**The candidates fork here and it is a live input to [#7](https://github.com/MBehtemam/Montaget/issues/7):**
`unicode-linebreak` documents a tailoring resolving `SA` to ordinary alphabetic
*regardless of General_Category*, so under `cosmic-text` Thai breaks at spaces only —
silently and plausibly wrong. `parley` declares a `complex-scripts` feature, backed
by ICU4X's LSTM models, that covers exactly those scripts. Neither has been run.

**`dir`** is an optional per-run override, `"ltr"`/`"rtl"`, with **isolate**
semantics only — never LRO/RLO overrides, which exist for legacy data and turn "my
text renders backwards" into an unfalsifiable mystery. It is not a new capability;
the bidi algorithm already takes an explicit paragraph level as input. The failure
it prevents is silent, invisible to any static check, and otherwise expressible only
by embedding invisible control characters in a string — in a format whose editing
model is exact-string matching against what you can see.

**`origin`** is the nine-way point of the text box that `x`,`y` places. **`align`**
is separate — `start`/`center`/`end`, how lines align to each other — and uses
start/end rather than left/right because RTL is in v1. ASS's `\an5` conflates the
two; keeping them apart is a bigger latent saving than the naming collision itself.

**Text bytes.** UTF-8, **NFC**, written as raw characters — the serializer never
emits `\uXXXX` above U+007F, because escaping destroys the unique-substring
guarantee *uniformly* rather than occasionally. **NFKC and NFKD are forbidden** to
every writer: canonical equivalence is a rendering-neutrality guarantee by design
(HarfBuzz re-normalizes internally to what the font supports), while compatibility
normalization demonstrably eats NBSP, ideographic space, Arabic presentation forms
and ligatures. **No tidying pass, ever** — whitespace inside a run is content, and
the fixture's `cobweb  -  cobweb` carries deliberate double spaces that no
normalization form touches but every tidier would. Two honest exceptions recorded:
HarfBuzz's Myanmar shaper requests no normalization, and Devanagari `0928 093C`
composes under NFC while U+0958–095F never do.

### The box

**A text element names the box it must fit inside**, and this is a *required*
consequence of literal size rather than a later refinement. ADR-0006 commits
`validate` to "text overflowing its own box, computed from the document" and parks
it pending this decision — but **the fixture models the card as a separate shape
element**, so the check as written has no input. It also names only vertical terms
(`size`, `line_height`, a `y` that means centre) and a top-edge example, so a line
that is simply too wide has no check at all.

Four agents then authored with this format and produced the same defect by three
different roads — lengthening a string, swapping the font, emphasising a word —
each time leaving every property individually valid. One of them named it exactly:

> the format has no representation of the box the text has to fit in — so the one
> property that determines whether the output is usable is the one property the file
> cannot state, `validate` cannot check, and the renderer will not enforce.

So `box` is part of the primitive, and ADR-0006's check gains a **width** term
alongside its height one.

## Evidence

Seventeen agent sessions across four rounds, each blind to the author's write-up.
Three juries reviewed the design; a fourth **used** it.

**The use exercise is where the design was actually tested.** Four agents authored
the fixture's text elements from the raw subtitle files and performed four edits.
Results:

- **4/4 invented `line_height` values and 4/4 invented font paths** — no font file
  exists anywhere in this repo, so every syntactically perfect `fonts` table an
  agent can write today is unverifiable and broken at render.
- **2/4 shipped segment-local times onto the absolute timeline**, and both "resolved"
  the resulting collision **by putting the elements in different tracks** — using a
  container ADR-0004 defines as *stacking, never timing* to paper over a timing
  collision, one of them writing "that's a workaround for missing data, not a real
  fix" while doing it.
- **4/4 flagged the worked example in the spec as actively teaching the bug**, because
  it carried segment-local times and the card's background colour on a text element.
  *"A worked example that is 90% ground truth and 10% fabricated is worse than one
  that is clearly synthetic — I had to actively distrust a document example, which is
  backwards."* This ADR's example is corrected accordingly.
- The font swap — one line under a fonts table — **silently invalidated every
  hand-tuned size and hand-placed break in the document**. The table is therefore
  adopted *with* a census and a font-swap finding, not as a bare win.

**Setup flaw, recorded rather than glossed:** all four agents were given the same
scratchpad path, so two of them had their files overwritten by the others mid-task.
Their later tasks are contaminated and only their first task and narrative were
scored. The flaw was caught by one of the contaminated agents, not by the author.

## Consequences

- `validate` gains: font-file resolution, glyph coverage across the chain
  (`error`), a **font census** (*"23 elements use `brand`; 1 uses `brand-old`"*), a
  **font-swap** finding naming which measured layouts are now unverified, a
  **grapheme-cluster** check that no run boundary splits a base from its combining
  mark, an **invisible-character** census (ZWJ/ZWNJ, RLM/LRM, variation selectors),
  and a **mixed-normalization** finding. Font files join ADR-0006's
  `(path, size, mtime)` probe cache — a font swapped in place is a silent
  whole-project render change that no census sees.
- A `fonts` discovery tool answers the authoring friction: it lists system fonts with
  paths and face indices and **vendors** a chosen one into the repo. The
  nondeterminism lives in a tool called once at authoring time whose entire output is
  a committed file. Generalised: *wherever this format refuses a convenience, the
  convenience belongs in an authoring-time tool whose output is inert.*
- **Font licensing is a first-class outcome, not an edge case.** The fixture's own
  font is not redistributable. A project depending on an unvendorable system font
  *is* machine-dependent, and the tool must say so and offer substitutes rather than
  let users vendor illegally or quietly regress to family names.
- `measure` (string + font + size → advance width, ascent, descent, line count; and
  inverted, string + font + box → largest size that fits) is the tool that makes
  literal size affordable. It is the identical arithmetic fit-to-box would do, run at
  authoring time, with the answer frozen in the file.
- Open, and belonging elsewhere: the full run style-delta set (colour, outline —
  with the ASS **BGR-with-alpha-nibble** trap and `ScaledBorderAndShadow` written
  down), and stable unique element ids, which `anchor`, `validate` findings and
  `measure` all already presume.
