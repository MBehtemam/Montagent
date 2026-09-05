# Jury I Exercises

## Exercise 1: Stress SPLIT where it is hardest

The three readings from round 1:
- **MOVE**: add delta to every keyframe time >= `at`
- **HOLD**: leave keyframes untouched
- **SPLIT**: evaluate property at `at`, insert keyframes at both `at` and `at+delta` with that value, then add delta to times >= `at`

Example fixture: `photo-06`, start 17472, end 30603, scale [[17472,1.0],[32472,1.08]]. shift(at=20000, delta=2000)

### Case (a): Non-linear easing

Taking photo-06's scale ramp. First, I need to understand: the ramp goes from 17472 to 32472 (15000 ms duration) from 1.0 to 1.08.
If this used CSS ease-in-out (which is NOT currently in the prototype, but let me use it as the test case for non-linear):
- The ramp progresses non-linearly over time
- At t=20000, we're 2528 ms into the 15000 ms ramp, which is 16.85% through
- Under cubic ease-in-out, at 16.85%, the value is NOT linearly interpolated

Let me compute the actual interpolated value using de Casteljau. CSS ease-in-out has control points:
P0=(0,0), P1=(0.42,0), P2=(0.58,1), P3=(1,1)

At t = 20000 - 17472 = 2528 ms, as fraction of 15000 ms: f = 2528/15000 = 0.1685333

For cubic Bezier with parameter f:
B(f) = (1-f)³*P0 + 3(1-f)²f*P1 + 3(1-f)f²*P2 + f³*P3

For y-component (with P0.y=0, P1.y=0, P2.y=1, P3.y=1):
y = 3(1-f)²f*0 + 3(1-f)f²*1 + f³*1 = 3(1-f)f² + f³

y = 3(1-0.1685)*(0.1685)² + (0.1685)³
y = 3*0.8315*0.02840 + 0.00479
y = 0.0708 + 0.00479
y = 0.0756

So the interpolated value at t=20000 would be: 1.0 + 0.0756 * (1.08 - 1.0) = 1.0 + 0.0756 * 0.08 = 1.00605

Three readings on this:

**MOVE**: Add 2000 to keyframe times
- [[17472, 1.0], [34472, 1.08]]
- Duration now 17000 ms instead of 15000 ms
- The ramp rate changes: scale changes by 0.08 over 17000 ms instead of 15000 ms
- At t=20000 (which is now 2528 ms into the ramp, but the ramp is now longer), the evaluated value would be different

Actually wait, let me reconsider. If we move the keyframe from 32472 to 34472, and the ease-in-out is still applied to the same duration... No, that's not how it works. The ease function is tied to the keyframe times themselves as the domain.

Actually, the question is: if we have a cubic Bezier ease from 17472 to 32472, how does that ease get reapplied if we change the end time to 34472?

The ease function describes how the property progresses over the interval [17472, 32472]. If we move it to [17472, 34472], the ease function stretches over a longer time period.

At t=20000:
- Original: f = (20000-17472)/(32472-17472) = 2528/15000 = 0.1685
- New: f = (20000-17472)/(34472-17472) = 2528/17000 = 0.1487

New y at f=0.1487: y = 3(1-0.1487)*(0.1487)² + (0.1487)³ = 3*0.8513*0.0221 + 0.00328 = 0.0565 + 0.00328 = 0.0598

Interpolated value = 1.0 + 0.0598 * 0.08 = 1.00478

So under MOVE: at t=20000, scale is 1.00478 (zoom rate changes silently)

**HOLD**: Leave keyframes untouched
- [[17472, 1.0], [32472, 1.08]]
- Element now runs 17472 to 32603
- At t=20000, still in the ramp, value is 1.00605 (the ease-in-out value at 16.85%)
- At t=32472, ramp ends with scale 1.08
- From 32472 to 32603, scale is frozen at 1.08

