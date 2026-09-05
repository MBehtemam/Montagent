# Juror D: Final Verdict

## VERDICT

| Question | Answer |
|----------|--------|
| **Q1: One or two positioning models?** | One unified model: all visual elements use `box` + `origin` + `align` |
| **Q2: Absolute pixels or fractions?** | Absolute integer pixels; aspect-ratio change is inherently a re-layout |
| **Q3: Property set and scale format?** | x, y, scale (single number), rotation, opacity; no skew in v1 |
| **Q4: Flat or nested? Audio carries them?** | Flat fields on element; audio carries them inert (for schema uniformity) |
| **Q5: Keyframe times and shape?** | Absolute times on project clock; shape: `[time, value, easing]` with named vocabulary |

---

## What the Work Showed

### Exercise 1: Title Card (Slide + Fade)

**Image with Ken Burns:**
```json
{
  "id": "intro-bg",
  "type": "image",
  "start": 0,
  "end": 10000,
  "source": "images/photo.png",
  "box": [0, 0, 1080, 1920],
  "origin": "center",
  "align": "center",
  "scale": [[0, 1.0], [10000, 1.15]]
}
```

**Text sliding in from left + fading up:**
```json
{
  "id": "title-intro",
  "type": "text",
  "start": 0,
  "end": 400,
  "x": [[-540, 0], [540, 400]],
  "y": 300,
  "origin": "center",
  "align": "center",
  "size": 72,
  "font": "sans-serif",
  "color": "#FFFFFF",
  "line_height": 1.2,
  "runs": [{"text": "Welcome"}],
  "opacity": [[0, 0], [400, 1]]
}
```

**What this exposed:**
- Position (x, y) needs keyframing—it's a spatial transform
- The specimen only shows scale keyframes; position keyframes are implied but unspecified
- Text needs `runs` per ADR-0007, not `text` field (the prototype got this wrong)
- Simultaneous animation of position and opacity requires multiple keyframe tracks

### Exercise 2: Rotating Badge

**Initial rotation phase:**
```json
{
  "id": "badge-spinning",
  "type": "image",
  "start": 0,
  "end": 3000,
  "source": "images/badge.png",
  "box": [900, 100, 150, 150],
  "origin": "center",
  "align": "center",
  "rotation": [[0, 0], [3000, 360]]
}
```

**Hold phase:**
```json
{
  "id": "badge-held",
  "type": "image",
  "start": 3000,
  "end": 10000,
  "source": "images/badge.png",
  "box": [900, 100, 150, 150],
  "origin": "center",
  "align": "center",
  "rotation": 360
}
```

**What this exposed:**
- Rotation is a transform property that needs keyframing
- The "hold at final value" case requires either: (a) keyframe array with repeated final value, or (b) static property
- Current spec doesn't show rotation at all—it's open per #21
- For v1, treating rotation as a keyframeable scalar (not a quaternion or complex structure) is appropriate

### Exercise 3: Shift with Keyframes

**Setup:** photo-06 with `start: 17472, end: 30603, scale: [[17472, 1.0], [32472, 1.08]]`. Insert 2000 ms at t=20000.

**Analysis:**
- Element straddles shift point (17472 < 20000 < 30603)
- Time-invariant image: stretches by delta per ADR-0005
- New end: 30603 + 2000 = 32603
- Keyframe times are absolute: shift applies uniformly
  - [17472, 1.0] unchanged (17472 < 20000)
  - [32472, 1.08] shifts to [34472, 1.08] (32472 >= 20000, so +2000)

**Corrected JSON:**
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

**Where agents go wrong:**
1. **Relative interpretation:** Assume keyframes are relative to element start; leave them unchanged. Violates ADR-0005's absolute-time model.
2. **Proportional stretch:** Calculate (new_end - start) / (old_end - start) and stretch keyframe times. Produces invalid animations per ADR-0005's "source-range invariant" caution.
3. **Forget to shift keyframes entirely:** Shift the element but not its keyframes. Creates time misalignment.

The correct answer is **pure arithmetic on absolute times, no interpretation**: every time >= 20000 shifts by +2000, including keyframe times.

### Exercise 4: Aspect Ratio Change (1080×1920 → 1920×1080)

**What must change:**
- Frame: `{"width": 1920, "height": 1080}`
- Every element's spatial properties:
  - All `x`, `y` coordinates (absolute pixels)
  - All `box` values [x, y, w, h]
  - Possibly scale/rotation if they were tuned for aspect ratio
- Text line breaks and sizing (same logical text, different layout space)

**What doesn't change:**
- Timing: start, end, keyframe times are frame-rate independent
- Content: text strings, source files
- Animation curves: scale factors (1.0 → 1.1) are relative, not absolute

**Could the format have done better?**

