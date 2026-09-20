# The Montaget project format: the rules the schema cannot express

The JSON Schema — served beside this document as `montaget://schema.json` — is the
authority on **shape**: which keys exist on which element type, which are required, and
which values are admitted. It is generated from the same Rust types that parse your file,
so it cannot fall out of step with what `validate` enforces.

This document is the other half. Everything below is a rule a schema has no way to say:
relationships between fields, between elements, and between the file and the bytes on disk.
Breaking one of these produces a legal-looking document that is the wrong video.

Read the schema for *what may be written*. Read this for *what it will mean*.

**Where these rules come from.** Every one of them is an accepted decision in Montaget's ADR
series, cited inline below by number. The ADRs are the specification and this document is
their agent-facing reading, not a second authority: where the two ever disagree, the ADR is
right. A test in `montaget-core` re-reads every ADR cited here and fails if one of them has
stopped being accepted, so a rule that is quietly superseded cannot go on being taught here.

## How you edit a project

**With your ordinary file tools** (ADR-0011). There is no CRUD API and there are no field-level edit
tools, deliberately: a tool that took an element id and a field name would cost you the
`diff` that proves your edit changed that element **and nothing else**. Montaget's tools
exist for the things a text editor cannot do — checking, measuring, querying, rendering.

That model has one prerequisite, and it is the reason for the writing convention below:
**your next exact-string replace must match exactly one element.**

### The canonical writing convention

A project file is written so that each element is a unique, matchable substring
(ADR-0005, ADR-0041, ADR-0007).

- Two-space indent. The header's keys one per line. `tracks` expanded, each track expanded.
- **Every element on exactly one line, whatever its length** — runs, keyframes, effects and
  all. There is no width threshold, deliberately: a formatter that reflowed long elements
  would be non-idempotent, and adding three characters would reflow an element into eight
  lines with a diff that has nothing to do with your edit.
- An element's own JSON is written *tight* — `{"id":"card-05","type":"rect",…}`, no spaces
  after `:` or `,`. The header and the track keys are written *spaced*. The difference is
  what makes an element line matchable; the same fields with spaces would be equally legal
  JSON and every replace you had written would find nothing.
- **Key order is the schema's property order** (ADR-0041). Not alphabetical, not the order you typed.
  Every type begins with the universal prefix `id, type, group, start, end, layer`, then
  its own fields, then `effects` last.
- One trailing newline, LF line endings, non-ASCII written raw rather than escaped.

`montaget fmt` restores all of this; `montaget fmt --check` reports what it would change
without touching the file. `validate` reports a departure from the convention under the
`LAYOUT` category on every run — which is **not** a severity and never blocks a render. A
file written outside the convention is unsafe to *edit*, not unsafe to *render*.

Two things `fmt` will never do to your file, both of which are content rather than
formatting: it never adds or removes a field that has a default (ADR-0030), and it never
rewrites a value — including choosing between `cover` and `contain` (ADR-0026) where an aspect ratio makes both
exactly true.

### Presence is content (ADR-0030)

Omitting a defaultable field and writing it explicitly at its default value are **two
different declarations**. Omission says *"give me whatever the default is"*; an explicit
`"opacity": 1` says *"I have pinned this to 1"*. They diverge the moment the default is
revisited. Nothing in Montaget will convert one into the other, `create_project` included:
the scaffold writes `background`, `duration` and `output` exactly when you asked for them.

## Time

- **One clock** (ADR-0005). Every time in the file is absolute integer milliseconds on the project's
  single timeline. There are no nested timelines, no per-track clocks and no relative times.
- **Ranges are half-open: `[start, end)`** (ADR-0005). An element whose `end` is 7500 is not on screen
  at 7500, and a neighbour starting at 7500 is. So a cut where one range ends and the next
  begins names a single instant — not an overlap, and not a one-millisecond gap.
- **There is no `duration` field on an element.** Its length is `end - start`, and a tool
  that wants the length computes it.
- **A keyframe's `t` is not a timeline time** (ADR-0012). It is the element's own animation geometry
  and travels with the element, which is why `shift` moves elements and carries their
  keyframes along. A keyframe outside its element's range is legal and ordinary: it is how
  a trimmed move is spelled.
- **Slack is invariant** (ADR-0032). The distance from one boundary to the next is content, not
  padding a tool may absorb. `shift` refuses an edit that would change an existing slack's
  size rather than quietly taking up the difference.

## Tracks, elements and stacking