**SPLIT**: Evaluate at t=20000 and insert keyframes
- Value at f=0.1685: 1.00605 (computed above)
- New keyframes: [[17472, 1.0], [20000, 1.00605], [22000, 1.00605], [34472, 1.08]]
- At t=22000 (20000 + delta), the scale is 1.00605 (holding)
- Then it ramps from 22000 to 34472

With SPLIT, the second keyframe at 22000 "holds" the interpolated value. But then we have a discontinuity: from 1.00605 to ramping to 1.08. That doesn't seem right either...

Actually, I think SPLIT is more sophisticated. After we insert the frozen values at `at` and `at+delta`, we might need to also adjust the property of the next keyframe so the ramp continues smoothly.

Let me reconsider SPLIT. The purpose is to "freeze" the value at the split point and then continue the animation on the other side. But with easing, "freezing" means: stop easing at that point, hold the value, then resume the easing from a "shifted" perspective.

Actually, I think the issue is that with an eased segment and SPLIT, we'd need to:
1. Evaluate the value at `at` under the ease: v = 1.00605
2. Insert [[at, v], [at+delta, v]]
3. For the remaining keyframe [32472, 1.08], we'd need to... what? Continue with the same ease?

If we keep ease-in-out on the segment [22000, 34472], that's a 12000 ms segment.
At t=20000 it was 16.85% through the original 15000 ms ramp.
At t=22000 in the new ramp [22000, 34472], we're 0% through.
But the value should be 1.00605, and then it should ramp to 1.08.

If we use the same ease-in-out function on [22000, 34472]:
- At f=0: value = 1.0 + 0*0.08 = 1.0
- At f=1: value = 1.08

But we need it to start at 1.00605. So we can't reuse the ease-in-out function.

This is where the problem becomes clear: **a subdivided eased segment cannot express its continuation in the same named ease vocabulary**. The new segment [22000, 34472] would need a cubic Bezier that:
- Starts at 1.00605 (not 1.0)
- Ends at 1.08
- Has the "right" progression

The progression would be: starting at progress 16.85% of the ease-in-out curve, continue to 100%.

So the intermediate ease would need control points adjusted to start from the midpoint and go to the end. This is NOT a standard named ease.

**Conclusion on (a)**: SPLIT cannot express its result in named eases for non-linearly eased segments.

### Case (b): `at` falls exactly on an existing keyframe

Let's say we shift at t=32472 (the end of the first keyframe).

Original: [[17472, 1.0], [32472, 1.08]]

**MOVE**: 
- [[17472, 1.0], [34472, 1.08]]
- The keyframe at 32472 doesn't exist anymore, it's moved to 34472
- Actually wait, if `at=32472` and we add delta=2000, we move "every time at or after `at`"
- The keyframe at 32472 is >= 32472, so it moves to 34472
- Result: [[17472, 1.0], [34472, 1.08]]

**HOLD**:
- [[17472, 1.0], [32472, 1.08]]
- Nothing changes
- Element runs 17472 to 32603, ramp ends at 32472, frozen at 1.08 for last 131 ms

