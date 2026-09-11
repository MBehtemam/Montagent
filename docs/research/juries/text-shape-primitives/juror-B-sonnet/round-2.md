# Q7 — The paint package: stroke, and alpha in colour

**Position first found attractive: A** (stroke + stroke_width on shapes and as a run-level
style delta on text; `#RRGGBB` or `#RRGGBBAA`).

The decisive signal is internal: ADR-0007 itself lists, as its own open item, "the full run
style-delta set (colour, outline — with the ASS BGR-with-alpha-nibble trap...)". The ADR that
invented `runs` already expected outline to be a *run*-level delta, filed next to colour, not
an element-level or effect-level thing. Combined with the fact that the effect vocabulary
(being designed elsewhere) attaches to *elements* while a `run` lives *inside* one, Position B
makes "outline one word" structurally inexpressible — not merely deferred, actually impossible
under the architecture two other tickets are independently committing to. That is a stronger
argument than anything Position B offers about alpha or animation interaction.

**Strongest attack I could mount on A:** the alpha-nibble and "interacts with scale/animation
like an effect" objections don't actually land — `#RRGGBBAA` is a different byte order from
ASS's BGR+nibble scheme, so the named trap is avoided by construction, not walked into; and a
stroke painted along the same path as the fill scales with the element exactly as the fill
does, unlike `clip`'s aperture, which was deliberately built *not* to scale (that's what makes
Ken Burns work). So the "stroke behaves like an effect" analogy is weaker than it reads. The
attack that actually bites is the stroke-geometry finding itself: a naive stroke silently makes
the *drawn* rect bigger or smaller than the declared one, which is precisely the "a number not
in the file" defect class ADR-0012/0013 spent three ADRs closing for fitted extents and `clip`.
Position A, if under-specified on geometry, reopens that defect the moment it ships.

**Survived?** Yes, but only by answering the sub-question decisively rather than leaving it
open.

**Verdict: Position A, confidence ~70%.**

## Stroke geometry sub-question

**Shapes: stroke draws inside the declared rect/ellipse.** The declared `width`/`height` stay
the exact drawn (and presumably collision/overlap) extent — inset each edge by `stroke_width`,
same idea for `ellipse`'s radius. This is forced by finding 1: the header panels and navy card
have *exactly* square corners at *exactly* (48,88) and (48,1453) in the published video, and
ADR-0013's "the declared rect is authoritative at render" principle (source resampled to the
declared rect, never the other way around) generalizes directly — a stroke that grows outward
or centres on the edge would make those numbers fiction the moment `stroke_width` is non-zero,
exactly the brief's own worked example (`card-05` becoming 992x177 at (44,1449) while the file
still reads 48/984/169). Inside is the only choice under which "the declared rect is what gets
drawn" survives stroke's addition unmodified.

