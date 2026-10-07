# Montagent

Montagent is a video editor whose project format is designed to be authored and
edited by an AI agent rather than dragged around in a GUI. A project states, by
being read, what is on screen at any given moment; Montagent renders it.

## Actors

**Montagent**:
The tool — a renderer and an MCP server. It is deterministic and contains no
model: given the same project and the same files it produces the same video
every time.
_Avoid_: calling Montagent an agent, or "the AI"

**Agent**:
An LLM client outside Montagent — Claude Code, or any other MCP client — that
authors and edits projects and calls Montagent's tools. Montagent never calls an
agent.
_Avoid_: user, bot, assistant, client (unqualified)

## Language

**Project**:
The whole video, as a single declarative document: frame size, frame rate,
background, output destination, an optional duration, and its tracks. Normally
stored as one JSON file in git, which is where its authority lives.
_Avoid_: composition, timeline, edit, movie

**Track**:
A named container holding elements, with an integer `layer` giving its place in
the stack. A track supplies *stacking*, never *timing*: it has no start, no
duration and no clock, its children carry absolute times on the project's one
timeline, and the order they appear in carries no meaning — for timing or for
stacking ([ADR-0060](docs/adr/0060-layer-tie-is-an-error-array-order-stays-meaningless.md)).
Children of a single track may not overlap in time — that is a validation error.
Elements that should overlap belong in different tracks.
_Avoid_: lane, channel, layer (as a container)

**Element**:
One thing placed on the timeline — an image, a video, an audio file, a piece of
text or a shape. Every element has a type, a required unique `id`, a time range
and an optional `group`, in that same shape whatever its type. It sits in a
track, which supplies its stacking position unless the element overrides it.
Audio is an element like any other; nothing owns it.
_Avoid_: clip, item, object, asset

**Id**:
A short, unique, author-chosen string naming one element, required on every
element regardless of type — `"id": "card-05"`. Its only job is to be a target,
for now an anchor's `below`/`above`
([ADR-0019](docs/adr/0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md)).
A separate namespace from a track's name: the two look alike as bare strings but
an anchor may only resolve against an element `id`, never a track name. Not a
tool-call argument — the write-tool invariant
([ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md)) forbids a
write tool taking a bare id as its argument, which is unrelated to an id
appearing as data on an element.
_Avoid_: key, name, index

**Layer**:
A place in the stack, as an integer — higher draws in front. Normally carried by
the track, so every element in it stacks together. An element may override its
track's layer with its own integer, or with an anchor. Two elements resolving to
the same layer is legal only while their boxes never overlap in time and space;
once they do, the document does not say which draws in front and `validate`
errors, refusing the render, until the author states an order explicitly via an
override or an anchor
([ADR-0060](docs/adr/0060-layer-tie-is-an-error-array-order-stays-meaningless.md)).
_Avoid_: z-index, depth

**Anchor**:
An element's layer stated relative to another element's `id` rather than as a
number — `{"below": "title"}` resolves to that element's layer minus one,
wherever either of them sits. The target must itself carry a plain integer
layer — one hop only, never another anchor — so resolving one is a lookup, not
a walk. Written so a dependent element cannot drift out of sync when the thing
it depends on moves.
([ADR-0019](docs/adr/0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md))
_Avoid_: parent, constraint, binding; and do **not** use for an element's
positioning origin — that is `origin`. See the note under Origin.

**Origin**:
The point of an element's own box that its `x`,`y` places, and about which transforms
pivot. Exactly nine keywords, vertical component first:

    top-left      top-center      top-right
    center-left   center          center-right
    bottom-left   bottom-center   bottom-right

The grammar is `{top|center|bottom}-{left|center|right}` with one elision: the middle is
`center` alone, and **`center-center` is a schema error naming `center`** — two spellings
of one value break the write-read round trip, because `fmt` normalises on write and the
agent's next exact-string replace then finds nothing. It is `center-left`, never
`middle-left`: `top-center` needs a horizontal-middle word regardless, so a separate
`middle` would spell one concept two ways depending on axis. Defaults to `center`, so a
bare element is conspicuous rather than plausible.
_Avoid_: anchor, alignment, pivot

**Transform**:
Where a visual element sits and how it is drawn — `x`, `y`, `origin`, `scale`,
`rotation`, `opacity` — as flat fields on the element in absolute integer pixels. There is
exactly one per element and it is never nested, never inherited and never composed: no
element's transform is relative to another's. An element's size is declared, never
defaulted from the source file.
_Avoid_: matrix, layout, placement (as a field), position (as a field name)

**Keyframe**:
One `{"t","t_from","v","ease"}` record in a list that makes an animatable property change
over time — `t_from` optional (see Recorded intent).
Its `t` is written in timeline milliseconds but is **not a timeline time** — it is the
element's own animation geometry, so `shift` moves elements and their keyframes are
carried with them. A keyframe outside its element's range is legal and ordinary: it is how
a trimmed move is spelled.
_Avoid_: key, waypoint, stop, tween

**Animatable property**:
A property the schema types as taking either a literal or a keyframe list. The list is
closed: a keyframe list on any other field is a schema error. A property joins it only if
its value type has a published interpolation, it resolves from the file and the instant
alone, and no `validate` check relies on it as one fixed value — which is why `clip` and a
fitted box are not on it
([ADR-0146](docs/adr/0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)).
_Avoid_: keyframable (property), animated property (that is one carrying a list right now)

**Easing**:
The shape of the interpolation **arriving at** a keyframe from the previous one — a name
published in the schema as its cubic bezier, or the four control points directly. Its
presence is a pure function of position: the first record of a list has nothing arriving at
it and carrying an `ease` there is an error, and every later record must carry one, with no
default published anywhere
([ADR-0038](docs/adr/0038-ease-is-required-on-every-non-first-keyframe-record.md)).
_Avoid_: timing function, curve, interpolation (as the field name), tween

**Recorded intent**:
A declaration of where a literal came from, written beside it, that **no renderer reads** —
the literal stays the sole author of what renders, and `validate` is the declaration's only
consumer. One pattern, several fields; exactly one field exists today, and a new one is an
ADR carrying a census rather than a convenience
([ADR-0086](docs/adr/0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md)).
Every instance is optional, and its **absence is no claim** — never a claim that the value
is independent. The time axis's instance is `t_from`, on a keyframe record, annotating that
record's own `t`: `{"rule": "element-start"}` says the instant was derived as the element's
own `start`, and `{"rule": "after-previous", "ms": 15000}` says it was derived as the
previous record's `t` plus that offset. Direction lives in the rule's name, never in the
sign of its argument, so `ms` is non-negative; rules do not compose, and no rule takes more
than one argument. When the arithmetic stops holding, `validate` reports `R-DERIVED-T` and
states the integer the rule now derives.
_Avoid_: reference, link, binding, constraint, formula, expression; and do **not** call a
`t_from` a *derived value* — the value is literal and the derivation is what is recorded

