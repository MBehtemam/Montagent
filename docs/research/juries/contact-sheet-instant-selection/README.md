# Jury: which instants go on the contact sheet

Three jurors, three different models, put to
[#398](https://github.com/MBehtemam/Montagent/issues/398) on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Resolved as
[ADR-0094](../../../adr/0094-the-sheets-instants-are-visual-states-sampled-at-the-first-painted-frame.md).

Each juror received `BRIEF.md` **verbatim and identical** — no assigned stance, no
persona, no sight of each other's ballots, dispatched in parallel. Ballots are
recorded here unedited.

| juror | model | ballot |
| --- | --- | --- |
| 1 | Opus 5 | [opus-5.md](opus-5.md) |
| 2 | Sonnet 5 | [sonnet-5.md](sonnet-5.md) |
| 3 | Fable 5.1 | [fable-5-1.md](fable-5-1.md) |

## What the panel settled without dissent

- The visual collapse is a `type` filter applied **inside `frame`**, never a new
  `query` mode — all three applied ADR-0074 rather than carving an exception, and
  all three independently identified the **re-merge** of now-identical intervals as
  the operation that does the work.
- The instant is the **first frame the project's grid actually paints** inside the
  run, by ADR-0077's `floor(n × 1000 / fps)`. Midpoint rejected 3–0, on a reason the
  brief did not supply: a midpoint is a synthetic instant no other verb computes, so
  the agent could not re-call `frame --at` and reproduce the tile.
- A run with no paintable frame gets **no tile and is named** with a reason.
- Uniform infill is a **flag, never a default**, and its tiles must carry a different
  label from document-derived ones.
- The disclosure must be a **structured field**, not prose alone.
- Easing midpoints earn nothing — 3–0, derived rather than authored.

## The one split: keyframes by default

Jurors 1 and 2 put keyframe endpoints in the default boundary set; Juror 3 put them
behind a flag. Resolved in ADR-0094 **for Juror 3**, on the ground that the
disclosure all three demanded dissolves Juror 1's objection: a named, counted
omission with a flag attached is not the silence that objection is about. See the
ADR's *"The keyframe split, and why the disclosure decides it"*.

## What the panel did not have

The brief summarised ADR-0074 but omitted its boundary table (47 / 19 / 28) and its
finding that a visual-only cut list *"reports an interval it believes is constant,
and is wrong about, with nothing in the output to suggest otherwise."* That passage
is load-bearing in ADR-0094 and no juror saw it. Juror 1 came closest independently,
by insisting the filtered-out audio be named.
