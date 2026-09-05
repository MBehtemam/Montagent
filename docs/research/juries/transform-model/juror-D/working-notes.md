# Juror D Working Notes

## Exercise 1: Lower-third title card (slide in + fade)

**Requirements:** Title slides in from left over 400ms AND fades up while it moves, sitting over a photo with Ken Burns zoom.

Photo element (Ken Burns):
```json
{
  "id": "bg-photo",
  "type": "image",
  "start": 0,
  "end": 10000,
  "source": "images/photo.png",
  "box": [0, 0, 1080, 1920],
  "fit": "cover",
  "align": "center",
  "scale": [[0, 1.0], [10000, 1.1]]
}
```

Title card element (slides in + fades):
```json
{
  "id": "title-card",
  "type": "text",
  "start": 0,
  "end": 400,
  "text": "Welcome",
  "x": 540,
  "y": 1700,
  "size": 72,
  "color": "#FFFFFF",
  "font": "sans-serif",
  "origin": "center",
  "align": "center",
  "line_height": 1.2,
  "x": [[0, -200], [400, 540]],
  "opacity": [[0, 0], [400, 1]]
}
```

Wait, I'm seeing a problem. Looking at the sample project, text elements don't have `x` or `y` as keyframe arrays—they're single values. And `origin` isn't explicitly in the sample, but `align` is. The sample shows these are flat properties.

Let me reconsider. The brief says the sample project is a PROTOTYPE with invented fields. Let me look at what ADR-0007 actually says about text positioning:

From ADR-0007: `x`, `y`, `origin` (nine keywords), `align` (start/center/end for line alignment).

So text does have x, y, origin. But the sample project uses `align` as if it's the positioning anchor. ADR-0007 says `align` is line-alignment (start/center/end). So there's confusion here.

Looking at the quote from ADR-0007:
```json
{"id":"word-05","type":"text","start":5318,"end":17472,"x":540,"y":1373,"origin":"center","font":"brand","size":88,"line_height":1.1,"color":"#245C8C","box":"card-05","runs":[{"text":"cobweb  -  cobweb"}]}
```

Ah! This shows the CORRECT format per ADR-0007: `x`, `y`, `origin`, and then `runs` (not `text`).

The sample project file from the prototype uses an older/draft format with:
- `text` field (not `runs`)
- `weight` field (which ADR-0007 rejects—should be in font file)
- `align` field doing double-duty (positioning + line alignment)

So I need to be careful: the prototype is INVENTED and contradicts the settled ADRs.

Per the brief: "Fields in it marked INVENTED are guesses at decisions that have never been made. It is a prototype, not a spec, and you may contradict it freely."

So for my exercises, I should follow the ADRs, not the prototype's errors.

Let me reconsider exercise 1 knowing:
1. Text positioning uses `x`, `y`, `origin` (nine-way positioning point)
2. Keyframes exist on transform properties only
3. Properties can animate (keyframe arrays)
4. The specimen shows `scale` as `[[time, value], ...]`

For a title sliding in from left + fading:
- Position changes: x from -200 to 540
- Opacity changes: 0 to 1
- But wait: are x/y keyframeable? The brief says "keyframes on transform properties only"

Looking at ADR-0005 and the sample project: `scale` is keyframed. But `x`, `y`—are those transform properties?

Rereading #21: "Position, scale, rotation, anchor point, opacity — which exist, in what units..."

So position (x, y), scale, rotation, opacity are the transform properties. But the ADRs haven't settled WHICH properties are keyframeable or HOW keyframes are formatted.

The sample project shows scale as `[[3018, 1.0], [18018, 1.08]]`—absolute times.

Since the brief says "keyframes on transform properties only" and shows scale keyframing in the fixture, I should assume x, y are keyframeable as position transforms.

Let me draft the exercise response treating position and opacity as keyframeable:

Title (slides in from left + fades up):
- x: -200 → 540 (over 400ms)
- opacity: 0 → 1 (over 400ms)

But now a key question from #21 lands: **are keyframe times absolute or relative?**

Looking at the sample: scale keyframes use absolute times like `[[3018, 1.0], [18018, 1.08]]`. The element starts at 3018 and ends at 17472. So times are absolute (on the project clock).

But wait—that's peculiar. The element ends at 17472, but the keyframe goes to 18018 (past the element's end). ADR-0006 explicitly says this is legal: "Keyframes outside their element's range are legal — a trimmed move".

So: **keyframe times are absolute**, matching the project's one absolute clock per ADR-0005.