**Clip**:
The static frame-space rectangle an element is drawn through. It does not rotate and does
not scale with the element, so `scale` moves the picture behind a window that stays put —
which is what a Ken Burns is. Shaped and soft masks are a different thing and are not this.
`clip` is also the aperture a `fit` rule works against (see Fit): under `cover` the derived
rect fills it exactly, but under `contain` the derived rect can be smaller, leaving slack.
Where that slack goes is not a separate placement field — it is ordinary `x`/`y`/`origin`
work against `clip`'s own corner and centre, the same as any other visual element's rect.
([#52](https://github.com/MBehtemam/Montagent/issues/52))
_Avoid_: crop, mask, viewport, bounds

**Run**:
One stretch of a text element's content, carrying its own text plus style deltas
over the element's base style. A text element's content is always an ordered array
of runs, even when there is only one. A run boundary is *style only* — it never
implies a line break; a line break is a `\n` character inside a run's text. A run
that holds exactly one unit of a stagger may single that unit out.
_Avoid_: span, segment, chunk (a **Paint chunk** is a stretch of frames, never text)

**Letter spacing**:
Space added after every grapheme of a line but the last, in thousandths of an em of the
size it is set at, so a size change keeps the same tracking. It belongs to the whole
element, never to one run, and it may change over time. None is added between two letters of
the same joining script such as Arabic, joined or not, because a gap there would tear the
cursive stroke
([ADR-0151](docs/adr/0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md),
[ADR-0153](docs/adr/0153-a-joined-piece-moves-as-one-and-joining-scripts-keep-their-ligatures-and-take-no-letter-spacing.md)).
_Avoid_: tracking (as the field name), kerning (the font's own pair adjustment), character spacing

**Stagger**:
One text element's animation run once per unit (a letter, word or line), each unit started a
fixed delay after the one before. It lives inside one element and never links two: cards
that enter one after another are separate elements with their own literal times. A unit is
singled out by splitting it into its own run, whose own values replace the derived ones
([ADR-0148](docs/adr/0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md),
[ADR-0151](docs/adr/0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)).
_Avoid_: cascade, offset animation

**Unit**:
One piece a stagger moves on its own. A letter is a grapheme cluster that is not whitespace;
a word is a word containing a letter, digit or pictograph, with its punctuation; a line is a
`\n` line with a letter in it. Units are counted in reading order across the whole element,
ignoring runs, and whitespace takes no step. The count comes from the text alone: letters that
the font merges into one glyph, or that make one joined piece, move together on the first
one's start but keep their own places in the count.
A unit's **delay** is how far after the first unit it starts.
([ADR-0151](docs/adr/0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md),
[ADR-0153](docs/adr/0153-a-joined-piece-moves-as-one-and-joining-scripts-keep-their-ligatures-and-take-no-letter-spacing.md))
_Avoid_: character (a code point is not a letter), glyph (shaping's unit, not the stagger's),
copy, instance

**Joined piece**:
A stretch of letters that cursive joining connects into one unbroken stroke, as in Arabic:
`السلام` is three pieces, `ا`, `لسلا` and `م`. Which letters join comes from the text alone.
A stagger by letter moves each piece as one, so the stroke never tears
([ADR-0153](docs/adr/0153-a-joined-piece-moves-as-one-and-joining-scripts-keep-their-ligatures-and-take-no-letter-spacing.md)).
_Avoid_: word (a word can hold several pieces), ligature (one glyph; a piece is several),
cluster

**Caption**:
A `text` element the four caption checks run on — `R-CAPTION-PACE`,
`R-CAPTION-MIN-DURATION`, `R-CAPTION-NO-AUDIO` and `R-CAPTION-REPEAT-DURATION`. That is any
text element that does not carry `caption: false`; omitted and `true` mean the same thing
([ADR-0136](docs/adr/0136-a-text-element-opts-out-of-the-caption-checks-with-caption-false.md)).
The author draws the line, and nothing infers it: not the track's name, not a `highlight`
window, not whether audio sits under it. Unrelated to the attribution block `frame` prints
beside a picture, which older ADRs also called a caption (see **Attribution obligation**).
_Avoid_: subtitle, decorative text (for what an opted-out element is — it is just text)

**Font**:
An ordered chain of font *files* the project declares under a semantic name, which
elements reference by that name. Always files, never a system family: a family name
is an entry in a table you cannot read, cannot commit, and that differs per machine.
The renderer opens nothing outside the chain.
_Avoid_: typeface, family, font stack

**Face**:
The one set of outlines and metrics a chain entry resolves to. A file is not a face: a
`.ttc` collection carries several, and the entry's `index` says which — *"defaulting to
0. 49 of the fonts in a stock macOS `/System/Library/Fonts` are collections; this is the
normal case"* ([ADR-0007](docs/adr/0007-text-runs-literal-size-declared-fonts.md)). The
face is **cut out of the collection into a file of its own before it is registered**, so
that an entry naming one resolves to it and to nothing else
([ADR-0102](docs/adr/0102-a-ttc-chain-entry-is-cut-down-to-its-face-before-it-is-registered.md)).
The distinction is load-bearing rather than pedantic: a chain entry, an attestation and
a font-swap census are all about the *file*, while shaping, coverage and every measured
width are about the *face*, and the period when one word covered both is the period when
a declared `index` reached no code that chose one.
_Avoid_: weight, style, variant, font (for a face)

**Attestation**:
What `fonts vendor` learned about one font file, recorded in the project's `fontVendor`
table under the file's path: the licence identifier it recognised or the author declared,
where the bytes came from, and their `sha256`. One path, one entry, however many font
chains reference the file ([ADR-0057](docs/adr/0057-font-vendoring-licence-gate-and-path-keyed-attestation.md)).
Written only by `fonts vendor`; `validate` checks the hash against the file on disk and
reports an entry no chain references, but never prunes one.
_Avoid_: licence record, font metadata, provenance (unqualified)

**Line height**:
A text element's line-height multiplier, restricted to one decimal digit — `1.0`, `1.1`,
`1.2`, … — always exactly representable as `n/10`. A line's height is the largest `size`
among its runs × `line_height`; the derived block height is `ceil` of that product over the
line count, evaluated in exact integer arithmetic, never IEEE double — the same ULP hazard
`fit`'s box sizing has (see Fit), on a field `fit`'s ADR never touched. Defaults to `1.2`
when omitted.
([ADR-0028](docs/adr/0028-text-block-arithmetic-is-exact-tenths.md))
_Avoid_: leading, line spacing (implies an additive gap, not a multiplier)

**Slot** / **Ink** / **Seam**:
Three things a line of text has, and the reason they are three words. A line's
**slot** is the space it reserves — the largest `size` among its runs ×
`line_height` — a function of two numbers the document declares and never of the
font. Its **ink** is where the glyphs actually are, read from their outlines. A
**seam** is what lies between one line's ink and the next line's, compared **per
glyph, where two glyphs share horizontal space**, and stated as an overlap:
positive means they collide. A seam is *absent* — not zero, not clear — when no
glyph of either line shares horizontal space with the other's, because those two
lines cannot meet at any `line_height`.
The three are not interchangeable, and that is the point. On Latin they agree
closely enough that one word would have done; on a script whose marks stack —
Thai's base plus upper vowel plus tone mark plus lower vowel — the ink runs past
the slot and through the next line's, with every field in the document valid.
`measure` reports all three; `validate`'s `R-LINE-INK-COLLISION` reports a
positive seam and never says which of the two legitimate repairs was meant.
([ADR-0007](docs/adr/0007-text-runs-literal-size-declared-fonts.md),
[ADR-0087](docs/adr/0087-thai-line-height-collision-is-a-font-selection-problem.md))
_Avoid_: line box (conflates the slot with the ink — the collision is exactly the
case where they differ), bounding box (unqualified: an advance box knows nothing
about a stacked mark, and a *whole-line* box was measured to be a false-positive
generator — ADR-0087), gap (a seam's ordinary sign is negative; see also Gap,
which is a fact about the *clock*)

**Shape**:
A drawn primitive with no source file — `rect`, `ellipse` or `path`, each its own element
type, never a `shape` field inside a shared one. An ellipse inscribes its declared rect and a
path draws inside it, so none needs a placement rule beyond the transform every visual
element already carries. A `rect` may declare a `radius`; the others may not.
_Avoid_: rectangle (as the type name), box, figure, primitive (unqualified)

**Path**:
A shape drawn through a list of **vertices**, open or `closed`. Each vertex is an `at`
position with optional `in` and `out` **handles**, and a segment between two vertices is a
cubic Bezier through their handles. Positions are integer pixels from the declared box's
top-left corner and handles are offsets from their own vertex, so the box bounds the drawing
but never scales it: there is one unit system and the extent is declared, not derived. A
vertex without handles is a corner. A path's `points` animate as one whole list of
unchanging shape.
([ADR-0154](docs/adr/0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md))
_Avoid_: polyline, polygon, line (as types), anchor point (for a vertex), control point (for
a handle as written; it is an offset), viewBox

**Reach factor**:
How far a path's stroke can reach from its outline, in half stroke widths: `k`, the larger
of the `stroke_miter_limit` (for a `"miter"` join) and √2 (for a `"square"` cap where a cap
draws), else 1. The **inset** `m = ceil(k × w / 2)` is the margin every vertex and handle
keeps from the declared box's edges, so the box contains the ink on every frame. It is a
worst-case bound, never a measured overlap.
([ADR-0158](docs/adr/0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md))
_Avoid_: padding, margin (for the inset), stroke extent

**Fill**:
The paint inside a shape's outline. Optional when a `stroke` is
present, giving an outlined shape; a shape with neither is a schema error naming both,
because an element that deliberately renders nothing and an element that forgot its paint
must not look alike. Only a closed **path** takes a fill; an open one is never filled by
implicit closing.
_Avoid_: background, colour (for a shape). Do not use for what a
time-based element does past the end of its source — that is `overrun`.

**Paint**:
A value, not a process: what a paint field holds — a flat colour or a gradient. The paint
fields are a shape's `fill` and `stroke` and a text element's element-level `color` and
`stroke`; run and highlight paint, the project `background` and every effect colour take
a colour only. The *painter* is the renderer that draws paints; it is not one.
([ADR-0149](docs/adr/0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md))
_Avoid_: colour (when a gradient is allowed), fill (for the union), style

**Gradient**:
A paint whose colour varies across the element's declared box: `linear`, at an `angle`, or
`radial`, from a `center` out to a `radius`, both measured as fractions of the box, with a
list of at least two stops of `offset` and colour. Past the first and last stop the end
colours extend. A named entry in a closed vocabulary, not an open syntax: the kind is an
enum and every parameter is a literal.
([ADR-0149](docs/adr/0149-a-gradient-is-a-paint-linear-or-radial-measured-against-the-declared-box.md))
_Avoid_: ramp, colour ramp, shader (for a gradient), location or position (for a stop's
`offset`)

**Stroke**:
A second paint on the same outline — `stroke` and `stroke_width` — sitting on the
primitive rather than among the effects, and addressable per *run* on text. It never
enlarges the declared rect, and that is what separates it from an effect: a blur is not a
paint on the outline and a drop shadow is not addressable by run. On a shape it falls
**inside** the declared rect, so a stroked `card-05` still occupies exactly 984×169. On a
**path** it is centred on the outline, and the box still contains it because every vertex
and handle must sit inside the box by the stroke's furthest **reach**: half the stroke
width, times the miter limit for a `miter` **join** or √2 for a `square` **cap**. A join is
how the stroke turns a corner and a cap how it ends; only a path chooses them, and a cap
exists only where the stroke ends, at an open path's ends or a dash's. On
text it falls **outside the glyph contour** — inside would thin the stems — and grows into
the box rather than past it, because a text element's `width`/`height` is a container
claim and not drawn geometry. It is in element space, so it scales with `scale`.
([ADR-0158](docs/adr/0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md))
_Avoid_: outline, border, bord; line join, line cap, end cap (say join and cap)

**Dash pattern**:
A stroke broken into dashes and gaps along its outline: `stroke_dash`, an even list of
integer pixel lengths starting with a dash, and `stroke_dash_offset`, how far into the
pattern the outline's start falls. The pattern runs from a fixed start in a fixed direction,
and where it fails to divide a closed outline it leaves a **seam** at that start; the lengths
drawn are always the ones written, never stretched to fit. Any shape takes one; text does
not. A zero-length dash is a dot, so it needs a cap that draws one.
([ADR-0158](docs/adr/0158-a-path-chooses-its-stroke-join-and-cap-and-every-shape-takes-a-dash-pattern.md))
_Avoid_: dasharray, dash array, line style, dotted (as a field or kind)

**Effect**:
A member of a closed, named, parameterised vocabulary in `effects: [...]` on an
element — never an open plugin architecture. A list, not a map or a single field,
because application order is semantically real: blur-then-shadow is a different
frame from shadow-then-drop, and two effects of the same name are ordinary rather
than forbidden. The vocabulary is twelve members — `blur`, `shadow`, `mask`
(shape-only), `chroma` (the Matte operation below), the four Colour filter
scalars below, and the named effects `grain`, `glow`, `posterize` and
`directional_blur`. A member joins only if its look can't already be composed, it
is one built-in filter or one small shader of Montagent's own, and it stays inside
the element's box or a reach it declares; each later member joins by its own ADR.
An effect filters the element's own pixels: there is no element that paints from
nothing, so a texture is a `rect` carrying an effect and a blend. Effects attach to whole
elements, never to a run — that boundary is what excludes `stroke` (a
run-addressable paint field) from this vocabulary, and what excluded per-word
Highlight, which needed run addressing and a timing model keyframes don't provide
and got its own construct instead. An open shader or script file is never an
effect: a look outside the vocabulary is Pre-rendered footage.
([ADR-0040](docs/adr/0040-effect-model-attachment-and-v1-vocabulary.md),
[ADR-0049](docs/adr/0049-v1-colour-filter-vocabulary-four-scalar-members.md),
[ADR-0088](docs/adr/0088-chroma-is-a-matte-operation-and-color-stays-literal.md),
[ADR-0156](docs/adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md))
_Avoid_: filter (for the whole concept — see Colour filter, below), plugin, stack,
shader (for a member), generator

**Seed**:
The literal integer a random-by-nature effect is written with. With the element's
own local instant, it fixes every draw, so the look changes on every frame yet the
same file always paints the same pixels, and a moved element keeps its pattern. Two
effects with the same seed, cell size and colour mode, on elements that start
together, draw the same pattern. Never keyed.
([ADR-0156](docs/adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md))
_Avoid_: random, noise seed, time input

**Pre-rendered footage**:
A look drawn in code with any tool, outside the project, and brought in as one
ordinary `video` with alpha in a lossless codec. Its **recipe** sits beside it,
outside the project: the code, the command, the tool versions and a hash of the
decoded frames, so it can be made again and checked. The project names only the
footage, never the code.
([ADR-0156](docs/adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md))
_Avoid_: shader file, script layer, generated clip (for the recipe), mogrt

**Colour filter**:
The four scalar `effects` members that change pixel colour rather than geometry:
`tint{color, amount}`, `saturation{amount}`, `brightness{amount}` and
`contrast{amount}`. Flat named members, each sitting in the same ordered list as
`blur` and `shadow` — never one `color{mode}` effect with an internal
discriminator, which would be the vocabulary's only two-level lookup. There is no
fifth, and no new one arrives without an ADR clearing the four-clause
admissibility rule: fixed arity, bounded scalar parameters, a documented identity
value, and not reproducible by composing two members that already exist.
`grayscale` is not a member — it is `saturation` at its zero endpoint — and
`sepia` is that composed with a warm `tint`. `tint.color` is the sole
grandfathered exception to "parameters are bounded scalars", and the exception is
closed.
([ADR-0049](docs/adr/0049-v1-colour-filter-vocabulary-four-scalar-members.md))
_Avoid_: grayscale, sepia, duotone, LUT, curve

**Matte operation**:
An `effects` member whose output is **transparency** rather than colour: it decides
which pixels survive, and the RGB it keeps is the RGB it was given. `mask` and
`chroma` are the two. The distinction is load-bearing rather than descriptive —
Colour filter's four-clause admissibility rule is scoped, in its own words, to a
*colour operation*, so it governs neither of them. `chroma`'s `spill` is the one
parameter that crosses back: it suppresses screen colour reflected onto retained
pixels, and it is admitted on the four clauses directly rather than by any
exception.
([ADR-0088](docs/adr/0088-chroma-is-a-matte-operation-and-color-stays-literal.md))
_Avoid_: key (for the whole concept — a key is what `chroma` computes), cutout

**Chroma**:
The `effects` member that keys a screen colour out of a source:
`chroma{color, tolerance, softness, spill}`. `color` is a literal `#RRGGBB` and not
a hue angle — a bare hue **inverts the key**, and supplying the missing saturation
and value is the colour restated in three fields. All four parameters are animatable
properties, `color` staying `#RRGGBB` in every record, so footage whose lighting
drifts mid-take takes a keyed `tolerance` rather than being cut at each drift.
`measure` reports the resulting alpha coverage per frame, which is how that drift is
found.
([ADR-0088](docs/adr/0088-chroma-is-a-matte-operation-and-color-stays-literal.md),
[ADR-0146](docs/adr/0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md))
_Avoid_: greenscreen (the technique, not the member), chromakey, despill (that is
`spill`, one parameter of this member)

**Mask**:
An `effects` vocabulary member: a closed shape (`circle`, `rect`, `ellipse`) that
keeps an element's rendered pixels where the shape is and erases the rest.
Shape-only: a mask sourced from an image is deferred, since it introduces a second
asset reference and unresolved fitting/colour-space questions, and a mask sourced
from text is refused — footage through letters is a paint, not a mask.
**Its parameters are one rect, shared by all three shapes**:
`x`, `y`, `width`, `height` name the rect the shape is inscribed in, and `shape`
selects which figure is drawn in it — never which fields exist, because a
per-shape field set would be the two-level lookup ADR-0049 refused. The four are
**all-or-none**, element-local integers measured from the element rect's top-left
whatever the `origin` keyword is, and their identity value is the element's own
rect — so a bare `{"name": "mask", "shape": "circle"}` is still the largest circle
inscribed in that rect, reached by the same arithmetic rather than by a second
rule. `radius` rounds the corners of a `rect` mask, identity `0`; on `circle` or
`ellipse` it is an unknown key, exactly as it is on a drawn ellipse (ADR-0014).
There is no ellipse rotation term — the element's own `rotation` turns the mask.
**The mask rides the transform**: it is declared inside the element's box in
unscaled units, so `scale` grows it and `rotation` turns it, the same rule that
makes a `blur` radius and a `stroke_width` scale. The rect, `radius` and `feather`
are animatable properties, like every numeric effect parameter; a plain mask whose
rect has no positive size hides the whole element, so a **reveal** is a keyed rect
from `0`. A bare `mask` key outside `effects` is a retired spelling. **`invert`** keeps the outside of the
shape instead of the inside. **`feather`** softens the mask's edge, centred on it,
read as a `blur` radius is, so an inverted feathered mask is the exact complement
of the plain one. Masks in one list intersect, so a mask and a smaller inverted one
keep a ring.
([ADR-0040](docs/adr/0040-effect-model-attachment-and-v1-vocabulary.md),
[ADR-0068](docs/adr/0068-the-bare-mask-key-retires-masks-are-effects-members.md),
[ADR-0084](docs/adr/0084-the-mask-rect-is-one-shape-independent-parameter-set.md),
[ADR-0146](docs/adr/0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md),
[ADR-0152](docs/adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md))
_Avoid_: clip (that name is the transform model's static frame-space aperture,
[ADR-0012](docs/adr/0012-flat-transform-keyframes-carried-by-their-element.md) — a
different concept that happens to sound alike; a mask is carried by the transform,
a `clip` is not, and a *shaped* static porthole is not expressible in v1)

**Transition**:
Its own element type — `type: "transition"` — with its own `id`, `start`/`end`,
a closed `kind` vocabulary, and two id references naming the elements it bridges.
Not a property on either bridged element, which would make ownership
unprincipled, and not an `effects` member, because an effect is element-local and
a transition reads two elements' pixels together. Its `start`/`end` must exactly
equal the intersection of the two bridged elements' own ranges: outside that
window only one of the two exists, so a wider declared range is a field the
renderer cannot honour. The per-track non-overlap rule is untouched — the two
bridged elements live on separate tracks, exactly as anything else needing
simultaneous visibility already does. **`kind` is `crossfade`, `wipe`, `slide` or
`push`.** A crossfade ramps the two elements' opacity. In a slide only the incoming
element moves, over a still outgoing one. In a push both move together, joined. In a
wipe a moving edge divides them, each keeping its own side. A slide, push or wipe moves
or cuts a whole frame's width or height, outside the elements' own transforms, and never
changes their layers. The same rule makes a dependency between two elements one the file
can show: a transition may move or cut what it bridges, but no element's pixels decide
where another shows. That is why there is no matte taken from another element.
([ADR-0059](docs/adr/0059-transitions-element-type-crossfade-only-exact-window.md),
[ADR-0150](docs/adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md))
_Avoid_: dissolve, transition effect, track matte (refused, not a synonym for a wipe)

**Direction**:
A slide, push or wipe's `direction`: **the way the motion travels**, `left`, `right`,
`up` or `down`. `"left"` means the content (or a wipe's edge) moves leftward, so the
incoming element enters from the right. It never names the edge something enters from,
and a crossfade has none.
([ADR-0150](docs/adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md))
_Avoid_: from (that is the outgoing element's id), side, edge

**Blend mode**:
How a finished element is composited into its **backdrop**, written as the element's
flat `blend` field beside `opacity`: `normal`, `multiply`, `screen`, `overlay` or `add`.
The backdrop is everything painted below the element in the stack, down to the
`background` — it is not the `background`, which is only its bottom. The finished
element is what blends: its effects, masks and shadow, then its `opacity`, composited
once. Not an `effects` member, because an effect is element-local and a blend mode
reads what is under the element; static, because a mode is not a number or a colour;
and whole-element, never per run. Omitted means `normal`. "Blend" is this entry's word
alone: a transition crossfades and a keyed colour interpolates.
([ADR-0147](docs/adr/0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md))
_Avoid_: blending mode, composite mode, `plus`, linear dodge, blend (for a crossfade or
a keyed colour)

**Motion blur**:
The smear an element's own motion leaves within one frame, written as the element's
`motion_blur` field: a **shutter**, the share of the frame the smear covers as an angle
out of 360°, centred on the frame's instant, and the number of **samples**, the instants
across it at which the element is painted and averaged. Everything the element's own
keyframes move follows the samples. What decides that the element is present, and which
frame of its footage shows, does not. So a video's footage is not blurred, and the smear
reaches only as far as the keyframes do. An element whose values are equal at every sample
is **still**, and is painted as if it had no field. Not an effect: it repaints the element,
effects and all. A smear with a fixed angle that ignores the motion is a directional blur,
not motion blur.
([ADR-0155](docs/adr/0155-motion-blur-is-a-per-element-field-that-accumulates-the-element-over-a-centred-shutter.md))
_Avoid_: directional blur (for this), shutter speed, keyframable shutter, motion trail,
echo

**Highlight**:
A run's optional timed window — `{start, end, ...style delta}` — during which it
wears a different style, falling back to its unconditional one outside it. The
karaoke construct, and deliberately *not* an effect: effects attach to elements
and a run lives inside one. It lives on the run rather than in a separate
element-level event array, because a `{t, run, style}` array would reintroduce
the join a reader has to perform by hand. Every highlighted word is its own run
and a run carries at most one window — spanning several words would need sub-run
character offsets, which a later text edit invalidates in silence. The times are
literal integers frozen at authoring time, never a reference into an external
alignment file: a highlight window is content, the same status a clip's
`start`/`end` already has.
([ADR-0048](docs/adr/0048-per-word-highlighting-is-a-timed-window-on-the-run.md))
_Avoid_: karaoke (as a field name), word timing, active word

**Colour**:
`#RRGGBB` or `#RRGGBBAA`, uppercase, and nothing else. No three-digit shorthand, no CSS
names, and `#RRGGBBFF` is an error naming the six-digit form — two spellings of one value
break the write-read round trip, as `center-center` does. Alpha lives here as well as in
`opacity` because `opacity` is the *animation* channel: a static per-run transparency, or
a translucent fill under an opaque stroke, has nowhere else to go. `fmt` never rewrites a
colour, including between the two forms. ASS writes `&HAABBGGRR` — byte-reversed, alpha
inverted — and every colour in the fixture was converted out of it by hand.
_Avoid_: color (in prose), rgba(), hex (as the field name)

**Group**:
An optional free-text label marking elements that belong together, such as every
element of one vocabulary item. It has no effect on rendering, timing or
stacking.
_Avoid_: scene, segment, section

**Source**:
The file an element draws on, written on the element itself as a path or a URL.
There is no table of files declared elsewhere and referred to by name, and no
top-level field that changes how a `source` resolves — a relative path is
always relative to the project file's own directory, and an absolute path is
permitted and resolved as-is. A project refers to a remote store by rewriting
`source` to URLs, not by editing a shared base
([ADR-0053](docs/adr/0053-asset-path-resolution-no-assetroot.md)). A URL is for
reference, not rendering: `validate` probes it, and `render` and `frame` use local
sources only, so rendering needs a local copy named by path
([ADR-0131](docs/adr/0131-render-and-frame-use-local-sources-only-and-validate-says-so.md)).
An image may name other files for timed windows of its range; its `source` is
what it draws outside them (see **Swap**).
_Avoid_: asset, resource, media reference, assetRoot

**Swap**:
A timed window in which an image element draws a different file in the same box,
falling back to its `source` outside every window. It changes the file and never
the box, and it steps rather than crossfades, so a run of swaps draws exactly what a
run of abutting image elements would. It is a highlight's counterpart for images:
a window over a base, not a keyframe
([ADR-0140](docs/adr/0140-an-image-element-changes-its-file-over-time-through-timed-swaps.md)).
_Avoid_: frame (a rendered picture), sprite, source keyframe, replacement

**Timeline range**:
Where an element sits on the project's one absolute clock, as a `start` and an
`end` in whole milliseconds. The range is half-open — an element is on screen from
its `start` up to but not including its `end` — so a cut where one range ends and
the next begins names a single instant, not an overlap and not a gap.
_Avoid_: duration (as a stored field), offset, timecode

**Source range**:
Which part of a file an element plays, as distinct from where the element sits on
the timeline. Only elements built on time-based media — video and audio — have
one; images, text and shapes have no insides and are simply held for their whole
timeline range. A time-based element's two ranges are the same length unless it
says otherwise. A video carrying a **Time remap** has no source range: the curve
names the source instead.
_Avoid_: trim (as a noun), in/out point

**Speed**:
A time-based element's playback-rate multiplier — `0.645` plays its source at
0.645× normal rate, slower. Strictly greater than zero; `0` and negative
values are schema errors. Changes what the source range's two ranges are
allowed to disagree by: `end - start` must equal `source range / speed`,
rounded to the nearest millisecond. Always static. A rate that changes over a
clip, and reverse playback, are a **Time remap**, never `speed`, and `speed` is
refused on an element carrying one.
([ADR-0020](docs/adr/0020-speed-overrun-hold-loop.md), [ADR-0157](docs/adr/0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md))
_Avoid_: rate, stretch factor (reads as the reciprocal and gets the direction
backwards), tempo

**Overrun**:
What a time-based element does once its (possibly speed-adjusted) source runs
out before its timeline range does: `"hold"` freezes the source's last frame
(video only — a schema error on audio, where the correct spelling of "then
silence" is a shorter element and a gap) or `"loop"` restarts the source from
its beginning with a hard cut, no crossfade. Composes with `speed` rather than
excluding it. Present only when needed — there is no `"none"` value. Refused on
an element carrying a **Time remap**, where a freeze is a flat stretch of the curve.
([ADR-0020](docs/adr/0020-speed-overrun-hold-loop.md))
_Avoid_: fill (spent — see Fill), extend, pad

**Time remap**:
A video's `source_time`: which moment of its file is on screen, as an animatable
property in source milliseconds. The rate is the curve's slope — steep is fast,
shallow is slow, flat is a freeze, falling plays in reverse — and a literal is a
freeze frame for the whole element. It is the only author of the source on its
element, so the source range, `speed` and `overrun` are refused there. Video only,
and the element's own sound must be silenced with `volume: 0`.
([ADR-0157](docs/adr/0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md))
_Avoid_: speed ramp, speed curve, retime, speed keyframes (for the field — a speed
ramp is what one draws), remap (as a field name), reverse (as a field)

**Feed**:
One `ffmpeg` decoding one video element's frames in timeline order, at the
element's declared size and a constant pace. It is opened at the first frame that
paints the element and closed at the first frame that does not.
_Avoid_: run (a text element's **Run**), stream (a container's audio/video
stream, as `ffprobe` names it), run of frames (ADR-0127's phrase, retired)

**Paint chunk**:
C consecutive timeline frames that one of `render`'s K painters paints in order, on a
painter of its own, while one encoder takes every painter's frames in timeline order.
Chunks are handed out in timeline order, and a chunk start paints exactly what a
`--from` render starting there would. Always "paint chunk" in prose: bare "chunk" is
on **Run**'s avoid list.
([ADR-0144](docs/adr/0144-render-paints-on-k-painters-over-chunks-and-the-spy-trailer-renders-in-a-minute.md))
_Avoid_: segment, slice, batch, range (a `--from`/`--to` render's), run

**Volume**:
An `audio` or `video` element's playback level, as a linear multiplier:
`0` is silent, `1` (the default) is the source's own level, and values
above `1` amplify. Negative is a schema error, the same class as `speed`'s;
the ceiling is deliberately open, and clipping past it is the renderer's
documented behaviour rather than a refusal. Flat on the element like `speed`,
and animatable with the same `{t,v,ease}` records every animatable property
carries, so a fade is two records rather than a dedicated field. There is no
`mute` — a `video` element's embedded audio is the same audio a `volume` of
`0` already silences. Automatic ducking (one element's level reacting to
another's presence) is out of scope; the same outcome is hand-authored as
ordinary keyframes.
([ADR-0055](docs/adr/0055-audio-mixing-model-volume-fades-ducking-deferred.md))
_Avoid_: gain, level (as a field name — ambiguous with other senses of
"level" in this glossary), mute

**Gap**:
A stretch of a track with no element in it. Gaps are legal and ordinary — the
silence between two narration lines is a gap. A gap is never an error, which is
why it is reported apart from an overlap rather than alongside one.
_Avoid_: hole, blank, silence (as a name for the general case)

**Knot**:
One connected group of elements of a single track that overlap each other — a
connected component of the overlap relation and deliberately not a clique, so
three elements where the first and third are disjoint but both meet the second
are **one** knot. It is the unit an author untangles, which is why the track's
one overlap finding carries a census whose groups are its knots and whose count
is the number of things to fix. It is never reduced to a single offending pair:
which member is misplaced is not readable off the document.
([ADR-0100](docs/adr/0100-one-track-overlap-finding-per-track-carrying-a-census-of-its-knots.md),
[ADR-0043](docs/adr/0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md))
_Avoid_: cluster, pile, collision, offending pair

**Slack**:
The timeline distance from one element's boundary to the nearest thing that
follows or precedes it — a neighbouring element's boundary in the same track
or a different one, or, for the project's own last boundary, the derived
`duration`. Every gap is slack; slack additionally names the cross-track case
a gap can't reach, such as the distance from the last narration's end to a
still photo's end. Slack currently in the file is invariant by default — its
size is content, not a default the renderer supplies — and `shift` refuses an
edit that would change it rather than absorbing the difference silently.
([ADR-0032](docs/adr/0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md))
_Avoid_: padding, buffer, margin (a spatial term already spoken for)

**Shift**:
Moving every time at or after some instant by an offset, so that inserting or
removing time carries the rest of the project with it. It is named because it is
the one edit that is arithmetic rather than authorship, and therefore the one
Montagent performs instead of the agent. It refuses an edit that would change an
existing slack's size, the same way it refuses to stretch a time-based
straddler, rather than silently absorbing the difference.
([ADR-0032](docs/adr/0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md))
_Avoid_: ripple (as the primary term), slide, nudge

**Presence set**:
Every element whose timeline range contains a given instant — the whole of what
the document puts on the clock there, audio included, since *"audio is an
element like any other"*
([ADR-0001](docs/adr/0001-flat-element-list.md)). Membership is decided by the
half-open range alone: an element ending at `t` is already out of the set at `t`.
It is a statement about the *document*, never about the picture — what covers
what, and what the frame actually looks like, is the stack's question and
`frame`'s. It is the same population in both `query` modes — ADR-0011's narrower
*"on-screen"* wording for the cut list is amended away
([ADR-0074](docs/adr/0074-the-cut-lists-presence-set-is-every-element.md)) — and
a caller wanting only the visual ones filters the `type` each member carries.
_Avoid_: active set, on-screen set (it includes audio), visible elements

**Cut list**:
What `query --from --to` answers with: the intervals over which the presence set
is constant, each carrying its members — and never sampled instants. It always
names the boundary immediately outside the range on each side, which is what
saves the caller guessing a window, and its intervals partition the range asked
for, so a stretch with nothing in it is one interval rather than a hole. An
interval is as long as the presence set stays the same: two neighbours a reader
could not tell apart are one interval
([ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md)). Elements the
document does not place on the clock cannot be in any interval and are **named**
in the answer rather than dropped
([ADR-0074](docs/adr/0074-the-cut-lists-presence-set-is-every-element.md))
_Avoid_: edit list, shot list, segments, keyframes (spoken for), sampling

**Resolved stack**:
What `query --at` answers with: the presence set at one instant, in painter's order — back
to front by resolved integer layer, an anchor having been resolved in exactly one hop — with
every animated property the element declares carried as the value it *has* there rather than
as its keyframe records. A resolved value is continuous and is never rounded: `x` is an
integer in the document and a number in the answer, because interpolation passes through
what lies between two integers and no rounding rule exists to hide it
([ADR-0035](docs/adr/0035-keyframe-grid-alignment-is-a-review-check-not-a-schema-rule.md)).
It states only what the document declares — no default is synthesised, since presence is
itself content
([ADR-0030](docs/adr/0030-defaultable-field-presence-is-content.md)). The half that reaches
outside the document — the offset into the source, the crop rectangle, the ink box and the
`NOT COVERED` region — is not in it yet
([#210](https://github.com/MBehtemam/Montagent/issues/210)).
([ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md))
_Avoid_: frame, render state, snapshot, sample

**Predicate**:
The expression `query --where` matches elements against: terms of the form
`<field> <op> <value>`, `<field> exists` or `<field> missing`, joined with `and`.
A field is a dotted path into the element, with `*` for every member of an
array and an all-digit segment for one, plus the reserved `track` — the name of
the track the element sits in, which the element itself does not carry. It
matches **what the document writes**: nothing is interpolated and no anchor is
resolved, so an animated `x` is reached by `x exists` and never by the value it
passes through — *"resolved values, never echoed fields"* is `--at`'s rule, and
`--where` has no instant to resolve at. Values match whole: there is no `or` and
no substring matching, and a term holds if **any** value at the path satisfies
it.
([ADR-0070](docs/adr/0070-the-where-predicate-is-a-conjunction-of-whole-value-terms.md)
for the grammar,
[ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md) for the mode
itself)
_Avoid_: filter, selector, query (the verb's name), expression

**Reference frame**:
A frame of the already-published short, extracted from
`fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4` and committed
beside it. Produced by a Python/FFmpeg/ASS pipeline that knows nothing about Montagent, so
nothing Montagent does can change it — which is what makes it the only thing in the
repository capable of **falsifying** the format rather than merely catching a change to
it. Compared against by SSIM over a stated region at a stated threshold, never by byte
equality, with the regions that differ for a known reason — the typeface substitution
([#186](https://github.com/MBehtemam/Montagent/issues/186)) — masked out of the gate and
measured beside it. A divergence is masked only while its cause is unfixed: the fixture's
Ken Burns pivot was masked out of the later frame until
[#276](https://github.com/MBehtemam/Montagent/issues/276) corrected the fixture, and that
frame now gates the photograph as a region of its own. **The typeface mask is the one
exception, and is permanent**: its cause is ADR-0057's licence gate, not a bug, so no work
discharges it short of re-rendering the reference — which would replace the falsifier with
a golden. The fixture's own declared layouts were measured under both faces and all 22 hold
([ADR-0085](docs/adr/0085-the-font-swap-census-holds-and-the-text-mask-is-permanent.md));
that they hold is a *layout* fact and does not make the *pixels* comparable, which is the
conflation #186 was written around.
`crates/montagent-core/tests/reference_frames.rs`.
(spec [#168](https://github.com/MBehtemam/Montagent/issues/168),
[ADR-0010](docs/adr/0010-skia-safe-rasterizer-text-beside-it.md))
_Avoid_: golden frame (it is the opposite — see below), expected output, baseline. The
bare word *reference* is also spoken for by Citation's avoid-list; this is the two-word
term and only ever the two-word term.

**Golden frame**:
A frame **Montagent rendered and committed**, under `crates/montagent-core/tests/golden/`
and `docs/research/prototypes/rust-rasterizer/frames/oracle/`. Self-confirming by
construction: it catches an unintended change — a `skia-safe` bump that shifts
antialiasing, a text change that moves every baseline — and it can never say the format is
wrong, because the thing under test produced it. Both kinds are useful and they are not
the same kind, so they never share a file or a name.
([ADR-0010](docs/adr/0010-skia-safe-rasterizer-text-beside-it.md), whose golden-frame
guard is *"not optional"*)
_Avoid_: reference frame, snapshot, approved output

**Proxy tier**:
The resolution `preview` actually rasterizes at, which is not the project's. The target is
720p — the project's **longer** edge scaled to at most 1280 px, aspect preserved, both
dimensions rounded to even — applied only where the project is larger; a cap, never a
fraction of the project, since half of 16K is still 8K. A preview that runs past the `<5 s`
scrub budget degrades exactly one tier, to 540p (long edge ≤ 960 px), and refuses rather
than degrade again. Every result **discloses** the tier it used, degraded or not: it is how
a caller knows whether the softness it is looking at is the project's or the proxy's. Never
applied to `render`, and never to `frame`, both of which are true pixels always.
The ladder is defined on **caps**, not on sizes, with two consequences worth stating: a
project between the two caps still degrades — from true pixels to a real 960 px proxy — and
a rung whose cap never engaged is named `native` wherever it is named, in the disclosure,
the attempt trace and a refusal alike. A tier name is always the frame that was
rasterized.
([ADR-0021](docs/adr/0021-preview-budget-and-graceful-degradation.md),
[ADR-0046](docs/adr/0046-proxy-preview-target-is-720p-long-edge-capped.md),
[ADR-0065](docs/adr/0065-preview-proxy-target-720p-540p-floor-disclosed-not-certified.md),
[ADR-0078](docs/adr/0078-preview-is-the-ninth-mcp-verb-and-its-unstated-readings-are-ratified.md))
_Avoid_: scale factor, downsample ratio, quality setting; and **do not call the 540p rung
"the floor" unqualified** — see below.

**Wall-clock give-up point** / **legibility floor**:
Two different limits that one word used to name, and they are never one number
([ADR-0067](docs/adr/0067-two-floors-a-wall-clock-give-up-point-and-a-legibility-refusal.md),
resolving [#178](https://github.com/MBehtemam/Montagent/issues/178)). The **wall-clock
give-up point** is 540p: where the *ladder* stops, because one degrade step buys only
~0.71 s at 8K and a miss larger than that cannot be rescued by another rung. The
**legibility floor** is 360p: where a *frame* stops being readable, measured on the real
fixture with the break point between 360p and 240p. One refusal is about time, the other
about pixels; each says which it is in its own words. Because the ladder stops at 540p the
legibility floor is unreachable by degradation today — it is kept as a guard on any future
rung and on any caller-specified proxy resolution, and the fact that it does not currently
fire is stated rather than tidied away.
**The legibility floor governs a standalone proxy *frame*, and not a contact-sheet tile**
([ADR-0095](docs/adr/0095-the-sheets-budget-is-served-tile-width-and-overflow-refuses.md)):
360p was measured on a frame judged alone in a viewport, while a tile is judged in a grid
beside its neighbours and under a label, and no tile count clears a 640 px long edge. The
sheet has its own pair of limits, below.
_Avoid_: *the* floor (there are three across these two entries), minimum resolution, cutoff

**Tile-width target** / **tile-width refusal**:
The contact sheet's own two limits, denominated in **served tile width** — the width in
pixels a tile is actually looked at, after the API downscales the whole sheet to its tier
([ADR-0095](docs/adr/0095-the-sheets-budget-is-served-tile-width-and-overflow-refuses.md)).
Not the width it was authored at, which overstates legibility by 5–6×. The **tile-width
target** is 180 px: what every range gets by default, 18 tiles for a 9:16 project. The
**tile-width refusal** is 140 px: the measured cliff past which fine detail stops being
visible, 30 tiles, reached by exactly one degrade step — and where the sheet **refuses,
naming sub-ranges that would fit**, rather than drawing more tiles than it can show a defect
in. Tile **count is derived** from the width and the tile's aspect, never set by the caller:
a cropped 3:1 band clears 180 px at 98 tiles where whole 9:16 frames manage 18. That
comparison is a **hypothetical** that shows why the currency is width and not count: **a
contact sheet is always whole frames**, and a per-tile crop is refused
([ADR-0103](docs/adr/0103-the-sheet-is-never-cropped-and-the-crop-stays-a-single-frame-instrument.md)),
because a band removes the only view in which a wrong-photo defect is a defect at all, and its
blindness is authored by the caller rather than fixed by the rule. A **crop is a single-frame
instrument**: `--crop` composes with `--at`, never with a range. Visual
tokens are *not* the currency — a sheet spends 1518–1568 of the tier's 1568 at every count
from 4 to 48, so a token cap never fires.
_Avoid_: tile count as a budget, *the* floor, authored tile width, sheet resolution, cropped
sheet / banded sheet / region sheet (no such artifact exists — a sheet is whole frames)

**Attribution obligation**:
What `frame` owes the agent alongside every picture: **the agent must be able to attribute
what it sees to what produced it, without a second call**
([ADR-0097](docs/adr/0097-the-range-is-from-to-on-both-surfaces-and-the-caption-becomes-an-attribution-obligation.md),
amending [ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md)). It is discharged
differently at the two scales. When the image is **one frame**, it is the `query --at` block
printed unconditionally beside it — unchanged, and still with no flag that suppresses it. When
the image is a **contact sheet**, eighteen such blocks would cost 28,410 characters against a
whole sheet answer's 1,413, so it is discharged instead by the fitted per-tile label
**together with** a range-level provenance list: one line per tile carrying the sampled
instant, the run boundary and the presence set as element ids. Both halves are required —
a label fitted to a 140 px tile can carry an instant and little else, so it **cannot** name
the presence set on its own — measurement later showed it cannot name it at the **180 px
target** either ([ADR-0098](docs/adr/0098-the-tile-label-is-a-floored-fitted-line-naming-the-change-at-its-own-boundary.md)), so the split rests on a number rather than a prediction.
What the label half carries is the **tile label**, below. The **resolved** stack (geometry, resolved keyframe values) stays
one call away by design, which ADR-0094's refusal of a synthetic midpoint is what makes
possible: every tile's instant is one the document itself produces, so `query --at <that
instant>` reproduces the tile exactly. The obligation is about attribution, never measurement —
ADR-0011 demotes `frame` from measuring and that is unchanged.
_Avoid_: the caption (ADR-0011's older word, and it now reads as the single-frame case only),
the `--describe` block, suppressing the stack

**Tile label**:
The one line beneath each tile on a contact sheet — in a **gutter strip** of 11% of tile
height, never burned over the tile's own pixels, because an overlay's legibility would depend
on the content it describes and the tile would stop being a faithful crop of what `frame --at`
returns ([ADR-0098](docs/adr/0098-the-tile-label-is-a-floored-fitted-line-naming-the-change-at-its-own-boundary.md)). Five fields in a **fixed order on every tile**: index, class
sigil, sampled instant, **signed boundary offset**, and a **signed element id**. Fixed arity is
load-bearing — a field that is sometimes absent is ambiguous between *"this tile has no such
element"* and *"the label ran out of room"*, and a variable-length field spreads served type
1.46× across one sheet with the smallest type landing on the busiest tile. The boundary is an
**offset from the instant, not a second absolute millisecond**: the two always differ by less
than one frame period, so three characters carry both losslessly, and `+0` prints so an on-grid
boundary is an assertion rather than an absence. The identifying field is **causal, not
discriminating** — the element whose presence *changed at this run's own boundary*, `+` entered
and `-` departed, highest layer, tie-broken on element id, whole-document-span elements
excluded. "Not present in every tile of this sheet" is rejected because it is
**range-dependent**: the same instant would label differently depending on the range asked for.
The label is a **pointer, not a census** — one id cannot name both planted defects, and the
provenance list is the census. The candidates are the elements that **entered**, and the ones
that departed only where nothing entered; the greatest element id breaks a layer tie. Every label
on a sheet is drawn at **one type size**, and a run tile with nothing nameable to report prints
`=`. The arity is fixed **within a sheet**, not across sheets: after the sheet-wide elision every
label has four fields, and the answer says so ([ADR-0128](docs/adr/0128-the-tile-label-is-fitted-at-one-size-per-sheet-and-names-the-topmost-entrant.md)).
_Avoid_: caption (that is a **Caption**, a text element; the single-frame block is the
**Attribution obligation**'s), tile title, gist, gutter text, the
discriminating element

**Served type floor**:
The minimum size, **8 px of served type**, below which a tile label is never drawn
([ADR-0098](docs/adr/0098-the-tile-label-is-a-floored-fitted-line-naming-the-change-at-its-own-boundary.md)). Fitting a label to tile width has **no lower bound** — it is how a label
carrying a full presence set reaches 1.4 px, satisfies every geometric definition of fitting,
and cannot be read, which is the trial's silence-as-coverage failure rebuilt inside the label.
So the floor is on the **type**, and **content is what gives way**: the identifying field elides
**sheet-wide, never per-tile**, so the busiest tile is never the one silently thinned, and if
the numeric core still will not fit the sheet **refuses**. Deliberately *not* 6.3 px, the figure
#396 read: that came from one primed observer who knew the defects, making it a ceiling on what
was ever read rather than a floor on what can be read cold. It is this spec's one
non-re-derivable constant, flagged as ADR-0095 flagged its 140 px cousin, and
[#422](https://github.com/MBehtemam/Montagent/issues/422) owns re-measuring both.
_Avoid_: the 6.3 px figure as a limit, minimum font size, *the* floor (there are now four),
shrink-to-fit

**Vendored UI face**:
The font Montagent draws its **own chrome** with — the tile label — as against the fonts a
*project* declares for its content ([ADR-0098](docs/adr/0098-the-tile-label-is-a-floored-fitted-line-naming-the-change-at-its-own-boundary.md) §9,
[#421](https://github.com/MBehtemam/Montagent/issues/421)). Never the project's declared font: it
may be Thai-only or carry no digits while the label is mostly digits, and this repo's own fixture
declares a path that does not exist, so chrome built on it fails hardest on the projects most
likely to be broken. Never a system font either — across ADR-0064's six targets the fitted type
size, and so the served type floor and the measured `295/chars` law, would mean a different thing
per platform. Tabular figures, and the build fails loudly rather than falling back.
It is **JetBrains Mono NL Regular**, embedded in the binary (`include_bytes!`, so its absence
is a compile error) and never registered where a project's text could reach it
([ADR-0122](docs/adr/0122-the-chrome-face-is-an-embedded-ligature-free-monospace-and-ofl-joins-the-allowlist.md)).
Monospaced and ligature-free, so for this face the `295/chars` law is exact arithmetic — every
character advances 600/1000 em. The code calls it the *chrome face* (`fonts::chrome`).
_Avoid_: the label font (ambiguous with a project's declared fonts), a fallback font, the
system face

**Blind spot**:
Something the contact sheet's **rule** structurally cannot see, whatever the document says —
change inside a visual state, easing between keyframes, detail below the served tile width, a
relation across two sheets, audio, motion. A blind spot is a property of the rule, never of the
document, so it is **never a finding**: it is the sheet's NOT CHECKED block, printed on every
answer including perfect ones, as a fixed list of `blind_to` tokens each bound to one sentence
([ADR-0105](docs/adr/0105-the-sheets-refusals-are-invocation-errors-its-blind-spots-are-not-findings-and-an-unpainted-state-is-quantization.md),
[ADR-0094](docs/adr/0094-the-sheets-instants-are-visual-states-sampled-at-the-first-painted-frame.md)).
Where one call can see past it, its sentence names that call.
_Avoid_: limitation, caveat, unchecked (a `U-` finding means *could not establish for this
document*, not *structurally cannot*)

**Reader check**:
The paragraph every contact sheet answer carries about its **reader**, not its rule
([ADR-0114](docs/adr/0114-the-sheet-carries-a-reader-check-a-handshake-on-its-first-label-that-names-no-reader.md)).
The served floors hold for a capable reader and mean nothing for a weak one, and Montagent
cannot know which is calling. So the answer offers a **handshake**, not a self-grade. It says
where the tile labels are (the strip beneath each tile, not the video's own captions) and quotes
the first tile's label exactly. It says the provenance list is the complete record and the sheet
a picture of it, and names `frame --at` for a reader whose strip does not match. It is
**not a blind spot**, since what a reader can see is not a property of the rule. It is never a
finding, and it names no model. It orients a reader that can see, and catches a weak reader
that reports honestly. **Its pass is not evidence of reading**: a reader that copies the quoted
string reports a true match, and nothing can tell that from a real one
([ADR-0116](docs/adr/0116-the-reader-check-orients-a-reader-that-can-see-and-its-pass-is-not-evidence-of-reading.md)).
_Avoid_: reader token, capability disclaimer, model class, legibility check (the floors are
the legibility checks; this checks the reader against them)

**Unpainted visual state**:
A visual state — a span over which the set of visual elements is constant — that contains no
frame the grid paints, because the two boundaries around it land on one frame. The document
declares it and the rendered video never shows it, so it is a fact about the **document**,
reported as `N-QUANTIZATION` at `review`
([ADR-0105](docs/adr/0105-the-sheets-refusals-are-invocation-errors-its-blind-spots-are-not-findings-and-an-unpainted-state-is-quantization.md)).
`validate` reports every one, and the contact sheet draws no tile for it and names it in
`skipped` with reason `no-grid-frame` — one finding, from one selection of visual states, in
both verbs.
Distinct from a **skipped** entry in general: an infill tile evicted under budget is also
skipped, and that is a fact about the answer, not the document. A span that differs from its
neighbours only in audio is not a visual state at all — the fixture's 4 ms interval at
56112–56116 is one such, absorbed into a run that paints once audio is set aside.
_Avoid_: short run, sub-frame run, dropped state

**Keyframe change point**:
One keyframe's `t` as the contact sheet sees it — the other kind of boundary the document states,
beside a visual state's own ([ADR-0106](docs/adr/0106-the-sheets-opt-ins-are-keyframes-and-infill-ceiling-and-a-keyframe-tile-is-sampled-where-its-change-first-paints.md)). The sheet counts only change
points **interior to a visual state, on an element visible there**: one on a run boundary is
already that run's tile, and one outside its element's lifetime (a trimmed move) is never on
screen — the fixture declares 14 keyframes and has **zero** such change points. Membership is read
off the document; whether one is **tiled** is a fact about the sheet. It is sampled, like a run,
at **the first frame the grid paints at or after it** — the only frame that shows a `step` at
all — so several may share one tile, one may land on a run's own tile and add nothing, and one
with no frame left in its run lands on the next run's tile. Every answer reports `tiled` and
`untiled`, and names each untiled one with its reason, `not-requested` or `no-grid-frame`, apart
from skipped states: an untiled change point is never a finding
([ADR-0129](docs/adr/0129-an-untiled-keyframe-point-is-named-in-the-census-and-never-in-skipped.md)).
`--keyframes` gives each `not-requested` one a tile of its own. Never an easing midpoint or a
curve's extremum: those are computed, not stated. A `volume` change is audible, so it is not one.
_Avoid_: keyframe (the record, not the instant), keyframe tile (the picture, not the instant),
animation point

**Infill ceiling**:
The longest span, in painted time, that a contact sheet may leave between two consecutive
tiles **of any class** — document-derived, keyframe or infill alike — asked for with
`--infill-ceiling <MS>` ([ADR-0106](docs/adr/0106-the-sheets-opt-ins-are-keyframes-and-infill-ceiling-and-a-keyframe-tile-is-sampled-where-its-change-first-paints.md)). It is a
**bound, not a count and not a period**: infill tiles are inserted only where the tiles the
document already produced sit further apart than the ceiling, so a busy range may gain none and
an empty one many. Infill is what closes the span; the span itself is measured across every
tile, and the last tile's span runs to the end of the range. Infill is fitted into the slots
left at the rung the other tiles fixed — the tiles that rung still admits, so a single long state
has seventeen — and never degrades the sheet, so a request may be honoured only coarser than
asked — and then the answer states the **achieved** ceiling beside the requested one, uniform
across the whole sheet, never a ceiling that holds in some stretches and not others
([ADR-0130](docs/adr/0130-infill-fills-the-rung-not-the-grid-at-the-latest-frame-within-the-ceiling.md)).
_Avoid_: gap ceiling (**Gap** is a stretch of a track with no element — a different thing),
max gap, infill count, infill interval / period

**Reference class**:
CapCut and Premiere: the tools whose mechanisms are Montagent's first source of precedent.
Read by mechanism, not by name; a mechanism confirmed in either tool is a precedent
([ADR-0145](docs/adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)).
_Avoid_: "in scope", "out of scope" — a capability outside the reference class can still
enter, and one inside it must still keep the invariants

**Entry test**:
What a capability with no precedent in the reference class must pass on top of the four
invariants every capability keeps (literal values, closed vocabulary, checkable by
`validate`, exact-string replace): the owner accepts a rendered prototype of it. Passing is
necessary, not sufficient
([ADR-0145](docs/adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)).
_Avoid_: approval, sign-off — passing lets the capability's ADR be argued; it does not
decide it

## Findings and reports

The vocabulary above is the document's. This is the tooling's: what Montagent has
to say about a document, and the shape it says it in. Every term here is defined
by an ADR rather than invented at the source tree, and the entries cite the ADR
that owns each one.

**Finding**:
One thing a verb has to say about a project, as a structured object: a stable
code, a class, a location, and the fields its own template names. Findings are
the only thing a verb reports — *"an error is a finding"*, including a malformed
file and a bad invocation, so there is exactly one thing to parse across the
surface. A finding states a fact derivable from the document, the media on disk
and the published rendering semantics, and never a verdict that requires knowing
what the video is for.
([ADR-0006](docs/adr/0006-validate-reports-facts-and-render-enforces.md),
[ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md))
_Avoid_: warning, violation, diagnostic, issue, problem — each prejudges a
badness that the class is what actually states

**Code**:
A finding's stable identifier, prefixed by the class it usually carries:
`E-SOURCE-OVERRUN`, `R-VISUAL-GAP`, `N-QUANTIZATION`. The prefix is a
convention and not a rule — `R-BOX-SLACK` is a `note`
([ADR-0058](docs/adr/0058-text-box-slack-is-a-note-with-sibling-census.md)) —
because the class is computed from the consequence while the code is fixed
when the check is written. One code, one field set, one template.
_Avoid_: rule id, error code (it names classes that are not errors)

**Check**:
The thing that emits findings: one question asked of the whole project, always
run, with no fast mode and no way to narrow what is checked. The check, not the
instance, is the unit of several rules — a check is refuse-class or
advise-class, and a check either is fact-only or borrows an external threshold —
so a check never triages its own matches.
([ADR-0006](docs/adr/0006-validate-reports-facts-and-render-enforces.md),
[ADR-0043](docs/adr/0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md))
_Avoid_: rule, lint, validator (a check is not a tool)

**Check set**:
A named group of checks that a run either completes or does not. A report records
the ones it completed, and records a set only once the set finishes. There are
five: `document` (validate's checks that read only the file and its fonts), `disk`
(validate's checks that need the media tools), `layout` (the one check `fmt` runs),
`drift` (`compare`'s own), and `deliverable` (`verify`'s measurement of the
rendered file). Running `validate` means running `document` and then
`disk`, and the split is there because a run can stop between the two: with
`ffprobe` missing, the document half's findings stand and the disk half never
runs. A project with no media completes `disk` trivially. A check set is not a
class. A verb that runs no check set can still raise a finding: a refusal is a
finding without being a check, and so is the `N-QUANTIZATION` that `frame`'s range
mode raises. So a zero means something only for a class that some completed set
could have raised.
([ADR-0112](docs/adr/0112-a-report-names-the-check-sets-that-ran-and-prints-no-zero-it-did-not-earn.md),
[ADR-0119](docs/adr/0119-verify-s-measurement-is-a-fifth-check-set-and-a-checkless-verb-s-finding-prints-after-its-scope.md))
_Avoid_: engine, scoreboard, scope (that is the NOT CHECKED block's), coverage
(reads as test coverage); `validate` as a set name (it is the verb that runs two)

**Class**:
Which of five kinds a finding is. Three are severities, named for what the
reader does rather than for how bad it is: `error` (the render is refused or is
guaranteed wrong), `review` (legal, renders, and you must look at a frame to
know if it was meant), `note` (a fact you may want and will not act on today).
Two are not severities at all: `UNCHECKED` (the question was unanswerable — an
unprobeable source, or one no observed identity can be matched to; `validate`-only,
because `render` must decode the source anyway, and `validate`'s unchecked set
therefore contains every source `render` declines to use) and `LAYOUT` (canonical key order; `validate`-only, and never a reason
to refuse a render). A class is computed from the consequence at an instant, not
fixed per check: the same gap is `review` when nothing else covers it and a
`note` when something does.
([ADR-0006](docs/adr/0006-validate-reports-facts-and-render-enforces.md),
[ADR-0013](docs/adr/0013-fitted-extents-floor-and-the-nine-origin-keywords.md),
[ADR-0041](docs/adr/0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md),
[ADR-0093](docs/adr/0093-renders-world-effects-are-findings-and-an-error-withholds-the-deliverable.md))
_Avoid_: severity (only three of the five are), level (the three-way ladder
only), category, priority

**Repair**:
The field every `error`-class finding carries, in one of exactly two forms.
*Advise-class* states a value, when the correct fix is fully determined by the
document, the media on disk and the published rendering semantics.
*Refuse-class* states `"none"`, when the fix depends on knowing what the author
meant — and that refusal is a guarantee no flag, force mode or write tool may
lift. Which of the two a check emits is decided once, when the check is written,
and holds for every instance it matches, including the ones that look safe.
Where the refusal is an intent fork, a fact that bears on only one branch, or
names a concrete candidate for one branch, is a repair by another name, and the
finding does not carry it; naming the complete set of legal options, unranked
and covering every branch, or the refusal's own consent handle is not one, and
nor is a census. A consent handle names the fact the finding reported and is
refused when it does not match; anything that lifts a refusal without naming
what it consents to is a bypass, and there is none. It is orthogonal to class,
not a fourth severity. Whether the binary reaches
findings that are not about a document at all — a file that would not parse, an
invocation that was wrong — is open
([#224](https://github.com/MBehtemam/Montagent/issues/224)).
([ADR-0043](docs/adr/0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md))
_Avoid_: fix, suggestion, autofix, quick fix — each implies something will
apply it

**Census**:
A grouping of the siblings a finding affects by an observable, document-derived
fact — *"four of five are at y = 1597, one is at 1537"* — which never ranks the
groups or says which is correct. It is what a refuse-class finding carries
instead of a repair: narrowing where to look is admissible where stating a fix
is not. Every sibling sits in exactly one group: a grouping whose
groups all hold the same members partitions nothing, and is not a census
however it is shaped.
([ADR-0043](docs/adr/0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md),
[ADR-0006](docs/adr/0006-validate-reports-facts-and-render-enforces.md))
_Avoid_: histogram, breakdown, majority (it does not have one)

**Citation**:
The inline record of where a borrowed number came from. Threshold provenance,
not severity, decides whether a check may state something: a number that decides
whether a finding fires must be derivable from the document or from the format's
own rendering semantics, and a check that borrows one from outside both may only
do so at `review` or `note`, must state the raw measurement as its substance,
and must cite the source in the finding and in its own ADR. Binding, not
best-effort — an optional citation requirement leaves no provenance to inspect.
([ADR-0061](docs/adr/0061-validate-judgment-boundary-threshold-provenance-and-a-fenced-exception.md))
_Avoid_: reference, source (already spoken for — see Source), attribution

**Report**:
One verb's whole answer: its findings, a count per class of the findings it
raised, the **check sets** that produced them, an exit code, the
`NOT CHECKED` block that states the report's own boundary, and — for a run that
read the disk — the **cache misses** and the **media facts** it established. The
cache miss is there because ADR-0006 put it there (*"report the cache miss,
unprompted, at the top"*) and ADR-0011 made it load-bearing rather than
incidental: the document records no source duration, so that line is the only
thing announcing a source that grew on disk. The media facts are there because a
run that measured a duration and printed none of it leaves the reader to
re-derive it. JSON is canonical and
the prose form is generated from it — `--json` prints the JSON *instead of* the
text, never alongside it. Errors and near-errors print in full while the
informational classes collapse to one counted line, because `0 errors, 47 notes`
reads as a pass and a noisy report manufactures false confidence faster than an
unrun one does. For the same reason the report prints a zero only for a class
some completed check set could have raised. A verb that runs none says
*"no checks run"* rather than six zeros shaped like a clean `validate`.
([ADR-0006](docs/adr/0006-validate-reports-facts-and-render-enforces.md),
[ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md),
[ADR-0112](docs/adr/0112-a-report-names-the-check-sets-that-ran-and-prints-no-zero-it-did-not-earn.md))
_Avoid_: output, results, log

**Probe sidecar**:
Where the local probe cache persists between runs: one JSON file per user under
the platform's cache directory, keyed on a canonicalised `(path, size, mtime)`
and holding the whole probe — the quad, the resolved dimensions and the `par`
that produced them, alpha and the audio facts. It lives outside every repository,
so it cannot be committed by accident and a project stays a movable unit. Its
purpose is the **cache miss**, not the saved work: without it, the line
announcing a source that grew on disk can only fire twice within one process, and
`probe` is CLI-only. Everything that can go wrong with it — absent, corrupt, an
unknown version, unwritable — is a cache miss and never a finding, because a
cache directory is not the project. Nothing about a **remote** source is ever
stored in it. Each entry also carries a **content guard**, and `montagent cache
clear` deletes the whole file — the one operation on it that reports a failure,
because there the delete *is* the request rather than something done in passing.
([ADR-0069](docs/adr/0069-probe-sidecar-is-a-per-user-json-cache-keyed-on-what-montagent-observed.md),
[ADR-0006](docs/adr/0006-validate-reports-facts-and-render-enforces.md),
[ADR-0056](docs/adr/0056-remote-source-probe-session-scoped-no-persistent-cache.md),
[ADR-0092](docs/adr/0092-a-probe-is-matched-on-an-observed-identity-and-guarded-by-its-contents.md))
_Avoid_: index, database, manifest, cache file (unqualified)

**Content guard**:
The cheap content fingerprint a probe-sidecar entry carries beside its key —
sha256 over the size, the first 64 KiB and the last 64 KiB — checked on every
cache **hit**. A **value**, never part of the key: the key decides whether the
bytes are worth looking at, and the guard decides whether anything actually
changed. It exists for the one invalidation `(path, size, mtime)` cannot see, a
**renumbering shuffle** — renaming files so each one's content lands on a
neighbour's existing name, which leaves every path present and, because `mv`
preserves mtime, leaves `size` as the only discriminator between two takes. A
mismatch is a `rewritten` miss, distinct from `changed` because both sides of the
key are identical. A guard that established nothing never invalidates anything.
([ADR-0092](docs/adr/0092-a-probe-is-matched-on-an-observed-identity-and-guarded-by-its-contents.md))
_Avoid_: checksum key, content key, hash key

**Tool qualification**:
The one null encode Montagent runs through a found `ffmpeg` to learn whether it
can do what Montagent asks of it: the three arguments that make up the floor
(`-fps_mode passthrough`, a filter graph read through `-/filter_complex`, and
`libx264`), on sixteen black pixels. It runs the first time a given `ffmpeg` is
resolved in a process, and its verdict is kept in memory and never on disk, so no
answer outlives an upgrade. It reads what the binary *does*, never its version
string, because a git build's version names no release and no version string
reveals a missing `libx264`. A failure is `E-TOOL-UNSUPPORTED`. It is not the
primary guard: that is the rule that a failed spawn is never an empty answer,
which also covers the breakages nobody has met yet.
([ADR-0115](docs/adr/0115-ffmpeg-7-1-with-libx264-is-the-floor-and-a-tool-qualification-finds-out.md),
[ADR-0113](docs/adr/0113-a-seek-whose-ffmpeg-failed-is-refused-never-read-as-no-frame.md))
_Avoid_: probe (the verb, and the probe cache), check (a check asks a question of
the project), version check

**Observed identity**:
The canonical path a probe recorded at the moment it ran (`Probe::identity`), and
the only thing a consumer may use to ask *"is this the same file?"*. It is
distinct from the probe's **source**, which is a *label* — whichever spelling the
run that first cached the probe happened to use, kept because it is what the
report names. Re-resolving that label is how the working directory became a third
input to `render`: a relative spelling cached by one run resolved nowhere in a run
started elsewhere, every audible element was declined, and the encoder emitted a
silent video at exit 0. `None` means *"do not know"* — a remote source, or no
local observation — and never a match.
([ADR-0092](docs/adr/0092-a-probe-is-matched-on-an-observed-identity-and-guarded-by-its-contents.md))
_Avoid_: path (unqualified), source path, resolved source

**World-effect**:
Something the render did to the world that the document did not ask for and cannot
be read off the document alone: an element not mixed, an element not painted, a
field parsed and then drawn without. It is an ordinary **finding** at an ordinary
class — `error`, because *"the render is refused **or is guaranteed wrong"*, with
nothing left over. The name is for the *signature*, not a sixth class: legal,
wrong, and `0 errors` anyway. There is one **code** per reason and the reason set
is closed, because a check's repair form is fixed when the check is written and one
code cannot be a `note` for one reason and refuse-class for another.
([ADR-0093](docs/adr/0093-renders-world-effects-are-findings-and-an-error-withholds-the-deliverable.md),
[ADR-0043](docs/adr/0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md))
_Avoid_: render warning, soft failure, degradation (the render did not degrade — it
was wrong), silent drop (it is no longer silent, which is the point)

**Promotion**:
The one rename that publishes a **deliverable**: until it happens the render's
output lives at a temp sibling and the declared `output` path does not exist. An
`error`-class finding **withholds** it — exit 1, and no file where one was asked
for — which is what makes the invariant *a file at the output path is a render with
zero errors* true. Declining to promote is the absence of a promotion, never
destruction. What is pre-flightable is decided before the encoder is spawned, so
wall clock is only ever spent on an error that is genuinely mid-loop; the audio mix
is wholly pre-flighted. A **preview** is not a deliverable and is not subject to
the rule.
([ADR-0093](docs/adr/0093-renders-world-effects-are-findings-and-an-error-withholds-the-deliverable.md),
[ADR-0021](docs/adr/0021-preview-budget-and-graceful-degradation.md))
_Avoid_: commit (spoken for by git), publish step, finalize, atomic write (that is
the mechanism, not the decision)

**Partial render**:
A `render` of a half-open range `[from, to)`, written to a derived path and refused
if that path is the declared `output`, so it is never a **deliverable**. Its
analysis is split, and its report says so: every `validate` check runs on the whole
project and any `error` refuses it, while **world-effects** are established only
inside the range. So a partial render can publish its file over a world-effect that
would refuse the full render, and a clean one says nothing about whether the full
render will pass. Its findings are the whole project's, printed as any report's are.
([ADR-0006](docs/adr/0006-validate-reports-facts-and-render-enforces.md),
[ADR-0093](docs/adr/0093-renders-world-effects-are-findings-and-an-error-withholds-the-deliverable.md),
[ADR-0121](docs/adr/0121-a-partial-render-s-world-effects-stop-at-its-range-and-its-report-says-so.md))
_Avoid_: scene render, draft render, preview (a different verb with its own budget)

**Attestation**:
What a **deliverable** carries saying which project wrote it — a `comment` tag on
the container holding the project file's **observed identity**, stamped at the
moment `render` encodes. It exists so that *"did this project produce the file
already at the output path?"* is a question Montagent **reads an answer to** rather
than infers: the cheap alternative, comparing the existing file's duration or frame
count against the document, is a re-derived label of exactly the kind
[ADR-0092](docs/adr/0092-a-probe-is-matched-on-an-observed-identity-and-guarded-by-its-contents.md)
ruled is not an identity, and it fails in both directions — silent on two language
cuts of one timeline, loud on a project's own second render. A file bearing another
project's attestation is refused **before the encoder is spawned**; one bearing none
is a `review`, because *no evidence* is weaker than *evidence of someone else* and
`error` means the render is refused or **guaranteed** wrong. A **preview** never
attests: it is not a deliverable, and a stamp would hand the next `render` a forged
licence to clobber. Since `montagent/2` it also carries a **staleness digest** and the
engine version, so it says *what* was rendered as well as *who* rendered it; ownership
is still the project identity alone, and a `montagent/1` stamp is still this project's.
([ADR-0104](docs/adr/0104-the-output-path-is-checked-for-a-foreign-deliverable-before-the-encoder-runs.md),
[ADR-0057](docs/adr/0057-font-vendoring-licence-gate-and-path-keyed-attestation.md),
[ADR-0117](docs/adr/0117-verify-measures-the-deliverable-with-the-decoder-and-a-stale-file-is-one-error.md))
_Avoid_: watermark (it is metadata, not pixels), signature (nothing is verified
cryptographically and nothing is meant to resist forgery), ownership, provenance
(too broad — this is one question, not a history)

**Stale**:
Said of a **deliverable** whose **attestation** is this project's and whose staleness
digest no longer matches the project as it stands: the document in canonical (`fmt`)
form, every local source's content fingerprint, and every font's `sha256`. A
whitespace or key-order edit is not staleness, because it renders identically; a
source swapped on disk is, even with its size and mtime unchanged (a **renumbering
shuffle**). The engine version is not in the digest, so an upgrade stales nothing.
`verify` reports a stale file as **one** `error` and measures nothing, because every
mismatch it would find descends from that one change. A digest that could not be
computed, or a stamp with none, is *staleness unknown*, never stale.
([ADR-0117](docs/adr/0117-verify-measures-the-deliverable-with-the-decoder-and-a-stale-file-is-one-error.md))
_Avoid_: out of date (says nothing about which input moved), dirty (git's word),
outdated render

**Should be heard**:
Where `verify` expects the mix to carry energy: an `audio` or `video` element is in
range, its source has an audio stream, and its resolved `volume` is above 0 on that
frame. A keyframed fade to 0 stops it being heard, and without `overrun: "loop"` an
element stops being heard where its source runs out. It is a claim about **spans**,
not elements: the file carries one mixed track, so a silent element under an audible
one cannot be seen in it, and `verify` says so in its `NOT CHECKED`. Where the mix is
below R128's −70 LUFS gate inside such a span for at least a 400 ms block, the census
splits the elements by whether their own source is silent there (the author's
material) or audible (the engine).
([ADR-0117](docs/adr/0117-verify-measures-the-deliverable-with-the-decoder-and-a-stale-file-is-one-error.md))
_Avoid_: audible (a measurement, not an expectation), expected audio, has sound

**NOT CHECKED**:
The block every report ends with, unconditionally, clean runs included: this
file was not compared against any prior version or instruction, and Montagent
cannot tell you whether it says what you meant it to say. It is there because
without it a clean run is read as *"the file is right"*, which is the rejected
`sequence` label wearing a `validate` label instead. A report that did not
complete both of validate's check sets also says here which did not run, so a
verb that checks nothing does not end with `validate`'s statement of scope. A verb
with limits of its own lists them beneath it, one bullet each (`not_checked_also`):
`render` names `verify`, and `verify` states what one mixed track cannot show.
([ADR-0006](docs/adr/0006-validate-reports-facts-and-render-enforces.md),
[ADR-0112](docs/adr/0112-a-report-names-the-check-sets-that-ran-and-prints-no-zero-it-did-not-earn.md),
[ADR-0117](docs/adr/0117-verify-measures-the-deliverable-with-the-decoder-and-a-stale-file-is-one-error.md))
_Avoid_: caveat, disclaimer, limitations

**Resource**:
Something the MCP server publishes for an agent to *read* rather than to call —
the published JSON Schema (`montagent://schema.json`) and the format docs
(`montagent://format.md`), and any smaller piece of the format published beside
them: the schema's pieces, and the format docs' pages, which `format.md` lists
and which are served but not listed. The count is not a rule; what is a rule is that **every resource is
generated from the same types that parse the project, or embedded verbatim** —
no fact about the format is written down twice. Being resources rather than
verbs is the whole point: an MCP tool schema costs the agent context on every
turn, and a resource costs no tool slot at all, so this is what makes *"how does
the agent know how to edit `project.json`"* answerable the same way
`package.json` is. The schema one is **generated on each read**, never a
committed copy served back, so the published schema and the enforced one stay one
artifact, and it stays **whole**: it is the full schema, its descriptions stay
beside their keys, and pieces are cut from it rather than replacing it.
Strictly the protocol's word, not the document's: an element's `source`
is never a resource — that noun is on **Source**'s avoid-list and stays there.
([ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md); the URIs, names
and media types are named by
[ADR-0080](docs/adr/0080-the-scaffold-writes-what-it-was-told-and-the-two-resources-are-named.md)
and do not move; the pieces and the generation rule are
[ADR-0137](docs/adr/0137-the-schema-is-also-served-in-pieces-through-one-index.md))
_Avoid_: asset, document, endpoint, attachment; and never for an element's
`source`

**Schema index**:
The **Resource** an agent reads first to learn the format's shape: every element
type with its required keys, every effect with its required parameters, and the
address of every **Schema piece**. It holds no rules — those stay in the pieces —
and its address is the only one of the pieces' family that does not move.
([ADR-0137](docs/adr/0137-the-schema-is-also-served-in-pieces-through-one-index.md))
_Avoid_: card, cheat sheet, summary, table of contents

**Schema piece**:
One part of the published schema served as its own **Resource**, small enough to
arrive in one tool result: the project's top-level keys, one element type, one
effect, or one other definition. Cut from the whole schema, never written
separately; reached through the **Schema index**, whose listing is the only
promise about where a piece lives.
([ADR-0137](docs/adr/0137-the-schema-is-also-served-in-pieces-through-one-index.md))
_Avoid_: chunk, fragment, slice, sub-schema

**Verb**:
One operation on the surface — `validate`, `query`, `frame`, `measure`,
`compare`, `render`, `preview`, `verify`, `shift`, `create_project`, `probe`, `fmt`,
`timeline`. The surface's job is to make reading, checking, comparing and
rendering cheap, not to provide editing verbs: the agent edits the file with the
tools it is already strongest with. Ten are MCP tools and thirteen are CLI
commands (nine and twelve until `verify`, [ADR-0117](docs/adr/0117-verify-measures-the-deliverable-with-the-decoder-and-a-stale-file-is-one-error.md)), and the asymmetry is deliberate — an MCP schema costs context on every
turn, a CLI subcommand costs nothing until invoked. Every write verb returns the
new state's findings, never an `ok`. (`preview` was missing from ADR-0011's
table, which this entry calls authoritative;
[ADR-0078](docs/adr/0078-preview-is-the-ninth-mcp-verb-and-its-unstated-readings-are-ratified.md)
gives it a row and settles the counts, resolving
[#295](https://github.com/MBehtemam/Montagent/issues/295). ADR-0011's own prose
still opens with *"nine verbs and two resources"* above a table of eleven, and
its *"eight MCP tools, eleven CLI commands"* line no longer holds — an ADR is
amended, never rewritten, so read its banner. The nine is asserted rather than
restated — `crates/montagent/tests/adapters.rs` reads `tools/list` off the running
server and names all nine. `create_project` is the one verb whose two surfaces
spell it differently: the MCP tool is `create_project`, the CLI command is
`create-project` with the underscore kept as a permanent alias, because
ADR-0011's write-tool invariant binds the MCP surface and not argv
([ADR-0080](docs/adr/0080-the-scaffold-writes-what-it-was-told-and-the-two-resources-are-named.md)).)
**The nine and the twelve survive `frame`'s range mode**
([ADR-0097](docs/adr/0097-the-range-is-from-to-on-both-surfaces-and-the-caption-becomes-an-attribution-obligation.md)):
that amendment gives the `frame` row arguments and adds no row, because **a range is arguments,
not a verb** — the same distinction ADR-0037 drew when it refused a tenth verb for a new
*capability*. A contact sheet was argued for as an extension of `frame` rather than a tenth
verb precisely on this surface's own cost model: an MCP schema costs context on every turn,
and *what does it look like* is the question `frame` already answers.
([ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md),
[ADR-0078](docs/adr/0078-preview-is-the-ninth-mcp-verb-and-its-unstated-readings-are-ratified.md),
[ADR-0080](docs/adr/0080-the-scaffold-writes-what-it-was-told-and-the-two-resources-are-named.md),
[ADR-0097](docs/adr/0097-the-range-is-from-to-on-both-surfaces-and-the-caption-becomes-an-attribution-obligation.md))
_Avoid_: command (the CLI spelling only), tool (the MCP spelling only),
endpoint, action

**Registry**:
The single declaration of every code Montagent can emit, and what is true of it:
which classes it may carry, whether it is refuse- or advise-class, whether its
threshold is internal or external, the ADR that owns it, and its template. It
exists so those facts are looked up rather than restated at each call site — the
same reason canonical key order is tied to the published schema rather than to a
parallel hand-maintained list.
([ADR-0041](docs/adr/0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md))
_Avoid_: catalogue, table, rule set

**Skill**:
End-user guidance an **Agent** installs alongside Montagent: workflow, recipes,
craft and capability discovery. A skill never states a format fact — the shape
belongs to the published schema, the rules to the format docs, and what is wrong
with a project to the **Finding**s — so it points at those instead of restating
them. When a skill and the installed Montagent disagree, Montagent wins.
_Avoid_: prompt, guide, docs; and never for guidance to agents developing
Montagent itself

**Router skill**:
The `montagent` **Skill**. It holds the loop from project file to delivered video
and the **Capability map**, beside a table of which **Job skill** covers each kind
of piece. Its description is written to load on any Montagent task; nothing enforces
that it loads first.
_Avoid_: entry skill, main skill

**Capability map**:
The one place an **Agent** learns what Montagent can do, held in the body of the
**Router skill** so it is read before anything is designed. It has three parts: what
Montagent does, grouped by area, each capability named with the keys that spell it;
what it cannot do yet, one item per capability, each a **Workaround** naming the issue
that retires it; and what it does not do by design, each with what to do instead and
the ADR that refused it. It names keys but never states their values or rules, which
belong to the schema and the format docs
([#700](https://github.com/MBehtemam/Montagent/issues/700)).
_Avoid_: feature map, feature list

**Job skill**:
A **Skill** the **Router skill** points to. It owns either the recipes for one kind
of piece (motion graphics, footage, character animation), or the cross-cutting
craft every piece is judged by.
_Avoid_: topic skill

**Workaround**:
A recipe in a **Skill** that compensates for something Montagent cannot yet do,
and names the issue whose resolution would retire it. It is written down as
temporary by design: when that issue closes, the workaround is either replaced
by the real capability or becomes plain recipe.
_Avoid_: hack, trick, fake

## Rejected terms

These words are deliberately absent. Each is standard vocabulary in a comparable
tool, which is exactly why using it here would mislead.

**Scene**:
Elsewhere a scene owns its own clock, so its children's times are relative to it.
A `track` is the one container Montagent has, and the distinction is exactly this:
a scene owns a clock, a track owns a stacking position. A track has no start, no
duration and no origin, so there is nothing for a child's time to be relative to.
Nesting still *invites* the assumption — JSON2Video documents the silent failure
that follows — which is why the times stay absolute and why the schema says so
loudly rather than relying on this paragraph.

**Clip**:
Elsewhere a clip pairs a visual with its audio, and its duration follows that
audio. In Montagent audio is an ordinary element with its own independent time
range, so no such pairing exists.

**Asset**:
Elsewhere an asset is an imported file declared once and referenced by id.
Montagent writes the file's location on the element that uses it. Note also that
an asset is *not* a reusable configured object in the sense of a Unity prefab —
that is templating, and it is out of scope for v1. The one carve-out is **Font**,
for reasons recorded in [ADR-0002](docs/adr/0002-inline-source-no-asset-table.md);
media sources never get a table.

**Anchor, for text positioning**:
Every comparable tool — ASS's `\an`, CSS, every GUI editor — calls the nine-way
positioning point an *anchor*. Montagent spends that word on layer-relative stacking
instead, so the positioning point is **`origin`**. The collision is recorded here
because it is near-certain to be rediscovered: the positioning concept appears on
essentially every text element, and layer anchoring appears rarely. The schema catches
the rediscovery **on shape, not presence**: `anchor` carrying a *string* is an error
naming `origin`; `anchor` carrying a below/above *object* is the ordinary feature above.

**Box**:
Retired, having meant two different things at once: a literal `[x,y,w,h]` rect on an image
and, in [ADR-0007](docs/adr/0007-text-runs-literal-size-declared-fonts.md), *the id of the
element you must fit inside*. A field that is sometimes a rect and sometimes a reference
cannot even produce a good error message. The rect is now `x`,`y`,`origin`,`width`,`height`
plus `clip`; the containment reference is now literal `width`/`height`, because fifteen of
the fixture's twenty-two text elements have no element behind them to name.

**Whether its error is refuse- or advise-class is open.** On an image the old `box` was the
*aperture*, not the drawn rect: the migration writes `clip` from the array and derives
`width`/`height` by `cover` against the **source's pixel dimensions**, which are on the
media rather than in the document. On a shape no such derivation is needed. So one spelling
has two repairs, one of which reads the disk — and `validate` cannot read it until the probe
lands. The check refuses meanwhile, under
[ADR-0043](docs/adr/0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md)'s
standing rule that a check refuses where any instance it can match could be load-bearing.
[ADR-0068](docs/adr/0068-the-bare-mask-key-retires-masks-are-effects-members.md)'s remark
that *"`box` was migrated by arithmetic script"* is a historical note about `migrate.py`,
not a classification. The placement is provisional; the ruling belongs to an ADR
([#228](https://github.com/MBehtemam/Montagent/issues/228), evidence in
[`docs/research/juries/retired-spelling-classes/`](docs/research/juries/retired-spelling-classes/README.md)).

**Align, for images**:
`align` means one thing: how a text element's lines align to each other
(`start`/`center`/`end`). On an image the same word once meant *which part of the source
survives the crop* — that job is now `x`/`y`/`origin` against the aperture's own `clip` (see
Clip), the same fields that place any other visual element's rect, including the slack a
`contain` fit leaves inside its aperture. `align` on a non-text element is a schema error
naming `x`/`y`/`origin` and `clip` as the replacement. The collision is recorded because it
caused a live misreading: the fixture's `align:"left"` on text was transcribing ASS `\an4`,
**left-middle**, so its `y` was a centre and nothing in the file said so.
([#52](https://github.com/MBehtemam/Montagent/issues/52))

**Its repair class is open for the same reason `box`'s is.** On an image the word meant
exactly what `gravity` meant, so repairing it needs the same absent fact, and no ADR has
ruled. The check refuses provisionally; see the note under Box
([#228](https://github.com/MBehtemam/Montagent/issues/228)).

**Line, polygon**:
Not element types. A line is a two-vertex open **path** and a polygon is a closed path of
corners; a second spelling of one drawing breaks exact-string replace. A `path` written as
an SVG `d` string is still refused: a mini-language inside a JSON string is unreadable by
reading and unmatchable by exact-string replace.
([ADR-0154](docs/adr/0154-a-point-list-enters-as-one-path-element-in-integer-pixels-from-the-declared-box.md))

**Repeat, repeater, clone**:
Elsewhere one layer draws as N offset copies. Montagent draws exactly one thing per element,
so every copy is written out as its own element with its own `id`. A drawn copy with no `id`
in the file is invisible to the checks and has no string an edit can target
([ADR-0148](docs/adr/0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md)).

**Gravity**:
Retired entirely by [ADR-0015](docs/adr/0015-fit-is-a-derivation-claim-and-gravity-retires.md).
It meant *which part of a source survives a crop* — a quantity the rect's `x`, `y`, `origin`
and the static frame-space `clip` already determine at render, with no separate degree of
freedom for a `gravity` field to spend. That structural argument is general and holds. A
schema error on every element type; on an image the message names `x`/`y`/`origin` and
`clip`, on text and shapes it names `origin`.

**Repairing a file that still carries one does not follow the same rule.** In the fixture,
every occurrence was `gravity:"top"`, and every one of those elements was already positioned
(`y:0` under a top-aligned `clip`) so the top of the source was what showed — the value was
inert *because of that specific geometry*, not because gravity itself never mattered. A
`gravity:"bottom"` element needs `y` recomputed to the aperture's opposite edge
(`clip.y + clip.h − height`, e.g. `1300 − 1912 = −612` for a 1912px rect in a 1300px
aperture) before the key is dropped; deleting it unmoved silently swaps which 1300px band of
the source is on screen. Measured, not assumed: six agents repaired a fork of the pre-ADR-0015
fixture with two `top`→`bottom` values flipped, given `validate` output alone versus
`validate` output plus this entry as it previously read — **2 of 3 correct without the entry,
0 of 3 with it**, because the entry's own framing (*"no freedom left to spend," met by
"copying a neighbouring element"*) read as license to discard the value on any element, not
only the ones where it happened not to matter.
([#76](https://github.com/MBehtemam/Montagent/issues/76),
[`docs/research/juries/format-versioning/experiment-gravity-fork/`](https://github.com/MBehtemam/Montagent/tree/main/docs/research/juries/format-versioning/experiment-gravity-fork))

**Derivation claim**:
A **renderer-ignored declaration recording how the author computed a literal**, whose only
consumer is `validate`. `fit` is the first one (see **Fit** below); ADR-0086 generalises the shape into a
pattern with six conditions of admission — no renderer reads it, `validate` is the only
consumer, the rule set is finite and published, at most one argument with its type fixed
per rule, rules do not compose, and the declaration is optional with absence meaning *no
claim*. A violated claim is **always `error`**; only the repair varies (advise-class when
the claim names a direction, refuse-class `"none"` when it is symmetric). A claim with no
arithmetic to re-derive cannot be violated at all.
([ADR-0086](docs/adr/0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md),
[ADR-0015](docs/adr/0015-fit-is-a-derivation-claim-and-gravity-retires.md))
_Avoid_: provenance (unqualified — already spoken for), annotation, hint, metadata

**`t_from`**:
The time axis's derivation claim: an optional member of a keyframe record annotating that
record's own `t`, always an object. Two rules, both measured 7 of 7 on the fixture —
`element-start` (no argument) and `after-previous` (one non-negative integer `ms`).
Direction is carried in the rule **name**, never in the sign of the argument, so no rule
needs signed arithmetic. *"Previous"* is positional and well-defined because ADR-0082
requires ascending `t`; it is not a reference. Key order is `t`, `t_from`, `v`, `ease`.
Defined in [ADR-0086](docs/adr/0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md).
_Avoid_: derivedFrom, anchor, t_rule

**Live versus recorded reference**:
The format has **zero *live* element-to-element references** — the restatement ADR-0086
makes of ADR-0012's invariant. A *live* reference is one the **renderer** resolves, and it
stays rejected: it breaks read-by-reading, degrades to a stale literal on the first shift,
and gives one element's geometry a silent second author. A *recorded* reference is read
only by `validate`; it does not degrade on a shift, it **detects** one. The permission to
name an id is **per-axis and earned**, granted only by an ADR carrying a measurement that
inference fails — and ADR-0086 grants it to no axis. A dangling recorded reference is an
`error` with a refuse-class repair, on referential integrity: a closed schema that admits a
reference type must require it to resolve.
([ADR-0086](docs/adr/0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md),
[ADR-0012](docs/adr/0012-flat-transform-keyframes-carried-by-their-element.md),
[ADR-0036](docs/adr/0036-shift-preambles-coincident-instants-validate-and-compare-stay-out.md))
_Avoid_: pointer, link, binding (a recorded reference binds nothing)

**Fit**:
A **derivation claim, not a layout mode**. `fit` records the rule by which the author computed
`width`/`height` from the source and the aperture; no renderer reads it, because the declared
rect is what gets drawn. Its only consumer is `validate`. The closed set is `cover`, `contain`
and `literal`, and the box a rule works against is `clip`'s `w`/`h`.
Defined in [ADR-0015](docs/adr/0015-fit-is-a-derivation-claim-and-gravity-retires.md).

**Source dimensions**:
The input to `fit`'s arithmetic. One type-generic pipeline: decode, resolve rotation
(the container's track-level display transform; codec-level orientation metadata is not
consulted), apply pixel aspect ratio, round to one integer pair. Images are the
degenerate case — PAR is `1:1` and EXIF orientation is the only rotation signal — so
this is the same rule ADR-0015 stated for images, generalised, not replaced.
Defined in [ADR-0023](docs/adr/0023-video-source-dimensions-par-and-rotation.md).

**Source alpha**:
Whether a source carries an alpha channel — a fact about the **file**, not about its
pixel format, and the two are not the same question. ProRes 4444 declares its alpha in
the pixel format (`yuva444p10le`); VP9-in-WebM keeps it in a **side stream** and reports
`pix_fmt=yuv420p` with an `alpha_mode` tag beside it, so a reading taken off the pixel
format alone answers `false` for a file that carries one. The probe answers from either
signal and reports which settled it (`pixel_format` or `container_declaration`), for the
reason rotation names its own source: two signals of unequal strength, one cached answer.
The side-stream case is also the only one that constrains the **decoder** — `libvpx-vp9`
is forced for it and for nothing else, since a user-supplied `ffmpeg` may have no libvpx.
Defined in [ADR-0089](docs/adr/0089-source-alpha-is-a-file-level-reading-and-vp9-needs-its-own-decoder.md).
_Avoid_: transparency, alpha channel (the file carries one; this term is the reading),
pix_fmt alpha

**Sampled frame**:
Which frame of a source a given instant shows: the **last frame whose start is at or before
it**, never the next one. The distinction is not pedantry — `ffmpeg`'s own seek answers the
opposite question, the first frame at or *after* the instant, so an element whose `start` is
off its source's frame grid used to draw every frame one early and to draw nothing at all at
the end of its range. The frame grid is a property of the **timestamps a source carries**, not
of the frame rate it declares: the repository's own reference MP4 runs at 25 fps from a
42.031 ms origin with four dropped frames, while its container reports `50/1` and its average
reports `24.9785`. So Montagent does not compute the grid — it asks the decoder for the frame
at or before the instant, over a bounded backward window, which is exact for constant,
fractional, dropped-frame and variable rates alike. The clamp is symmetric at both ends: an instant past the end of the
source resolves to its final frame, which is what `overrun: "hold"` means, and an instant
*before* the first frame — a source whose own origin is late, as the reference MP4's is by
42 ms — resolves to that first frame rather than refusing. An instant further outside the
source than the window reaches is an `error`, because then the source really does not cover
the range the document declares.
Defined in [ADR-0096](docs/adr/0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md).
_Avoid_: nearest frame (it is not the nearest — it is the one at or before), seek target,
current frame

**PAR (pixel aspect ratio)**:
Applied, not ignored — silence here would repeat the EXIF-orientation divergence ADR-0015
already legislated against, and would leave `validate` disagreeing with any renderer whose
decode path applies it. Declared explicitly on the element as `par: [num, den]`, an exact
integer-pair rational populated by an ingest/authoring tool, never silently re-probed from
whichever of the container/bitstream layers that can disagree with each other. Defaults to
`[1, 1]`; a schema error on any non-video element. Defined in
[ADR-0023](docs/adr/0023-video-source-dimensions-par-and-rotation.md).

**Fill / none / stretch, for `fit`**:
`fill` is spent — it is a shape's paint colour ([ADR-0014](docs/adr/0014-stroke-is-paint-the-text-box-is-required.md)).
`none` is a **false friend**: CSS `object-fit: none` means *intrinsic size*, which this format
cannot express, so it is a schema error naming `literal`. `stretch` was rejected as naming an
effect that is constant across the vocabulary — every value resamples to the declared rect —
and so distinguishes nothing. The collisions are recorded because they are the CSS-habit
reach: three of eight agents tried `none` or `fill` before finding the legal value.

**Literal, versus declared**:
The escape value is `literal`, continuing [ADR-0007](docs/adr/0007-text-runs-literal-size-declared-fonts.md)'s
*"Literal `size`. No fit-to-box."* — this format's established opposition between a literal
number and a fit rule. `declared` was rejected despite being a jury plurality: the corpus
already spends it generically, 14 times as "declared rect" and 6 as "declared extent", so it
describes all three values and distinguishes none of them.

**Weight / bold**:
Not a field. A different weight is a different font file — with one declared file
and no family to search, `bold: true` could only mean synthetic emboldening, which
is renderer-specific and machine-dependent. Expect to reach for it out of CSS habit;
the schema rejects it and names the replacement.

**Its error is refuse-class, provisionally.** The replacement names a *file*, and the file
may not be vendored yet — so the repair is an instruction rather than a value, and an
advise-class finding whose repair cannot be applied verbatim spends the guarantee that
field exists to give. What is missing is an asset rather than an intent, which fits neither
arm of [ADR-0043](docs/adr/0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md)
cleanly; the ruling belongs to an ADR
([#228](https://github.com/MBehtemam/Montagent/issues/228)). The spelling sits on the text
element as often as on a run — the one real project file carried `"weight": "bold"` on all
22 of its text elements.
