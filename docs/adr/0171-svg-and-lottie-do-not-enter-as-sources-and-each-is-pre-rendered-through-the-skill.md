---
status: accepted
amends: 0027 (discharges its "reopens as a fresh scope question" clause: the question was asked and both formats are refused, each on grounds recorded here, with its reopen triggers), 0156 (§7's pre-render route widens from a code-drawn piece to an SVG still and a Lottie animation, under shared size and text rules)
---

# SVG and Lottie do not enter as sources, and each is pre-rendered through the skill

[#790](https://github.com/MBehtemam/Montagent/issues/790), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663): capability 12, the map's last.
[ADR-0027](./0027-vector-sources-out-of-scope.md) ruled vector sources out of v1 and said the
question "reopens as a fresh scope question" if vector support is proposed. Since then the map
admitted the entry test ([ADR-0145](./0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)),
a `path` element ([ADR-0154](./0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md))
and the pre-render route ([ADR-0156](./0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7).
The research for this ticket is
[`docs/research/vector-sources-789.md`](https://github.com/MBehtemam/Montagent/blob/research/vector-sources-789/docs/research/vector-sources-789.md)
(branch `research/vector-sources-789`, [#789](https://github.com/MBehtemam/Montagent/issues/789)).
It reads published sources and ran nothing.

Settled by two `/court` rounds of three jurors each (Opus, Sonnet and Fable), judged by how agents
would work with each choice, with the owner ruling with the Judge's read. The first round was
unanimous on all three questions. The second was unanimous on §5 and §6 and split two to one on §4,
where the owner took the two-slice reading. The jurors shared one framing of the options, so their
agreement is weak evidence on its own. The grounds below stand on repository rules and published
facts, not on the vote.

## The decision

### 1. SVG does not enter as a source

No element accepts an SVG file. The grounds, in ADR-0027's three terms and one new one:

- **The pin.** Skia's `svg` module needs a new CPU-only prebuilt key on all six targets (none is
  published; the CPU-only `svg` key covers macOS only) and pulls in `textlayout`. `skia_pin.rs`
  forbids `textlayout`, because [ADR-0010](./0010-skia-safe-rasterizer-text-beside-it.md) refuses a
  second shaper: the one `measure` reports would not be the one that draws. Taking the route reopens
  ADR-0008 and ADR-0010 as well as ADR-0027.
- **A parallel backend.** `resvg` shares `tiny-skia`, `harfrust` and `skrifa` with Montagent, but
  drawing through it is a second raster backend, which the workspace pin names as the thing never to
  have. Its "identical on every platform" claim is a design goal: its CI runs the tests on Linux only,
  with a tolerance of one level per channel, and its filters use `powf`, `sin` and `cos`, which Rust
  does not promise to be deterministic. The map's byte-identical painting rule cannot be shown to
  hold on that evidence.
- **The reference class.** Neither Premiere nor CapCut imports SVG natively in any source read
  (Adobe's own format page refused the fetch, and CapCut's desktop list is an image). Under ADR-0145
  that is no precedent, so the entry test would need an owner-accepted prototype, and every route
  to one costs one of the two grounds above.
- **The size.** An SVG root may carry no absolute size. Skia reports 0×0 when either dimension is a
  percentage and never reads `viewBox`; `usvg` takes the size from `viewBox` and otherwise falls back
  to 100×100. The two disagree on the same file. See §6.

### 2. Lottie does not enter as a source

Decided separately, on grounds that are not SVG's:

- **The pin.** `skottie` has the same `textlayout` problem and appears only in a Linux key that
  carries GPU code.
- **No safe player.** `velato` has no text, images or stroke dash. `rasterlottie` is a young
  renderer that rejects text animators, layer effects and expressions in its default profile.
  ThorVG ignores a frame change under 0.001, so a seek depends on the previous seek and parallel
  painters would not agree. Skottie's seek mutates its animation object and its "randomize order"
  text shuffles with a library-defined algorithm. `rlottie` is archived.
- **The clock and the text.** A Lottie carries its own frame rate and in/out points, so a source
  element would need a rule reconciling that clock with the timeline's absolute integer
  milliseconds ([ADR-0005](./0005-absolute-integer-milliseconds.md)). The published Lottie spec (1.0.1) has no text layer, so text is
  defined by what After Effects exports and players accept, and expressions are literal-value
  violations by ADR-0145.
- **The reference class.** Premiere does not import Lottie JSON. The one documented path is
  LottieFiles' extension, which delivers pre-rendered MP4 or transparent MOV: footage.

### 3. Both are pre-rendered through `montagent-prerender`

An SVG becomes **one PNG in an ordinary `image` element**. It needs no footage, no codec and no
frame hash beyond its own, and the rasteriser runs once, outside the render, so every painter
decodes the same PNG and painting stays byte-identical by construction. A Lottie becomes
**PNG-in-MOV** (`-c:v png -pix_fmt rgba`, measured pixel-exact by
[#723](https://github.com/MBehtemam/Montagent/issues/723)) in an ordinary `video` element.

What the agent pays: a fixed resolution (a zoom past it resamples a raster), a recipe toolchain to
reproduce the piece (not to render the project), and footage size for Lottie, about 6 MB per second
smooth to 141 MB per second noisy at 1080p. The route needs no format change, so it keeps all four
invariants.

### 4. Shared rules, two slices

The two routes share three rules, written once in the skill's common section, and differ in
everything else, so the work is two slices ([#810](https://github.com/MBehtemam/Montagent/issues/810),
then [#811](https://github.com/MBehtemam/Montagent/issues/811)):

- **Size.** Rasterise at the largest size the piece is ever shown, as a literal pixel width and
  height, written in the recipe.
- **Text.** Vendored fonts or outlines only, never a system font. A font that cannot be found fails
  the build; it never drops a span silently.
- **Recipe.** The skill's existing recipe, with a decoded-pixel hash and a rebuild that never
  compares file bytes. An SVG's recipe is one frame.

The Lottie route adds a harness rule: **one fresh player per frame**, because of §2's seek facts.
It refuses a Lottie whose expressions, effects or text the player skips. The capability map gains a
row per route and one "Not by design" line, "a native SVG or Lottie source", citing this ADR; the
map had no can't-do item for either, so none retires.

### 5. Considered alternatives and what reopens them

- **`resvg` as a raster at render time.** Refused by §1's second ground. It reopens only with the
  native-source triggers below.
- **An SVG-to-`path` conversion skill.** Since ADR-0154 an SVG's shapes can be hand-translated into
  `path` elements, which would stay scalable where a PNG does not. No source is needed, so every
  invariant holds. It is out of this decision: the demand is unmeasured, and a skill that refuses
  gradients, filters and text would leave the PNG route needed for the rest. **It reopens when three
  or more agent projects are found to carry `path` points copied by hand from an SVG.** Three is a
  chosen threshold, not a measured one; the owner may move it.
- **A native source for either format.** **Reopens only if a reference-class tool gains native
  import of it, or a separate decision lifts the `textlayout` ban.** Those are the two facts the
  refusals rest on. A player fixing its seek or text gaps does not reopen it.
- **No trigger of their own:** the `resvg` second-backend ground and Lottie's seek and clock grounds.
  They are covered only through the two triggers above, by design.

### 6. A recorded lean on the size rule, nothing adopted

With no source there is no rule to adopt, and ADR-0027's "not settled here" entry is discharged
with "not adopted". If a source is ever reopened, the lean is **a literal pixel `width` and `height`
on the source, never derived from `viewBox`**. It keeps the literal-value invariant, is checkable
by `validate` and editable by exact-string replace, avoids the Skia against `usvg` disagreement
above, and matches Lottie, whose `w` and `h` are always stated. It binds nothing: no schema key
exists, and a reopening decides afresh.

## Consequences

- No schema, painter or `validate` change. ADR-0027 stays accepted, with a banner pointing here;
  ADR-0156's banner gains this ADR.
- An agent handed an SVG or a Lottie has a sanctioned route with stated costs, not a refusal.
- This closes capability 12 and, with it, the map.

## Not settled here

- Animated SVG (SMIL or CSS). The static route does not cover it, and no route is chosen for it.
- Whether a pre-rendered SVG should keep its source beside the project so a larger re-render is one
  command. The recipe already carries the copy; the layout is the slice's.