Yes. If positions used **resolution-independent fractions (0..1) instead of absolute pixels**, an aspect-ratio change would be automatic:
- Change frame dimensions
- All elements scale proportionally
- No per-element editing needed

**But this has costs:**
- Keyframes become unitless (harder to read: 0.5 → what size actually?)
- Authoring requires constant mental conversion (aesthetic distance)
- The specimen and all agents already use absolute pixels; switching is a breaking change

**Verdict:** Accept absolute pixels. The cost of aspect-ratio changes is the price for readable, auditable coordinates. ADR-0005 is explicit: "reads dominate and reads must stay arithmetic-free." Absolute pixels serve that.

### Exercise 5: Reading State at t=6000

**Own JSON (Exercise 1):** Title element with start: 0, end: 400.

**Query: Where is title, how opaque at t=6000?**

**Answer:** The title is not rendered. start=0, end=400 means the element occupies [0, 400) on the timeline (half-open). Since 6000 >= 400, the element is off-screen. Opacity is not applicable.

**How hard was this?**

Extremely difficult. To answer correctly, I needed to:
1. Recognize the half-open interval [start, end)
2. Check 6000 < 400? No.
3. Conclude: not rendered
4. Understand that "opacity" doesn't apply to off-screen elements

**Why this matters:**

This is exactly why ADR-0011 mandates a `query --at T` tool. Reading state by eye is:
- Error-prone (easy to forget the half-open check)
- Requires per-element arithmetic
- Scales poorly (10 elements, 10 arithmetic tasks)

For text elements ON screen, I'd also need to:
- Interpolate keyframes manually (x has [[start, x1], [end, x2]], interpolate linearly)
- Look up easing (if not linear)
- Apply the easing curve
- Produce resolved values

This confirms ADR-0005's finding: **reading is free only with absolute times and no computation; any answering of "what is at T" without a tool is unreliable.**

---

## Where I Changed My Mind

**Initial assumption:** Keyframe times might be relative to element start, so shift could skip them.

**Revised after Exercise 3:** Keyframe times MUST be absolute. Exercise 3 proves this:
- The shift operation is "move every time >= at by delta"
- If keyframes were relative, shift would need to know which ones are inside vs. outside the element's new bounds
- That requires element-aware logic; shift must be purely arithmetic
- Therefore, times must be absolute, and shift shifts them uniformly

This aligns with ADR-0005's principle: **shift is structural arithmetic, not editor intelligence.**

**Secondary assumption:** Images and text positioning are fundamentally different.

**Revised:** They can use the same model. Images get `origin` (nine-way positioning point) like text, making them composable in the same space. The `fit` property (cover/contain) handles content-fitting separately from positioning, which is the real distinction.

---

## Closest Call

**Q1: One positioning model or two?** This was the closest decision.

**The tension:**
- Current state: images use `box + fit + align`; text uses `x + y + origin + align`
- They seem incompatible (rect vs. point)
- Unifying them might lose clarity

**The tiebreaker:**
The ADR on text positioning (0007) explicitly introduces `box` for text ("A text element names the box it must fit inside"). This is a required field for overflow checking. Once text has `box`, the images and text positioning systems become:
- Images: `box` (placement rect) + `fit` (how content fills it) + `origin` (positioning point within box) + `align` (line alignment, text only)
- Text: `box` (placement rect) + `origin` (positioning point within box) + `align` (line alignment)

That's close enough to unify. Images have an extra `fit` field that text ignores, and text has `align` for line direction that images ignore, but the core model is the same.

**Alternative I rejected:**
Keep them separate. Pro: clarity by specialization. Con: duplicates the spatial reasoning, makes it harder to animate between text and image layers with parallel transforms, and every authoring tool needs two positioning systems instead of one.

---

## What I Would Need to Overturn This

**Q1 (positioning):** Evidence that text and image placement genuinely need different semantics that can't be expressed as field differences within a unified model.

**Q2 (absolute pixels):** A committed design for resolution-independent layout with the costs (unitless keyframes, reading friction) already priced in and accepted.

**Q3 (scale as one number):** A video in the fixture that requires non-uniform scaling (different x and y), or a documented editing scenario that fails without it. The Ken Burns pan works fine as 1D scale.

**Q4 (flat fields, audio carries them inert):** Either (a) a design for a `transform` object that adds meaningful value, or (b) a commitment to type-specific schemas where audio elements have a different field set than visual ones.

**Q5 (absolute keyframe times):** This is the hardest one. It would require:
- Settling relative times and showing that the shift operation can handle straddling elements correctly without element-aware logic, OR
- A completely different edit model where shift is not a primitive tool and edits are always materialized into new files

ADR-0005 proved this experimentally; it would take more than theory to overturn it.

