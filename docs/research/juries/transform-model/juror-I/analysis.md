# Jury I Analysis

## Working Through the Evidence

### Exercise 1: SPLIT Stress Test - Key Findings

Tested SPLIT against four hard cases:

**Case (a) - Non-linear easing (ease-in-out):**
At the shift point t=20000 with ease-in-out control points (0.42,0)→(0.58,1):
- MOVE: ramp duration stretches 15000→17000ms; zoom rate changes ~0.9% at midpoint (~10px framing difference)
- HOLD: ramp ends at 32472, freezes at 1.08 for remaining 131ms of element
- SPLIT: interpolated value is 1.00605; creates [[17472,1.0],[20000,1.00605],[22000,1.00605],[34472,1.08]]

**SPLIT BREAKS**: Cannot express the remainder ramp [22000→34472,1.08] in named eases. Would need custom Bezier starting at 1.00605, not 1.0.

**Case (b) - `at` exactly on existing keyframe (t=32472):**
- MOVE: [[17472,1.0],[34472,1.08]] ✓ clean
- HOLD: [[17472,1.0],[32472,1.08]] ✓ clean (freeze tail)
- SPLIT: Would insert at existing keyframe then shift: [[17472,1.0],[32472,1.08],[34472,1.08],[34472,1.08]]

**SPLIT BREAKS**: Produces duplicate keyframes. Semantically questionable.

**Case (c) - Element entirely after `at` (photo-07, shift at t=20000 before element start 30603):**
- MOVE: [[32603,1.0],[47603,1.08]] ✓ maintains proportional animation
- HOLD: [[30603,1.0],[45603,1.08]] creates element that picks up partway through ramp (f=0.2 at start)
- SPLIT: Cannot evaluate property at t=20000; shift point is before element exists

**SPLIT BREAKS**: Inapplicable when shift point is outside element.

**Case (d) - Keyframe past element end / trimmed move (photo-05-quiz start 53856, end 64016, keyframe at 68856):**
Shift at t=62000, delta=3000. Element becomes [56856,67016].
- MOVE: [[53856,1.0],[71856,1.08]] - first keyframe now before start, entire ramp stretches
- HOLD: [[53856,1.0],[68856,1.08]] - keyframe still 1840ms past end
- SPLIT: [[53856,1.0],[62000,1.04343],[65000,1.04343],[71856,1.08]] - introduces intermediate freeze

All three produce different ramp progression. All legal. All pass validation.

### Exercise 2: Easing Subdivisions

CSS ease-in-out: P0=(0,0), P1=(0.42,0), P2=(0.58,1), P3=(1,1)

Split at f=0.5 (temporal midpoint) using de Casteljau:

**First half control points** (0→0.5):
- Start: (0, 0)
- C1: (0.21, 0)
- C2: (0.355, 0.25)  [unnormalized: 0.5 scaled to 1.0 range]
- End: (0.5, 0.5)

**Second half** (0.5→1.0):
- Start: (0.5, 0.5)
- C1: (0.645, 0.75)
- C2: (0.79, 1)
- End: (1, 1)

**Neither matches any named ease:**
- ease vs our-first-half: (0.25,0.1)→(0.25,1) ≠ (0.21,0)→(0.355,0.5)
- ease-in vs either: (0.42,0)→(1,1) doesn't match

**Conclusion**: A closed named easing vocabulary **cannot survive shift**. Custom cubic-bezier form is required.

### Exercise 3: Keyframe Notation Cost

Seven Ken Burns elements as currently stored (positional pairs):

```json
"scale": [[0,1.0],[15000,1.08]]
"scale": [[3018,1.0],[18018,1.08]]
"scale": [[17472,1.0],[32472,1.08]]
"scale": [[30603,1.0],[45603,1.08]]
"scale": [[42763,1.0],[57763,1.08]]
"scale": [[53856,1.0],[68856,1.08]]
"scale": [[64016,1.0],[79016,1.08]]
```

As round-1 object form:

```json
"scale": [{"t":0,"v":1.0},{"t":15000,"v":1.08}]
"scale": [{"t":3018,"v":1.0},{"t":18018,"v":1.08}]
"scale": [{"t":17472,"v":1.0},{"t":32472,"v":1.08}]
"scale": [{"t":30603,"v":1.0},{"t":45603,"v":1.08}]
"scale": [{"t":42763,"v":1.0},{"t":57763,"v":1.08}]
"scale": [{"t":53856,"v":1.0},{"t":68856,"v":1.08}]
"scale": [{"t":64016,"v":1.0},{"t":79016,"v":1.08}]
```

**Costs**:
- Positional: 18% more compact; harder to extend (adding `ease`, `easing_dir`, other keyframe properties)
- Object: self-documenting; future-proof; costs 4-5 extra bytes per keyframe
- "fmt normalizing scalar on write": **this is a dodge**. If format allows both `scale: 1.08` and `scale: [{t,v}]`, normalizer creates false impression of consistency. Real fix: require one form everywhere.

**Honest assessment**: Reading object form is trivial (each keyframe names its fields). The positional pair form is the one that optimizes for the wrong operation (compact storage vs. agent clarity).

