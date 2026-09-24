# Brief (round 2): you are an agent that authors and edits Montagent projects

Montagent is a video editor whose project format is a single declarative JSON file,
authored and edited by an AI agent rather than dragged around in a GUI. **You are that
agent** — the consumer who will live in this format daily. Not a reviewer. Answer from
what it is like to use, and back every claim with something you actually did.

Repo: `/Users/mohammedehtemam/projects/github/Montagent` (read-only — see Rules).

## Read first

- `CONTEXT.md`, and `docs/adr/0001`, `0003`, `0005`, `0006`, `0007`, `0011`.
- **`gh issue view 21 --comments`** — the ticket AND its five comments. The comments carry
  prior jury evidence and are load-bearing. Do not skip them.
- The only project file ever written (a prototype, contradictable, fields marked INVENTED
  are guesses):
  `git show prototype/sample-project-file:docs/research/prototypes/sample-project/en-halloween-decorating.montagent.json`
  plus `FINDINGS.md` and `JURY-EDIT-EXERCISE.md` beside it.

## Settled in round 1 — build on these, do not reopen

A five-agent panel settled these. Treat them as given:

- **One positioning model** for every visual element: `x`, `y`, `origin`, and a size.
- **Absolute integer pixels** in the project's frame space. Not fractions.
- **Flat fields** on the element, never a nested `transform` object.
- **`"x"` on an audio element is a schema error**, not a silently-ignored field.
- **Properties**: `x`, `y`, `origin`, `scale`, `rotation`, `opacity`. Skew is out.
  `rotation` is unwrapped degrees (1080 = three turns), never normalised to [0,360).
- **Keyframe times are absolute** milliseconds on the project clock.
- **Keyframe records are objects** — `{"t":…, "v":…, "ease":…}` — not positional pairs.
- **Clamp at both ends**: before the first keyframe the value is the first value; after
  the last it is the last. So "then holds" costs no syntax.

## The round-1 evidence you need for Q6

`shift(at=20000, delta=2000)` on the fixture's `photo-06`: `start 17472`, `end 30603`,
`scale [[17472,1.0],[32472,1.08]]`. It is a time-invariant straddler, so ADR-0005
stretches `end` to 32603. What happens to its keyframes is the open question. Three
readings have been produced by agents, all legal, all validating clean:

- **MOVE** — add delta to every keyframe time >= `at`: `[17472 -> 34472]`. The 15000 ms
  ramp becomes 17000 ms; ends at 1.0712 instead of 1.0800.
- **HOLD** — leave keyframes untouched: `[17472 -> 32472]`. Ramp tops out 131 ms before
  the element ends, then freezes.
- **SPLIT** — evaluate the property at `at`, insert keyframes at `at` and `at+delta` both
  carrying that value, then add delta to every keyframe time >= `at`:
  `[{17472,1.0},{20000,1.013483},{22000,1.013483},{34472,1.08}]`.

Judge these yourself against the fixture and `shift`'s own contract. Do not assume the
panel was right.

## Do the work first — this is what the answers must come from

Write everything into your own scratch directory.

1. **Stress SPLIT where it is hardest.** Apply all three readings to: (a) a segment whose
   `ease` is NOT linear; (b) an element where `at` falls exactly on an existing keyframe;
   (c) an element lying entirely after `at`; (d) an element with a keyframe past its own
   `end` (the fixture has seven — a "trimmed move"). Report where each reading breaks.
   Compute real numbers.
2. **Try to subdivide an eased segment exactly.** Take the CSS `ease-in-out` cubic bezier
   and split one segment at an arbitrary interior point. Is either half expressible as any
   named ease? Show the arithmetic (de Casteljau, or equivalent). This decides whether a
   closed named easing vocabulary can survive `shift`.
3. **Write the fixture's seven Ken Burns lists both ways** — `"v":1.08` and
   `"v":[1.08,1.08]` — as they would really appear. Then read them. Say honestly what the
   pair costs and whether `fmt` normalising a scalar on write is a real fix or a dodge.
4. **Rewrite real fixture elements under a `box`/`align` retirement.** Take `photo-06`
   (`"box":[0,0,1080,1300],"fit":"cover","align":"top"`), `card-05`
   (`"box":[48,1453,984,169]`), and a text element with ADR-0007's `"box":"card-05"`.
   Express all three in the round-1 model. Does anything become inexpressible or worse?
   Note that `align` currently means image gravity on images and line alignment on text.
5. **Author a bare element** — type, source, start, end, nothing else — and say where it
   should render, at what size and opacity. Then author one that is deliberately hard to
   find, and say which default would have saved you.

## Then answer these five

- **Q6.** Which reading does `shift` take — MOVE, HOLD, SPLIT, or something else? State the
  complete rule for every case in exercise 1. If SPLIT, say what the rounding rule is for
  the interpolated value it writes, since nothing publishes one.
- **Q7.** Easing: a closed named vocabulary only, or named plus a raw cubic-bezier form?
  Exercise 2 decides it. If names cannot be subdivided, what does `shift` do — emit
  beziers, refuse, or something else? Also settle: does `ease` describe the segment
  **entering** its keyframe or **leaving** it? Both conventions exist and they render
  differently.
- **Q8.** Is `scale` always `[sx, sy]`, scalar-only, or a union of both?
- **Q9.** What replaces `box`? It currently means both a literal `[x,y,w,h]` rect and the
  id of an element you must fit inside. Name the replacement(s). Does `align` get the same
  treatment, and if so what happens to image gravity?
- **Q10.** The no-transform default. An element with no transform fields must render
  somewhere, at some size, at some opacity. What is it, and what does `origin` default to?

## Rules

- **Do not modify the repository.** No commits, no branches, no `gh` writes. Read with
  `git show`. Write only inside your own scratch directory.
- Cite the repo's own text. Where you are guessing, say so.
- Disagreeing with round 1's settled list, or with an accepted ADR, is in scope **if you
  say so loudly and separately** rather than quietly designing around it.
- Do not soften. Close calls get named as close, with the tiebreaker.

## Return

Your final message is the whole deliverable:

- **VERDICT** — Q6..Q10, under 12 words each.
- **What the work showed** — findings from the five exercises, with the numbers and JSON.
- **Where I changed my mind.**
- **Closest call** — and the tiebreaker.
- **What I would need to overturn this.**
