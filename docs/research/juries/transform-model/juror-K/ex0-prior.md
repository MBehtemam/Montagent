# Exercise 0 — juror K, written from instinct before reading anything else

My keyframe list (first thing I wrote, unrevised):

```json
[
  {"t": 0,    "v": -400, "ease": "linear"},
  {"t": 1000, "v": -400, "ease": "hold"},
  {"t": 1400, "v": 100,  "ease": "ease-out"},
  {"t": 2000, "v": 540,  "ease": "linear"}
]
```

Where I put each ease, and what I meant by it:

- t=0, "linear": placeholder. I had nothing arriving at t=0 and nothing I cared about
  leaving it. I wrote it because the shape has an `ease` slot and leaving it out felt
  incomplete. This is the tell that my model is confused at the boundary.
- t=1000, "hold": I meant the ONE SECOND OF SITTING STILL, i.e. the segment from t=0
  to t=1000 — the segment ARRIVING at this keyframe.
- t=1400, "ease-out": I meant the 400 ms slide from -400 to 100 — the segment from
  t=1000 to t=1400, i.e. the segment ARRIVING at this keyframe.
- t=2000, "linear": I meant the 600 ms continuation to 540 — the segment from t=1400
  to t=2000, i.e. the segment ARRIVING at this keyframe.

So: my unprompted convention was ENTERING. Every meaningful ease I wrote describes the
segment that terminates at the keyframe carrying it.

Was it ambiguous? Partly, and I am recording the ambiguity honestly:
- The t=0 record is junk under my own convention (nothing arrives at t=0). I still
  wrote an `ease` on it, because the record shape offered the field. Under a LEAVING
  convention that t=0 "linear" would have been WRONG in a rendering sense: it would have
  made the still second linear-interpolate, which happens to look identical because the
  value does not change — so my error would have been silent even to me.
- The "hold" at t=1000 is the one place I hesitated. "Hold" reads to me as a verb about
  the future ("hold this value onward"), which is a LEAVING intuition, and it pulled
  against the ENTERING reading I used everywhere else. If I had written `"hold"` on the
  t=0 record instead I would have been using LEAVING for that one field. I noticed this
  pull only while writing this note down, not while writing the JSON.

Net: instinct = ENTERING, with a local LEAVING pull specifically on the word `hold`.
