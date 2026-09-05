# Exercise 0 — juror L, unprompted prior (written before reading anything else)

## The keyframe list I wrote from instinct

```json
"x": [
  {"t": 0,    "v": -400, "ease": "linear"},
  {"t": 1000, "v": -400, "ease": "linear"},
  {"t": 1400, "v": 100,  "ease": "ease-out"},
  {"t": 2000, "v": 540,  "ease": "linear"}
]
```

## Where I put each ease and what I meant by it

- t=0 / "linear": filler. The first record has no segment before it, so nothing to
  describe. I wrote "linear" only because the shape `{"t","v","ease"}` looked like it
  wanted the field filled. This is itself a tell: the first record was the one with
  nothing to say.
- t=1000 / "linear": the hold from 0→1000 ms. The segment ARRIVING at t=1000.
- t=1400 / "ease-out": the 400 ms slide from -400 to 100. The segment ARRIVING at t=1400.
- t=2000 / "linear": the 600 ms continue from 100 to 540. The segment ARRIVING at t=2000.

## Which convention did I use, unprompted?

ENTERING. Every ease I wrote describes the segment arriving at the keyframe it sits on.
I did not notice I was choosing; I read the brief's prose left-to-right ("over 400 ms
slides ... with an ease-out feel") and attached the feel to the endpoint I had just
computed. The ease and the value it eases toward were written in the same breath, on the
same line.

Note against my own answer: I am saturated in CSS, where `animation-timing-function`
inside `@keyframes` is LEAVING. My instinct still came out ENTERING. The pull was the
authoring act, not the CSS memory — I named the destination and its feel together.