Now for the title card:

Photo (Ken Burns zoom):
```json
{
  "id": "bg-photo",
  "type": "image",
  "start": 0,
  "end": 10000,
  "source": "images/photo.png",
  "box": [0, 0, 1080, 1920],
  "fit": "cover",
  "align": "center",
  "scale": [[0, 1.0], [10000, 1.1]]
}
```

Title card (slides + fades):
```json
{
  "id": "title-card",
  "type": "text",
  "start": 0,
  "end": 400,
  "text": "Welcome",
  "x": [[-200, 0], [540, 400]],
  "y": 1700,
  "size": 72,
  "color": "#FFFFFF",
  "font": "default",
  "origin": "center",
  "align": "center",
  "line_height": 1.2,
  "opacity": [[0, 0], [400, 1]]
}
```

Hmm, but I'm inventing the format for `x` as a keyframe. The brief and ADRs don't show this. Let me look at what the specimen shows... it only shows `scale` as keyframed.

But the question is open (#21): which properties are keyframeable?

Since I'm supposed to be an agent authoring this format, and the exercises are designed to make me discover issues, let me try authoring what seems natural and then note the uncertainty.

Actually, re-reading the instructions more carefully: "Write the actual JSON for both elements."

So I need to give my best attempt. Given that:
- Scale is keyframed in the sample
- Position (x, y) are transform properties per #21
- Opacity is mentioned as a transform property per #21

I'll author it with x and opacity as keyframes, noting the uncertainty about the exact format.

Wait, let me look at the specimen again more carefully:

```json
{"id":"photo-05","type":"image","group":"item-05","start":3018,"end":17472,"source":"images/05.png","box":[0,0,1080,1300],"fit":"cover","align":"top","scale":[[3018,1.0],[18018,1.08]]}
```

Format is: `"scale": [[time1, value1], [time2, value2]]`

So a single animation property is an array of [time, value] pairs (no easing in the specimen).

For multiple properties animating:
```json
{
  "x": [[0, -200], [400, 540]],
  "opacity": [[0, 0], [400, 1]],
  "scale": [[0, 1.0], [400, 1.0]]
}
```

Let me draft my full answer for Exercise 1:

---

## Exercise 1 Answer

Photo with Ken Burns (slow zoom):
```json
{
  "id": "title-bg-photo",
  "type": "image",
  "start": 0,
  "end": 10000,
  "source": "images/photo.png",
  "box": [0, 0, 1080, 1920],
  "fit": "cover",
  "align": "center",
  "scale": [[0, 1.0], [10000, 1.15]]
}
```

Title card text (slides from left + fades):
```json
{
  "id": "title-card",
  "type": "text",
  "start": 0,
  "end": 400,
  "x": [[-540, 0], [540, 400]],
  "y": 300,
  "origin": "center",
  "align": "center",
  "text": "Welcome to the Show",
  "size": 72,
  "color": "#FFFFFF",
  "font": "sans-serif",
  "line_height": 1.2,
  "opacity": [[0, 0], [400, 1]]
}
```

**Issues identified:**
1. The format for position keyframes (x, y as arrays) is not shown in the specimen and is INVENTED here
2. The specimen only shows scale keyframes; position keyframes are implied by #21 but not formalized
3. Easing is absent from the specimen—only [time, value] pairs; this will be a design question

---

## Exercise 2: Rotating badge (3s then hold)

Badge that rotates continuously for 3 seconds then holds steady:

```json
{
  "id": "rotating-badge",
  "type": "image",
  "start": 0,
  "end": 3000,
  "source": "images/badge.png",
  "box": [900, 100, 150, 150],
  "fit": "contain",
  "align": "center",
  "rotation": [[0, 0], [3000, 360]]
}
```

Then continue holding:
```json
{
  "id": "rotating-badge-hold",
  "type": "image",
  "start": 3000,
  "end": 10000,
  "source": "images/badge.png",
  "box": [900, 100, 150, 150],
  "fit": "contain",
  "align": "center",
  "rotation": 360
}
```

**Issues identified:**
1. `rotation` property is assumed to exist and be keyframeable (per #21 scope)
2. Rotation units are degrees (assumed; also implied by 360 for full rotation)
3. Rotation is specified as absolute degrees, not radians
4. After rotation stops, I've modeled it as a static value (360) rather than a continued keyframe

Wait, should the second element have a rotation keyframe that holds, like `[[0, 360], [10000, 360]]`? Or just a static value?

The guard in #21 says the reference class is CapCut/Premiere, and I should know from those that once you stop rotating, you typically just hold the last value.

But in this format, if rotation is a keyframe array, how do you express "hold the current value"? Do you keep the keyframe and have it repeat at the same value? Or does a missing keyframe mean "no animation, use the property value as static"?

This is exactly the kind of detail the exercise is supposed to surface.

For now, I'll model it as: if a property is a static value (not an array), it doesn't animate. If it's an array, it does.

---

## Exercise 3: Shift operation

**Given:** photo-06 with `"scale": [[17472, 1.0], [32472, 1.08]]` running from 17472 to 30603.
**Task:** Insert 2000 ms at t=20000. What's the corrected JSON?

Analysis:
- photo-06 currently: start=17472, end=30603, duration=13131ms
- Shift point: t=20000 (this is within photo-06's range: 17472 < 20000 < 30603)
- Element is a time-invariant image, so per ADR-0005: "shift stretches the straddler's end by delta"

So:
- new start: 17472 (unchanged; shift is AFTER it starts... wait, 20000 is AFTER 17472 but BEFORE 30603)
- This element STRADDLES the shift point
- Per ADR-0005: time-invariant elements are STRETCHED by delta
- new end: 30603 + 2000 = 32603

Now for scale keyframes:
- Original: `[[17472, 1.0], [32472, 1.08]]`
- After shift: Both keyframe times must move because they're absolute times

Wait, let me re-read ADR-0005 on this:

> **Straddling elements are handled by type**, because a single rule is wrong for at least one kind:
> - **Time-invariant elements** (image, text, shape) have no internal clock. `shift` **stretches** the straddler's `end` by `delta`. This was the hand-fix two agents had to apply to close a three-second hole they had punched in the stills track.

So stretching means: the element extends by delta, but its content doesn't change duration internally.

For keyframes: "Photo-06" has scale that goes from 17472 to 32472. That's a 15000ms duration of animation. When the element is stretched by 2000ms:
- Element was 30603 - 17472 = 13131ms long
- Element becomes 30603 + 2000 - 17472 = 15131ms long
- The scale animation should... stay the same length? Or stretch?

Re-reading ADR-0005 more carefully:

> Stretching speech violates the source-range invariant — one agent's "obvious" uniform-stretch policy produced an element with an 8.1s timeline range against a 5.2s source, an invalid file.

So stretching the animation keyframes would be WRONG. The keyframes should stay as they are—the animation plays at normal speed but the element is held longer after the animation ends.

Hmm, but that means the keyframe at 32472 is now OUTSIDE the element's range (32472 > 32603)... wait, no: 32472 < 32603? Let me recalculate:

Old element: 17472-30603
New element: 17472-32603

Old keyframes: [[17472, 1.0], [32472, 1.08]]

The keyframe at 32472 is BEYOND the old end (32472 > 30603), which is already allowed per ADR-0006 ("Keyframes outside their element's range are legal").

After shift at t=20000 with delta=2000:
- Every time >= 20000 shifts forward by 2000
- Keyframe at 17472: unchanged (17472 < 20000)
- Keyframe at 32472: becomes 34472 (32472 >= 20000)

So the new keyframes should be:
`"scale": [[17472, 1.0], [34472, 1.08]]`

And the new element:
```json
{
  "id": "photo-06",
  "type": "image",
  "group": "item-06",
  "start": 17472,
  "end": 32603,
  "source": "images/06.png",
  "box": [0, 0, 1080, 1300],
  "fit": "cover",
  "align": "top",
  "scale": [[17472, 1.0], [34472, 1.08]]
}
```

Wait, but the brief says: "Produce the corrected JSON for `photo-06`, which carries `"scale": [[17472, 1.0], [32472, 1.08]]` and runs from 17472 to 30603. State exactly what the right answer is and why, and note where a plausible agent gets it wrong."

So the prompt is asking me to think through what goes wrong. Let me think about common mistakes:

**Mistake 1: Agent thinks the keyframe at 32472 is "after the shift point" so it shifts too**
- Shift at t=20000
- Keyframe at 32472 >= 20000, so shift it: 32472 + 2000 = 34472
- This is what I calculated above

**Mistake 2: Agent thinks stretch means "extend the animation duration proportionally"**
- Original animation: 32472 - 17472 = 15000ms of animation
- Original element: 30603 - 17472 = 13131ms
- Stretched element: 32603 - 17472 = 15131ms
- Scale animation duration should be: 15131 ms? → from 17472 to 17472+15131=32603?
- But that would make the keyframes [[17472, 1.0], [32603, 1.08]]
- This is WRONG per ADR-0005

**Mistake 3: Agent doesn't shift the keyframes at all**
- Thinks "the element moved, but keyframes are relative to the element start"
- Keyframes stay [[17472, 1.0], [32472, 1.08]]
- WRONG per ADR-0005 which says shift takes times on the absolute timeline

**The correct answer:**
- New element: start=17472, end=32603
- New scale keyframes: [[17472, 1.0], [34472, 1.08]]
- **Why:** Keyframe times are absolute. Shift moves every time >= 20000 by +2000. The second keyframe at 32472 >= 20000, so it becomes 34472. The element's end is stretched by +2000 per ADR-0005, but the keyframes shift on the absolute clock.

---

## Exercise 4: Retarget frame aspect ratio

**Given:** frame: {width: 1080, height: 1920} (portrait)
**Target:** 1920x1080 (landscape)

What needs to change?

1. **The frame itself:** `"frame": {"width": 1920, "height": 1080}`

2. **Every element's positioning and sizing:**
   - All `x`, `y` coordinates need to be remapped
   - All `box` values (x, y, width, height) need remapping
   - Scale factors might need adjustment if they're dependent on frame size

3. **Specific transforms:**
   - A box [0, 0, 1080, 1300] (roughly full width, aspect-aware height) becomes... what?
   - If fit=cover, the aspect ratio and positioning change
   - text elements with specific x, y placements need repositioning
   - Layer structure stays the same, but visual layout changes completely

4. **Possible things that might NOT change:**
   - Timing/start/end (these are frame-rate independent)
   - source/source_range
   - Most animation keyframe values might stay the same IF they're relative scaling (like scale: 1.0 → 1.1)

**Is this inherently a re-layout?**

Yes. Because the format stores **absolute pixel coordinates** per the sample project. If you want to maintain the same visual composition in a different aspect ratio, you must touch:
- Every x, y coordinate
- Every box rect
- Possibly rotation/scale if positions are aspect-dependent

**Could the format have done better?**

Yes, #21 asks exactly this: "Absolute pixels in the project's frame space, or resolution-independent fractions (0..1)?"

If the format used fractions (0..1), a frame resize would be:
- Change frame dimensions
- Done. All coordinates automatically scale.

Current format requires hand-remapping every coordinate.

---

## Exercise 5: Read title position and opacity at t=6000

From my own Exercise 1 JSON:
```json
{
  "id": "title-card",
  "type": "text",
  "start": 0,
  "end": 400,
  "x": [[-540, 0], [540, 400]],
  "y": 300,
  "origin": "center",
  "align": "center",
  "text": "Welcome to the Show",
  "size": 72,
  "color": "#FFFFFF",
  "font": "sans-serif",
  "line_height": 1.2,
  "opacity": [[0, 0], [400, 1]]
}
```

Question: Where is the title at t=6000 and how opaque?

**Answer:** The title is not on screen at t=6000. It ends at t=400, so at t=6000 it's completely gone (not rendered at all). Opacity is undefined/not applicable.

**How hard was that?** Very hard! I had to:
1. Recognize that start=0, end=400 means only [0, 400)
2. Notice that 6000 >= 400
3. Conclude the element isn't rendered
4. Guess that "opacity" becomes meaningless for elements not on screen

If I want to know about an element at a specific time, I need to:
1. Check start <= t < end to know if it's rendered
2. If yes, interpolate all keyframed properties
3. For x keyframes [[t1, v1], [t2, v2]], if t1 <= t <= t2, interpolate between v1 and v2

Without a query tool, this is error-prone arithmetic per element. This is exactly why ADR-0011 wants a `query --at` tool: because reading it by eye fails.

---

## Summary of Exercises

These exercises surface several critical design questions:

1. **Position keyframes format:** Is it `"x": [[t, v], ...]` or something else?
2. **Which properties are keyframeable:** Position, scale, rotation, opacity—all of them?
3. **Keyframe time domain:** Absolute times (on project clock) vs relative to element start
4. **How to express "hold" after animation ends:** Static value? Repeated final keyframe?
5. **Reading by eye:** "What is on screen at 6000?" requires element filtering, interval arithmetic, keyframe interpolation—all without tools
6. **Aspect ratio change:** Requires hand-remapping every coordinate if stored as absolute pixels

