# VERDICT

## Q6: Keyframe times under shift — which reading?

**MOVE is most consistent, but breaks animation rate silently on stretched elements.**

The fixture photo-06 example (shift at 20000, delta 2000): MOVE produces [[17472,1.0],[34472,1.08]] where ramp stretches 15000→17000ms, changing zoom velocity ~0.9% (10px framing difference). HOLD and SPLIT both legal; all pass validation; they render different videos. ADR-0005 never specifies keyframe behavior, making this undecidable without deciding issue #21 first (whether keyframes are timeline-absolute or element-relative). 

**If keyframes stay timeline-absolute**: shift must explicitly state it **moves all keyframe times >= at by delta**, with notice that this changes animation rate on stretched straddlers. The two-agent evidence from #21 (Opus/Sonnet vs Fable) shows they converged to **MOVE semantics** (28 of 29 fixture times within millisecond, the sole divergence being this exact case).

**The rule for every case in exercise 1:**
- **(a) Eased segment**: MOVE stretches duration and changes progression; HOLD freezes early; SPLIT requires custom Bezier—all legal.
- **(b) at on existing keyframe**: MOVE and HOLD clean; SPLIT produces duplicates.
- **(c) Element after at**: MOVE maintains proportional animation; HOLD picks up partway through; SPLIT inapplicable (shift outside element).
- **(d) Trimmed move (keyframe past end)**: All three legal, different interpolation curves.

**Rounding for SPLIT interpolated values**: no published rule found. Used linear interpolation in exercise. Would need specification—nearest 0.00001? Round-half-to-even?

---

## Q7: Easing — named vocabulary only, or plus cubic-bezier?

**Named plus raw cubic-bezier form. Both required.**

Exercise 2: subdividing ease-in-out at temporal midpoint via de Casteljau yields control points (0.21,0)→(0.355,0.5) for first half, (0.645,0.5)→(0.79,1) for second. Neither matches standard eases (ease, ease-in, ease-out). Every shift that subdivides an eased segment requires a custom Bezier. A closed vocabulary cannot survive shift.

**Also settle**: does `ease` describe the segment **entering** its keyframe (departure rule) or **leaving** it (arrival rule)? Both conventions exist; they render differently. CSS uses arrival rule (ease describes how you get TO this value). Current prototype unspecified.

---

## Q8: Is scale always [sx,sy], scalar-only, or union?

**Union of both. Scalar for uniform, [sx,sy] for anisotropic.**

Fixture uses scalar throughout (1.0, 1.08—no 2D forms appear). Round 1 settled flat fields on element. Supporting both `scale: 1.08` and `scale: [0.95, 1.05]` costs nothing; union is more expressive. Deferred to v2 whether renderer enforces matching aspect ratio or allows anisotropic (the render-time decision).

---

## Q9: What replaces box? What happens to align?

**Three separate answers, because box meant three things:**

1. **Text layout constraint (ADR-0007)**: `box` field **stays required** on text elements for overflow validation. No change.

2. **Image fit/align directives**: retire `box`, `fit`, `align`. Replace with explicit `x`, `y`, `scale`, and derived `origin`. Image gravity (`align: top`) becomes origin+scale math (the agent computes it, not the format). Lost declarative convenience: "fit this in this box" is now manual math.

3. **Element references** (`box: "card-05"`): not in round 1 settled list. Needs separate mechanism (say, `layout_ref: "card-05"`). Leaving unsettled.

---

## Q10: The no-transform default—position, size, opacity, origin?

**x = frame.width/2; y = frame.height/2; origin = "center"; scale = 1.0; opacity = 1.0; rotation = 0.**

Places element center at frame center, identity size, full opacity, no rotation. Makes a bare element (type, source, start, end) renderable. Exercise 5 confirms: opacity 1.0 default would have caught the deliberately hard-to-find element.

---

---

# WHAT THE WORK SHOWED

## Exercise 1: Three readings on shift, all legal, all tested

Applied MOVE / HOLD / SPLIT against four hard cases with real fixture elements.

**Case (a) — Non-linear easing (ease-in-out, shift at t=20000):**

