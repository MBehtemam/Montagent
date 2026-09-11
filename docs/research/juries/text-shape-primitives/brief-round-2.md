# Brief: three unresolved questions about a video format's text and shape primitives

You are an **agent that authors and edits video projects** in a declarative JSON format
called Montaget — its consumer, not its reviewer. The repo is
`/Users/mohammedehtemam/projects/github/Montaget`. You edit the project file with
ordinary file tools (exact-string replace) and check your work with tools called
`validate`, `frame`, `measure` and `query`.

A previous group of agents answered six design questions. Three were settled and three
came back **split**. Your job is the three splits. For each, both positions are stated
below as they were argued. They are unattributed on purpose. **Pick the position you
find more attractive, then genuinely try to break it, and only then give your verdict.**
A verdict that survives your own attack is worth more than a confident first impression.

## Read these first (settled; you may not reopen them)

- `CONTEXT.md` — the vocabulary. Note `Origin`, `Transform`, `Clip`, `Run`, `Font`, and
  the **Rejected terms** section (`Box`, `Align, for images`, `Weight/bold`).
- `docs/adr/0007-text-runs-literal-size-declared-fonts.md` — text is styled runs at a
  literal size; no fit-to-box, no automatic wrapping. Note the open item it records:
  "the full run style-delta set (colour, outline — with the ASS BGR-with-alpha-nibble
  trap and ScaledBorderAndShadow written down)".
- `docs/adr/0008-line-breaks-belong-to-the-agent.md`
- `docs/adr/0012-flat-transform-keyframes-carried-by-their-element.md` — flat `x`, `y`,
  `origin`, `width`, `height`, `scale`, `rotation`, `opacity`, `clip`; keyframes are
  `{"t","v","ease"}`. It bans a size defaulted from the source file.
- `docs/adr/0013-fitted-extents-floor-and-the-nine-origin-keywords.md` — note its
  principle that a correct file should produce no findings, and its refusal to create a
  schema value by implication.
- `docs/adr/0006-validate-reports-facts-and-render-enforces.md` — note its noise budget:
  a validator that emits many findings on a correct file manufactures false confidence.
- `docs/adr/0003-general-video-editor-not-channel-tooling.md` — reference class is
  **CapCut/Premiere; After Effects is out of scope**.
- `fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json` — the only
  real project file that exists (22 `text`, 10 `rect`, 8 `image`, 20 `audio`), and
  `fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4`, the video it
  describes.

**Standing rule you must obey:** the fixture is test data and a regression guard, never
a scope boundary. It is evidence a capability is **needed**, never evidence one is
**unneeded**. "The fixture doesn't use it" is not an argument against a feature. The
last jury was given this paragraph and one juror still argued "all 10 shapes in the
fixture are rectangles, so ship rect only". Do not be that juror. Equally, do not
design for this one channel.

## Already settled — do not reargue

- Shapes are **sibling element types** (`type:"rect"`, `type:"ellipse"`), not a
  `type:"shape"` with a discriminator. Unanimous.
- **`rect` and `ellipse` are in v1. `path` is out.**
- **`gravity` is decided with the `fit` vocabulary in another ticket**, not here. This
  ticket owes only the rule that `gravity` on a text or shape element is a schema error.

## Facts established since (verify any of them)

1. The header panels and the navy sentence card in the published MP4 have **exactly
   square corners** at exactly (48,88) and (48,1453). The fixture README calling them
   "rounded" is wrong.
2. `brand/logo-en.png` is 800x800 RGBA with **corner alpha 0** — the circular badge is
   baked into the asset, so `mask:"circle"` on `handle-logo` changes nothing.
3. The old pipeline authored these visuals in ASS, whose drawing language **includes
   cubic beziers**. Across all seven files in
   `fixtures/en-halloween-decorating/reference/subtitles/` there are six distinct
   drawings and **every one is four-point and axis-aligned** (`m`/`l` only, not one `b`).
   A real authoring system with an unrestricted path language shipped only rectangles.
4. For **15 of 22** text elements the declared `height` is exactly
   `ceil(size * line_height * line_count)`. For the other 7 it is copied from a
   container: 5 from the navy card (984x169), 2 from a header panel (height 84).
   The declared **`width` is externally sourced in all 22** (984 = the card's width;
   238 and 472 = a panel's right edge minus the text's `x`).
5. All 10 `rect` elements carry exactly `fill`, a flat `#RRGGBB`, and no other paint field.
6. All seven fixture ASS files carry `ScaledBorderAndShadow: yes`.

