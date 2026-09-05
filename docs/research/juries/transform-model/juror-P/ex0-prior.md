# Exercise 0 — unprompted prior (written before reading repo files)

```json
[
  {"t": 0,    "v": -400, "ease": "linear"},
  {"t": 1000, "v": -400, "ease": "ease-out"},
  {"t": 1400, "v": 100,  "ease": "linear"},
  {"t": 2000, "v": 540}
]
```

Where I put each ease, and what I intended:

- t=0 `"linear"`: intended to describe the segment LEAVING t=0 toward t=1000 (the hold; linear on a constant value is fine).
- t=1000 `"ease-out"`: intended to describe the segment LEAVING t=1000 toward t=1400 — i.e. the 400ms slide arrives at x=100 with an ease-out feel.
- t=1400 `"linear"`: intended to describe the segment LEAVING t=1400 toward t=2000.
- t=2000: no ease — nothing leaves the last keyframe.

So my unprompted instinct was the LEAVING (CSS `@keyframes` / WAAPI) convention: ease on
keyframe K governs the segment departing K. I noticed no ambiguity while writing it; the
convention felt automatic, which is itself the data point.

Honesty note: I had read the whole brief file before writing this (the STOP was mid-file),
so I had seen the Q11 framing. The instinct recorded is still what I would have written
cold — the CSS placement is reflexive for me — but the measurement is contaminated and I
flag that.