**SPLIT**:
- Value at t=32472 under linear interpolation: 1.08 (it's right on the keyframe)
- Insert [[17472, 1.0], [32472, 1.08], [34472, 1.08], [34472, 1.08]]
- Wait, that's two identical keyframes. Then we move times >= 32472:
- [[17472, 1.0], [32472, 1.08], [34472, 1.08], [34472, 1.08]]
- The last [34472, 1.08] is the original [32472, 1.08] shifted by 2000
- But we have duplicate keyframes at 34472

Actually, I think I'm misunderstanding SPLIT. Let me re-read: "evaluate the property at `at`, insert keyframes at `at` and `at+delta` both carrying that value, then add delta to every keyframe time >= `at`"

So:
1. Evaluate value at at
2. Insert keyframes at both at and at+delta with that value
3. Add delta to times >= at

So for our case:
1. Value at 32472: 1.08
2. Insert at 32472: we already have a keyframe there
3. Insert at 34472: new keyframe with value 1.08
4. Add delta=2000 to times >= 32472: the keyframe [32472, 1.08] becomes [34472, 1.08]

Result: [[17472, 1.0], [34472, 1.08], [34472, 1.08]]

We have duplicate keyframes. This is odd. In a sense, SPLIT produces redundant data when applied exactly at a keyframe.

### Case (c): Element lying entirely after `at`

photo-07: start 30603, end 42763, scale [[30603, 1.0], [45603, 1.08]]
shift at 20000 (before the element starts)

**MOVE**:
- [[30603, 1.0], [45603, 1.08]]
- Both keyframes are >= 20000, so both move
- [[32603, 1.0], [47603, 1.08]]
- The element doesn't move (its start/end are moved by shift), but relative to the new timeline, the keyframes are offset

**HOLD**:
- [[30603, 1.0], [45603, 1.08]]
- Nothing changes
- But the element moved from [30603, 42763] to [32603, 44763]
- Now the last keyframe at 45603 is outside the element entirely (12840 ms past the end)
- The element freezes at the last keyframe value (1.08) for its entire duration

Wait, that's not quite right either. If the element's end is 44763, and the keyframe is at 45603, then:
- The element runs from 32603 to 44763
- The keyframe at 45603 is past the element's end
- The ramp from 30603 to 45603 gets clipped to 30603 to 44763... no wait

Let me reconsider. The element's keyframes are in absolute timeline time. If the element runs from 30603 to 42763, and keyframes are at [30603, 1.0] and [45603, 1.08]:
- The first keyframe is at the start, value 1.0
- The second keyframe is past the end
- So the animation progresses from 1.0 at the start toward 1.08, but the element ends before reaching it

The animation ramps from 30603 to 45603 (15000 ms), but the element only exists from 30603 to 42763 (12160 ms). At 42763:
- f = (42763 - 30603) / (45603 - 30603) = 12160 / 15000 = 0.8107
- Under linear interpolation: 1.0 + 0.8107 * 0.08 = 1.06486

So at the element's end, scale is 1.06486. It's a trimmed move.

After shift: element becomes [32603, 44763]
Keyframes remain [[30603, 1.0], [45603, 1.08]] under HOLD

But now the element's start (32603) is AFTER the first keyframe (30603). This is the "element with a keyframe before its start" case. The element starts partway through the ramp.

At element start 32603:
- f = (32603 - 30603) / (45603 - 30603) = 2000 / 15000 = 0.1333
- Value = 1.0 + 0.1333 * 0.08 = 1.01067

At element end 44763:
- f = (44763 - 30603) / (45603 - 30603) = 14160 / 15000 = 0.944
- Value = 1.0 + 0.944 * 0.08 = 1.07552

So HOLD creates a case where the element picks up partway through its ramp.

**MOVE**: 
- Keyframes become [[32603, 1.0], [47603, 1.08]]
- Element is [32603, 44763]
- The first keyframe is right at the start, value 1.0 ✓
- The second keyframe at 47603 is past the end
- At element end 44763: f = (44763 - 32603) / (47603 - 32603) = 12160 / 15000 = 0.8107
- Value = 1.0 + 0.8107 * 0.08 = 1.06486

Same trimmed-move value as before shift.

**SPLIT**:
- Value at at=20000... but 20000 is before the element even starts
- Hmm, that's interesting. The element doesn't exist at 20000
- SPLIT would evaluate the keyframe ramp at a time before the ramp even starts
- At t=20000, this element's scale is undefined (element doesn't exist)
- So SPLIT doesn't really make sense here

In this case, SPLIT breaks because the shift point is outside the element.

### Case (d): Keyframe past element's end (trimmed move)

This is actually already exemplified by photo-05-loop in the fixture: start 64016, end 65216, scale [[64016, 1.0], [79016, 1.08]]
The second keyframe is 13800 ms past the project end.

