You are a juror. Answer from the packet below alone. Do not use tools, do not read files, do not edit anything, do not offer recommendations about process. Answer the four questions and nothing else.

# Packet: the `mask` effect's parameter set in Montagent

## What Montagent is

Montagent is an agent-first video editor. A single declarative JSON project file is the source of truth; agents author and edit it with ordinary file tools. The file must be fully understandable by *reading* it — no evaluation, no code. A small CLI/MCP surface (validate, query, frame, render, compare, ...) does what a text editor cannot. It will be open source and used by people other than its author, so primitives are shaped by what a video editor must express, never by what one project happens to use.

The repo carries one real fixture: a 9:16 language-teaching YouTube short. The fixture is **evidence a capability is needed, never evidence one is unneeded**.

## The settled model around this question

**Element placement (ADR-0012).** Every visual element is placed by flat fields in absolute integer pixels on the project's frame: `x`, `y`, `origin` (nine keywords, e.g. `top-left`, `center`), a declared `width`/`height`, plus `scale` (always `[sx, sy]`), `rotation` (degrees clockwise, never normalised), `opacity` (0..1). There is exactly one transform per element; it is never nested, never inherited, never relative to another element. `x`/`y`/`width`/`height`/`clip` are integers; `scale`/`rotation`/`opacity` are floats. Any transform property may be a value or a list of `{"t","v","ease"}` keyframe records on the project's one absolute clock. **Keyframes are transform-only** — no effect parameter is keyframable in v1.

**`clip` (ADR-0012, ADR-0025).** A **static frame-space** rectangle an element is drawn through. It does **not** rotate and does **not** scale with the element, so `scale` moves the picture behind a window that stays put — which is what a Ken Burns is. It is permanently static (explicitly ruled not keyframable). The domain glossary's entry for `clip` says "Shaped and soft masks are a different thing and are not this", and its entry for Mask says "_Avoid_: clip (that name is the transform model's static frame-space aperture — a different concept that happens to sound alike)".

