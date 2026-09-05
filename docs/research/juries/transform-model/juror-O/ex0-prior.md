# Exercise 0 — prior, written before reading past that point in the brief

Instinctive keyframe list for x:

```json
[
  {"t": 0,    "v": -400, "ease": "hold"},
  {"t": 1000, "v": -400, "ease": "ease-out"},
  {"t": 1400, "v": 100,  "ease": "linear"},
  {"t": 2000, "v": 540,  "ease": "linear"}
]
```

Where I put each ease, and what I intended:

- t=0: "hold" — intended to describe the segment LEAVING this keyframe (0→1000ms,
  sit still). Honestly, "hold" here is my way of saying the value doesn't change;
  with equal values a linear would also sit still, so this is belt-and-braces.
- t=1000: "ease-out" — intended to describe the segment LEAVING this keyframe
  (1000→1400ms slide from -400 to 100).
- t=1400: "linear" — intended to describe the segment LEAVING this keyframe
  (1400→2000ms, 100→540).
- t=2000: "linear" — a filler; under my leaving-convention this keyframe's ease
  describes nothing (there is no segment after it). I wrote it anyway because the
  record shape demands an "ease" field, which is itself a smell: the last record's
  ease is dead weight under my convention.

Verdict on my own instinct: I unambiguously used the OUTGOING/"leaving" convention
(CSS @keyframes style: the timing function on a keyframe applies from it to the
next keyframe). I was not tempted by the arriving convention while writing it, but
I note the last-keyframe dead field and the first-keyframe "hold" show the leaving
convention pushes meaning onto records whose ease is sometimes meaningless.
