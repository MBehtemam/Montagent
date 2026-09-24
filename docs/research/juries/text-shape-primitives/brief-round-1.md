# Brief: the text and shape primitives of a video project format

You are an **agent that authors and edits video projects** in a declarative format
called Montagent — you are its consumer, not its reviewer. Montagent is a real repo at
`/Users/mohammedehtemam/projects/github/Montagent`. Files in (images, video, audio),
video out. The project file is a single JSON document in git and is the source of
truth; you would edit it with ordinary file tools (exact-string replace) and check
your work with tools called `validate`, `frame`, `measure` and `query`.

Six design questions are open. Answer them **as the agent who has to write and edit
these files**, not as a language designer. Where you can, ground an answer in an edit
you would actually perform and what would go wrong.

## Read these first (they are settled and you may not reopen them)

- `CONTEXT.md` — the domain vocabulary. Note `Origin`, `Transform`, `Clip`, `Run`,
  `Font`, and the **Rejected terms** section (`Box`, `Align, for images`, `Weight/bold`).
- `docs/adr/0007-text-runs-literal-size-declared-fonts.md` — text is styled runs at a
  literal size; no fit-to-box, no automatic wrapping.
- `docs/adr/0008-line-breaks-belong-to-the-agent.md`
- `docs/adr/0012-flat-transform-keyframes-carried-by-their-element.md` — every visual
  element carries flat `x`, `y`, `origin`, `width`, `height`, `scale`, `rotation`,
  `opacity`, `clip`, with keyframes as `{"t","v","ease"}`.
- `docs/adr/0013-fitted-extents-floor-and-the-nine-origin-keywords.md`
- `docs/adr/0003-general-video-editor-not-channel-tooling.md` — **the reference class is
  CapCut/Premiere. After Effects is out of scope.** Montagent is a general, open-source
  video editor.
- `fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json` — the only
  real project file that exists. 60 elements: 22 `text`, 10 `rect`, 8 `image`, 20 `audio`.
  `fixtures/en-halloween-decorating/README.md` describes what is on screen.
  `fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4` is the
  published video the file describes.

**A standing rule of this project, which you must obey:** the fixture is *test data and
a regression guard*, never a scope boundary. It is evidence that a capability is
**needed**, never evidence that one is **unneeded**. "The fixture does not use it" is
not an argument against a feature. Equally, do not design for this one channel.

## Measured facts you may rely on (verify any of them if you want)

1. The two header panels and the navy sentence card in the published MP4 have
   **exactly square corners**, at exactly (48,88) and (48,1453). The fixture README's
   phrase "a rounded cream panel" is wrong.
2. `brand/logo-en.png` is 800x800 RGBA with **corner alpha 0** — the circular badge is
   baked into the asset, so the `mask:"circle"` written on `handle-logo` changes nothing.
3. All 10 `rect` elements carry exactly `fill`, a flat `#RRGGBB`, and no other paint field.
4. For **15 of the 22** text elements, the declared `height` is exactly
   `ceil(size * line_height * line_count)` — derived from the element's own typography.
   For the other 7 it comes from a container: 5 from the navy card (984x169) and 2 from
   a header panel (height 84). The declared **`width` is externally sourced in all 22**
   (984 = the card's width; 238 and 472 = a panel's right edge minus the text's `x`).
5. `gravity` is inert on 8 of 8 image elements: the photos are 1536x2720 into 1080x1912
   (aspect 0.5647 vs 0.5648) and the logo is 800x800 into 68x68, so `cover` leaves
   essentially no slack to position.

## The six questions

**Q1 — Is a shape one element type or many?** `type:"rect"` and `type:"ellipse"` as
sibling element types, or one `type:"shape"` carrying a `shape:"rect"` discriminator?

**Q2 — Which shapes exist in v1?** Candidates include `rect`, `ellipse`, `line`,
`polygon`, `path`. Say which are in, which are out, and which are merely unevidenced
rather than rejected.

**Q3 — How is a colour spelled?** `#RRGGBB` only; `#RRGGBB` plus `#RRGGBBAA`; also
3-digit shorthand and/or CSS colour names? Note that ADR-0012 already gives every
element an `opacity`.

**Q4 — Does a text element still declare a `height`?** Given fact 4 above. Options:
(a) `width` and `height` both stay required, and the vertical overflow check is a no-op
on 15 of 22 elements; (b) `width` required, `height` optional, where absent means no
vertical container is claimed; (c) both required, and `validate` emits a finding when
`height` equals the derived value. Note that (b) collides with ADR-0012 and CONTEXT.md,
which say an element's size is **declared, never defaulted**.

**Q5 — Is `stroke` (an outline on a shape, and on text) a field on the primitive, or
does it belong to a separate closed vocabulary of *effects* that is being designed in a
different ticket?** If it is a field, say where the line falls between a primitive's
paint fields and an effect — in one sentence.

**Q6 — Scope.** `gravity` means "which part of the source survives the crop" on an
image. A different open ticket owns the `fit` vocabulary that `gravity` modifies.
Should `gravity` be decided together with the text and shape primitives, or does it
belong with `fit`?

## How to answer

Write your answer to the output path you are given, as Markdown. For each question:
your answer, your confidence, and the *reason* — preferring a concrete authoring or
editing failure over an aesthetic preference. Then a final section:

- **What would bite me first?** The one thing in this design most likely to make you
  ship a broken video while believing it was fine.
- **Where you disagree with the question itself** — a question that is malformed, or
  that presupposes something false, is worth saying so.

Do not consult any write-up of the author's reasoning; there is a GitHub issue #13 for
this work and **you must not read it**, nor issue #2. Work from the repo and the brief.