shift at 62000 (before the element), delta 3000, so element becomes [67016, 68216] (I'm shifting the whole thing by 3000)

Wait, let me re-read the brief. The fixture shift is: shift(at=20000, delta=2000) on photo-06 with start 17472, end 30603.

Let me use photo-05-quiz as the example for case (d):
start 53856, end 64016, scale [[53856, 1.0], [68856, 1.08]]

shift at 62000, delta 3000
Element becomes [56856, 67016] (moved by 3000)
Element's keyframes in case (d):

**MOVE**:
- [[53856, 1.0], [68856, 1.08]]
- Both >= 62000? Only the second one (68856 >= 62000)
- First keyframe at 53856 doesn't move (< at)
- But wait, the element moved. So the keyframes are absolute timeline times
- First keyframe 53856 is now BEFORE the element's start 56856
- [[53856, 1.0], [71856, 1.08]] (68856 + 3000)
- Element is [56856, 67016]
- Keyframe 1 is 3000 ms before element start
- At element start 56856: f = (56856 - 53856) / (71856 - 53856) = 3000 / 18000 = 0.1667
- Value = 1.0 + 0.1667 * 0.08 = 1.01333
- Keyframe 2 at 71856 is 4840 ms past element end 67016
- At element end: f = (67016 - 53856) / (71856 - 53856) = 13160 / 18000 = 0.7311
- Value = 1.0 + 0.7311 * 0.08 = 1.05849

**HOLD**:
- [[53856, 1.0], [68856, 1.08]]
- Element is [56856, 67016]
- Keyframe 1 at 53856 is 3000 ms before element start
- At element start: f = (56856 - 53856) / (68856 - 53856) = 3000 / 15000 = 0.2
- Value = 1.0 + 0.2 * 0.08 = 1.016
- Keyframe 2 at 68856 is 1840 ms past element end 67016
- At element end: f = (67016 - 53856) / (68856 - 53856) = 13160 / 15000 = 0.8773
- Value = 1.0 + 0.8773 * 0.08 = 1.07018

**SPLIT**:
- Value at at=62000: f = (62000 - 53856) / (68856 - 53856) = 8144 / 15000 = 0.5429
- Value = 1.0 + 0.5429 * 0.08 = 1.04343
- Insert at 62000 and 65000 (62000 + 3000): [[53856, 1.0], [62000, 1.04343], [65000, 1.04343], [71856, 1.08]]
- Element is [56856, 67016]
- At element start 56856: f = (56856 - 53856) / (71856 - 53856) = 3000 / 18000 = 0.1667
- But we have keyframes now at intermediate values, so the interpolation changes
- From 56856 to 62000, we interpolate from 1.0 to 1.04343 over 5144 ms
- At 56856: f = (56856 - 53856) / (62000 - 53856) = 3000 / 8144 = 0.3684
- Value = 1.0 + 0.3684 * 0.04343 = 1.01600
- From 62000 to 65000, value is frozen at 1.04343
- From 65000 to 67016, we interpolate from 1.04343 to 1.08
- At 67016: f = (67016 - 65000) / (71856 - 65000) = 2016 / 6856 = 0.294
- Value = 1.04343 + 0.294 * 0.03657 = 1.05412

So SPLIT on a trimmed move produces intermediate steps and changes the interpolation curve.

All three readings are legal for case (d), they just produce different results.

## Exercise 2: Subdividing an eased segment

CSS ease-in-out cubic Bezier: (0.42, 0) (0.58, 1)

Split at f=0.5 (midpoint of time, not value)

First half: [0, 0.5] in time parameter
Use de Casteljau to find the control points that describe the curve from 0 to 0.5

P0 = (0, 0)
P1 = (0.42, 0)
P2 = (0.58, 1)
P3 = (1, 1)

At t=0.5:

First iteration (lerp between adjacent points at t=0.5):
L0 = (1-0.5)*P0 + 0.5*P1 = 0.5*(0,0) + 0.5*(0.42,0) = (0.21, 0)
L1 = (1-0.5)*P1 + 0.5*P2 = 0.5*(0.42,0) + 0.5*(0.58,1) = (0.5, 0.5)
L2 = (1-0.5)*P2 + 0.5*P3 = 0.5*(0.58,1) + 0.5*(1,1) = (0.79, 1)

Second iteration:
M0 = 0.5*L0 + 0.5*L1 = 0.5*(0.21,0) + 0.5*(0.5,0.5) = (0.355, 0.25)
M1 = 0.5*L1 + 0.5*L2 = 0.5*(0.5,0.5) + 0.5*(0.79,1) = (0.645, 0.75)

Third iteration:
N = 0.5*M0 + 0.5*M1 = 0.5*(0.355,0.25) + 0.5*(0.645,0.75) = (0.5, 0.5)

So the curve passes through (0.5, 0.5) at parameter t=0.5.

The first half of the curve (from t=0 to t=0.5) has:
- Start: P0 = (0, 0)
- Control point 1: L0 = (0.21, 0)
- Control point 2: M0 = (0.355, 0.25)
- End: N = (0.5, 0.5)

So the first half would be described by cubic Bezier:
Start: (0, 0)
C1: (0.21, 0)
C2: (0.355, 0.25)
End: (0.5, 0.5)

Normalized back to [0,1] output range: this curve goes from y=0 to y=0.5, so to get the full [0,1] range, we'd need to scale the y-values by 2:

Start: (0, 0)
C1: (0.21, 0)
C2: (0.355, 0.5)  
End: (0.5, 1)

Let me check if this matches any named CSS easing...

Named eases in CSS:
- linear
- ease: (0.25, 0.1) (0.25, 1)
- ease-in: (0.42, 0) (1, 1)
- ease-out: (0, 0) (0.58, 1)
- ease-in-out: (0.42, 0) (0.58, 1)

Our first half normalized: (0.21, 0) (0.355, 0.5) - doesn't match any standard

The second half: from N to P3 with control M1 and L2:
Start: N = (0.5, 0.5)
C1: M1 = (0.645, 0.75)
C2: L2 = (0.79, 1)
End: P3 = (1, 1)

Normalized to [0, 1] output:
Start: (0.5, 0)
C1: (0.645, 0.5)
C2: (0.79, 1)
End: (1, 1)

Doesn't match standard either.

**Conclusion for Exercise 2**: A subdivided ease-in-out segment at its midpoint cannot be expressed in named eases. Both halves require custom Bezier curves.

## Exercise 3: The seven Ken Burns scale lists - both notations

Looking at the fixture's seven photo elements with scale:

photo-05-intro: [[0,1.0],[15000,1.08]]
photo-05: [[3018,1.0],[18018,1.08]]
photo-06: [[17472,1.0],[32472,1.08]]
photo-07: [[30603,1.0],[45603,1.08]]
photo-08: [[42763,1.0],[57763,1.08]]
photo-05-quiz: [[53856,1.0],[68856,1.08]]
photo-05-loop: [[64016,1.0],[79016,1.08]]

Currently all use array-of-arrays form. Scalar `1.08` would be: could we represent constant scale?

Actually wait. The task says write them "both ways" as `"v":1.08` and `"v":[1.08,1.08]`.

I think the question is about representing a scalar constant value vs. an array for keyframes.

If an element has no animation, just a constant scale, we could write:
- `"scale": 1.08` (scalar)
- OR `"scale": [1.08, 1.08]` (array with start and end equal)

Let me interpret the exercise as: write the seven Ken Burns as constant scales (before and after), showing both notations.

Actually, re-reading, I think the exercise is asking: show how the seven keyframe arrays would be written in two different formats. One format is the current one `[[t1,v1],[t2,v2]]`, the other might be... hmm, `[{t:t1,v:v1},{t:t2,v:v2}]`? Or scalar vs array?

Looking at ADR-0011's settled decisions, keyframes are "Keyframe records are objects — `{"t":…, "v":…, "ease":…}` — not positional pairs."

So maybe the two ways are:
1. Current form (positional arrays): [[t,v],[t,v]]
2. Object form (as specified): [{t,v},{t,v}]

Let me write both:

**Positional pairs** (NOT the settled form):
```json
{"id":"photo-05-intro","scale":[[0,1.0],[15000,1.08]]}
{"id":"photo-05","scale":[[3018,1.0],[18018,1.08]]}
{"id":"photo-06","scale":[[17472,1.0],[32472,1.08]]}
{"id":"photo-07","scale":[[30603,1.0],[45603,1.08]]}
{"id":"photo-08","scale":[[42763,1.0],[57763,1.08]]}
{"id":"photo-05-quiz","scale":[[53856,1.0],[68856,1.08]]}
{"id":"photo-05-loop","scale":[[64016,1.0],[79016,1.08]]}
```

**Object form** (the settled form per round 1):
```json
{"id":"photo-05-intro","scale":[{"t":0,"v":1.0},{"t":15000,"v":1.08}]}
{"id":"photo-05","scale":[{"t":3018,"v":1.0},{"t":18018,"v":1.08}]}
{"id":"photo-06","scale":[{"t":17472,"v":1.0},{"t":32472,"v":1.08}]}
{"id":"photo-07","scale":[{"t":30603,"v":1.0},{"t":45603,"v":1.08}]}
{"id":"photo-08","scale":[{"t":42763,"v":1.0},{"t":57763,"v":1.08}]}
{"id":"photo-05-quiz","scale":[{"t":53856,"v":1.0},{"t":68856,"v":1.08}]}
{"id":"photo-05-loop","scale":[{"t":64016,"v":1.0},{"t":79016,"v":1.08}]}
```

The cost:
- Positional: more compact, but harder to extend (adding `ease` field requires changing the array structure)
- Object: explicit field names, clearer what each value means, allows future extensions

The "fmt normalizing a scalar on write" question: if some elements have `scale: 1.0` (no animation) vs `scale: [{"t":start,"v":1.0}]`, a normalizing writer could ensure consistency. But is that a fix or a dodge?

I think it's a dodge if the format allows both scalar and keyframe-array forms. True fix would be to pick one and require it everywhere.

Reading the object form is straightforward - each keyframe explicitly states its time and value.

## Exercise 4: Rewrite photo-06, card-05, and text element without box/fit/align

Current photo-06:
```json
{"id":"photo-06","type":"image","group":"item-06","start":17472,"end":30603,"source":"images/06.png","box":[0,0,1080,1300],"fit":"cover","align":"top","scale":[[17472,1.0],[32472,1.08]]}
```

This says:
- Image 06.png, which is presumably some larger size (not stated)
- Place in a box [0,0,1080,1300] at the frame's top
- Use "cover" fit with "top" alignment for Ken Burns
- Scale animates 1.0 to 1.08

In round-1 model (no box/fit/align):
- Image has width/height properties? (not in the model)
- Or: we must specify final x,y,width,height directly?

Actually, looking at the brief it says "express all three in the round-1 model. Does anything become inexpressible or worse?"

The round-1 model has: `x`, `y`, `origin`, and a size. It has `scale` (already present). 

So we need to express the image's final position and size in the round-1 terms.

If the box is [0,0,1080,1300] and the image is 1536x2720, and fit="cover" with align="top":
- The box is 1080x1300
- The image is 1536x2720 (aspect ratio 1536/2720 = 0.564)
- The box is 1080x1300 (aspect ratio 1080/1300 = 0.831)
- Cover means: scale to cover the box, maintaining aspect ratio
- If we scale the image to cover 1080x1300:
  - Scale by width: 1080 / 1536 = 0.703
  - Scale by height: 1300 / 2720 = 0.478
  - Take max: 0.703
  - Actual size: 1536 * 0.703 = 1080, 2720 * 0.703 = 1912
  - So the image becomes 1080x1912
  - Crop: box is 1300 tall, image is 1912 tall, center at "top" → crop from 0 to 1300, align top
  - Visible region: top 1300 of the 1912

But in the round-1 model, there's no "crop" or "fit" concept. We just have x, y, scale, opacity.

So photo-06 in round-1 would be:
```json
{"id":"photo-06","type":"image","group":"item-06","start":17472,"end":30603,"source":"images/06.png","x":540,"y":650,"origin":"top-center","scale":[{"t":17472,"v":1.0},{t":32472,"v":1.08}]}
```

Where:
- x=540 is the frame center (1080/2)
- y=650 is the vertical center of the box [0,0,1080,1300] → 0 + 1300/2 = 650
- origin="top-center" so the center of the image is placed at (540, 650)
- scale evolves from 1.0 to 1.08

Actually wait. In the current model, fit="cover" is doing the scaling. But in round-1, scale is applied to... what? The original source size? Or a default size?

I think scale in round-1 applies to the element's size relative to some default. If the image has no intrinsic size in the format, we need to establish one.

The question becomes: does anything become inexpressible?

Yes - we lose the ability to specify "fit inside this box". We're now directly specifying x, y, and scale, which is more explicit but requires hand-calculation of the right scale value.

Current card-05:
```json
{"id":"card-05","type":"rect","group":"item-05","start":10468,"end":17472,"box":[48,1453,984,169],"fill":"#1E344C"}
```

In round-1:
```json
{"id":"card-05","type":"rect","group":"item-05","start":10468,"end":17472,"x":540,"y":1537.5,"width":984,"height":169,"origin":"center","fill":"#1E344C"}
```

Where:
- x = 48 + 984/2 = 540 (center of the box)
- y = 1453 + 169/2 = 1537.5 (center of the box)
- width and height from box dimensions
- origin="center" so the rectangle is centered at that point

Text element with box reference (from the proto, word-05):
```json
{"id":"word-05","type":"text","group":"item-05","start":5316,"end":17472,"text":"cobweb  -  cobweb","font":"SF Pro Rounded","weight":"bold","size":88,"color":"#245C8C","x":540,"y":1373,"align":"center","line_height":1.1}
```

It doesn't have an explicit box field, but ADR-0007 says text elements have a box field. Let me check card-05's box again: [48,1453,984,169].

If word-05's box is also [48,1453,984,169], then in round-1:
```json
{"id":"word-05","type":"text","group":"item-05","start":5316,"end":17472,"text":"cobweb  -  cobweb","font":"SF Pro Rounded","size":88,"color":"#245C8C","x":540,"y":1537,"origin":"center","align":"center","line_height":1.1}
```

Where the x and y are derived from the box (same as the card).

The "weight" field is rejected in round-1 (the font file specifies weight).

Does anything become inexpressible? 

The "fit" and "align" fields provided image scaling and alignment directives. In round-1, we specify the exact x, y, and scale, which is more explicit but requires computation.

The text "box" field in ADR-0007 is required for validation (checking overflow), but in round-1 we might not have it as a separate field if we just use x, y, width, height (for rects) or origin + sizing derived from content.

## Exercise 5: Author bare elements

A bare element with only type, source, start, end:

```json
{"type":"image","source":"images/05.png","start":0,"end":3000}
```

Where does it render?
- x, y defaults: probably center of frame? But we don't have defaults specified
- size: intrinsic? No, there's no size field
- opacity: 1.0 (full opacity)

Without defaults, this element is inexpressible. We need at least x, y, and scale/size.

A deliberately hard-to-find element:
```json
{"type":"image","source":"images/05.png","start":64015,"end":65216,"x":540,"y":960,"origin":"center","scale":1.0,"opacity":0.01}
```

This element:
- Appears only in the last ~1.2 seconds (the loop tail)
- Rendered at very low opacity (1% visible)
- Centered on frame, normal size
- Hard to find because: opacity is nearly invisible, starts near the project end

Default that would save us: opacity=1.0 by default would make it visible.