## Three findings from the last round that no vote captured

- **Stroke geometry.** If `stroke_width: 8` is added to `card-05` and the stroke centres
  on the path, the drawn card becomes 992x177 at (44,1449) — eating 4px of the 48px
  margin the layout rests on — while every number in the file still reads 48/984/169 and
  `validate` passes. What is drawn is a number not in the file.
- **The container-copied heights are not bindings.** The 7 elements that copy a height
  from a card or panel have no relationship to it in the document. Resize the card and
  `validate` stays green on a now-stale caption height.
- **The two overflow axes are not alike.** Height overflow is arithmetic on numbers
  already in the document. Width overflow requires `measure` against the current font
  binary, and all 22 widths are externally sourced. One is free and self-contained; the
  other goes stale silently whenever a font or a string changes.

## Q7 — The paint package: stroke, and alpha in colour

These two resolve together: alpha in a colour is largely motivated by wanting two paints
(a fill and a stroke) with different transparency on one element, and ADR-0012 already
gives every element a single keyframable `opacity`.

**Position A.** Shapes carry `stroke` and `stroke_width`; text carries a stroke too, and
on text it is available as a **run**-level style delta, not only element-level. Colour is
spelled `#RRGGBB` **or** `#RRGGBBAA`. The argument: a paint property is consumed while
the element rasterizes in its own geometry, so it is part of the primitive; and the
effect vocabulary being designed elsewhere attaches effects to **elements**, while a run
lives *inside* an element — so if text stroke is an effect, "outline one word" is
inexpressible. Element `opacity` is the animation channel and cannot express a
translucent fill under an opaque stroke, nor per-run alpha.

**Position B.** No stroke in the primitive at all — it belongs to the closed effect
vocabulary being designed in another ticket. Colour is `#RRGGBB` only; all transparency
goes through `opacity`. The argument: a stroke interacts with `scale` and with animation
in a way the settled `clip` field explicitly does not, which is effect territory; an
alpha nibble inside a colour string is the ASS BGR-with-alpha-nibble trap that ADR-0007
names; and shipping a paint field the renderer must grow rules for is a field that reads
declarative and behaves surprisingly.

**If you land on Position A, also answer:** does a stroke fall **inside** the declared
rect, **outside** it, or **centred** on its edge — and is the rule the same for shapes
and for text? Justify it against finding 1 above.

## Q8 — Does a text element still declare a `height`?

**Option (a): `width` and `height` both stay required.** The argument: the self-derived
height is *frozen at authoring time*. It is vacuous only at the instant it is written —
add a `\n` in a later edit without updating it and the overflow check fires correctly.
A frozen number is a regression guard. It also keeps "an element's size is declared,
never defaulted" true uniformly across every element type.

**Option (b): `width` required, `height` optional**, its absence meaning the element
claims no vertical container and its extent is simply the typography. The argument: a
check input the author computes from the very thing being checked checks nothing;
`width` is externally sourced 22/22 while `height` is only 7/22, *because most captions
have nothing behind them to overflow*; and the apparent collision with
"declared, never defaulted" is false, because ADR-0012 bans defaults whose **inputs are
not in the document**, whereas a derived height's inputs (`size`, `line_height`, the
`\n` count) are all on the same line.

**Option (c): both required, and `validate` reports a finding** when `height` equals the
derived value. The argument against it, which you should weigh: it fires 15 findings on
the only correct project file that exists.

Consider specifically: which *edits* does each option catch, and which does it miss?
Lengthening an existing line and adding a new line are different edits.

## Q9 — Are `line` and `polygon` rejected, or merely unevidenced?

**Position A — rejected, on placement grammar.** Every visual element in this format is
placed by `x`, `y`, `origin`, `width`, `height`. A line or polygon is a *point list*,
which needs a second placement grammar that no transform, origin or clip rule covers. A
thin rotated `rect` already draws any segment. Recording the rejection *with this reason*
says something falsifiable: the day a genuine point-list shape is needed, that is a new
ADR about placement, not a schema addition.

**Position B — unevidenced, not rejected.** They are ordinary primitives in the
CapCut/Premiere reference class and nothing so far excludes them. Recording them as
rejected overstates what has actually been established.

## How to answer

Write to the output path you are given, as Markdown. For each question: the position you
first found attractive, **the strongest attack you could mount on it**, whether it
survived, and your verdict with confidence. Then:

- **What would bite me first** under the design you just endorsed.
- **Any question you think is malformed or presupposes something false.**

Do not read GitHub issues 13 or 2, and do not read any other agent's answer file.
