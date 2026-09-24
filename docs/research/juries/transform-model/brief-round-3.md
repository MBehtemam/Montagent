# Brief (round 3): two questions, and you must rule on both

You are an agent that authors and edits Montagent projects — a video editor whose project
format is one declarative JSON file, authored by an AI agent rather than a GUI. You are
the consumer who will live in this format. Not a reviewer.

Repo: `/Users/mohammedehtemam/projects/github/Montagent` (read-only).

**You must return a decision on BOTH questions. "It depends", "either is defensible", and
"the panel should choose" are not answers. If it is close, say it is close, then decide.**

## STOP — do Exercise 0 before you read anything else in this file

Do this now, before reading further, before reading any repo file. It is measuring your
prior, and reading ahead destroys the measurement.

Write, from your own instinct, a JSON keyframe list for an element that: sits still for
one second, then over 400 ms slides from x=-400 to x=100 with an ease-out feel, then
over 600 ms continues to x=540 with a linear feel. Use the record shape
`{"t": <ms>, "v": <number>, "ease": <name>}`. Absolute milliseconds. Element starts at
t=0.

Then, without changing it, write down: **on which keyframe did you put each `ease`, and
did you intend it to describe the segment arriving at that keyframe, or the segment
leaving it?** Save this verbatim to `ex0-prior.md` in your scratch directory. Do not
revise it later. If you got it ambiguous or wrong, that is data — record it as it is.

## Now read

- `CONTEXT.md`; `docs/adr/0001`, `0005`, `0006`, `0007`, `0011`.
- `gh issue view 21 --comments` and `gh issue view 22`.
- The sample project:
  `git show prototype/sample-project-file:docs/research/prototypes/sample-project/en-halloween-decorating.montagent.json`

## Settled — do not reopen

One positioning model (`x`, `y`, `origin`, plus a size). Absolute integer pixels. Flat
fields. `"x"` on audio is a schema error. Properties: `x`, `y`, `origin`, `scale`,
`rotation`, `opacity`; skew out; `rotation` unwrapped degrees. Keyframe times absolute.
Records are objects `{"t","v","ease"}`. Clamp at both ends. `scale` is always `[sx,sy]`.

**And `shift` uses SPLIT**, established by measurement (frame error 0.000 px, against
8.45 px for moving keyframes and 11.52 px for leaving them). The rule: `shift` moves
*elements*; keyframes are carried. For a time-invariant straddler, evaluate the property
at `at`, insert keyframes at `at` and `at+delta` both carrying that value, shift keys
strictly after `at` by delta, and stretch `end`.

**Established arithmetic:** splitting CSS `ease-in-out` at an interior point yields
`cubic-bezier(0.362866, 0, 0.693025, 0.369169)` and `(0.365106, 0.225535, 0.568419, 1)`.
Neither is any named ease; snapping to the nearest costs ~10 px of framing. So named eases
are sugar over beziers, `shift` emits raw beziers when it subdivides, and only `linear`
and `hold` are closed under subdivision. Do not re-derive this; build on it.

---

# Q11 — does `ease` describe the segment ENTERING its keyframe, or LEAVING it?

Both conventions exist. They render differently. The field is one word wide and the wrong
guess is silent.

- **ENTERING**: `ease` on keyframe K governs the segment arriving at K. The first keyframe
  in a list can carry no ease.
- **LEAVING**: `ease` on keyframe K governs the segment departing K toward the next. The
  last keyframe in a list can carry no ease. This is the CSS `@keyframes` and Web
  Animations convention.

Arguments already on the table, both sides, in no order and with no vote counts:

- Under SPLIT, subdividing an eased segment must write the left half-bezier somewhere.
  Under ENTERING that lands on the newly inserted keyframe at `at`; every record `shift`
  touches then has `t >= at`. Under LEAVING it lands on the keyframe *before* the edit
  point — a record that keeps its own `t` and `v`, so it looks untouched in a diff, in a
  format whose editing model is exact-string replace over a diff a human eyeballs.
- LEAVING matches CSS, and the agents authoring this format are saturated in CSS. ADR-0007
  records the fixture carrying `weight:"bold"` on all 22 text elements — a CSS habit
  firing against a format that does not have the field. A convention that fights the prior
  produces a wrong guess that renders differently with nothing flagging it.
- Appending a keyframe to the end of a list is a one-site edit under ENTERING and a
  two-site edit under LEAVING (you must also set the ease on the previously-last record).
  Prepending is the mirror.

## Exercises for Q11 — do these, do not just weigh the arguments above

1. Take your own Exercise 0 answer. Which convention did you use, unprompted? Say so
   plainly even if it embarrasses the position you end up arguing.
2. Write a real 4-keyframe list with a *different* ease on each segment, under BOTH
   conventions, for the same intended motion. Put them side by side.
3. Now `shift` both files (SPLIT, delta=2000, `at` strictly inside the second segment).
   Produce both resulting files. **Diff each against its own pre-shift version and count
   the changed records, and how many changed records have an unchanged `t` and `v`.**
4. Append a fifth keyframe to both. Then delete the *middle* keyframe from both. Count the
   edit sites each operation needs under each convention.
5. Write the one-sentence schema field description for `ease` under each convention. Which
   one is self-teaching to an agent that has the wrong prior?

# Q12 — does the clipping aperture belong in this ADR, or wait for the effect model (#22)?

The fixture's `photo-06` is `"box":[0,0,1080,1300],"fit":"cover"`, source 1536x2720. Cover
scales it to 1080x1912.5 — but the box is only 1300 tall. The box was silently doing two
jobs: placing the element AND clipping it. Retire `box` to `x`/`y`/`w`/`h` with no
aperture and the photo paints ~612 px over the background that belongs there. **The
fixture's primary visual renders wrong.**

One proposal is a static frame-space `clip` rect that does not rotate or scale with the
element — the aperture stays put while `scale` moves the picture behind it, which is what
a Ken Burns is. A source-space `crop` was considered and rejected: under `crop`, `scale`
grows the drawn rect past the aperture and spills.

The question is not whether an aperture is needed. It is **where it is decided**: in the
transform ADR now, or in #22's effect/mask model later.

## Exercises for Q12

1. Verify the 612 px claim yourself against the real file. Compute the cover factor and
   the overflow. Report the actual numbers.
2. **Try to express the aperture using only the settled property list.** Is it genuinely
   inexpressible, or is there a composition of settled fields that does it?
3. Read `gh issue view 22`. Does the aperture fall naturally inside the effect model's
   stated scope, or is it a different kind of thing? Quote the ticket.
4. Census the fixture: how many elements other than `photo-06` need clipping? Does any
   *text* or *rect* element need it? Does the answer change if the aperture is transform
   or effect?
5. Say what breaks if this ADR ships with no aperture and #22 adds one in three months —
   concretely, what files and what decisions have to be revisited.

---

## Rules

- Do not modify the repository. Read with `git show`. Write only in your scratch directory.
- Cite the repo's own text; where you guess, say so.
- Do not soften, and do not hedge into a non-answer. Both questions get a decision.

## Return

- **VERDICT** — Q11 and Q12, one line each, decided.
- **Exercise 0 result** — which convention your unprompted instinct used. State it first,
  before any argument. This is the panel's only measurement of the prior and honesty here
  matters more than consistency with your verdict.
- **What the work showed** — with the diffs, counts and numbers.
- **Where I changed my mind.**
- **What I would need to overturn this.**