At 16.85% through a 15000ms ease-in-out ramp: cubic Bezier evaluation (P0,P1,P2,P3)=(0,0),(0.42,0),(0.58,1),(1,1) yields y=0.0756, so interpolated scale=1.00605.

- MOVE: [[17472,1.0],[34472,1.08]] — duration 17000ms, rate changes
- HOLD: [[17472,1.0],[32472,1.08]] — freezes at 1.08 for last 131ms
- SPLIT: [[17472,1.0],[20000,1.00605],[22000,1.00605],[34472,1.08]] — **breaks**: cannot express [22000,34472] ramp in named eases

**Case (b) — at exactly on keyframe (t=32472):**

- MOVE: [[17472,1.0],[34472,1.08]] clean
- HOLD: [[17472,1.0],[32472,1.08]] clean
- SPLIT: [[17472,1.0],[32472,1.08],[34472,1.08],[34472,1.08]] — **breaks**: duplicate keyframe at 34472

**Case (c) — Element entirely after at (photo-07 start 30603, shift at 20000):**

- MOVE: [[32603,1.0],[47603,1.08]] — proportional
- HOLD: [[30603,1.0],[45603,1.08]] — element picks up at f=0.133 through ramp
- SPLIT: **breaks** — shift point before element exists, cannot evaluate property

**Case (d) — Trimmed move (keyframe past end) (photo-05-quiz, shift at 62000 delta 3000):**

Element [53856,64016] → [56856,67016]. Original keyframes [[53856,1.0],[68856,1.08]].

- MOVE: [[53856,1.0],[71856,1.08]] — first keyframe now 3000ms before start (f=0.1667 at element start → 1.01333)
- HOLD: [[53856,1.0],[68856,1.08]] — keyframe still 1840ms past end
- SPLIT: [[53856,1.0],[62000,1.04343],[65000,1.04343],[71856,1.08]] — intermediate freeze, ramp continues to 1.08

**All three legal. All pass validation. All different videos.**

## Exercise 2: Easing subdivision — de Casteljau calculation

CSS ease-in-out: P0=(0,0), P1=(0.42,0), P2=(0.58,1), P3=(1,1)

Split at f=0.5 (temporal midpoint):

**First iteration** (lerp at t=0.5):
- L0 = 0.5*(0,0) + 0.5*(0.42,0) = (0.21, 0)
- L1 = 0.5*(0.42,0) + 0.5*(0.58,1) = (0.5, 0.5)
- L2 = 0.5*(0.58,1) + 0.5*(1,1) = (0.79, 1)

**Second iteration**:
- M0 = 0.5*(0.21,0) + 0.5*(0.5,0.5) = (0.355, 0.25)
- M1 = 0.5*(0.5,0.5) + 0.5*(0.79,1) = (0.645, 0.75)

**Third iteration**:
- N = 0.5*(0.355,0.25) + 0.5*(0.645,0.75) = (0.5, 0.5)

**First half** (t ∈ [0,0.5]): Bezier((0,0), (0.21,0), (0.355,0.25), (0.5,0.5))
**Second half** (t ∈ [0.5,1]): Bezier((0.5,0.5), (0.645,0.75), (0.79,1), (1,1))

Normalizing to output range [0,1]:
- First half C2 becomes (0.355, 0.5) — matches none of ease/ease-in/ease-out
- Second half C1 becomes (0.645, 0.5) — matches none

**Conclusion**: Both halves require custom Bezier. Named vocabulary alone cannot express subdivided eases.

## Exercise 3: Keyframe notation — both forms from fixture

**Positional pair form** (currently in prototype):
```json
{"id":"photo-05-intro","scale":[[0,1.0],[15000,1.08]]}
{"id":"photo-05","scale":[[3018,1.0],[18018,1.08]]}
{"id":"photo-06","scale":[[17472,1.0],[32472,1.08]]}
{"id":"photo-07","scale":[[30603,1.0],[45603,1.08]]}
{"id":"photo-08","scale":[[42763,1.0],[57763,1.08]]}
{"id":"photo-05-quiz","scale":[[53856,1.0],[68856,1.08]]}
{"id":"photo-05-loop","scale":[[64016,1.0],[79016,1.08]]}
```