**Text: the question doesn't transfer.** "Inside/outside/centred on the edge" presupposes a
filled path with an edge the stroke straddles — true of a rect or ellipse's boundary, not true
of a text element's declared box, which is a layout container, not itself a drawn shape. A
run-level text stroke is glyph ink (ASS `\bord`-style outlining around each character's own
path); it has no relationship to the box edge at all. So the rule is **not the same for shapes
and text** — for shapes it's a geometric inset rule; for text there is no analogous rule to
state, because the box was never the thing being painted. (This does leave a real gap: the
existing text-overflow check computes content height/width from glyph metrics and doesn't yet
account for stroke half-width bleeding past the box on wide or thick strokes. That's a natural
follow-up to ADR-0007's overflow check, not something this ticket needs to close.)

---

# Q8 — Does a text element still declare a `height`?

**Position first found attractive: (b)** — `width` required, `height` optional. The 22/22 vs
7/22 asymmetry is real and the "checks nothing" framing of a self-derived required field is
intuitively clean.

**Strongest attack I could mount on (b):** if `height` is absent, something in the pipeline
still has to resolve where a text element's own box's edges are — `origin` (a bottom-center or
center pivot needs a vertical extent to pivot *around*), `clip`, and any future
overlap/collision check all need it. Under (b) that number is no longer frozen in the document;
it must be computed by `measure` at render/query time from whatever font binary happens to be
installed. That is *exactly* the failure mode ADR-0007 fought hardest to kill for `size`
("a property whose value is HarfBuzz's opinion in the version you happen to have installed
fails 'understandable by reading'") and that ADR-0007 explicitly says gets solved by moving the
measurement to authoring time and freezing the result ("that loop does not disappear — it moves
to authoring time and freezes its result in the file, where a shaper upgrade cannot move it").
Option (b) reintroduces a renderer-computed, shaper-version-dependent number for exactly the
15/22 elements it makes optional — reopening the argument the format already won for `size`.

**Survived?** No — this attack changes my verdict. Option (a)'s defense ("vacuous only at the
instant it's written; a frozen number is a regression guard") is actually the *same move*
ADR-0007 already made for `size`, not an awkward workaround for it. Treating `height` the same
way — required, computed by `measure` once, frozen — keeps text's size and extent handled
identically, and keeps "an element's size is declared, never defaulted" uniform across every
element type without carving out a text-only, computed-on-demand exception.

Reject (c) outright and with high confidence: firing 15 findings on the only correct project
file that exists is a direct, undisputed violation of ADR-0006's noise budget ("a noisy
validator manufactures false confidence faster than an unrun one does"). No version of (c) that
matches equal-to-derived survives that ADR.

**Which edits each option catches/misses**, since the brief asks specifically:
- *Lengthening an existing line* (no new `\n`): height is unaffected by definition (the derived
  formula depends on line **count**, not line **length**) — none of (a)/(b)/(c) has anything to
  say about it via the height axis; it's purely a width/`measure` problem, orthogonal to Q8.
- *Adding a new line* (`\n`): under (a), the *existing* overflow check (declared height vs.
  content height computed from `size`/`line_height`/line count) fires correctly the moment the
  stored, frozen height no longer covers the new content — this is the case the option (a)
  argument is built on, and it is real. Under (b), for the 7/22 with a real container the same
  check fires the same way (height is present there too); for the 15/22 with no declared
  height, nothing fires — which is *correct* only if nothing downstream ever needed that extent,
  and per the argument above, something does (origin/clip resolution).

**Verdict: Option (a), confidence ~60%.** This is the closest of the three questions — (b)'s
honesty about which 7 elements have a real container is a genuine improvement I'd like to keep
in spirit (e.g., as a `validate` distinction between "container-derived" and "typography-only"
height, a census not a schema split), just not as an optional field.

---

# Q9 — Are `line` and `polygon` rejected, or merely unevidenced?

**Position first found attractive: A** (rejected, on placement grammar) — it's a principled,
falsifiable argument independent of channel content, so it doesn't fall into the "fixture
doesn't use it" trap the brief explicitly warns against, and it's consistent with `path`
already being decided out for the same underlying reason (a point-list needs a second
placement grammar nothing else in the format supplies).

**Strongest attack I could mount on A:** its own justification — "a thin rotated rect already
draws any segment" — only defends rejecting **`line`**. It says nothing about **`polygon`**. A
straight segment between two points is exactly what a thin rotated rect draws; an arbitrary
closed polygon (a triangle badge, a five-point star, a chevron, a speech-bubble tail) is not
expressible by any composition of axis-aligned-or-rotated rects and ellipses — there is no
existing primitive standing in for it the way there is for `line`. Position A's text quietly
bundles two different shapes under one argument that only actually covers one of them, and
flatly rejecting `polygon` on that basis overstates what's established — which is exactly
Position B's own complaint, just localized correctly.

**Survived?** No, not as a single bundled verdict. It survives cleanly for `line`, and fails for
`polygon`.

**Verdict, split: `line` — rejected** (Position A's argument, unweakened: it needs a second
placement grammar and is already fully covered by a thin rotated `rect`, so the day a point-list
segment is genuinely needed it is a placement-grammar ADR, not a schema addition).
**`polygon` — unevidenced, not rejected** (Position B): nothing here shows it's covered by
existing primitives, nothing shows it's unneeded (badges/arrows/callouts are ordinary in the
CapCut/Premiere reference class per ADR-0003), and recording it as rejected today would be the
"the fixture doesn't use it" fallacy wearing a placement-grammar costume. Confidence ~70% on the
split; ~55% on exactly where `polygon` lands if evidence ever arrives (it may still turn out
`rejected` once someone works out its placement grammar and finds it costs more than it's worth
— that's a future ticket's job, not a closure now).

---

# What would bite me first

Authoring a stroked element and assuming one geometric convention covers both shapes and text.
Under this design a stroke on a `rect`/`ellipse` draws **inside** the declared box (the box
stays authoritative), but a stroke on **text** is glyph ink with no box-edge relationship at
all — the same field name (`stroke`/`stroke_width`) means two different geometric things
depending on element type, and nothing in the schema enforces the distinction beyond the ADR
prose. I'd expect to author a thick outlined title inside a card, have it validate clean and
look fine at the sizes I tested, and then get bitten the day a longer word or a bigger
`stroke_width` makes the glyph outline visibly bleed past the card's edge — because the
existing text-overflow arithmetic (`size` × `line_height` × line count) doesn't yet have a
stroke term, and this ticket didn't add one. Second-order bite: under the Q8 verdict, `height`
on a text element now carries the same "must re-`measure` after every text edit" discipline
ADR-0007 already imposed on `size` — forgetting to refresh it after lengthening a caption with
a new line is silent until the (real, useful) overflow check catches it, which is the intended
behavior, but it means every text edit is now a two-field measure-and-freeze operation instead
of one.

# Malformed or false-presupposing questions

Q9 as framed presupposes `line` and `polygon` share one fate and one justification. They don't:
`line` has a costless substitute already in the schema (a thin rotated `rect`), `polygon` has
none. Any vote that picks one of the two stated positions wholesale is picking the wrong answer
for at least one of the two shapes. The stroke-geometry sub-question under Q7 has a milder
version of the same issue — "is the rule the same for shapes and text" presupposes the
inside/outside/centred vocabulary is even meaningful for text, when text's declared box was
never the drawn boundary in the first place, so "the same" and "not the same" are both slightly
wrong framings; the honest answer is that the question only fully applies to shapes.
