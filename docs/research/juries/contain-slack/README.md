# Where `contain`'s slack goes — [#52](https://github.com/MBehtemam/Montaget/issues/52)

**Outcome: no schema change.** The slack is placed by `x`/`y`/`origin`, exactly as every
other element is placed. What was missing is a sentence, not a field.

`contain_slack_scan.py` re-derives every number below and exits non-zero if any of it
stops reproducing. Run it from the repo root.

## Method

Four rounds, fifteen jurors across four models (Opus, Sonnet, Haiku, Fable), each blind to
the author's write-up and to prior rounds' worklogs. Rounds 1–2 made jurors **author the
element** before opining, so the headline numbers are measurements rather than opinions.
Rounds 3–4 were adversarial: jurors were told to refute named claims, including claims the
earlier rounds had produced.

The worklogs are summarised here rather than reproduced verbatim. The load-bearing artifact
is the scan script — per `docs/agents/domain.md`, the evidence a decision rests on must be
checkable by a later reader, and a re-executable script is the form that survives.

## What the jury settled

**No placement field.** Unanimous in round 1, and it survived the round-2 reversal attempt.
ADR-0015's structural argument transfers verbatim: with the declared rect authoritative,
`x`/`y`/`origin` against a static `clip` leave no freedom for a `place` field to spend. Such
a field would either restate what `x`/`y`/`origin` already say — the two-spellings defect
that retired `center-center` and `#RRGGBBFF` — or be read at render, repealing
declared-rect-authoritative. There is no third state.

**The idiom is `origin` plus the aperture's own integers.** For the badge case (aperture
`[840,1700,200,160]`, source 1200x400, derived rect 200x66):

    origin:"center",  x = 840 + 200//2 = 940,  y = 1700 + 160//2 = 1780

Both operands come from `clip`. **The derived `66` is not an input.** The hand-computed
spelling four of eight consumers wrote — `y = 1700 + (160-66)//2 = 1747` — denotes the same
rect but consumes the derived extent, so it goes stale when the source is replaced. Both are
legal; they are not equivalent under maintenance.

**The 4-centred/4-top-anchored split was a prompt artifact.** The task said "entirely
visible", not "centred", and `contain` delivers entirely-visible at every `y` that keeps the
rect inside the aperture. Eight agents were asked a question with many correct answers and
produced two of them. The real finding is the other number: **8 of 8 reached for a field
that does not exist**, and that number is the same under any prompt, because it is a fact
about what the docs and the corpus show.

**Accepted cost, not a decision:** when parity makes the exact centre half-integral, integer
`x`/`y` cannot spell it. Corner-anchoring is exact iff the slack is even; centre-anchoring
iff the box dimension is even. On the driving axis the slack is always zero, so corner
spelling is unconditionally exact there. Bounded by 0.5px.

## Three claims that were asserted during the investigation and then falsified

Recorded because each was believed, acted on, and written into a draft before it broke.

**1. "The fixture proves a floor rule for origin resolution."** It does not, and no such
rule should ship. The 10 odd-height `origin:"center"` elements are **4 distinct geometries**,
only **1** of which has a corroborating `top-left` twin — ADR-0013's own "one geometry counted
seven times" deflation, recurring. The one corroboration is contaminated twice: `migrate.py`
copies the text `x`/`y` straight through from the source ASS `\pos`, and back-fills
`CARD = (984, 169)` onto the sentences, so the coincidence is manufactured by the migration.
And **ADR-0012 already records this exact number as an error** — *"its centre is `y=1537.5`
and the fixture's text sits at 1537, half a pixel off… the default origin cannot be relied on
for pixel-exact layout."* Reading it as evidence for a rule reverses the recorded intent.

The rule was also **misnamed**: `y - height//2` is floor of the *half-extent*, which is
**ceil of the edge**. `floor(1452.5)` is 1452 and does not match. Publishing it as "floor"
would have split implementers by 1px on the first odd element.

And it applies to **nothing**: 0 of 60 elements exercise it. All 18 non-text visual elements
are `top-left` (where resolution is the identity), every `width` in the file is even, and the
only `center`-origin elements are text — which are placed by their **block**, not their box.

**2. "Where the text block sits inside its declared box is undefined."** It is defined.
ADR-0007 places the *block*, by `origin`, and its worked example — *size 55, line_height 1.1,
centred on 1537 → 1506.75–1567.25* — is `sentence-05` itself, computed from `y` and the
derived block height with the declared `height` appearing nowhere. ADR-0014 closes it from
the other side: the box *"is a container claim and not drawn geometry."*
`docs/research/prototypes/rust-rasterizer/src/text.rs` already implements it. Residual
ambiguity on every committed element: **0px**. The 108.5px box-vs-block gap is real but names
two things never composited against each other.

*Honest caveat:* the worked example cannot discriminate "placed by origin" from "centred in
box", since those are algebraically identical when the vertical origin is `center`. It does
eliminate the two non-centred readings. The tie is broken by ADR-0007's prose and ADR-0014's
"container claim", not by the example.

**3. "Once `origin:"center"` is always expressible, the stale-placement bug cannot occur."**
Refuted. `top-left` stays legal, ADR-0012 actively *recommends* it for odd chrome, and 4 of 8
consumers chose it unprompted. The honest form is *avoidable by a disciplined author* — which
is what ADR-0006 exists to refuse. Worse, the prescription is catastrophic if unscoped: under
`cover` the rect is **larger** than the clip, so its centre is not the clip's centre. Writing
`photo-05` at the clip centre gives a rect top of **-306** against a committed 0 — a 306px
crop shift on a 1300px aperture, silently re-implementing the `gravity:"top"` ADR-0015 retired.

## Findings outside #52's scope

Each is filed as its own ticket.

- **Text block arithmetic has no stated numeric domain.** `line_height` is the format's only
  non-integer field; `55 * 1.1` is `60.50000000000001` in IEEE double. `ceil` of the block
  diverges between float and exact decimal on **76 of 1197** size x line-count cases
  (**6.35%**). The fixture escapes on all of its pairs — exactly how ADR-0013 describes
  `cover` escaping its ULP bug — but it is **one `\n` away**: add a break to `sentence-05`
  and the derived height is 121 exact, 122 float. ADR-0013 banned floats from the fit
  arithmetic over a **4.466%** divergence; the equivalent clause is missing here.
- **The baseline within the line slot is unwritten.** ADR-0007 gives the line *slot* but never
  says where the glyph baseline sits in it. Live conventions span ~5px against the fixture's
  actual font, on 22 of 22 text elements.
- **ADR-0006's `sentence-quiz` overhang is false.** It claims the element *"overhangs its
  card's top edge by ~6 px"*. The element has one run and no `\n`, so under ADR-0008's
  no-automatic-wrapping it is one line: the block sits **53.75px inside** the card. Reproducing
  ~6px needs about three lines, which the format forbids. It is a fourth stale fact in an ADR
  that already retracts three, and it motivates a check that may not be needed.
- **`fmt` and defaulted fields.** Nothing states whether `fmt` inserts or strips defaults.
  ADR-0012 defaults six things; if `fmt` materialises any of them it breaks exact-string
  replace on all of them.
