# Exercise 0 — unprompted prior

Motion: hold at x=-400 for 1000ms, then 400ms ease-out to x=100, then 600ms linear to x=540.
Element starts at t=0.

My instinctive JSON:

```json
[
  {"t": 0,    "v": -400, "ease": "hold"},
  {"t": 1000, "v": -400, "ease": "ease-out"},
  {"t": 1400, "v": 100,  "ease": "linear"},
  {"t": 2000, "v": 540}
]
```

## Which keyframe did I put each ease on, and did I mean entering or leaving?

- `ease: "hold"` on t=0: describes the segment leaving t=0 (the still period from 0->1000).
- `ease: "ease-out"` on t=1000: I meant this to describe the segment LEAVING t=1000, i.e.
  the 1000->1400 slide that has the ease-out feel. I did NOT intend it to describe the
  segment arriving at 1000 (which is just the hold, and arguably shouldn't carry an
  interesting ease at all under an entering scheme).
- `ease: "linear"` on t=1400: again LEAVING — describes the 1400->2000 segment.
- Last keyframe (t=2000) carries no ease, because there's nothing after it to describe.
  This is exactly the shape the brief describes as the LEAVING/CSS convention (last
  keyframe can carry no ease).

So: unprompted, I used the LEAVING convention, modeled directly on CSS `@keyframes`
percentage-rule timing functions. I did not consider the ENTERING convention at all until
I saw it named in the brief — it did not occur to me as an alternative. This is recorded
as-is, without revision, even though it may not match my final verdict.
