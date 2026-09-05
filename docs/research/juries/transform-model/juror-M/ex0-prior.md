# Exercise 0 — unprompted prior

Task: element sits still 1s, then over 400ms slides x=-400 -> x=100 with ease-out feel,
then over 600ms continues x=100 -> x=540 with linear feel.

My unprompted answer:

```json
[
  {"t": 0,    "v": -400, "ease": "hold"},
  {"t": 1000, "v": -400, "ease": "ease-out"},
  {"t": 1400, "v": 100,  "ease": "linear"},
  {"t": 2000, "v": 540,  "ease": "linear"}
]
```

(I hesitated on the first record's ease — wrote "hold" as a placeholder for "nothing
happens until the next keyframe," then moved on without fully deciding if a no-op ease
name is even legal. Not revising it now.)

## Which convention did I use, and where did I put each ease?

- `ease-out` is on the keyframe AT t=1000 (the start of the moving segment, value still
  -400).
- `linear` is on the keyframe AT t=1400 (the start of the second moving segment, value
  100).
- The final keyframe (t=2000, v=540) also got "linear" — on reflection this was me
  copying the previous record's value out of uncertainty about what the last record's
  ease field should hold, not a deliberate choice. If forced to leave it blank/null I
  would have.

Intent: I meant "ease-out" to describe the segment LEAVING t=1000 and arriving at t=1400.
I meant "linear" (on the t=1400 record) to describe the segment LEAVING t=1400 and
arriving at t=2000.

So: unprompted, I used the **LEAVING** convention (ease on keyframe K governs the segment
departing K), matching CSS/Web Animations. I did this without consciously choosing it —
it just felt natural to attach the "how it moves next" adjective to the keyframe where
that motion starts. I was not confident what belongs on the very first keyframe (before
any motion starts) or the very last one (after all motion ends) — I defaulted to
duplicating the last real ease onto the terminal keyframe rather than leaving it empty,
which suggests my instinct doesn't crisply expect a keyframe to be "unable to carry an
ease."
