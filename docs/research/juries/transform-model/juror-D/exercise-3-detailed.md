# Exercise 3: Shift Operation - Detailed Analysis

## Setup
- photo-06: starts at 17472, ends at 30603 (13,131 ms duration)
- Scale keyframes: [[17472, 1.0], [32472, 1.08]]
- Shift point: t=20000
- Shift delta: +2000 ms

## Key Question: What does shift do to this element?

### Classification
- Element type: image (time-invariant, no internal clock)
- Element straddles the shift point: 17472 < 20000 < 30603 ✓

### Per ADR-0005: Time-Invariant Straddlers
"shift **stretches** the straddler's `end` by `delta`"

- New end: 30603 + 2000 = **32603**
- Start: 17472 (unchanged)

### Keyframe Handling

The critical insight from ADR-0005:

> Stretching speech violates the source-range invariant — one agent's "obvious" uniform-stretch policy produced an element with an 8.1s timeline range against a 5.2s source, an invalid file.

This is about AUDIO (time-based). For images (time-invariant), stretching the display window makes sense, but the animation keyframes need careful handling.

#### Original keyframes: [[17472, 1.0], [32472, 1.08]]

These are absolute times on the project clock.

When we shift at t=20000 by +2000:
- Every time >= 20000 moves forward by 2000

Keyframe at 17472: 
  - 17472 < 20000, so unchanged

Keyframe at 32472:
  - 32472 >= 20000, so shifts to 32472 + 2000 = 34472

#### Interpretation: Two plausible agents, two different answers

**Agent A (Absolute Shift):**
Shifts all keyframe times >= 20000:
```json
"scale": [[17472, 1.0], [34472, 1.08]]
```
Result: animation runs from 17472-34472 (17000 ms duration). After 30603, the element holds at 1.08x.

**Agent B (Relative Interpretation):**
"The keyframes are inside the animation, so they're relative to element start"
Tries to keep them unchanged:
```json
"scale": [[17472, 1.0], [32472, 1.08]]
```
Result: Same animation in same absolute time space. WRONG—ignores that times are absolute per ADR-0005.

**Agent C (Proportional Stretch):**
"Element got longer, so animation should too"
- Original animation span: 32472 - 17472 = 15000 ms
- Original element span: 30603 - 17472 = 13131 ms
- Stretched element span: 32603 - 17472 = 15131 ms
- Ratio: 15131 / 13131 = 1.1524
- Stretch animation keyframes: 
  - 17472 stays
  - 32472 becomes 17472 + (32472-17472) × 1.1524 = 17472 + 17306 = 34778
```json
"scale": [[17472, 1.0], [34778, 1.08]]
```
Result: Animation stretches proportionally. WRONG per ADR-0005—this is the "uniform-stretch policy" that violates the source-range invariant.

### The Right Answer

**Agent A is correct:** Shift operates on absolute times uniformly.

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

### Why this matters

ADR-0005 explicitly states: "Two of the three had their working directory overwritten by a concurrent respondent and redid the exercise in isolation; one saw a fragment of another's file before stopping."

And: "Two of the three had to apply to close a three-second hole they had punched in the stills track" via hand-fixing stretches.

The memo then says: "The answer here is clear: **all three agents independently shipped a project whose narration outran its last visual, and in all three cases it survived every check they ran.**"

This suggests that getting shift wrong is common and subtle. The insight is:
- Shift is purely arithmetic on absolute times
- No interpretation, no intelligence
- Keyframes shift wholesale like everything else
