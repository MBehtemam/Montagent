# Montaget

Montaget is a video editor whose project format is designed to be authored and
edited by an AI agent rather than dragged around in a GUI. A project states, by
being read, what is on screen at any given moment; Montaget renders it.

## Actors

**Montaget**:
The tool — a renderer and an MCP server. It is deterministic and contains no
model: given the same project and the same files it produces the same video
every time.
_Avoid_: calling Montaget an agent, or "the AI"

**Agent**:
An LLM client outside Montaget — Claude Code, or any other MCP client — that
authors and edits projects and calls Montaget's tools. Montaget never calls an
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
One `{"t","v","ease"}` record in a list that makes a transform property change over time.
Its `t` is written in timeline milliseconds but is **not a timeline time** — it is the
element's own animation geometry, so `shift` moves elements and their keyframes are
carried with them. A keyframe outside its element's range is legal and ordinary: it is how
a trimmed move is spelled.
_Avoid_: key, waypoint, stop, tween

**Easing**:
The shape of the interpolation **arriving at** a keyframe from the previous one — a name
published in the schema as its cubic bezier, or the four control points directly. The
first record of a list has nothing arriving at it and carrying an `ease` there is an error.
_Avoid_: timing function, curve, interpolation (as the field name), tween

**Clip**:
The static frame-space rectangle an element is drawn through. It does not rotate and does
not scale with the element, so `scale` moves the picture behind a window that stays put —
which is what a Ken Burns is. Shaped and soft masks are a different thing and are not this.
`clip` is also the aperture a `fit` rule works against (see Fit): under `cover` the derived
rect fills it exactly, but under `contain` the derived rect can be smaller, leaving slack.
Where that slack goes is not a separate placement field — it is ordinary `x`/`y`/`origin`
work against `clip`'s own corner and centre, the same as any other visual element's rect.
([#52](https://github.com/MBehtemam/Montaget/issues/52))
_Avoid_: crop, mask, viewport, bounds

**Run**:
One stretch of a text element's content, carrying its own text plus style deltas
over the element's base style. A text element's content is always an ordered array
of runs, even when there is only one. A run boundary is *style only* — it never
implies a line break; a line break is a `\n` character inside a run's text.
_Avoid_: span, segment, chunk

**Font**:
An ordered chain of font *files* the project declares under a semantic name, which
elements reference by that name. Always files, never a system family: a family name
is an entry in a table you cannot read, cannot commit, and that differs per machine.
The renderer opens nothing outside the chain.
_Avoid_: typeface, family, font stack

**Line height**:
A text element's line-height multiplier, restricted to one decimal digit — `1.0`, `1.1`,
`1.2`, … — always exactly representable as `n/10`. A line's height is the largest `size`
among its runs × `line_height`; the derived block height is `ceil` of that product over the
line count, evaluated in exact integer arithmetic, never IEEE double — the same ULP hazard
`fit`'s box sizing has (see Fit), on a field `fit`'s ADR never touched. Defaults to `1.2`
when omitted.
([ADR-0028](docs/adr/0028-text-block-arithmetic-is-exact-tenths.md))
_Avoid_: leading, line spacing (implies an additive gap, not a multiplier)

**Shape**:
A drawn primitive with no source file — `rect` or `ellipse`, each its own element type,
never a `shape` field inside a shared one. An ellipse inscribes its declared rect, so
neither needs a placement rule beyond the transform every visual element already carries.
A `rect` may declare a `radius`; an ellipse may not.
_Avoid_: rectangle (as the type name), box, figure, primitive (unqualified)

**Fill**:
The paint inside a shape's outline, as a single flat colour. Optional when a `stroke` is
present, giving an outlined shape; a shape with neither is a schema error naming both,
because an element that deliberately renders nothing and an element that forgot its paint
must not look alike. A gradient is not a fill — a flat colour is a value, a gradient is a
paint description, and descriptions belong in a closed vocabulary rather than an open
syntax.
_Avoid_: background, colour (for a shape), paint. Do not use for what a
time-based element does past the end of its source — that is `overrun`.

**Stroke**:
A second paint on the same outline — `stroke` and `stroke_width` — sitting on the
primitive rather than among the effects, and addressable per *run* on text. It never
enlarges the declared rect, and that is what separates it from an effect: a blur is not a
paint on the outline and a drop shadow is not addressable by run. On a shape it falls
**inside** the declared rect, so a stroked `card-05` still occupies exactly 984×169. On
text it falls **outside the glyph contour** — inside would thin the stems — and grows into
the box rather than past it, because a text element's `width`/`height` is a container
claim and not drawn geometry. It is in element space, so it scales with `scale`.
_Avoid_: outline, border, bord

**Effect**:
A member of a closed, named, parameterised vocabulary in `effects: [...]` on an
element — never an open plugin architecture. A list, not a map or a single field,
because application order is semantically real: blur-then-shadow is a different
frame from shadow-then-drop. v1's vocabulary is `blur`, `shadow` and `mask`
(shape-only). Effects attach to whole elements, never to a run — that boundary is
what excludes `stroke` (a run-addressable paint field) from this vocabulary, and
what excludes per-word text highlighting, which needs run addressing and a
timing model keyframes don't provide (keyframes animate transform properties
only). Static in v1: no effect parameter is keyframable.
([ADR-0040](docs/adr/0040-effect-model-attachment-and-v1-vocabulary.md))
_Avoid_: filter (for the whole concept — see Colour filter, deferred, below), plugin, stack

**Mask**:
An `effects` vocabulary member: a closed shape (`circle`, `rect`, `ellipse`) with
numeric parameters only, that clips an element's rendered pixels. Shape-only in
v1 — a soft or alpha mask sourced from an image is deferred, since it introduces
a second asset reference and unresolved fitting/colour-space questions.
([ADR-0040](docs/adr/0040-effect-model-attachment-and-v1-vocabulary.md))
_Avoid_: clip (that name is the transform model's static frame-space aperture,
[ADR-0012](docs/adr/0012-flat-transform-keyframes-carried-by-their-element.md) — a
different concept that happens to sound alike)

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
permitted and resolved as-is. Relocating a project to a remote store means
rewriting `source` to URLs, not editing a shared base
([ADR-0053](docs/adr/0053-asset-path-resolution-no-assetroot.md)).
_Avoid_: asset, resource, media reference, assetRoot

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
says otherwise.
_Avoid_: trim (as a noun), in/out point

**Speed**:
A time-based element's playback-rate multiplier — `0.645` plays its source at
0.645× normal rate, slower. Strictly greater than zero; `0` and negative
values are schema errors. Changes what the source range's two ranges are
allowed to disagree by: `end - start` must equal `source range / speed`,
rounded to the nearest millisecond. Reverse playback is not `speed`'s job and
is undecided, not ruled out.
([ADR-0020](docs/adr/0020-speed-overrun-hold-loop.md))
_Avoid_: rate, stretch factor (reads as the reciprocal and gets the direction
backwards), tempo

**Overrun**:
What a time-based element does once its (possibly speed-adjusted) source runs
out before its timeline range does: `"hold"` freezes the source's last frame
(video only — a schema error on audio, where the correct spelling of "then
silence" is a shorter element and a gap) or `"loop"` restarts the source from
its beginning with a hard cut, no crossfade. Composes with `speed` rather than
excluding it. Present only when needed — there is no `"none"` value.
([ADR-0020](docs/adr/0020-speed-overrun-hold-loop.md))
_Avoid_: fill (spent — see Fill), extend, pad

**Volume**:
An `audio` or `video` element's playback level, as a linear multiplier:
`0` is silent, `1` (the default) is the source's own level, and values
above `1` amplify. Flat on the element like `speed`, and keyframable with
the same `{t,v,ease}` records every animatable property carries, so a fade
is two records rather than a dedicated field. There is no `mute` — a
`video` element's embedded audio is the same audio a `volume` of `0`
already silences. Automatic ducking (one element's level reacting to
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
Montaget performs instead of the agent. It refuses an edit that would change an
existing slack's size, the same way it refuses to stretch a time-based
straddler, rather than silently absorbing the difference.
([ADR-0032](docs/adr/0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md))
_Avoid_: ripple (as the primary term), slide, nudge

## Findings and reports

The vocabulary above is the document's. This is the tooling's: what Montaget has
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

**Class**:
Which of five kinds a finding is. Three are severities, named for what the
reader does rather than for how bad it is: `error` (the render is refused or is
guaranteed wrong), `review` (legal, renders, and you must look at a frame to
know if it was meant), `note` (a fact you may want and will not act on today).
Two are not severities at all: `UNCHECKED` (the question was unanswerable — an
unprobeable source; `validate`-only, because `render` must decode the source
anyway) and `LAYOUT` (canonical key order; `validate`-only, and never a reason
to refuse a render). A class is computed from the consequence at an instant, not
fixed per check: the same gap is `review` when nothing else covers it and a
`note` when something does.
([ADR-0006](docs/adr/0006-validate-reports-facts-and-render-enforces.md),
[ADR-0013](docs/adr/0013-fitted-extents-floor-and-the-nine-origin-keywords.md),
[ADR-0041](docs/adr/0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md))
_Avoid_: severity (only three of the five are), level (the three-way ladder
only), category, priority

**Repair**:
The field every `error`-class finding carries, in one of exactly two forms.
*Advise-class* states a value, when the correct fix is fully determined by the
document, the media on disk and the published rendering semantics.
*Refuse-class* states `"none"`, when the fix depends on knowing what the author
meant — and that refusal is a guarantee no flag, force mode or write tool may
lift. Which of the two a check emits is decided once, when the check is written,
and holds for every instance it matches, including the ones that look safe. It
is orthogonal to class, not a fourth severity. Whether the binary reaches
findings that are not about a document at all — a file that would not parse, an
invocation that was wrong — is open
([#224](https://github.com/MBehtemam/Montaget/issues/224)).
([ADR-0043](docs/adr/0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md))
_Avoid_: fix, suggestion, autofix, quick fix — each implies something will
apply it

**Census**:
A grouping of the siblings a finding affects by an observable, document-derived
fact — *"four of five are at y = 1597, one is at 1537"* — which never ranks the
groups or says which is correct. It is what a refuse-class finding carries
instead of a repair: narrowing where to look is admissible where stating a fix
is not.
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
One verb's whole answer: its findings, a count per class, an exit code, and the
`NOT CHECKED` block that states the report's own boundary. JSON is canonical and
the prose form is generated from it — `--json` prints the JSON *instead of* the
text, never alongside it. Errors and near-errors print in full while the
informational classes collapse to one counted line, because `0 errors, 47 notes`
reads as a pass and a noisy report manufactures false confidence faster than an
unrun one does.
([ADR-0006](docs/adr/0006-validate-reports-facts-and-render-enforces.md),
[ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md))
_Avoid_: output, results, log

**NOT CHECKED**:
The block every report ends with, unconditionally, clean runs included: this
file was not compared against any prior version or instruction, and Montaget
cannot tell you whether it says what you meant it to say. It is there because
without it a clean run is read as *"the file is right"*, which is the rejected
`sequence` label wearing a `validate` label instead.
([ADR-0006](docs/adr/0006-validate-reports-facts-and-render-enforces.md))
_Avoid_: caveat, disclaimer, limitations

**Verb**:
One operation on the surface — `validate`, `query`, `frame`, `measure`,
`compare`, `render`, `shift`, `create_project`, `probe`, `fmt`, `timeline`. The
surface's job is to make reading, checking, comparing and rendering cheap, not
to provide editing verbs: the agent edits the file with the tools it is already
strongest with. Eight are MCP tools and eleven are CLI commands, and the
asymmetry is deliberate — an MCP schema costs context on every turn, a CLI
subcommand costs nothing until invoked. Every write verb returns the new state's
findings, never an `ok`. (ADR-0011's prose opens with *"nine verbs and two
resources"* while its own table lists eleven and its own count says *"eight MCP
tools, eleven CLI commands"*. The eleven above are the table's; the word "nine"
is unreconciled and should not be quoted as a count.)
([ADR-0011](docs/adr/0011-tool-surface-reads-checks-renders.md))
_Avoid_: command (the CLI spelling only), tool (the MCP spelling only),
endpoint, action

**Registry**:
The single declaration of every code Montaget can emit, and what is true of it:
which classes it may carry, whether it is refuse- or advise-class, whether its
threshold is internal or external, the ADR that owns it, and its template. It
exists so those facts are looked up rather than restated at each call site — the
same reason canonical key order is tied to the published schema rather than to a
parallel hand-maintained list.
([ADR-0041](docs/adr/0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md))
_Avoid_: catalogue, table, rule set

## Rejected terms

These words are deliberately absent. Each is standard vocabulary in a comparable
tool, which is exactly why using it here would mislead.

**Scene**:
Elsewhere a scene owns its own clock, so its children's times are relative to it.
A `track` is the one container Montaget has, and the distinction is exactly this:
a scene owns a clock, a track owns a stacking position. A track has no start, no
duration and no origin, so there is nothing for a child's time to be relative to.
Nesting still *invites* the assumption — JSON2Video documents the silent failure
that follows — which is why the times stay absolute and why the schema says so
loudly rather than relying on this paragraph.

**Clip**:
Elsewhere a clip pairs a visual with its audio, and its duration follows that
audio. In Montaget audio is an ordinary element with its own independent time
range, so no such pairing exists.

**Asset**:
Elsewhere an asset is an imported file declared once and referenced by id.
Montaget writes the file's location on the element that uses it. Note also that
an asset is *not* a reusable configured object in the sense of a Unity prefab —
that is templating, and it is out of scope for v1. The one carve-out is **Font**,
for reasons recorded in [ADR-0002](docs/adr/0002-inline-source-no-asset-table.md);
media sources never get a table.

**Anchor, for text positioning**:
Every comparable tool — ASS's `\an`, CSS, every GUI editor — calls the nine-way
positioning point an *anchor*. Montaget spends that word on layer-relative stacking
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

**Whether its error is refuse- or advise-class is open.** Both repairs need a fact the
document does not carry — the pivot the 4-array left implicit, and the dimensions of an
element fifteen of twenty-two captions never had — which is `gravity`'s fork in `box`'s
spelling. [ADR-0068](docs/adr/0068-the-bare-mask-key-retires-masks-are-effects-members.md)
remarks in passing that *"`box` was migrated by arithmetic script"* and
[ADR-0043](docs/adr/0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md)
is explicit that it *"classifies `gravity` and nothing else"*, so no ADR has ruled. Until
one does, the check refuses, under ADR-0043's own standing rule that a check refuses where
any instance it can match could be load-bearing. The placement is provisional; the ruling
belongs to an ADR ([#228](https://github.com/MBehtemam/Montaget/issues/228)).

**Align, for images**:
`align` means one thing: how a text element's lines align to each other
(`start`/`center`/`end`). On an image the same word once meant *which part of the source
survives the crop* — that job is now `x`/`y`/`origin` against the aperture's own `clip` (see
Clip), the same fields that place any other visual element's rect, including the slack a
`contain` fit leaves inside its aperture. `align` on a non-text element is a schema error
naming `x`/`y`/`origin` and `clip` as the replacement. The collision is recorded because it
caused a live misreading: the fixture's `align:"left"` on text was transcribing ASS `\an4`,
**left-middle**, so its `y` was a centre and nothing in the file said so.
([#52](https://github.com/MBehtemam/Montaget/issues/52))

**Its repair class is open for the same reason `box`'s is.** On an image the word meant
exactly what `gravity` meant, so repairing it needs the same absent fact, and no ADR has
ruled. The check refuses provisionally; see the note under Box
([#228](https://github.com/MBehtemam/Montaget/issues/228)).

**Path, line, polygon**:
Every visual element is placed by `x`, `y`, `origin`, `width`, `height`. A point list is
not — it has **no declared extent**, and giving it one needs either a second unit system
or an extent derived from its own content, both of which are already closed. So these are
rejected rather than merely absent, and the reopening condition is stated so it can be
met: admitting a point-list shape is a new ADR about *placement*, not a schema addition.
Do not reach for "a thin rotated `rect` draws a line" as the reason — it is false in the
way that matters, because the endpoints never appear in the file. A `path` is additionally
a mini-language inside a JSON string: unreadable by reading, unmatchable by exact-string
replace. Commit an SVG or a PNG instead.

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
([#76](https://github.com/MBehtemam/Montaget/issues/76),
[`docs/research/juries/format-versioning/experiment-gravity-fork/`](https://github.com/MBehtemam/Montaget/tree/main/docs/research/juries/format-versioning/experiment-gravity-fork))

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
