# Exercise 0 — Prior Instinct

## My keyframe list (unprompted):

```json
[
  {"t": 0, "v": -400},
  {"t": 1000, "v": -400},
  {"t": 1400, "v": 100, "ease": "ease-out"},
  {"t": 2000, "v": 540, "ease": "linear"}
]
```

## Convention I used:

I put the `ease` on the keyframe that the segment ARRIVES AT (ENTERING convention).

- `ease-out` on the keyframe at t=1400 (the keyframe that the eased segment enters/arrives at)
- `linear` on the keyframe at t=2000 (the keyframe that the linear segment enters/arrives at)

I intended the ease to describe the segment that is ENTERING that keyframe — i.e., the segment being traveled to reach that keyframe.

The first static keyframe at t=1000 has no ease because it is not entered by motion; and the final keyframe at t=2000 logically could have no ease since there is no segment leaving it (though I put linear there out of habit).
