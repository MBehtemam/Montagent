# Round 2 brief — Montaget issue #48, the `fit` vocabulary

Repo: /Users/mohammedehtemam/projects/github/Montaget, branch `main`.
Montaget is an agent-first declarative video editor: a JSON project file is the source of
truth and an LLM agent authors it with ordinary file tools.

## Read first

1. `gh issue view 48` — the ticket.
2. `docs/adr/0013-fitted-extents-floor-and-the-nine-origin-keywords.md` — ALL of it. Especially
   "The declared rect is authoritative at render", "What `validate` checks", "Not settled here".
3. `docs/adr/0012-flat-transform-keyframes-carried-by-their-element.md` — the `clip`/aperture section.
4. `docs/adr/0014-stroke-is-paint-the-text-box-is-required.md` — "`gravity` is not decided here".
5. `docs/adr/0006-validate-reports-facts-and-render-enforces.md` — opt-in checks, omitted fields, NOT CHECKED.
6. `docs/adr/0005-absolute-integer-milliseconds.md` — the `speed` divergence (`3368/0.645`), a cautionary precedent.
7. `docs/adr/0003-general-video-editor-not-channel-tooling.md` — THE SCOPE RULE. The fixture channel is
   evidence a capability is NEEDED, never evidence one is UNNEEDED. "The fixture doesn't use it" is NOT
   an argument against a field.
8. `CONTEXT.md` — the glossary and its recorded term collisions (`anchor`, `align`, `box`, `gravity`, `path`).
9. `fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json` — all 8 `"type":"image"` elements.
10. Run `python3 docs/research/sample-project-migration/fit_rounding_scan.py`.

## Settled in round 1 — treat as established, do not re-litigate

A prior jury settled these. They are your premises:

- **`fit` is a derivation claim, not a render instruction.** Under declared-rect-authoritative the
  renderer resamples the source to exactly the declared `width`/`height` regardless of `fit`. `fit`
  records HOW the author computed those extents. Its consumer is `validate` (and `render`'s copy of
  those checks).
- **The box the rule fits into is `clip`'s width/height.** Verified: source 1536x2720 into clip
  1080x1300 gives exactly the published 1080x1912. The project frame (1080x1920) gives 1084x1920 and
  is falsified. Note separately that using the element's own declared rect as the box is an
  ALGEBRAIC FIXED POINT — it reproduces 1912 on all 8 elements and so cannot be discriminated by
  equality with the published number.
- **`fit` is required** on elements carrying a raster source; omission is a schema error.
- **`gravity` is retired** — a schema error naming its replacement. The argument is structural: under
  declared-rect-authoritative nothing leaves the crop underdetermined, so the quantity `gravity`
  names does not exist.
- **The fit-deviation check is promoted from `note` to `error`**, with this predicate: the declared
  driving axis must equal the box dimension with zero grace, and the declared slack axis must be in
  `{floor(exact), ceil(exact)}` in integer arithmetic. It breaks 0 of the 8 committed elements.

## Your questions

Answer all five. For each give **Decision**, **Why**, and **Strongest counter** (the best argument
against your own answer). Where a question admits a computation, compute it — do not guess.

**R2-Q1. The escape value's name.** The vocabulary needs a member meaning "do not derive my extents;
these integers are mine." Candidates, alphabetically: `declared`, `exact`, `none`, `stretch`. You may
propose a different name. Consider: which names are already spent elsewhere in this format; which
collide with CSS `object-fit` semantics in a way that would mislead an agent arriving from CSS; and
whether the name should describe the CAUSE (no derivation rule was applied) or the EFFECT (the source
may be anisotropically resampled). This name is permanent.

**R2-Q2. Does `contain` ship in v1, or is the set `cover` + escape only?** If it ships, state its
inequality and rounding direction. Then resolve this specific collision, which is the live cost:
ADR-0013's aperture-coverage error requires the declared rect to CONTAIN `clip`, and it is hard-coded
to the cover direction. A correct `contain` element is SMALLER than its clip and would trip that
error. Say exactly how you scope or parameterise that error, or why `contain` should be deferred
instead. Weigh ADR-0003's asymmetry rule when reasoning about the fixture's non-use of `contain`.

**R2-Q3. What does a declared `fit` mean when `clip` is absent?** `clip` is optional. Candidate
answers include: the box falls back to the element's own declared rect; the element is `UNCHECKED`;
the box falls back to the project frame; or a declared `fit` without `clip` is a schema error.
Remember the fixed-point property noted above when evaluating the first of these — and consider what
ADR-0006 says about a check that silently weakens to nothing.

**R2-Q4. Define "the source's dimensions" normatively.** The rule's input is the source's pixel
dimensions, and no document defines what that means. A JPEG can carry an EXIF orientation flag that
transposes width and height; a video stream can carry a non-square pixel aspect ratio. So two
conforming implementations can read different dimensions from the same file and compute different
integers — in a rule whose entire purpose is that independent implementers land on the same integer.
Compare ADR-0005's `speed` divergence. Decide the normative definition, and decide whether
`validate` must print the dimensions it used.

**R2-Q5. Does this ADR cover `video` elements, or only `image`?** Every rule here was specced against
8 `image` elements. ADR-0003 commits Montaget to video clips. Decide whether the rules are written
type-generically now or scoped to `image` and extended later, and name any wrinkle video introduces
that images do not.

## Output

Write your verdict to the path you are given, as markdown, with `## R2-Q1`..`## R2-Q5` sections each
containing **Decision:**, **Why:**, **Strongest counter:**. Add a final `## Anything these five
missed` section if you find a hole.

Your final message back must be compact: one line per question giving just the decision, plus the path.
