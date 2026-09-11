# Answer D — text and shape primitives, from the authoring seat

Everything below is argued from one seat: I am the agent that opens
`en-halloween-decorating.montaget.json`, edits it with exact-string replace, and checks
myself with `validate` / `frame` / `measure` / `query`. Settled documents relied on:
CONTEXT.md, ADR-0007/0008/0012/0013, and the fixture itself (I re-read `card-05`,
`sentence-05`, `handle-logo` from the committed file).

---

## Q1 — One element type or many?

**Sibling types: `type:"rect"`, `type:"ellipse"`. No `type:"shape"` with a `shape:`
discriminator.** Confidence: high.

Reasons, in editing terms:

1. **The fixture already ships `type:"rect"` on 10 elements.** A `shape` supertype is a
   migration of committed data plus a rewrite of ADR-0012's published example vocabulary
   ("zero text and zero rect elements need it"), for no capability gained. ADR-0013's
   tiebreak (1) applies verbatim: the sibling spelling is a supplement; the
   discriminator is a correction.
2. **`type` is the substring I grep.** The editing model is exact-string replace over
   one-line elements. "Find every rect" is `grep '"type":"rect"'` today. Under a
   discriminator it becomes a two-field query, and — worse — `"type":"shape"` is the one
   `type` value that does not tell me what the element is. Every other type (text, image,
   audio, rect) answers "what is this?" by itself.
3. **ADR-0012 says "the schema varies by type" and builds its error strategy on it** —
   `"x"` on audio is a schema error *because* type discriminates the legal field set. A
   sub-discriminator adds a second axis to that machinery: "`fill` is legal on shape but
   which fields are legal depends on `shape:`" is exactly the two-shapes-one-consumer
   cost that killed the `scale` union ("one of them will get the rarer branch wrong").
4. The argument *for* a supertype — shared paint fields — is void: this format already
   shares fields across sibling types (`x`,`y`,`origin`,`width`,`height` on text, image,
   rect alike). Sharing fields never required sharing a type.

## Q2 — Which shapes exist in v1?

**In: `rect`, `ellipse`. Rejected: `path`. Unevidenced, not rejected: `line`,
`polygon`.** Confidence: medium-high on the in/rejected split; high that the
unevidenced pair must be recorded as unevidenced rather than rejected.

The real cut is not a shape list — it is a cut between **geometry models**:

- **Box-geometry shapes are free.** `rect` and `ellipse` are both fully specified by
  the transform every visual element already carries (`x`,`y`,`origin`,`width`,`height`,
  ADR-0012): a rect fills the declared box; an ellipse inscribes it. No new geometry
  fields, no new validate arithmetic, keyframes and `clip` work unchanged. `rect` is
  10-of-10 evidenced. `ellipse` has real evidence of *need* despite zero fixture
  instances: the fixture's one circle is **baked into `logo-en.png`** (corner alpha 0,
  measured fact 2) and its `mask:"circle"` is inert — i.e. the only real project wanted
  a circle and had to pre-bake it in an external asset the document cannot read. That is
  the format's own named failure (meaning outside the document), and `ellipse` is the
  box-geometry piece of the fix (soft/shaped *masks* stay #22's, per ADR-0012's seam).
- **Point-geometry (`line`, `polygon`) is a second geometry model** — endpoints or a
  vertex list instead of a box — which breaks the "every visual element is placed the
  same way" sentence ADR-0012 opens with, forces a decision about whether points are
  frame-space or box-relative, and gives `origin`/`scale`/`clip` a second meaning to
  define. Neither has evidence, and both horizontal/vertical rules and even diagonal
  dividers are already spellable as a thin `rect` (+ `rotation`). Per the standing rule,
  their absence from the fixture is not an argument against them — so they are
  **unevidenced and deferred**, with the recorded condition: the first real project that
  needs a divider I cannot spell as a rotated rect reopens them.
- **`path` is rejected, not deferred.** A bezier path field is artwork-as-data: it fails
  "understandable by reading" the same way an expression language does, it is the After
  Effects side of ADR-0003's line (CapCut/Premiere shape tools are rect/ellipse/line;
  pen-tool artwork is motion-graphics territory), and the correct spelling already
  exists — commit the artwork as an SVG/PNG and reference it as an image `source`,
  which keeps the bytes in git where the format's authority lives.