**Effects (ADR-0040).** `effects: [...]`, an ordered list field on the element. Order is semantically real (blur-then-shadow differs from shadow-then-blur); two effects of the same name are ordinary. The vocabulary is **closed, named and published in the schema** — never a plugin architecture. v1 has seven members: `blur{radius}`, `shadow{dx, dy, radius, color, opacity}`, `mask{shape: "circle"|"rect"|"ellipse", ...shape params}`, and four colour scalars. Effects attach to whole elements, never to a run. Rule: **numeric parameters only, enumerable from the schema alone**. Image-source / alpha / soft masks are deferred (they'd drag in a second asset reference and unresolved fitting/luminance questions).

**The colour-filter admissibility standard (ADR-0049).** The four colour members are `tint{color, amount}`, `saturation{amount}`, `brightness{amount}`, `contrast{amount}`. No fifth arrives without clearing a **four-clause rule**: fixed arity, bounded scalar parameters, a documented identity value, and not reproducible by composing two members that already exist. `grayscale` is not a member (it is `saturation` at zero); `sepia` is that composed with a warm `tint`. ADR-0049 spelled out every member's fields and gave an example literal — which is exactly what ADR-0040 did **not** do for `mask`.

**Defaultable-field presence is content (ADR-0030).** Whether a project file writes a defaultable field explicitly or omits it is **itself a declaration** and is preserved. So the *spelling* of a default matters, not just its value.

**Canonical key order (ADR-0041).** Key order is schema property order, `validate` checks it, `fmt` enforces it. Whichever ADR introduces a field fixes its position. `effects` appends after a type's existing fields; for `image`: `source, x, y, origin, width, height, fit, clip, scale, effects`. The order **within** a `mask` member is this question's to fix.

**Closed schema, no escape hatch (ADR-0017).** An unknown key is an error.

**No union spellings (ADR-0012, ADR-0013, ADR-0068).** This project has repeatedly retired "two ways to spell one thing". The mechanism it cites: `fmt` normalises on write, so an agent writes one spelling, `fmt` re-emits the other, and the agent's next exact-string replace gets **zero hits**. `scale` is always `[sx, sy]` and never a bare number for exactly this reason, at a deliberate +26% cost on the fixture's Ken Burns lists.

## What was decided most recently, and must not be reopened

ADR-0040 wrote the mask union member as `mask{shape: "circle"|"rect"|"ellipse", ...shape params}` and **never wrote the ellipsis**. ADR-0068 found that gap while migrating the fixture, and stated **the minimum the migration forced and no more**:

> `{"name": "mask", "shape": "circle"}` with no geometry parameters means the largest circle inscribed in the element's own rect — diameter `min(width, height)`, centred on that rect. The `rect` and `ellipse` shapes take the element's rect itself under the same rule.

Its evidence was the only real instance: the fixture's `handle-logo`, an 800x800 RGBA badge whose stored alpha is *already exactly* the inscribed circle, placed in a square 68x68 slot with `clip` equal to the element rect. ADR-0068 explicitly said the full parameter surface — explicit centre and radius, corner radii, two axes, behaviour under keyframed `scale` or non-`top-left` `origin` — is **graduated to its own ticket**, and that ADR-0049's four-clause discipline is the standard the eventual parameter set should be held to "rather than something to guess at now". It also warned that the param-less default "is **not** evidence that the explicit form should look any particular way".

ADR-0068 also ruled that **admitting both a bare `mask` key and the `effects` member was refused** — one spelling only.

## The shipped renderer's current behaviour (unratified by any ADR)

A later ticket implemented the `mask` member in Rust/Skia. No accepted document states any of the following; the code simply does it, and golden frames lock it in:

- Effects run **in element space**. Drawing order per element is: `clip` is applied first, in frame space, outside everything; then translate to `(x, y)`; then `rotation`; then `scale`; then the box is moved back by the `origin`'s fraction of itself; and **only then** are the effect layers opened. Inside, the element's box is `(0, 0, width, height)` in **unscaled** units.
- Therefore the mask **rides the transform**: it scales with a keyframed `scale` and rotates with `rotation`. A `rect` mask on a rotated element paints a rotated rectangle on screen. This is the same rule that makes a `blur` of radius 8 on an element at `scale: 2` sixteen frame pixels wide, and that makes a text `stroke_width` scale with the element.
- The mask's `(0,0)` is the rect's top-left **regardless of `origin`**, because the origin-offset translate happens before the effect layers open.
- A mask is **not** implemented as a clip: it is the *complement* of the shape, painted antialiased in `Clear` over the element's own layer. Deliberately so — a clip established before an enclosing layer would restrict that layer's bounds, so `[mask, blur]` would hand the blur an input already cut to the mask and lose every contribution from just outside it.

**A caution about deferring to this code.** A prior ADR (ADR-0075) exists precisely because an earlier ADR asserted a rendering fact ("the change is pixel-inert / the rendered frame is unchanged") that turned out to be false when measured: the renderer minifies the 800x800 badge 11.8x into a 68x68 slot with no mipmaps, so the *drawn* badge reaches marginally past the circle the *stored* badge does not, and the mask trims that rim — 112 pixels of a 540x960 frame changed. An unratified renderer reading has been wrong before.

## Constraints any answer must respect

- **Numeric parameters only**, enumerable from the schema alone.
- **Backward compatibility**: `{"name":"mask","shape":"circle"}` must keep meaning what ADR-0068 says, or the committed fixture migrates again.
- **Defaultable-field presence is content** — the default's spelling matters.
- One spelling per concept; no unions.
- The answer is an ADR held to ADR-0049's four-clause standard.

---

# The four questions

## Q1 — Does v1 gain explicit mask geometry at all, or does this ticket ratify param-less-only?

The ticket is framed as "decide the parameter set", but that presumes a prior question: does `mask` get explicit geometry parameters **in v1**, or does this ticket ratify the param-less form as the only spelling and record where explicit geometry would go if it ever arrives?

Considerations on the record, in no order and with no weighting implied:
- An off-centre circle, a rounded-corner rect, or an ellipse narrower than its box is **inexpressible today by any means**. There is no composition workaround (unlike a vignette, which ADR-0040 says to express as a shape element with a feathered mask, or a glow, which waits on blend modes).
- There is **zero evidence** for explicit geometry. The only real instance in the repo is the badge, which the param-less form already means exactly. ADR-0040 admitted `mask` at all on the strength of one observed authoring gesture in the fixture.
- The param-less form is **derived from the element's rect**, so its meaning changes silently whenever `width`/`height` change. Resizing an element from 68x68 to 68x120 yields a different mask with no edit to the mask member.
- ADR-0049's four-clause rule was written for admitting new *members*, not new *parameters on an existing member*; whether it transfers is itself part of the question.

Options: (a) add explicit geometry in v1; (b) ratify param-less-only and put explicit geometry in "not yet specified" pending a forcing case; (c) something else.

## Q2 — Coordinate space: element-local or frame-absolute? And where is the origin of that space?

Element-local (coordinates measured against the element's own rect) or frame-absolute (the same numbers `clip` uses, measured on the project's frame)?

And if element-local: is `(0,0)` the rect's top-left **regardless** of the element's `origin` keyword, or does the `origin` keyword move the mask's coordinate origin too?

Note the fixture cannot distinguish these: `handle-logo` has `origin: "top-left"` and a `clip` equal to its own rect, so every candidate answer produces the same pixels there. The project's ADR-0003 holds that fixture silence is **not** evidence a question doesn't matter.

## Q3 — Does the mask live in pre-transform or post-transform space?

Does the mask ride the element's `scale` and `rotation` (so a keyframed Ken Burns grows the masked circle with the picture, and a rotated element's `rect` mask paints a rotated rectangle), or does it sit in a space the transform does not reach (a fixed porthole the picture moves behind)?

Relevant: `clip` already **is** the fixed, non-rotating, non-scaling frame-space porthole, and is permanently static. Also relevant: `blur` radius and text `stroke_width` both scale with the element under the current model.

Also worth deciding explicitly: `scale` and `rotation` are keyframable, and effect parameters are not. If the mask rides the transform, a mask's on-screen geometry is animated *indirectly* by keyframes on `scale`/`rotation`. Is that acceptable, or does it violate the spirit of "no effect parameter is keyframable in v1"?

## Q4 — Does `circle` on a non-square element warrant a finding?

`min(width, height)` is determinate but may be nobody's intent: a `circle` on a 1080x1912 element inscribes a 1080px circle and erases 832px of the element. On the fixture's square 68x68 badge the same arithmetic is exactly right.

Options: (a) silence — it is determinate and documented; (b) a `review`-severity finding when a **param-less** `circle` sits on a non-square rect, naming a repair; (c) an error; (d) something else — e.g. `circle` on a non-square element means something other than the inscribed circle.

Note a `review` finding is only honest if it can name a repair, which may depend on Q1. Note also that `ellipse` inscribed in a non-square rect is an ordinary oval and plausibly needs no finding at all — so an answer should say whether the concern is specific to `circle`.

---

Answer in exactly this block, once per question, naming the model that backs you:

🗳️ **Juror <n>** (<model>) — **VOTE Q1: <chosen option or one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <what it costs, or why not the others>

...and the same for Q2, Q3, Q4.

You may reject a question's framing outright if the options are all bad or the question is wrong — that is a valid ballot.
