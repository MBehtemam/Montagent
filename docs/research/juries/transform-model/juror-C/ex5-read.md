Read test on my own ex1 JSON, t=6000, by eye:
- lower-third: start 5000 <= 6000 < 9000 -> on screen. Local t = 6000-5000 = 1000.
  Last keyframe of x and of opacity is t=400; 1000 > 400 -> both hold final values:
  x=80 (origin bottom-left, y=1700), opacity=1.0.
- Cost: one subtraction (6000-5000), done in my head, then two "past the last
  keyframe?" comparisons. Maybe 10 seconds. The subtraction is the price of
  relative times; the operands sit on the same line ("start":5000 is 40 chars away).
- Presence ("is it on screen") needed no arithmetic at all — that read stays free
  either way, and it is the read ADR-0001/0005 actually defend.
- Honest failure: if t had been 5200 (mid-ramp) the answer is an interpolation —
  x = -420 + 500*(200/400) = -170 — that is arithmetic under EITHER time base.
  Absolute times only save the one subtraction, never the interpolation.