## Q3 — How is a colour spelled?

**`#RRGGBB` and `#RRGGBBAA`, uppercase, exactly one spelling per value: 3-digit
shorthand is a schema error naming the 6-digit form, CSS names are a schema error
naming hex, and `#RRGGBBFF` is a schema error naming `#RRGGBB`.** Confidence: high on
"one spelling per value"; medium-high on admitting `AA`.

1. **One spelling per value is forced by the write–read round trip**, and this is the
   `center-center` argument verbatim (CONTEXT.md, ADR-0013): ADR-0011 requires `fmt` to
   normalise on write, so an accepted alias (`#fff`, `red`, `#FF0000FF`) is silently
   rewritten and my next exact-string replace on the string I just wrote gets zero
   hits. Rejection-with-named-replacement is the established pattern (`weight:"bold"`,
   `anchor`-as-string, `center-center`); leniency is the one option this repo has ruled
   out repeatedly. It also keeps the colour census greppable: "every navy element" is
   `grep '#1E344C'` only if navy has one spelling. The fixture is already uniformly
   6-digit uppercase (fact 3), so canonical-uppercase costs zero migration.
2. **`AA` is needed and element `opacity` does not substitute**, despite Q3's hint.
   `opacity` is one scalar over the *whole element*, and it is a keyframe channel.
   Three concrete authoring failures without `AA`:
   - a run-level colour delta (ADR-0007's open style-delta set) cannot make one word
     semi-transparent — element opacity fades every run;
   - once `stroke` exists (Q5), "translucent fill, opaque stroke" on one shape is
     inexpressible;
   - a scrim rect at a standing 50% that also fades in must bake the 0.5 into every
     opacity keyframe (`0 → 0.5`), so editing the standing level later means recomputing
     every `v` — the exact base-geometry-and-animation-share-one-channel failure
     ADR-0013 recorded for `scale`. With `fill:"#00000080"` the keyframes stay `0 → 1`.
3. CSS names additionally smuggle a lookup table outside the document — the family-name
   argument from Fonts, in miniature.

## Q4 — Does a text element still declare a `height`?

**(a): `width` and `height` both stay required.** And I reject (c)'s finding as
specified. Confidence: high.

The decisive point is that **Q4's premise — "the vertical overflow check is a no-op on
15 of 22 elements" — is only true at the instant of authoring.** A self-derived height
is `ceil(size × line_height × line_count)` *frozen at write time*. The moment I perform
the ordinary edit — add a `\n`, grow a run, bump a `size` — the derived value moves and
the declared one does not, and the check fires. It is not a no-op; it is a regression
guard, exactly the structure ADR-0013 built for fit deviation ("on a correct file the
note fires zero times"). Concrete edit: I change `sentence-05`'s run to a two-line
string. Under (a), `validate` immediately reports derived 2×55×1.1=121 (…ceil per line
count) against declared 169 — still fits the card; make it three lines and it errors.
Under (b) with `height` absent, nothing fires and the text quietly grows over whatever
sits below it, every field individually valid — ADR-0007's "invisible defect class"
reintroduced by the option that was supposed to be honest.

On (b) specifically: it is *less* incoherent than the brief implies — a text element's
line count is document-computable with no shaper (no auto-wrap, ADR-0008: lines =
mandatory breaks + 1), so absent-height would not be the unreadable
natural-source-size default ADR-0012 killed. But it still violates the letter of
"declared, never defaulted," it makes `height` the one transform field that is
sometimes absent (a shape test in every consumer, the `scale`-union cost), and it
deletes the regression guard above. Not worth it.

On (c): a finding that fires whenever `height == derived` fires **15 times on the
committed, correct fixture**. That violates the zero-findings-on-a-correct-file
principle ADR-0013 just established, and trains me to ignore findings — the
false-confidence failure ADR-0006 exists to prevent. If validate wants to say anything
here, it is a *census* line ("15 heights self-derived; 7 container-sourced"), not a
per-element finding.

## Q5 — Is `stroke` a paint field or an effect?

**A field on the primitive — on shapes, and on text as a base style / run delta.**
Confidence: medium-high.

The line, in one sentence: **a paint field participates in rasterising the element's
own geometry; an effect operates on the element's pixels after it is drawn.** Stroke is
literally the other paint style of the same geometry (fill the region / stroke its
boundary — one rasteriser call in Skia, which ADR-0010 already chose); shadow, glow,
blur, and shaped masks consume the finished pixels, and they are #22's closed
vocabulary. ADR-0012 already drew this seam once (`clip` on the element; masks to #22)
and ADR-0007 already assigned outline to the run style-delta set as open business of
the text primitive ("colour, outline — with the ASS BGR-with-alpha-nibble trap written
down").

Authoring grounding: outlined text is subtitle bread-and-butter in the CapCut/Premiere
reference class, and it is per-run ("emphasise one word" = restyle one run). If stroke
lived in a separate effects vocabulary, a per-run outline needs the effect system to
address *runs inside* an element — a reference into an array that reorders under the
splice-edits ADR-0007 designed for. As a field it is a style delta like `color`, edited
in place. Spelling: `stroke:{"color":"#RRGGBB[AA]","width":N}` with Q3's colour rule —
which quietly retires the ASS BGR-with-alpha-nibble trap by never importing that
encoding.

## Q6 — Where does `gravity` get decided?

**With `fit` (#48). Not here.** Confidence: high.

`gravity` has no meaning except as a modifier of a fit-driven crop: its legal values,
and whether it exists at all, depend on what `fit` values exist (`contain` crops
nothing, so gravity is inert under it by construction; `cover` with a tight `clip` is
measured-inert on 8 of 8 fixture elements — fact 5). Deciding it in the text-and-shapes
ticket would legislate semantics for a vocabulary that is undefined — precisely the
"create a schema value by implication" move ADR-0013 refused for `contain`'s rounding.
As the authoring agent I never touch `gravity` while writing a text or shape element;
I touch it only in the same breath as `fit` on an image. ADR-0013 already routed the
inertness fact onward "not a decision"; this ticket should do the same.

One carve-out this ticket *does* owe: the field-legality table. `gravity` on a text or
shape element must be a schema error (the twice-ruled "a field the renderer cannot
honour is worse than no field"), and that error's wording should not presume #48's
outcome — "gravity is an image-fit field, see fit" is enough.

---

## What would bite me first

**The vertical check's green light laundering confidence over the horizontal check I
skipped.** Height overflow is pure document arithmetic (size × line_height × line
count) — `validate` computes it always, for free. Width overflow needs the shaper: the
declared `width` is externally sourced on **all 22** text elements (fact 4), and the
actual advance width exists only in `measure`'s output for a specific font file. So my
authoring loop has one check that is automatic and one that is a discipline. I edit a
run to a longer string *without* adding a line, run `validate`, see the height term
pass and no width finding (because I didn't re-run `measure`, or because `measure` ran
before someone swapped the font file — ADR-0007's recorded font-swap invalidation),
and ship a line that paints past the navy card's edge with every number in the file
individually valid. The mitigation belongs in this ticket's consequences: ADR-0007's
width term must be stated as *stale-unless-measured*, and `validate`'s summary should
count text elements whose width term is UNCHECKED (ADR-0013's category, reused) rather
than printing an undifferentiated pass.

## Where I disagree with the questions

- **Q4 presupposes the no-op.** "The vertical overflow check is a no-op on 15 of 22"
  conflates the authoring instant with the file's lifetime. A frozen derived value is a
  regression guard against the most common future edit; the 15 self-derived elements
  are where the check earns its keep, not where it is vacuous. The question's framing
  makes (b) look honest and (a) look like ritual; it is the reverse.
- **Q3's aside ("ADR-0012 already gives every element an `opacity`") implies AA is
  redundant.** It is redundant only for an element with exactly one paint and no
  animation — element-scalar alpha and per-paint alpha come apart the moment runs,
  stroke, or an opacity keyframe list exist, and two of those three are being created
  by this very ticket.
- **Q2 asks for a shape list when the decision is a geometry-model list.** Box-geometry
  shapes are nearly free under ADR-0012; point-geometry and path-geometry each cost a
  second placement model. Answering "which shapes" without naming that line invites
  relitigating `line` and `polygon` one shape at a time instead of once.
- **Q6 answers itself by being in this brief.** That it had to be asked here is the
  evidence it belongs elsewhere: nothing in the text/shape primitive needs to know the
  answer.