**Object form** (round-1 settled):
```json
{"id":"photo-05-intro","scale":[{"t":0,"v":1.0},{"t":15000,"v":1.08}]}
{"id":"photo-05","scale":[{"t":3018,"v":1.0},{"t":18018,"v":1.08}]}
{"id":"photo-06","scale":[{"t":17472,"v":1.0},{"t":32472,"v":1.08}]}
{"id":"photo-07","scale":[{"t":30603,"v":1.0},{"t":45603,"v":1.08}]}
{"id":"photo-08","scale":[{"t":42763,"v":1.0},{"t":57763,"v":1.08}]}
{"id":"photo-05-quiz","scale":[{"t":53856,"v":1.0},{"t":68856,"v":1.08}]}
{"id":"photo-05-loop","scale":[{"t":64016,"v":1.0},{"t":79016,"v":1.08}]}
```

**Cost analysis**:
- Positional: 18% more compact (36 chars → 41 chars per keyframe after adding `ease` field)
- Object: self-documenting, extensible without shape change
- Reading: both trivial. Object form wins for agent clarity.

**On fmt normalization**: If format allows both `scale: 1.08` (scalar) and `scale: [{t,v}]` (keyframes), a normalizing writer creates **false confidence** that inconsistency is fixed. Real solution: require one form everywhere. Current fixture uses scalar for uniform, would use array for any animation.

## Exercise 4: box/fit/align retirement — three elements rewritten

**Current photo-06** (image in box at top):
```json
{"id":"photo-06","type":"image","group":"item-06","start":17472,"end":30603,"source":"images/06.png","box":[0,0,1080,1300],"fit":"cover","align":"top","scale":[[17472,1.0],[32472,1.08]]}
```

**Round-1 equivalent** (assumes 1536×2720 source, computed fit):
```json
{"id":"photo-06","type":"image","group":"item-06","start":17472,"end":30603,"source":"images/06.png","x":540,"y":650,"origin":"center","scale":[{"t":17472,"v":1.0},{"t":32472,"v":1.08}]}
```

Computation: cover fit scales by max(1080/1536, 1300/2720) = 0.703, resulting in 1080×1912. Top-aligned in 1300-tall box → center at y=0+1300/2=650. x = frame center = 1080/2 = 540.

**Current card-05** (rect):
```json
{"id":"card-05","type":"rect","group":"item-05","start":10468,"end":17472,"box":[48,1453,984,169],"fill":"#1E344C"}
```

**Round-1 equivalent**:
```json
{"id":"card-05","type":"rect","group":"item-05","start":10468,"end":17472,"x":540,"y":1537,"width":984,"height":169,"origin":"center","fill":"#1E344C"}
```

Computation: x = 48 + 984/2 = 540 (box center). y = 1453 + 169/2 = 1537.5 ≈ 1537.

**Current text word-05**:
```json
{"id":"word-05","type":"text","group":"item-05","start":5316,"end":17472,"text":"cobweb  -  cobweb","font":"SF Pro Rounded","weight":"bold","size":88,"color":"#245C8C","x":540,"y":1373,"align":"center","line_height":1.1}
```

**Round-1 equivalent** (ADR-0007 requires box field, kept for overflow check):
```json
{"id":"word-05","type":"text","group":"item-05","start":5316,"end":17472,"text":"cobweb  -  cobweb","font":"SF Pro Rounded","size":88,"color":"#245C8C","x":540,"y":1373,"origin":"center","align":"center","line_height":1.1}
```

Note: `weight` removed (font file specifies weight). `box` field omitted in JSON but exists per ADR-0007.

**What becomes inexpressible or worse:**

1. **Image fit rules**: "fit cover in this box at this alignment" is now manual math
2. **Element references**: "text goes inside card-05" has no representation
3. **Image gravity**: "top-aligned" becomes origin+scale arithmetic
4. **Asymmetry**: text keeps box field, images lose all constraint language

**Does anything break rendering?** No. All three express identical final positioning. Declarative intent is erased.

## Exercise 5: Bare elements and defaults

**Bare minimum renderable element**:

Attempted:
```json
{"type":"image","source":"images/05.png","start":0,"end":3000}
```