- **A track supplies stacking, never timing** (ADR-0004). It has no start, no duration and no clock.
- **Array order carries no meaning** (ADR-0060) — not for timing and not for stacking. Reordering the
  elements in a track changes nothing about the video.
- **Children of one track may not overlap in time** (ADR-0004). That is a validation error, and it is
  what tracks are for. Elements that should overlap belong in different tracks.
- **A gap in a track is legal and ordinary** (ADR-0006) — the silence between two narration lines is a
  gap. A gap is never an error, which is why it is reported apart from an overlap.
- **`layer` is an integer; higher draws in front.** It normally lives on the track. An
  element may override it with its own integer or with an anchor.
- **A layer tie is an error once the boxes actually overlap** (ADR-0060) in both time and space,
  because at that point the document does not say which draws in front. Two elements
  resolving to the same layer whose boxes never meet is fine.
- **An anchor is one hop, never a chain** (ADR-0019). `{"below": "title"}` resolves to that element's
  layer minus one. Its target must carry a plain integer `layer`; an anchor pointing at
  another anchor, at itself, or at a missing id is an error. A track's `name` and an
  element's `id` are separate namespaces — an anchor resolves only against an element `id`.
- **Every element carries a required, unique `id`,** whatever its type. Its only job is to
  be a target.
- **`group` is render-inert.** It says two elements belong to one authorial unit and
  nothing more — no timing, no stacking, no drawing consequence.

## Geometry

- **An element's size is declared, never inferred from its source file.** `width` and
  `height` are what the element occupies; Montaget will not read a PNG and fill them in.
- **`origin` is the point of the element's own box that `x`,`y` places** (ADR-0013), and the point
  transforms pivot about. Nine keywords, vertical first: `top-left`, `top-center`,
  `top-right`, `center-left`, `center`, `center-right`, `bottom-left`, `bottom-center`,
  `bottom-right`. The middle elides to `center` alone — `center-center` is an error naming
  it, because two spellings of one value break the write-read round trip.
- **There is exactly one transform per element, and it is flat** (ADR-0012). `x`, `y`, `origin`,
  `scale`, `rotation`, `opacity`, as fields on the element. No transform is nested,
  inherited or composed, and no element's transform is relative to another's.
- **`scale` is always `[sx, sy]`** (ADR-0012), never a bare number.
- **`clip` is a static frame-space rectangle** and never animates (ADR-0025): it is simultaneously the
  aperture the source is drawn through and the fixed denominator a `fit` claim is checked
  against. It does not rotate and does not scale with the element — which is exactly what
  makes a Ken Burns move work: `scale` moves the picture behind a window that stays put.
- **`fit` is a claim about how you computed `width`/`height`, not a layout mode** (ADR-0015). Montaget
  checks your arithmetic against the source's real dimensions; it does not do the fitting
  for you. Where an aspect ratio makes `cover` and `contain` exactly equivalent, both
  spellings stand and nothing will normalise one to the other. **No renderer reads it**: the
  declared rect is authoritative, so a source is resampled to exactly `width`×`height` and
  `frame` draws the same picture whether `fit` says `cover`, says `contain`, or is absent.
- **`radius` is a field on `rect` and not on `ellipse`** (ADR-0014). A single integer,
  defaulting to 0 — one corner radius, not four. An ellipse inscribes its declared rect and
  has no corners to round, so `radius` on one is an unknown key rather than a field that
  quietly does nothing.
- **`stroke` never enlarges the declared rect** (ADR-0014). On a shape it falls inside it; on text it
  falls outside the glyph contour and grows into the box rather than past it.
- **Effects are an ordered list, and the order is semantically real** (ADR-0040). Blur-then-shadow is a
  different frame from shadow-then-blur. They attach to whole elements, never to a run, and
  no effect parameter is keyframable in v1.

## Text

- **Content is always an ordered array of runs,** even when there is only one.
- **A run boundary is style only** (ADR-0008). It never implies a line break. A line break is a `\n`
  character inside a run's text — the agent chooses every break, and the renderer chooses
  none.
- **Sizes are literal pixels** (ADR-0007), and a text element's `width`/`height` is a container claim
  rather than drawn geometry. Use `measure` to find out what a string actually occupies in
  the font the project declares; do not estimate it.
- **`line_height` is a multiplier restricted to one decimal digit** (ADR-0028) — `1.0`, `1.1`, `1.2` —
  so it is always exactly `n/10`. A line's height is the largest `size` among its runs times
  `line_height`; the block height is the `ceil` of that over the line count, computed in
  exact integer arithmetic. Defaults to `1.2`.
