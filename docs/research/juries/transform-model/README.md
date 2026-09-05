# The transform-model jury — the evidence behind ADR-0012

Fifteen agents, three rounds, four model tiers, for [#21](https://github.com/MBehtemam/Montaget/issues/21).
Every juror was briefed as **an agent that authors and edits Montaget projects**, not as a
reviewer, and every one was made to *do the work* — author the JSON, run the shift, compute
the bezier — before answering anything. No juror saw the author's recommendations. Round 2
and 3 jurors saw the previous round's **findings** but never its vote counts.

The briefs are `brief-round-1.md`, `brief-round-2.md`, `brief-round-3.md`. Each `juror-*/`
directory holds that juror's own working files, unedited. Copies of repo files the jurors
fetched to work against are not reproduced here.

| round | jurors | models | question |
| --- | --- | --- | --- |
| 1 | A B C D E | opus, sonnet, fable, haiku, opus | the property set, units, shape, scope, keyframe time base |
| 2 | F G H I J | opus, sonnet, fable, haiku, opus | the `shift` rule, easing, `scale` shape, the `box` retirement, defaults |
| 3 | K L M N O P Q | opus, opus, sonnet, sonnet, fable, fable, haiku | `ease` direction, and whether the aperture is decided here or in #22 |

Only four model tiers were available, so rounds 1 and 2 each carry one duplicated tier.
Round 3 is seven jurors so the two questions could not tie.

## Round 1 — the property set

| Q | A (opus) | B (sonnet) | C (fable) | D (haiku) | E (opus) |
| --- | --- | --- | --- | --- | --- |
| one positioning model | ✅ `fits_in` | ✅ `fit_box` | ✅ `within` | ✅ keeps `box` | ✅ `must_fit` |
| units | absolute px | absolute px | absolute px | absolute px | absolute px |
| properties | `number\|[sx,sy]` | `[sx,sy]` | `[sx,sy]` | scalar | `[sx,sy]` |
| flat / audio | flat, errors | flat, errors | flat, errors | flat, **inert** | flat, errors |
| keyframe time | absolute + plateau | absolute + follow-`start` | **relative** | absolute + move-all | absolute + split-and-hold |

A and E, independently and on separate contexts, produced the **same** third answer to the
stretched-straddler question and the same interpolated constant to six places. C argued the
opposite case — element-relative times — and its own decisive finding (that moving a keyframe
changes frames *before* the edit point) is what the others used to build the rule that beat it.

## Round 2 — the `shift` rule and the retirement

| Q | F (opus) | G (sonnet) | H (fable) | I (haiku) | J (opus) |
| --- | --- | --- | --- | --- | --- |
| Q6 shift | element-anchored + SPLIT | HOLD; "SPLIT unimplementable" | element-anchored + SPLIT | MOVE | SPLIT |
| Q7 easing | named + bezier, **entering** | named; **leaving** | named + bezier; **leaving** | named + bezier | named + bezier; **entering** |
| Q8 `scale` | `[sx,sy]` | `[sx,sy]` | scalar-only | union | `[sx,sy]` |
| Q9 `box` | literal `w`/`h` | id-ref needs a field | literal `w`/`h` | — | + a new `clip` |
| Q10 default | centred, **size required** | native, **size required** | centred | centred | frame-sized |

G's objection — that SPLIT was unimplementable at `at`-on-a-keyframe and on elements outside
the straddle — was answered by H and F, who published the complete rule including both cases.
Two jurors (G, I) independently *mis-applied* SPLIT to elements it does not cover, which is
why ADR-0012 states the scoping as a table rather than a sentence.

The measurement that decided it (F, over every millisecond the element is on screen):

| reading | max scale error vs the pre-edit picture | px of width @1080 |
| --- | --- | --- |
| SPLIT | 3.3e-07 | **0.000** |
| MOVE | 0.0078 | 8.452 |
| HOLD | 0.0107 | **11.520** |

**HOLD is the worst of the three**, not the cautious one. The "131 ms freeze" framing describes
only its tail and hides that the whole post-`at` shot plays 2000 ms early against a ramp that
did not move with it.

## Round 3 — `ease` direction, and where the aperture is decided

Exercise 0 measured each juror's **unprompted prior** before it read anything: write a keyframe
list with mixed easing from instinct, then say which segment each `ease` was meant to describe.
The files are `juror-*/ex0-prior.md`, unrevised.

| | K | L | M | N | O | P | Q | |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| prior | entering | entering | leaving | leaving | leaving | leaving\* | entering | **4–3 leaving** |
| Q11 direction | ENT | ENT | ENT | ENT | ENT | LEAV | LEAV | **5–2 entering** |
| Q12 aperture | now | now | now | now | now | now | now | **7–0 now** |

\* P declared its own measurement contaminated: the STOP instruction sat mid-brief, and an agent
that reads a file top-to-bottom has already seen both conventions. The flaw is the brief's.

Three jurors ruled against their own measured prior. **Five of seven wrote an `ease` on the
record where their own convention had no segment to describe** — which is why the "one keyframe
is structurally forced to omit `ease`" check catches the wrong prior under *either* convention,
and therefore does not discriminate between them. K found this and withdrew its own argument.

The CSS prior turned out to be 4–3, not the rout the case for `leaving` assumed. L located why:
the CSS construct matching this shape is per-keyframe `animation-timing-function` inside
`@keyframes`, an obscure corner; the *strong* prior is `transition-timing-function`, which
belongs to the whole transition and attaches to neither endpoint, so it discriminates nothing.
Lottie's `o`/`i` is genuine `leaving` evidence. The reference class is split.

Two arguments each side raised and then withdrew: the schema-sentence test (K — symmetric,
catches nothing), and diff-visibility (L — ADR-0007 puts the element on one line, so any
keyframe change reflows it regardless). What survived is the **write-set invariant**, and
O's observation that the settled SPLIT rule as worded is complete under `entering` and
incomplete under `leaving`.

## Where the jury corrected the author

- The `shift` rule first proposed here — "translate every keyframe time by delta" — is MOVE,
  disqualified three separate ways.
- HOLD's cost was described as a 131 ms freeze. It is 11.52 px, the worst of the three.
- `fmt` normalising a scalar `scale` on write was proposed as the fix for the union. F and J
  both killed it with the same mechanism: you write `"v":1.08`, `fmt` rewrites it to
  `"v":[1.08,1.08]`, and your next exact-string replace on the string you wrote gets zero hits.
- Round 1's brief did not point jurors at `gh issue view 21 --comments`, so all five missed the
  prior evidence on the ticket. Round 2 and 3 fixed it.