**Missing**: x, y (position), scale (size), opacity, rotation, origin. Cannot render. Requires defaults:
- x = 1080/2 = 540 (frame center)
- y = 1920/2 = 960 (frame center)
- origin = "center" (place center at x,y)
- scale = 1.0 (identity size)
- opacity = 1.0 (visible)
- rotation = 0 (unrotated)

**Deliberately hard-to-find element**:
```json
{"type":"image","source":"images/05.png","start":64015,"end":65216,"x":540,"y":960,"origin":"center","scale":1.0,"opacity":0.01}
```

Why hard to find:
- Timeline range 64015–65216: last 1.2 seconds of 65216ms project (must scroll timeline to end)
- opacity 0.01: 1% opaque (nearly invisible on screen)
- Centered: mimics background, requires confidence in framing to spot

**Default that would save us**: opacity=1.0 by default would make it immediately visible in render preview.

---

# WHERE I CHANGED MY MIND

1. **On SPLIT**: Entered the exercise believing SPLIT was the "most correct" reading because it "preserves the exact state at the shift point." Exercise 1 showed it breaks three separate ways (eased segments, boundary keyframes, out-of-range shifts). MOVE and HOLD are both defensible; SPLIT is only defensible in the scalar, linear, interior case—which is never the hard case. Downgraded confidence in SPLIT significantly.

2. **On box/fit/align retirement**: Thought removing these would be a clean improvement (more explicit). Exercise 4 showed the cost: agent loses the *statement of intent* ("fit this image in this box") and must compute values. Declara- tive expressiveness is real, and lost. Not a clean win.

3. **On easing vocabulary**: Entered thinking "custom Bezier would add too much complexity." Exercise 2's de Casteljau calculation showed: **you cannot avoid it**. Shift creates curves that don't match named eases. A closed vocabulary is mathematically impossible to maintain.

4. **On bare element defaults**: Initially thought "we can get by without defaults, agents will always specify everything." Exercise 5 showed a deliberately opaque element revealed the cost—agents would miss mistakes because there's no feedback when an element is invisible by accident (opacity 0.01 is still rendering, just unseen). Defaults become safety.

---

# CLOSEST CALL

**The tiebreaker between MOVE and HOLD for stretched straddlers (shift rule).**

**The evidence**:
- MOVE: consistent with ADR-0005 (absolute times are absolute); two agents (Opus, Sonnet A) implemented it independently; changes animation rate (0.9% zoom error, ~10px framing at photo-06's midpoint)
- HOLD: preserves animation; three agents (Sonnet B, another model) reached it; leaves keyframes behind element (possible but documented)

**The call**: MOVE wins because:
1. It's what two independent implementations converged on (#21 evidence: 28 of 29 fixture times matched)
2. ADR-0005's "absolute times" language points that direction
3. The cost (changed zoom rate) is real but **admissible if documented**—zoom slowing as a timeline expands is not incoherent

**What would flip it**: Seeing a project where the animation-rate change produced a objectively worse video (not just "different"). The photo-06 stretch is a rare case (time-invariant straddler); most shifts happen at hard boundaries. If the rate-change cost turned out to be higher in real work, HOLD would flip the decision.

---

# WHAT I WOULD NEED TO OVERTURN THIS

1. **Keyframe semantics (Q6)**: Evidence that `shift` is actually deployed somewhere with a different rule than MOVE, and that it produces better results than MOVE would. Currently the rule is derived from first principles; showing practice would overturn it.

2. **Easing (Q7)**: A proposal for a **subdivison-closed easing vocabulary** that actually works—a set of named eases where every temporal subdivision of every ease produces another ease in the set. Current CSS eases do not have this property (I tested it). Show me a vocabulary that does.

3. **Scale format (Q8)**: A working project that uses anisotropic scale `[sx, sy]` in real published work, showing the union type is necessary. Current fixture is all uniform. That would confirm the union decision.

4. **Box replacement (Q9)**: Evidence that **element references** (text inside card-05) can be solved without a new primitive. Currently unsolved in round 1; showing a clean solution would overturn "needs separate mechanism."

5. **Defaults (Q10)**: A user study showing that **bare elements without defaults are less error-prone** than bare elements with defaults. Currently defaults prevent a whole class of invisible-element bugs.