### Exercise 4: box/fit/align Retirement

**Current photo-06**:
```json
{"box":[0,0,1080,1300],"fit":"cover","align":"top","scale":[[17472,1.0],[32472,1.08]]}
```

**Round-1 version**:
```json
{"x":540,"y":650,"origin":"center","scale":[{"t":17472,"v":1.0},{"t":32472,"v":1.08}]}
```

(Assumes 1536×2720 image, cover fit to 1080×1300 box at top = scale by 0.703, position at box center)

**Current card-05**:
```json
{"box":[48,1453,984,169],"fill":"#1E344C"}
```

**Round-1 version**:
```json
{"x":540,"y":1537.5,"width":984,"height":169,"origin":"center","fill":"#1E344C"}
```

**Current word-05** (text with box):
```json
{"text":"cobweb  -  cobweb","font":"SF Pro Rounded","x":540,"y":1373,"align":"center"}
```

**Round-1 version** (same, no box field shown but ADR-0007 says it exists):
```json
{"text":"cobweb  -  cobweb","font":"SF Pro Rounded","x":540,"y":1373,"origin":"center","align":"center"}
```

**What becomes inexpressible/worse**:
1. **Declarative fit rules are gone** - agent must compute scale value explicitly (no longer says "fit this image in this box")
2. **Element references for layout are lost** - cannot say "text goes inside card-05" as a named constraint
3. **Image alignment is lost** - "top" or "bottom" or "center" gravity becomes explicit coordinate math
4. **Text box field required by ADR-0007** - must be kept for overflow validation, creating asymmetry (text has box, images don't)

**Does anything break design?** No - all three express identical final rendering. But the declarative intent is erased.

### Exercise 5: Bare Element Defaults

**Bare minimum** element (type, source, start, end):
```json
{"type":"image","source":"images/05.png","start":0,"end":3000}
```

Missing:
- `x`, `y` - where to place it
- `scale` or `width`,`height` - what size
- `opacity` - visibility
- `rotation` - orientation
- `origin` - which point to place

**Cannot render without defaults.** Need at minimum: position (center frame?), opacity (1.0?).

**Deliberately hard-to-find element**:
```json
{"type":"image","source":"images/05.png","start":64015,"end":65216,"x":540,"y":960,"scale":1.0,"opacity":0.01}
```

Why hard to find:
- Last 1.2 seconds of 65-second project (have to scroll timeline far)
- 1% opacity (nearly invisible on screen)
- Centered, so might mistake it for background at low confidence

**Default that would save us**: `opacity=1.0` by default - would make it visible immediately when checking render.

---

## Cross-Examination Against Issue #21

Issue #21 identifies the exact ambiguity my Exercise 1 confirms:

> "Both are legal, both pass every check, different video"

The three readings from Opus/Sonnet (HOLD) vs Fable (MOVE) on photo-06 stretch:

| Reading | Keyframes after shift | Ramp duration | Final value at element end |
| --- | --- | --- | --- |
| HOLD | [17472,1.0],[32472,1.08] | 15000ms | 1.0800 (freeze tail 172ms) |
| MOVE | [17472,1.0],[34472,1.08] | 17000ms | 1.0712 (still zooming) |

My exercise (d) with photo-05-quiz shows SPLIT adds a third option, all legal.

**Issue #21's open question remains unresolved**: keyframe times must be either:
1. Element-relative (SPLIT becomes no-op, "trimmed move" becomes clearer, but reading keyframe time requires addition per ADR-0001)
2. Timeline-absolute (current model), but then shift needs explicit rule for stretched straddlers

I cannot decide this - it is exactly what #21 names as blocking shift.

---

## The Five Questions: Raw Verdicts

Q6: SHIFT keyframe rule
- MOVE consistent with ADR-0005 (absolute times move)
- But breaks animation rates on stretched elements (zoom changes 0.9% silently)
- HOLD preserves animation but leaves keyframes behind
- SPLIT breaks easing and boundary cases
- **There is no right answer without deciding #21 first**
- If forced: **MOVE is more consistent, but needs explicit schema rule** that this changes animation rate on stretched elements

Q7: Easing vocabulary
- **Named eases + cubic-bezier forms required**
- Exercise 2 proves subdivided eases don't match standard names
- Also needs: explicit statement of whether `ease` describes segment entering or leaving keyframe

Q8: Scale format
- The fixture uses scalar throughout (1.0, 1.08, no 2D array forms appear)
- Round 1 settled "flat fields on element"
- **Union type: scalar for uniform, [sx,sy] for anisotropic**
- Fixture tests only scalar case

Q9: Box replacement
- **Text keeps box field (ADR-0007); images become x,y,scale,opacity,rotation explicitly**
- **Align: text keeps horizontal alignment; image gravity becomes origin+scale math**
- **Element references (text inside card-05) need separate mechanism** (not settled in round 1)

Q10: No-transform default
- **x = frame.width/2, y = frame.height/2** (center of frame)
- **origin = "center"** (places the center of element at x,y)
- **scale = 1.0** (identity)
- **opacity = 1.0** (fully opaque)
- **rotation = 0** (no rotation)