- **Fonts are declared as an ordered chain of font *files*, never a system family name**
  (ADR-0007, ADR-0057).
  The renderer opens nothing outside that chain, which is what makes a project that renders
  on your machine render on a clean one.
- **Every font file is vendored through `montaget fonts vendor`, and the `fontVendor` table
  is its receipt** (ADR-0057). The table is keyed by file path — one path, one licence, one
  `sha256` — however many `fonts` chains reference the file, and it is written by the tool,
  never by hand. `validate` checks that every chain path has an entry whose hash matches the
  bytes on disk (`error` when it does not) and that every entry is still referenced (a
  `note` when not; nothing prunes it for you). It re-checks integrity, never licence law.
  `fonts vendor` runs its licence check *before* any bytes are copied: a font on the
  blocklist of known non-redistributable fonts is refused with no override, a recognised
  open licence is recorded, and anything else needs a `--licence` you have verified. Use
  `montaget fonts list` to see each installed font's status before you try.

## Values

- **Colours are `#RRGGBB` or `#RRGGBBAA`, uppercase, and nothing else** (ADR-0014). No three-digit
  shorthand, no CSS names. `#RRGGBBFF` is an error naming the six-digit form, because a
  fully-opaque eight-digit colour is a second spelling of a value the six-digit form already
  says. Alpha lives here as well as in `opacity` because `opacity` is the *animation*
  channel.
- **`speed` is strictly greater than zero** (ADR-0020, ADR-0045), and it changes what the two ranges may
  disagree by: `end - start` must equal the source span divided by `speed`, rounded to the
  nearest millisecond, evaluated exactly rather than in floating point.
- **`overrun` has no `"none"` value** (ADR-0020) — it is present when needed and absent otherwise.
  `"hold"` is video-only; the correct spelling of "then silence" on audio is a shorter
  element and a gap.
- **`ease` is required on every keyframe record except the first, and forbidden on the
  first** (ADR-0038). Easing describes the interpolation *arriving at* a keyframe, and nothing arrives
  at the first one. Presence is a pure function of position.

## Sources

- **A `source` is written on the element itself** (ADR-0002) — a path or a URL. There is no table of
  assets declared elsewhere and referred to by name, and no top-level field that changes
  how a `source` resolves.
- **A relative path is always relative to the project file's own directory** (ADR-0053). An absolute
  path is permitted and resolved as-is. Moving a project to a remote store means rewriting
  each `source` to a URL, not editing a shared base.
- **The document records no source duration** (ADR-0011). What the file on disk actually is gets
  established by probing it, which is why `validate` always probes and why the cache-miss
  line is printed unprompted rather than treated as an optimisation detail.

## Versioning

**There is no format version field** (ADR-0016, ADR-0017). An older binary meeting a file authored against a
newer schema discovers it through the unknown-key error, which is why the schema is closed
at every object level — including inside `run` and `keyframe`. An unknown key is an error
naming the key, never a silently ignored field, and there is no escape hatch object for
arbitrary extra data.

Likewise, a retired spelling is an error that names its replacement rather than being
quietly accepted: `gravity`, `box`, `align` on a non-text element, `anchor` as a string,
`center-center`, `#RRGGBBFF`, `bold`, and `none`/`fill` as a `fit` value are all errors you
can repair from the message without reading an ADR.

## What the tools will and will not tell you

`validate` reports **facts** (ADR-0006) — findings with stable codes, each carrying the numbers it
rests on. It never reports a verdict that would require knowing what the video is for.
Every finding has a class: `error` (the render is refused or is guaranteed wrong), `review`
(legal, renders, and you must look at a frame to know whether it was meant), `note` (a fact
you may want), plus `UNCHECKED` (the question could not be answered) and `LAYOUT` (the
writing convention), neither of which is a severity.

Two consequences worth holding onto:

- **`render` runs the identical checks and refuses on any `error`** (ADR-0006). Validation is not an
  optional step you can skip; it is what the render does before it draws anything.
- **Every write tool returns the new state's findings, never `ok`** (ADR-0011). You do not run
  `validate` after a write — the write hands you `validate`'s answer.

And the boundary every report ends with, which is not modesty but the most important
sentence in the tool: **Montaget verifies that the file is internally legal. It cannot tell
you whether the file says what you meant it to say.**
